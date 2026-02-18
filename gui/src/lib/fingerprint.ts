// Site Fingerprint / WAF Detection Engine
// Runs entirely in the frontend — matches HTTP response headers, cookies, body patterns

export interface FingerprintRule {
	id: string;
	name: string;
	category: 'waf' | 'cdn' | 'server' | 'framework' | 'security' | 'session' | 'bot_protection';
	confidence: 'high' | 'medium' | 'low';
	match: {
		cookie?: string;       // cookie name pattern (regex)
		header?: string;       // header name to check (case-insensitive)
		headerValue?: string;  // header value pattern (regex)
		bodyPattern?: string;  // response body regex
		statusCode?: number;
	};
	description: string;
	details?: string;
}

export interface FingerprintMatch {
	rule: FingerprintRule;
	evidence: string[];
}

export interface ResponseInfo {
	status_code: number;
	headers: Record<string, string>;
	cookies: Record<string, string>;
	body?: string;
}

export interface StackEntry {
	name: string;
	category: string;        // 'waf' | 'cdn' | 'server' | 'framework'
	confidence: 'high' | 'medium' | 'low';
}

export interface FingerprintResult {
	stack: StackEntry[];      // top-level summary: "Cloudflare, nginx, PHP"
	matches: FingerprintMatch[];
	securityHeaders: { name: string; present: boolean; value?: string }[];
	raw: { headers: Record<string, string>; cookies: Record<string, string> };
}

// ─── Detection Rules ─────────────────────────────────────────────────

const rules: FingerprintRule[] = [
	// ── WAF ──────────────────────────────────────────────────────────
	{
		id: 'akamai-abck', name: 'Akamai Bot Manager', category: 'waf', confidence: 'high',
		match: { cookie: '^_abck$' },
		description: 'Akamai Bot Manager detected via _abck cookie',
		details: 'Requires sensor data generation to bypass',
	},
	{
		id: 'akamai-akbmsc', name: 'Akamai Bot Manager', category: 'waf', confidence: 'high',
		match: { cookie: '^ak_bmsc$' },
		description: 'Akamai Bot Manager detected via ak_bmsc cookie',
	},
	{
		id: 'akamai-bmsz', name: 'Akamai Bot Manager', category: 'waf', confidence: 'medium',
		match: { cookie: '^bm_sz$' },
		description: 'Akamai Bot Manager detected via bm_sz cookie',
	},
	{
		id: 'akamai-bmsv', name: 'Akamai Bot Manager', category: 'waf', confidence: 'medium',
		match: { cookie: '^bm_sv$' },
		description: 'Akamai Bot Manager detected via bm_sv cookie',
	},
	{
		id: 'akamai-header', name: 'Akamai', category: 'waf', confidence: 'medium',
		match: { header: 'x-akamai-transformed' },
		description: 'Akamai WAF/CDN detected via x-akamai-transformed header',
	},
	{
		id: 'cloudflare-cfray', name: 'Cloudflare', category: 'waf', confidence: 'high',
		match: { header: 'cf-ray' },
		description: 'Cloudflare detected via cf-ray header',
		details: 'May require challenge solving for protected endpoints',
	},
	{
		id: 'cloudflare-server', name: 'Cloudflare', category: 'waf', confidence: 'high',
		match: { header: 'server', headerValue: '^cloudflare$' },
		description: 'Cloudflare detected via Server header',
	},
	{
		id: 'cloudflare-cfbm', name: 'Cloudflare Bot Management', category: 'waf', confidence: 'high',
		match: { cookie: '^__cf_bm$' },
		description: 'Cloudflare Bot Management detected via __cf_bm cookie',
	},
	{
		id: 'cloudflare-clearance', name: 'Cloudflare Challenge', category: 'waf', confidence: 'high',
		match: { cookie: '^cf_clearance$' },
		description: 'Cloudflare JS Challenge detected via cf_clearance cookie',
		details: 'Requires JS challenge solving or Turnstile token',
	},
	{
		id: 'ddosguard-server', name: 'DDoS-Guard', category: 'waf', confidence: 'high',
		match: { header: 'server', headerValue: 'ddos-guard' },
		description: 'DDoS-Guard detected via Server header',
		details: 'May require cookie solving or JS challenge bypass',
	},
	{
		id: 'ddosguard-cookie1', name: 'DDoS-Guard', category: 'waf', confidence: 'high',
		match: { cookie: '^__ddg[0-9]' },
		description: 'DDoS-Guard detected via __ddg cookie',
	},
	{
		id: 'ddosguard-cookie2', name: 'DDoS-Guard', category: 'waf', confidence: 'medium',
		match: { cookie: '^__ddgid_' },
		description: 'DDoS-Guard detected via __ddgid cookie',
	},
	{
		id: 'ddosguard-mark', name: 'DDoS-Guard', category: 'waf', confidence: 'high',
		match: { cookie: '^__ddgmark_' },
		description: 'DDoS-Guard detected via __ddgmark cookie',
	},
	{
		id: 'datadome-cookie', name: 'DataDome', category: 'waf', confidence: 'high',
		match: { cookie: '^datadome$' },
		description: 'DataDome detected via datadome cookie',
		details: 'Requires DataDome interstitial/captcha handling',
	},
	{
		id: 'datadome-header', name: 'DataDome', category: 'waf', confidence: 'high',
		match: { header: 'x-datadome-cid' },
		description: 'DataDome detected via x-datadome-cid header',
	},
	{
		id: 'datadome-server', name: 'DataDome', category: 'waf', confidence: 'high',
		match: { header: 'server', headerValue: 'datadome' },
		description: 'DataDome detected via Server header',
	},
	{
		id: 'incapsula-visid', name: 'Imperva/Incapsula', category: 'waf', confidence: 'high',
		match: { cookie: '^visid_incap_' },
		description: 'Imperva/Incapsula WAF detected via visid_incap cookie',
	},
	{
		id: 'incapsula-ses', name: 'Imperva/Incapsula', category: 'waf', confidence: 'high',
		match: { cookie: '^incap_ses_' },
		description: 'Imperva/Incapsula WAF detected via incap_ses cookie',
	},
	{
		id: 'imperva-cdn', name: 'Imperva/Incapsula', category: 'waf', confidence: 'high',
		match: { header: 'x-cdn', headerValue: 'imperva' },
		description: 'Imperva CDN detected via X-CDN header',
	},
	{
		id: 'imperva-iinfo', name: 'Imperva/Incapsula', category: 'waf', confidence: 'high',
		match: { header: 'x-iinfo' },
		description: 'Imperva/Incapsula detected via X-Iinfo header',
	},
	{
		id: 'perimeterx-cookie', name: 'PerimeterX/HUMAN', category: 'waf', confidence: 'high',
		match: { cookie: '^_px' },
		description: 'PerimeterX (HUMAN) detected via _px cookie',
		details: 'Requires sensor data and challenge solving',
	},
	{
		id: 'perimeterx-header', name: 'PerimeterX/HUMAN', category: 'waf', confidence: 'high',
		match: { header: 'x-px-' },
		description: 'PerimeterX (HUMAN) detected via x-px header',
	},
	{
		id: 'aws-waf', name: 'AWS WAF', category: 'waf', confidence: 'high',
		match: { header: 'x-amzn-waf-' },
		description: 'AWS WAF detected via x-amzn-waf header',
	},
	{
		id: 'aws-alb', name: 'AWS ALB', category: 'waf', confidence: 'medium',
		match: { cookie: '^(awsalb|AWSALB)$' },
		description: 'AWS Application Load Balancer detected via AWSALB cookie',
	},
	{
		id: 'sucuri', name: 'Sucuri WAF', category: 'waf', confidence: 'high',
		match: { header: 'x-sucuri-id' },
		description: 'Sucuri WAF detected via x-sucuri-id header',
	},
	{
		id: 'sucuri-server', name: 'Sucuri WAF', category: 'waf', confidence: 'high',
		match: { header: 'server', headerValue: 'sucuri' },
		description: 'Sucuri WAF detected via Server header',
	},
	{
		id: 'reese84', name: 'Reese84 (Shape/F5)', category: 'waf', confidence: 'high',
		match: { cookie: '^reese84$' },
		description: 'Reese84 (Shape Security / F5) detected via reese84 cookie',
		details: 'Requires sensor data generation',
	},
	{
		id: 'f5-bigip', name: 'F5 BIG-IP', category: 'waf', confidence: 'high',
		match: { cookie: '^BIGipServer' },
		description: 'F5 BIG-IP load balancer detected via BIGipServer cookie',
	},
	{
		id: 'kasada-header', name: 'Kasada', category: 'waf', confidence: 'high',
		match: { header: 'x-kpsdk-' },
		description: 'Kasada bot protection detected via x-kpsdk header',
		details: 'Requires Kasada SDK challenge solving',
	},
	{
		id: 'shape-cookie', name: 'Shape Security', category: 'waf', confidence: 'medium',
		match: { cookie: '^_imp_apg_r_$' },
		description: 'Shape Security detected via _imp_apg_r_ cookie',
	},
	{
		id: 'forcepoint', name: 'Forcepoint', category: 'waf', confidence: 'medium',
		match: { header: 'x-cnection', headerValue: 'close' },
		description: 'Forcepoint WAF detected via x-cnection header',
	},
	{
		id: 'wordfence', name: 'Wordfence', category: 'waf', confidence: 'high',
		match: { cookie: '^wfvt_' },
		description: 'Wordfence (WordPress WAF) detected via wfvt_ cookie',
	},
	{
		id: 'modsecurity', name: 'ModSecurity', category: 'waf', confidence: 'high',
		match: { header: 'server', headerValue: 'mod_security' },
		description: 'ModSecurity WAF detected via Server header',
	},
	{
		id: 'stackpath', name: 'StackPath', category: 'waf', confidence: 'high',
		match: { header: 'x-sp-' },
		description: 'StackPath WAF detected via x-sp header',
	},
	{
		id: 'edgecast', name: 'Edgecast/Verizon', category: 'waf', confidence: 'medium',
		match: { header: 'server', headerValue: 'ecacc' },
		description: 'Edgecast (Verizon Digital Media) detected via Server header',
	},
	{
		id: 'qrator', name: 'Qrator', category: 'waf', confidence: 'high',
		match: { header: 'x-qrator-' },
		description: 'Qrator WAF detected via x-qrator header',
	},
	{
		id: 'stormwall', name: 'StormWall', category: 'waf', confidence: 'high',
		match: { cookie: '^swp_token' },
		description: 'StormWall DDoS protection detected via swp_token cookie',
	},

	// ── CDN ──────────────────────────────────────────────────────────
	{
		id: 'cloudfront-id', name: 'CloudFront', category: 'cdn', confidence: 'high',
		match: { header: 'x-amz-cf-id' },
		description: 'Amazon CloudFront CDN detected via x-amz-cf-id header',
	},
	{
		id: 'cloudfront-pop', name: 'CloudFront', category: 'cdn', confidence: 'high',
		match: { header: 'x-amz-cf-pop' },
		description: 'Amazon CloudFront CDN detected via x-amz-cf-pop header',
	},
	{
		id: 'fastly-served', name: 'Fastly', category: 'cdn', confidence: 'high',
		match: { header: 'x-served-by' },
		description: 'Fastly CDN detected via x-served-by header',
	},
	{
		id: 'fastly-timer', name: 'Fastly', category: 'cdn', confidence: 'medium',
		match: { header: 'x-timer' },
		description: 'Fastly CDN detected via x-timer header',
	},
	{
		id: 'cloudflare-cache', name: 'Cloudflare CDN', category: 'cdn', confidence: 'high',
		match: { header: 'cf-cache-status' },
		description: 'Cloudflare CDN detected via cf-cache-status header',
	},
	{
		id: 'varnish-via', name: 'Varnish', category: 'cdn', confidence: 'medium',
		match: { header: 'via', headerValue: 'varnish' },
		description: 'Varnish cache detected via Via header',
	},
	{
		id: 'varnish-header', name: 'Varnish', category: 'cdn', confidence: 'high',
		match: { header: 'x-varnish' },
		description: 'Varnish cache detected via x-varnish header',
	},
	{
		id: 'keycdn', name: 'KeyCDN', category: 'cdn', confidence: 'high',
		match: { header: 'x-edge-location' },
		description: 'KeyCDN detected via x-edge-location header',
	},
	{
		id: 'bunnycdn', name: 'BunnyCDN', category: 'cdn', confidence: 'high',
		match: { header: 'cdn-pullzone' },
		description: 'BunnyCDN detected via CDN-PullZone header',
	},

	// ── Server ───────────────────────────────────────────────────────
	{
		id: 'server-nginx', name: 'nginx', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: '^nginx' },
		description: 'nginx web server detected',
	},
	{
		id: 'server-apache', name: 'Apache', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: '^apache' },
		description: 'Apache HTTP Server detected',
	},
	{
		id: 'server-iis', name: 'Microsoft IIS', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: 'microsoft-iis' },
		description: 'Microsoft IIS web server detected',
	},
	{
		id: 'server-litespeed', name: 'LiteSpeed', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: 'litespeed' },
		description: 'LiteSpeed web server detected',
	},
	{
		id: 'server-caddy', name: 'Caddy', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: '^caddy' },
		description: 'Caddy web server detected',
	},
	{
		id: 'server-kestrel', name: 'Kestrel', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: 'kestrel' },
		description: 'ASP.NET Kestrel web server detected',
	},
	{
		id: 'server-cowboy', name: 'Cowboy (Erlang)', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: '^cowboy' },
		description: 'Cowboy (Erlang/Elixir) web server detected',
	},
	{
		id: 'server-gunicorn', name: 'Gunicorn', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: 'gunicorn' },
		description: 'Gunicorn (Python) WSGI server detected',
	},
	{
		id: 'server-openresty', name: 'OpenResty', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: 'openresty' },
		description: 'OpenResty (nginx + Lua) web server detected',
	},
	{
		id: 'server-envoy', name: 'Envoy', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: '^envoy' },
		description: 'Envoy proxy detected',
	},
	{
		id: 'server-tengine', name: 'Tengine', category: 'server', confidence: 'high',
		match: { header: 'server', headerValue: 'tengine' },
		description: 'Tengine (Alibaba nginx fork) web server detected',
	},

	// ── Framework ────────────────────────────────────────────────────
	{
		id: 'aspnet-powered', name: 'ASP.NET', category: 'framework', confidence: 'high',
		match: { header: 'x-powered-by', headerValue: 'asp\\.net' },
		description: 'ASP.NET framework detected via X-Powered-By header',
	},
	{
		id: 'aspnet-version', name: 'ASP.NET', category: 'framework', confidence: 'high',
		match: { header: 'x-aspnet-version' },
		description: 'ASP.NET version detected via X-AspNet-Version header',
	},
	{
		id: 'aspnet-session', name: 'ASP.NET', category: 'framework', confidence: 'high',
		match: { cookie: '^ASP\\.NET_SessionId$' },
		description: 'ASP.NET detected via ASP.NET_SessionId cookie',
	},
	{
		id: 'aspnet-affinity', name: 'Azure App Service', category: 'framework', confidence: 'medium',
		match: { cookie: '^ARRAffinity' },
		description: 'Azure App Service detected via ARRAffinity cookie',
	},
	{
		id: 'php-powered', name: 'PHP', category: 'framework', confidence: 'high',
		match: { header: 'x-powered-by', headerValue: 'php' },
		description: 'PHP detected via X-Powered-By header',
	},
	{
		id: 'php-session', name: 'PHP', category: 'framework', confidence: 'high',
		match: { cookie: '^PHPSESSID$' },
		description: 'PHP detected via PHPSESSID cookie',
	},
	{
		id: 'express-powered', name: 'Express.js', category: 'framework', confidence: 'high',
		match: { header: 'x-powered-by', headerValue: 'express' },
		description: 'Express.js (Node.js) detected via X-Powered-By header',
	},
	{
		id: 'express-session', name: 'Express/Connect', category: 'framework', confidence: 'medium',
		match: { cookie: '^connect\\.sid$' },
		description: 'Express/Connect session detected via connect.sid cookie',
	},
	{
		id: 'rails-runtime', name: 'Ruby on Rails', category: 'framework', confidence: 'high',
		match: { header: 'x-runtime' },
		description: 'Ruby on Rails detected via X-Runtime header',
	},
	{
		id: 'django-csrf', name: 'Django', category: 'framework', confidence: 'high',
		match: { cookie: '^csrftoken$' },
		description: 'Django detected via csrftoken cookie',
	},
	{
		id: 'spring-jsessionid', name: 'Java/Spring', category: 'framework', confidence: 'medium',
		match: { cookie: '^JSESSIONID$' },
		description: 'Java application server detected via JSESSIONID cookie',
	},
	{
		id: 'spring-context', name: 'Spring Framework', category: 'framework', confidence: 'high',
		match: { header: 'x-application-context' },
		description: 'Spring Framework detected via X-Application-Context header',
	},
	{
		id: 'laravel-session', name: 'Laravel', category: 'framework', confidence: 'high',
		match: { cookie: '^laravel_session$' },
		description: 'Laravel (PHP) detected via laravel_session cookie',
	},
	{
		id: 'laravel-xsrf', name: 'Laravel', category: 'framework', confidence: 'medium',
		match: { cookie: '^XSRF-TOKEN$' },
		description: 'Laravel or Angular detected via XSRF-TOKEN cookie',
	},
	{
		id: 'nextjs-header', name: 'Next.js', category: 'framework', confidence: 'high',
		match: { header: 'x-nextjs-' },
		description: 'Next.js detected via x-nextjs header',
	},
	{
		id: 'nextjs-cookie', name: 'Next.js', category: 'framework', confidence: 'medium',
		match: { cookie: '^__next' },
		description: 'Next.js detected via __next cookie',
	},

	// ── Bot Protection (body patterns) ───────────────────────────────
	{
		id: 'challenge-page', name: 'JS Challenge Page', category: 'bot_protection', confidence: 'medium',
		match: { statusCode: 403, bodyPattern: 'challenge-platform|please enable javascript|checking your browser' },
		description: 'JavaScript challenge page detected (403 with challenge pattern)',
	},
	{
		id: 'ratelimit-page', name: 'Rate Limiting', category: 'bot_protection', confidence: 'medium',
		match: { statusCode: 429 },
		description: 'Rate limiting detected (HTTP 429)',
	},
	{
		id: 'captcha-page', name: 'CAPTCHA Page', category: 'bot_protection', confidence: 'medium',
		match: { bodyPattern: 'recaptcha|hcaptcha|funcaptcha|arkose|captcha-delivery' },
		description: 'CAPTCHA challenge detected in response body',
	},

	// ── Security Headers ─────────────────────────────────────────────
	{
		id: 'sec-hsts', name: 'HSTS', category: 'security', confidence: 'high',
		match: { header: 'strict-transport-security' },
		description: 'HTTP Strict Transport Security is enabled',
	},
	{
		id: 'sec-csp', name: 'CSP', category: 'security', confidence: 'high',
		match: { header: 'content-security-policy' },
		description: 'Content Security Policy is enabled',
	},
	{
		id: 'sec-xfo', name: 'X-Frame-Options', category: 'security', confidence: 'high',
		match: { header: 'x-frame-options' },
		description: 'X-Frame-Options header is set',
	},
	{
		id: 'sec-xcto', name: 'X-Content-Type-Options', category: 'security', confidence: 'high',
		match: { header: 'x-content-type-options' },
		description: 'X-Content-Type-Options header is set',
	},
	{
		id: 'sec-xxss', name: 'X-XSS-Protection', category: 'security', confidence: 'high',
		match: { header: 'x-xss-protection' },
		description: 'X-XSS-Protection header is set',
	},
	{
		id: 'sec-referrer', name: 'Referrer-Policy', category: 'security', confidence: 'high',
		match: { header: 'referrer-policy' },
		description: 'Referrer-Policy header is set',
	},
	{
		id: 'sec-permissions', name: 'Permissions-Policy', category: 'security', confidence: 'high',
		match: { header: 'permissions-policy' },
		description: 'Permissions-Policy header is set',
	},
	{
		id: 'sec-cors', name: 'CORS', category: 'security', confidence: 'high',
		match: { header: 'access-control-allow-origin' },
		description: 'CORS headers are present',
	},
];

// ─── Security headers to check for presence ─────────────────────────

const SECURITY_HEADERS = [
	'strict-transport-security',
	'content-security-policy',
	'x-frame-options',
	'x-content-type-options',
	'x-xss-protection',
	'referrer-policy',
	'permissions-policy',
	'access-control-allow-origin',
];

// ─── Engine ──────────────────────────────────────────────────────────

function matchesRule(rule: FingerprintRule, resp: ResponseInfo): string[] {
	const evidence: string[] = [];
	const m = rule.match;

	// Check cookie name pattern
	if (m.cookie) {
		const re = new RegExp(m.cookie, 'i');
		for (const cookieName of Object.keys(resp.cookies)) {
			if (re.test(cookieName)) {
				evidence.push(`Cookie: ${cookieName}=${resp.cookies[cookieName].slice(0, 40)}${resp.cookies[cookieName].length > 40 ? '...' : ''}`);
			}
		}
	}

	// Check header presence and optionally value
	if (m.header) {
		const headerLower = m.header.toLowerCase();
		for (const [name, value] of Object.entries(resp.headers)) {
			const nameLower = name.toLowerCase();
			// Support prefix matching (e.g. "x-px-" matches "x-px-something")
			if (nameLower === headerLower || (headerLower.endsWith('-') && nameLower.startsWith(headerLower))) {
				if (m.headerValue) {
					const re = new RegExp(m.headerValue, 'i');
					if (re.test(value)) {
						evidence.push(`Header: ${name}: ${value}`);
					}
				} else {
					evidence.push(`Header: ${name}: ${value}`);
				}
			}
		}
	}

	// Check body pattern
	if (m.bodyPattern && resp.body) {
		const re = new RegExp(m.bodyPattern, 'i');
		if (re.test(resp.body)) {
			evidence.push(`Body pattern: ${m.bodyPattern}`);
		}
	}

	// Check status code
	if (m.statusCode !== undefined) {
		if (resp.status_code === m.statusCode) {
			if (evidence.length > 0 || (!m.cookie && !m.header && !m.bodyPattern)) {
				evidence.push(`Status: ${resp.status_code}`);
			}
		} else if (!m.cookie && !m.header && !m.bodyPattern) {
			return [];
		}
	}

	return evidence;
}

export function fingerprint(responses: ResponseInfo[]): FingerprintResult {
	const matchMap = new Map<string, FingerprintMatch>();

	// Merge all headers/cookies across responses for raw evidence
	const allHeaders: Record<string, string> = {};
	const allCookies: Record<string, string> = {};

	for (const resp of responses) {
		for (const [k, v] of Object.entries(resp.headers)) allHeaders[k] = v;
		for (const [k, v] of Object.entries(resp.cookies)) allCookies[k] = v;
	}

	for (const rule of rules) {
		for (const resp of responses) {
			const evidence = matchesRule(rule, resp);
			if (evidence.length > 0) {
				const key = `${rule.category}:${rule.name}`;
				const existing = matchMap.get(key);
				if (existing) {
					for (const e of evidence) {
						if (!existing.evidence.includes(e)) existing.evidence.push(e);
					}
					if (rule.confidence === 'high') existing.rule = { ...existing.rule, confidence: 'high' };
				} else {
					matchMap.set(key, { rule, evidence: [...evidence] });
				}
			}
		}
	}

	// Auto-detect Server header value even if no specific rule matched
	const serverHeader = Object.entries(allHeaders).find(([k]) => k.toLowerCase() === 'server');
	if (serverHeader) {
		const [hdrName, hdrValue] = serverHeader;
		const hasServerMatch = Array.from(matchMap.values()).some(
			m => m.rule.category === 'server' || m.evidence.some(e => e.toLowerCase().includes('header: server:'))
		);
		if (!hasServerMatch) {
			// No rule matched the Server header — add it as a generic detection
			matchMap.set('server:' + hdrValue, {
				rule: {
					id: 'server-auto', name: hdrValue, category: 'server', confidence: 'high',
					match: {}, description: `Server: ${hdrValue}`,
				},
				evidence: [`Header: ${hdrName}: ${hdrValue}`],
			});
		}
	}

	// Auto-detect X-Powered-By if no framework rule matched it
	const poweredBy = Object.entries(allHeaders).find(([k]) => k.toLowerCase() === 'x-powered-by');
	if (poweredBy) {
		const [hdrName, hdrValue] = poweredBy;
		const hasFrameworkMatch = Array.from(matchMap.values()).some(
			m => m.rule.category === 'framework' && m.evidence.some(e => e.toLowerCase().includes('x-powered-by'))
		);
		if (!hasFrameworkMatch) {
			matchMap.set('framework:' + hdrValue, {
				rule: {
					id: 'framework-auto', name: hdrValue, category: 'framework', confidence: 'high',
					match: {}, description: `X-Powered-By: ${hdrValue}`,
				},
				evidence: [`Header: ${hdrName}: ${hdrValue}`],
			});
		}
	}

	// Build security headers summary
	const headersLower = new Map<string, string>();
	for (const [k, v] of Object.entries(allHeaders)) {
		headersLower.set(k.toLowerCase(), v);
	}

	const securityHeaders = SECURITY_HEADERS.map(name => ({
		name,
		present: headersLower.has(name),
		value: headersLower.get(name),
	}));

	// Sort matches
	const categoryOrder: Record<string, number> = {
		waf: 0, bot_protection: 1, cdn: 2, server: 3, framework: 4, session: 5, security: 6,
	};
	const confidenceOrder: Record<string, number> = { high: 0, medium: 1, low: 2 };

	const matches = Array.from(matchMap.values()).sort((a, b) => {
		const catDiff = (categoryOrder[a.rule.category] ?? 9) - (categoryOrder[b.rule.category] ?? 9);
		if (catDiff !== 0) return catDiff;
		return (confidenceOrder[a.rule.confidence] ?? 9) - (confidenceOrder[b.rule.confidence] ?? 9);
	});

	// Build top-level stack summary (unique names from non-security matches)
	const seen = new Set<string>();
	const stack: StackEntry[] = [];
	for (const m of matches) {
		if (m.rule.category === 'security') continue;
		const name = m.rule.name;
		if (seen.has(name)) continue;
		seen.add(name);
		stack.push({ name, category: m.rule.category, confidence: m.rule.confidence });
	}

	return {
		stack,
		matches,
		securityHeaders,
		raw: { headers: allHeaders, cookies: allCookies },
	};
}

// Group matches by display category
export function groupMatches(matches: FingerprintMatch[]): { label: string; matches: FingerprintMatch[] }[] {
	const groups: { label: string; categories: string[]; matches: FingerprintMatch[] }[] = [
		{ label: 'WAF / Bot Protection', categories: ['waf', 'bot_protection'], matches: [] },
		{ label: 'Server & Infrastructure', categories: ['cdn', 'server'], matches: [] },
		{ label: 'Framework & Language', categories: ['framework', 'session'], matches: [] },
	];

	for (const m of matches) {
		if (m.rule.category === 'security') continue;
		const group = groups.find(g => g.categories.includes(m.rule.category));
		if (group) group.matches.push(m);
	}

	return groups.filter(g => g.matches.length > 0).map(g => ({ label: g.label, matches: g.matches }));
}
