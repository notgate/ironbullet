// Site Fingerprint / WAF Detection Engine
// Runs entirely in the frontend — matches HTTP response headers, cookies, body patterns

export interface FingerprintRule {
	id: string;
	name: string;
	category: 'waf' | 'cdn' | 'server' | 'framework' | 'cms' | 'analytics' | 'captcha' | 'antibot' | 'identity' | 'hosting';
	confidence: 'high' | 'medium' | 'low';
	match: {
		cookie?: string;
		header?: string;
		headerValue?: string;
		bodyPattern?: string;
		statusCode?: number;
	};
	description: string;
	details?: string;
	bypassHint?: string;
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
	category: string;
	confidence: 'high' | 'medium' | 'low';
}

export interface CookieAnalysis {
	name: string;
	value: string;
	provider: string;
	purpose: string;
	category: 'antibot' | 'session' | 'analytics' | 'tracking' | 'functional' | 'unknown';
	risk: 'critical' | 'high' | 'medium' | 'low';
	bypassRequired: boolean;
	details: string;
}

export interface FingerprintResult {
	stack: StackEntry[];
	matches: FingerprintMatch[];
	securityHeaders: { name: string; present: boolean; value?: string }[];
	cookieAnalysis: CookieAnalysis[];
	raw: { headers: Record<string, string>; cookies: Record<string, string> };
}

// ─── Detection Rules ─────────────────────────────────────────────────

const rules: FingerprintRule[] = [
	// ── Akamai ──
	{ id: 'akamai-abck', name: 'Akamai Bot Manager', category: 'antibot', confidence: 'high', match: { cookie: '^_abck$' }, description: 'Akamai Bot Manager via _abck cookie', bypassHint: 'Requires sensor data header (akamai-sensor-data) with valid payload from real browser events', details: 'Collects 100+ mouse/keyboard/device signals. One of the hardest systems to bypass.' },
	{ id: 'akamai-akbmsc', name: 'Akamai Bot Manager', category: 'antibot', confidence: 'high', match: { cookie: '^ak_bmsc$' }, description: 'Akamai Bot Manager via ak_bmsc cookie' },
	{ id: 'akamai-bmsz', name: 'Akamai Bot Manager', category: 'antibot', confidence: 'medium', match: { cookie: '^bm_sz$' }, description: 'Akamai BM via bm_sz (session fingerprint)' },
	{ id: 'akamai-bmsv', name: 'Akamai Bot Manager', category: 'antibot', confidence: 'medium', match: { cookie: '^bm_sv$' }, description: 'Akamai BM via bm_sv (score/validation)' },
	{ id: 'akamai-header', name: 'Akamai CDN', category: 'cdn', confidence: 'medium', match: { header: 'x-akamai-transformed' }, description: 'Akamai CDN via x-akamai-transformed header' },
	{ id: 'akamai-edge', name: 'Akamai CDN', category: 'cdn', confidence: 'high', match: { header: 'x-check-cacheable' }, description: 'Akamai Edge Cache indicator' },
	// ── Cloudflare ──
	{ id: 'cf-ray', name: 'Cloudflare', category: 'cdn', confidence: 'high', match: { header: 'cf-ray' }, description: 'Cloudflare via cf-ray request ID', bypassHint: 'Use cf_clearance cookie or residential/ISP proxies to avoid blocks' },
	{ id: 'cf-server', name: 'Cloudflare', category: 'cdn', confidence: 'high', match: { header: 'server', headerValue: '^cloudflare$' }, description: 'Cloudflare via Server header' },
	{ id: 'cf-bm', name: 'Cloudflare Bot Management', category: 'antibot', confidence: 'high', match: { cookie: '^__cf_bm$' }, description: 'Cloudflare Bot Management via __cf_bm cookie', bypassHint: 'Requires valid __cf_bm token — use residential IPs or headless browser', details: 'Cloudflare Bot Management ML scoring token. Short-lived, fingerprint-tied.' },
	{ id: 'cf-clearance', name: 'Cloudflare JS Challenge', category: 'antibot', confidence: 'high', match: { cookie: '^cf_clearance$' }, description: 'Cloudflare JS Challenge via cf_clearance cookie', bypassHint: 'Solve Cloudflare JS/Turnstile challenge with headless browser or service' },
	{ id: 'cf-turnstile', name: 'Cloudflare Turnstile', category: 'captcha', confidence: 'high', match: { bodyPattern: 'challenges\\.cloudflare\\.com/turnstile' }, description: 'Cloudflare Turnstile CAPTCHA in page body' },
	{ id: 'cf-mitigated', name: 'Cloudflare (Mitigated)', category: 'waf', confidence: 'high', match: { header: 'cf-mitigated' }, description: 'Cloudflare active mitigation (WAF block)' },
	// ── DataDome ──
	{ id: 'datadome-cookie', name: 'DataDome', category: 'antibot', confidence: 'high', match: { cookie: '^datadome$' }, description: 'DataDome via datadome cookie', bypassHint: 'Requires DataDome interstitial solver — headless browser or 2Captcha/CapMonster', details: 'ML-based behavioral bot detection. Analyzes TLS, HTTP/2 fingerprints, mouse patterns. Extremely difficult.' },
	{ id: 'datadome-cid', name: 'DataDome', category: 'antibot', confidence: 'high', match: { header: 'x-datadome-cid' }, description: 'DataDome via x-datadome-cid response header' },
	{ id: 'datadome-server', name: 'DataDome', category: 'antibot', confidence: 'high', match: { header: 'server', headerValue: 'datadome' }, description: 'DataDome via Server header' },
	{ id: 'datadome-body', name: 'DataDome Captcha Page', category: 'antibot', confidence: 'high', match: { bodyPattern: 'datadome\\.co/captcha|dd-cookie' }, description: 'DataDome interstitial captcha page detected' },
	// ── DDoS-Guard ──
	{ id: 'ddosguard-server', name: 'DDoS-Guard', category: 'waf', confidence: 'high', match: { header: 'server', headerValue: 'ddos-guard' }, description: 'DDoS-Guard WAF via Server header', bypassHint: 'Solve JS challenge to obtain __ddg cookie series' },
	{ id: 'ddosguard-ddg', name: 'DDoS-Guard', category: 'waf', confidence: 'high', match: { cookie: '^__ddg[0-9]' }, description: 'DDoS-Guard via __ddg# cookie' },
	{ id: 'ddosguard-ddgid', name: 'DDoS-Guard', category: 'waf', confidence: 'medium', match: { cookie: '^__ddgid_' }, description: 'DDoS-Guard persistent device ID cookie' },
	{ id: 'ddosguard-mark', name: 'DDoS-Guard', category: 'waf', confidence: 'high', match: { cookie: '^__ddgmark_' }, description: 'DDoS-Guard marking cookie' },
	// ── Imperva / Incapsula ──
	{ id: 'incapsula-visid', name: 'Imperva/Incapsula', category: 'waf', confidence: 'high', match: { cookie: '^visid_incap_' }, description: 'Imperva WAF visitor ID cookie', bypassHint: 'Requires valid referrer chain + valid incap_ses cookie pair' },
	{ id: 'incapsula-ses', name: 'Imperva/Incapsula', category: 'waf', confidence: 'high', match: { cookie: '^incap_ses_' }, description: 'Imperva WAF session cookie' },
	{ id: 'imperva-cdn', name: 'Imperva CDN', category: 'cdn', confidence: 'high', match: { header: 'x-cdn', headerValue: 'imperva' }, description: 'Imperva CDN via X-CDN header' },
	{ id: 'imperva-iinfo', name: 'Imperva/Incapsula', category: 'waf', confidence: 'high', match: { header: 'x-iinfo' }, description: 'Imperva WAF request info header' },
	// ── PerimeterX / HUMAN ──
	{ id: 'px-cookie', name: 'PerimeterX/HUMAN', category: 'antibot', confidence: 'high', match: { cookie: '^_px[0-9a-z]' }, description: 'PerimeterX/HUMAN Security via _px cookie', bypassHint: 'Requires PX cookie generation via JS execution or solver service', details: 'HUMAN Security (formerly PerimeterX). Tracks 200+ behavioral + browser signals. Token expires quickly.' },
	{ id: 'px-pxhd', name: 'PerimeterX/HUMAN', category: 'antibot', confidence: 'high', match: { cookie: '^_pxhd$' }, description: 'PerimeterX hardware fingerprint cookie' },
	{ id: 'px-pxvid', name: 'PerimeterX/HUMAN', category: 'antibot', confidence: 'high', match: { cookie: '^_pxvid$' }, description: 'PerimeterX visitor ID cookie' },
	{ id: 'px-header', name: 'PerimeterX/HUMAN', category: 'antibot', confidence: 'high', match: { header: 'x-px-cookies' }, description: 'PerimeterX response cookie injection header' },
	{ id: 'px-body', name: 'PerimeterX/HUMAN Block Page', category: 'antibot', confidence: 'high', match: { bodyPattern: 'px-captcha|PerimeterX|perimeterx\\.com' }, description: 'PerimeterX block/captcha page in body' },
	// ── Kasada ──
	{ id: 'kasada-cookie', name: 'Kasada', category: 'antibot', confidence: 'high', match: { cookie: '^kpsdk-' }, description: 'Kasada SDK cookie', bypassHint: 'Requires Kasada SDK bypass — TLS fingerprint + JS POW challenge', details: 'SDK-level protection with TLS JA3 fingerprinting and JavaScript proof-of-work.' },
	{ id: 'kasada-header', name: 'Kasada', category: 'antibot', confidence: 'high', match: { header: 'x-kpsdk-ct' }, description: 'Kasada challenge token response header' },
	{ id: 'kasada-body', name: 'Kasada Protection Page', category: 'antibot', confidence: 'high', match: { bodyPattern: 'kpsdk|kasada\\.io' }, description: 'Kasada protection page detected in body' },
	// ── F5 BIG-IP / Shape Security ──
	{ id: 'f5-bigip-server', name: 'F5 BIG-IP', category: 'waf', confidence: 'high', match: { header: 'server', headerValue: 'BigIP|BIG-IP' }, description: 'F5 BIG-IP load balancer detected' },
	{ id: 'f5-ts-cookie', name: 'F5 BIG-IP / Shape Security', category: 'antibot', confidence: 'high', match: { cookie: '^TS[0-9a-f]{8}' }, description: 'F5 BIG-IP persistence / Shape Security cookie', bypassHint: 'Shape Security tracks TLS JA3 fingerprint — requires TLS mimicry or residential IPs', details: 'TS* cookie used for both BIG-IP persistence and Shape Security (F5 bot defense) tracking.' },
	// ── AWS WAF ──
	{ id: 'awswaf-token', name: 'AWS WAF', category: 'waf', confidence: 'high', match: { cookie: '^aws-waf-token$' }, description: 'AWS WAF JavaScript challenge token', bypassHint: 'Requires AWS WAF token generation (JavaScript challenge must be solved)' },
	{ id: 'awswaf-header', name: 'AWS WAF', category: 'waf', confidence: 'high', match: { header: 'x-amzn-waf-action' }, description: 'AWS WAF action response header' },
	{ id: 'cloudfront-cfid', name: 'AWS CloudFront', category: 'cdn', confidence: 'high', match: { header: 'x-amz-cf-id' }, description: 'AWS CloudFront request ID' },
	{ id: 'cloudfront-pop', name: 'AWS CloudFront', category: 'cdn', confidence: 'high', match: { header: 'x-amz-cf-pop' }, description: 'AWS CloudFront Point of Presence header' },
	// ── CAPTCHA systems ──
	{ id: 'recaptcha-body', name: 'Google reCAPTCHA', category: 'captcha', confidence: 'high', match: { bodyPattern: 'google\\.com/recaptcha|grecaptcha' }, description: 'reCAPTCHA v2/v3 embedded in page', bypassHint: 'Use 2Captcha, CapMonster, or Anti-Captcha solver APIs' },
	{ id: 'hcaptcha-body', name: 'hCaptcha', category: 'captcha', confidence: 'high', match: { bodyPattern: 'hcaptcha\\.com|data-hcaptcha-sitekey' }, description: 'hCaptcha embedded in page', bypassHint: 'Use hCaptcha solver (2Captcha, Nopecha, CapMonster)' },
	{ id: 'geetest-body', name: 'GeeTest', category: 'captcha', confidence: 'high', match: { bodyPattern: 'geetest\\.com|initGeetest|gt\\.js' }, description: 'GeeTest CAPTCHA (slide/click challenge)', bypassHint: 'Requires GeeTest solver — slide completion with correct trajectory data' },
	{ id: 'arkose-body', name: 'Arkose Labs / FunCaptcha', category: 'captcha', confidence: 'high', match: { bodyPattern: 'arkoselabs\\.com|funcaptcha|fc\\.js' }, description: 'Arkose FunCaptcha in page', bypassHint: 'Requires FunCaptcha solver (Arkose token)' },
	// ── Dynatrace RUM ──
	{ id: 'dynatrace-dtcookie', name: 'Dynatrace RUM', category: 'analytics', confidence: 'high', match: { cookie: '^dtCookie$' }, description: 'Dynatrace Real User Monitoring session cookie', details: 'Performance monitoring only — not a security/antibot control.' },
	{ id: 'dynatrace-rxvt', name: 'Dynatrace RUM', category: 'analytics', confidence: 'high', match: { cookie: '^rxvt$' }, description: 'Dynatrace RUM session expiry tracking' },
	{ id: 'dynatrace-dtsa', name: 'Dynatrace RUM', category: 'analytics', confidence: 'medium', match: { cookie: '^dtSa$' }, description: 'Dynatrace sampling decision cookie' },
	{ id: 'dynatrace-dtlatc', name: 'Dynatrace RUM', category: 'analytics', confidence: 'medium', match: { cookie: '^dtLatC$' }, description: 'Dynatrace latency measurement cookie' },
	{ id: 'dynatrace-header', name: 'Dynatrace', category: 'analytics', confidence: 'high', match: { header: 'x-dynatrace' }, description: 'Dynatrace tracing header in response' },
	// ── Forter ──
	{ id: 'forter-cookie', name: 'Forter (Fraud Prevention)', category: 'antibot', confidence: 'high', match: { cookie: '^forterToken$' }, description: 'Forter fraud/device fingerprint via forterToken cookie', bypassHint: 'Requires valid Forter token — headless browser with proper device signals', details: 'Forter device intelligence for e-commerce fraud prevention.' },
	// ── ThreatMetrix / LexisNexis ──
	{ id: 'threatmetrix-cookie', name: 'ThreatMetrix (LexisNexis)', category: 'antibot', confidence: 'high', match: { cookie: '^__tmx' }, description: 'ThreatMetrix device ID/fingerprint cookie', details: 'Used heavily in financial services for fraud risk scoring.' },
	// ── Sucuri ──
	{ id: 'sucuri-id', name: 'Sucuri WAF', category: 'waf', confidence: 'high', match: { header: 'x-sucuri-id' }, description: 'Sucuri WAF via x-sucuri-id header' },
	{ id: 'sucuri-cache', name: 'Sucuri WAF', category: 'waf', confidence: 'high', match: { header: 'x-sucuri-cache' }, description: 'Sucuri WAF cache indicator header' },
	// ── Google ──
	{ id: 'google-socs', name: 'Google Cookie Consent', category: 'identity', confidence: 'high', match: { cookie: '^SOCS$' }, description: 'Google cookie consent choice (SOCS)' },
	{ id: 'google-aec', name: 'Google Anti-Abuse', category: 'antibot', confidence: 'high', match: { cookie: '^AEC$' }, description: 'Google AEC anti-abuse cookie', bypassHint: 'Required for Google account actions — difficult to spoof' },
	{ id: 'google-nid', name: 'Google NID', category: 'tracking', confidence: 'high', match: { cookie: '^NID$' }, description: 'Google NID preference/tracking cookie' },
	// ── Fastly ──
	{ id: 'fastly-served', name: 'Fastly CDN', category: 'cdn', confidence: 'high', match: { header: 'x-served-by', headerValue: 'cache-' }, description: 'Fastly CDN via x-served-by cache header' },
	{ id: 'fastly-surrogate', name: 'Fastly CDN', category: 'cdn', confidence: 'medium', match: { header: 'surrogate-key' }, description: 'Fastly CDN via surrogate-key purging header' },
	{ id: 'fastly-cache', name: 'Fastly CDN', category: 'cdn', confidence: 'medium', match: { header: 'x-cache', headerValue: 'HIT|MISS' }, description: 'Fastly cache hit/miss indicator' },
	// ── Varnish ──
	{ id: 'varnish', name: 'Varnish Cache', category: 'cdn', confidence: 'high', match: { header: 'x-varnish' }, description: 'Varnish HTTP Cache accelerator' },
	// ── nginx / Apache / IIS ──
	{ id: 'nginx', name: 'nginx', category: 'server', confidence: 'high', match: { header: 'server', headerValue: '^nginx' }, description: 'nginx web server' },
	{ id: 'apache', name: 'Apache', category: 'server', confidence: 'high', match: { header: 'server', headerValue: '^Apache' }, description: 'Apache HTTP Server' },
	{ id: 'iis', name: 'Microsoft IIS', category: 'server', confidence: 'high', match: { header: 'server', headerValue: '^Microsoft-IIS' }, description: 'Microsoft IIS — commonly used with ASP.NET' },
	{ id: 'litespeed', name: 'LiteSpeed', category: 'server', confidence: 'high', match: { header: 'server', headerValue: '^LiteSpeed' }, description: 'LiteSpeed web server' },
	{ id: 'caddy', name: 'Caddy', category: 'server', confidence: 'high', match: { header: 'server', headerValue: '^Caddy' }, description: 'Caddy web server' },
	{ id: 'openresty', name: 'OpenResty (nginx+Lua)', category: 'server', confidence: 'high', match: { header: 'server', headerValue: '^openresty' }, description: 'OpenResty (nginx + Lua scripting)' },
	// ── Framework / Language ──
	{ id: 'php', name: 'PHP', category: 'framework', confidence: 'high', match: { header: 'x-powered-by', headerValue: '^PHP' }, description: 'PHP runtime via X-Powered-By header' },
	{ id: 'aspnet', name: 'ASP.NET', category: 'framework', confidence: 'high', match: { header: 'x-powered-by', headerValue: 'ASP\\.NET' }, description: 'ASP.NET via X-Powered-By header' },
	{ id: 'aspnet-version', name: 'ASP.NET MVC', category: 'framework', confidence: 'high', match: { header: 'x-aspnet-version' }, description: 'ASP.NET MVC version exposed via header' },
	{ id: 'aspnetcore', name: 'ASP.NET Core', category: 'framework', confidence: 'high', match: { header: 'x-powered-by', headerValue: 'ASP\\.NET Core' }, description: 'ASP.NET Core via X-Powered-By' },
	{ id: 'express', name: 'Express.js', category: 'framework', confidence: 'medium', match: { header: 'x-powered-by', headerValue: '^Express' }, description: 'Express.js Node.js framework' },
	// ── CMS ──
	{ id: 'wordpress', name: 'WordPress', category: 'cms', confidence: 'high', match: { cookie: '^wordpress_' }, description: 'WordPress CMS via cookie prefix' },
	{ id: 'wordpress-logged', name: 'WordPress (Logged In)', category: 'cms', confidence: 'high', match: { cookie: '^wordpress_logged_in' }, description: 'WordPress authenticated session cookie' },
	{ id: 'joomla', name: 'Joomla', category: 'cms', confidence: 'high', match: { cookie: '^joomla_' }, description: 'Joomla CMS via cookie prefix' },
	// ── Shopify ──
	{ id: 'shopify-cookie', name: 'Shopify', category: 'hosting', confidence: 'high', match: { cookie: '^_shopify_' }, description: 'Shopify e-commerce platform cookie', details: 'Shopify uses Cloudflare by default. May also have Shopify-specific bot detection.' },
	{ id: 'shopify-header', name: 'Shopify', category: 'hosting', confidence: 'high', match: { header: 'x-shopify-stage' }, description: 'Shopify via x-shopify-stage header' },
	// ── Analytics ──
	{ id: 'ga4', name: 'Google Analytics 4', category: 'analytics', confidence: 'high', match: { cookie: '^_ga' }, description: 'Google Analytics 4 / Universal Analytics' },
	{ id: 'gtm', name: 'Google Ads/GTM', category: 'analytics', confidence: 'medium', match: { cookie: '^_gcl_' }, description: 'Google Ads / Tag Manager conversion cookie' },
	{ id: 'meta-pixel', name: 'Meta Pixel', category: 'analytics', confidence: 'high', match: { cookie: '^_fbp$' }, description: 'Meta (Facebook) Pixel browser fingerprint' },
	// ── BunnyCDN ──
	{ id: 'bunnycdn', name: 'BunnyCDN', category: 'cdn', confidence: 'high', match: { header: 'bunnycdn-cache-status' }, description: 'BunnyCDN via cache status header' },
];

// ─── Cookie Classifier ────────────────────────────────────────────────

const COOKIE_PATTERNS: Array<{
	pattern: RegExp;
	provider: string;
	purpose: string;
	category: CookieAnalysis['category'];
	risk: CookieAnalysis['risk'];
	bypassRequired: boolean;
	details: string;
}> = [
	{ pattern: /^_abck$|^ak_bmsc$|^bm_sz$|^bm_sv$/, provider: 'Akamai Bot Manager', purpose: 'Bot detection — sensor/behavioral collection', category: 'antibot', risk: 'critical', bypassRequired: true, details: 'Tracks 100+ mouse/keyboard/scroll events. Requires valid akamai-sensor-data header with sensor payload to bypass.' },
	{ pattern: /^__cf_bm$/, provider: 'Cloudflare Bot Management', purpose: 'Bot score token (ML-based)', category: 'antibot', risk: 'critical', bypassRequired: true, details: 'Short-lived ML scoring token. Requires matching browser fingerprint — TLS, HTTP/2, canvas, fonts must all match.' },
	{ pattern: /^cf_clearance$/, provider: 'Cloudflare JS Challenge', purpose: 'JS challenge completion proof', category: 'antibot', risk: 'critical', bypassRequired: true, details: 'Cannot be generated without actually solving the Cloudflare challenge in a real browser environment.' },
	{ pattern: /^datadome$/, provider: 'DataDome', purpose: 'Behavioral bot fingerprint token', category: 'antibot', risk: 'critical', bypassRequired: true, details: 'ML behavioral analysis engine. Analyzes TLS JA3, HTTP/2 SETTINGS, mouse movement, timing. Considered one of the hardest to bypass.' },
	{ pattern: /^_px[0-9a-z]|^_pxhd$|^_pxvid$/, provider: 'PerimeterX / HUMAN Security', purpose: 'Device fingerprint token (200+ signals)', category: 'antibot', risk: 'critical', bypassRequired: true, details: 'HUMAN Security behavioral analysis. Tracks JS execution environment, WebGL, fonts, timing, mouse events. Token is IP/device bound.' },
	{ pattern: /^visid_incap_|^incap_ses_/, provider: 'Imperva / Incapsula', purpose: 'WAF visitor/session tracking', category: 'antibot', risk: 'high', bypassRequired: true, details: 'Imperva WAF session pair. Requires correct referrer chain and both cookies set together to pass through.' },
	{ pattern: /^__ddg[0-9]|^__ddgid_|^__ddgmark_/, provider: 'DDoS-Guard', purpose: 'JS challenge cookie chain', category: 'antibot', risk: 'high', bypassRequired: true, details: 'DDoS-Guard JavaScript challenge series. Must all be obtained from the same challenge solving session.' },
	{ pattern: /^forterToken$/, provider: 'Forter', purpose: 'Device intelligence / fraud signal', category: 'antibot', risk: 'high', bypassRequired: true, details: 'Forter fraud prevention. Primarily targets account takeover and payment fraud.' },
	{ pattern: /^__tmx/, provider: 'ThreatMetrix (LexisNexis)', purpose: 'Device identity hash', category: 'antibot', risk: 'high', bypassRequired: true, details: 'Device fingerprinting for financial fraud prevention. Common in banking/fintech.' },
	{ pattern: /^TS[0-9a-f]{8}/, provider: 'F5 BIG-IP / Shape Security', purpose: 'Load balancer persistence / bot detection', category: 'antibot', risk: 'high', bypassRequired: false, details: 'F5 BIG-IP persistence cookie. When Shape Security is enabled, tracks TLS JA3 fingerprint alongside.' },
	{ pattern: /^kpsdk-/, provider: 'Kasada', purpose: 'SDK proof-of-work token', category: 'antibot', risk: 'critical', bypassRequired: true, details: 'Kasada SDK JavaScript proof-of-work. Includes TLS fingerprinting check on server side.' },
	{ pattern: /^aws-waf-token$/, provider: 'AWS WAF', purpose: 'JavaScript challenge clearance', category: 'antibot', risk: 'high', bypassRequired: true, details: 'AWS WAF JavaScript challenge token. Must be solved with real JS execution.' },
	{ pattern: /^dtCookie$|^rxvt$|^dtSa$|^dtLatC$/, provider: 'Dynatrace RUM', purpose: 'Real User Monitoring (performance only)', category: 'analytics', risk: 'low', bypassRequired: false, details: 'Dynatrace performance monitoring — not a security control. Safe to ignore or replicate.' },
	{ pattern: /^SOCS$/, provider: 'Google', purpose: 'Cookie consent preference', category: 'functional', risk: 'low', bypassRequired: false, details: 'Google cookie consent state. Required on Google properties for EU compliance.' },
	{ pattern: /^AEC$/, provider: 'Google Anti-Abuse', purpose: 'Anti-abuse check token', category: 'antibot', risk: 'medium', bypassRequired: false, details: 'Google anti-abuse cookie. Required for certain account actions but not typically a blocker for requests.' },
	{ pattern: /^_ga|^_gid$|^_gat$/, provider: 'Google Analytics', purpose: 'Analytics visitor/session tracking', category: 'analytics', risk: 'low', bypassRequired: false, details: 'Google Analytics 4 / Universal Analytics visitor ID. Not security-related.' },
	{ pattern: /^_gcl_|^_gac_/, provider: 'Google Ads', purpose: 'Ad conversion / click tracking', category: 'tracking', risk: 'low', bypassRequired: false, details: 'Google Ads conversion linker cookie.' },
	{ pattern: /^_fbp$|^_fbc$/, provider: 'Meta Pixel', purpose: 'Facebook/Meta ad attribution', category: 'tracking', risk: 'low', bypassRequired: false, details: 'Meta Pixel browser fingerprint for ad attribution.' },
	{ pattern: /^wordpress_|^wp-settings-|^wordpress_logged_in/, provider: 'WordPress', purpose: 'CMS authentication / preferences', category: 'session', risk: 'low', bypassRequired: false, details: 'Standard WordPress session and settings cookies.' },
	{ pattern: /^PHPSESSID$|^JSESSIONID$|^ASP\.NET_SessionId$|^CFID$|^CFTOKEN$/, provider: 'Application Session', purpose: 'Server-side session identifier', category: 'session', risk: 'medium', bypassRequired: false, details: 'Standard server-side session cookie. Session fixation attacks may be possible if improperly implemented.' },
	{ pattern: /^_shopify_/, provider: 'Shopify', purpose: 'E-commerce session / cart', category: 'session', risk: 'medium', bypassRequired: false, details: 'Shopify platform session, cart, and storefront tracking.' },
];

function classifyCookie(name: string, value: string): CookieAnalysis | null {
	for (const p of COOKIE_PATTERNS) {
		if (p.pattern.test(name)) {
			return {
				name,
				value: value.length > 48 ? value.slice(0, 45) + '...' : value,
				provider: p.provider,
				purpose: p.purpose,
				category: p.category,
				risk: p.risk,
				bypassRequired: p.bypassRequired,
				details: p.details,
			};
		}
	}
	// Unknown private prefix cookies — possibly custom bot protection
	if (name.startsWith('__') || name.startsWith('_')) {
		return {
			name,
			value: value.length > 48 ? value.slice(0, 45) + '...' : value,
			provider: 'Unknown',
			purpose: 'Unrecognized private/tracking cookie',
			category: 'unknown',
			risk: 'medium',
			bypassRequired: false,
			details: 'Private-prefix cookie with no known provider match. Could be custom bot protection, analytics, or session management.',
		};
	}
	return null;
}

// ─── Main Fingerprint Function ────────────────────────────────────────

export function fingerprint(responses: ResponseInfo[]): FingerprintResult {
	const mergedHeaders: Record<string, string> = {};
	const mergedCookies: Record<string, string> = {};
	let mergedBody = '';

	for (const r of responses) {
		for (const [k, v] of Object.entries(r.headers)) {
			mergedHeaders[k.toLowerCase()] = v;
		}
		for (const [k, v] of Object.entries(r.cookies)) {
			mergedCookies[k] = v;
		}
		if (r.body) mergedBody += r.body;
	}

	const rawMatches: FingerprintMatch[] = [];

	for (const rule of rules) {
		const evidence: string[] = [];
		let matched = false;

		if (rule.match.cookie) {
			const rx = new RegExp(rule.match.cookie, 'i');
			for (const [k, v] of Object.entries(mergedCookies)) {
				if (rx.test(k)) {
					evidence.push(`Cookie: ${k}=${v.slice(0, 36)}${v.length > 36 ? '...' : ''}`);
					matched = true;
				}
			}
		}
		if (rule.match.header) {
			const hval = mergedHeaders[rule.match.header.toLowerCase()];
			if (hval !== undefined) {
				if (!rule.match.headerValue || new RegExp(rule.match.headerValue, 'i').test(hval)) {
					evidence.push(`Header: ${rule.match.header}: ${hval.slice(0, 70)}`);
					matched = true;
				}
			}
		}
		if (rule.match.bodyPattern && mergedBody) {
			if (new RegExp(rule.match.bodyPattern, 'i').test(mergedBody)) {
				evidence.push(`Body: matched /${rule.match.bodyPattern}/`);
				matched = true;
			}
		}
		if (rule.match.statusCode) {
			if (responses.some(r => r.status_code === rule.match.statusCode)) {
				evidence.push(`Status: ${rule.match.statusCode}`);
				matched = true;
			}
		}

		if (matched) rawMatches.push({ rule, evidence });
	}

	// Deduplicate by name — merge evidence, prefer high confidence
	const seenNames = new Map<string, FingerprintMatch>();
	for (const m of rawMatches) {
		const existing = seenNames.get(m.rule.name);
		if (!existing) {
			seenNames.set(m.rule.name, { rule: m.rule, evidence: [...m.evidence] });
		} else {
			if (m.rule.confidence === 'high' && existing.rule.confidence !== 'high') {
				seenNames.set(m.rule.name, { rule: m.rule, evidence: [...existing.evidence, ...m.evidence] });
			} else {
				existing.evidence.push(...m.evidence);
			}
		}
	}
	const matches = Array.from(seenNames.values());

	// Build stack summary
	const stack: StackEntry[] = Array.from(
		new Map(
			matches
				.filter(m => ['waf', 'cdn', 'server', 'framework', 'cms', 'antibot', 'captcha', 'hosting'].includes(m.rule.category))
				.map(m => [m.rule.name, { name: m.rule.name, category: m.rule.category, confidence: m.rule.confidence }])
		).values()
	);

	// Security headers
	const SECURITY_HEADERS = [
		'strict-transport-security', 'content-security-policy', 'x-frame-options',
		'x-content-type-options', 'x-xss-protection', 'referrer-policy',
		'permissions-policy', 'access-control-allow-origin',
		'cross-origin-opener-policy', 'cross-origin-resource-policy',
	];
	const securityHeaders = SECURITY_HEADERS.map(h => ({
		name: h,
		present: h in mergedHeaders,
		value: mergedHeaders[h],
	}));

	// Cookie analysis
	const cookieAnalysis: CookieAnalysis[] = [];
	for (const [name, value] of Object.entries(mergedCookies)) {
		const analysis = classifyCookie(name, value);
		if (analysis) cookieAnalysis.push(analysis);
	}
	// Sort by risk severity
	const riskOrder: Record<string, number> = { critical: 0, high: 1, medium: 2, low: 3 };
	cookieAnalysis.sort((a, b) => (riskOrder[a.risk] ?? 4) - (riskOrder[b.risk] ?? 4));

	return {
		stack,
		matches,
		securityHeaders,
		cookieAnalysis,
		raw: { headers: mergedHeaders, cookies: mergedCookies },
	};
}

export function groupMatches(matches: FingerprintMatch[]): { label: string; icon: string; matches: FingerprintMatch[] }[] {
	const ORDER = ['antibot', 'waf', 'captcha', 'cdn', 'server', 'framework', 'cms', 'hosting', 'analytics', 'tracking', 'identity'];
	const LABELS: Record<string, string> = {
		antibot: 'Bot Protection', waf: 'WAF / Firewall', captcha: 'CAPTCHA',
		cdn: 'CDN', server: 'Server', framework: 'Framework',
		cms: 'CMS', hosting: 'Hosting Platform',
		analytics: 'Analytics / RUM', tracking: 'Ad Tracking', identity: 'Identity',
	};
	const ICONS: Record<string, string> = {
		antibot: 'shield-x', waf: 'shield', captcha: 'puzzle',
		cdn: 'globe', server: 'server', framework: 'code',
		cms: 'layers', hosting: 'cloud', analytics: 'bar-chart', tracking: 'target', identity: 'user',
	};
	const groups: Record<string, FingerprintMatch[]> = {};
	for (const m of matches) {
		const cat = m.rule.category;
		if (!groups[cat]) groups[cat] = [];
		groups[cat].push(m);
	}
	return ORDER
		.filter(cat => groups[cat]?.length)
		.map(cat => ({ label: LABELS[cat] ?? cat, icon: ICONS[cat] ?? 'shield', matches: groups[cat] }));
}
