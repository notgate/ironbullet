import type { BlockDoc } from './types';

export const BYPASS_DOCS: BlockDoc[] = [
	{
		type: 'CaptchaSolver',
		name: 'Captcha Solver',
		category: 'Bypass',
		description: 'Solves captchas using a third-party solver service API. Supports reCAPTCHA v2, hCaptcha, FunCaptcha, Image Captcha, and Cloudflare Turnstile.',
		parameters: [
			{ name: 'solver_service', type: 'enum', required: true, description: 'Service: capsolver, 2captcha, anticaptcha, capmonster' },
			{ name: 'captcha_type', type: 'enum', required: true, description: 'Type: RecaptchaV2, HCaptcha, FunCaptcha, ImageCaptcha, Turnstile' },
			{ name: 'api_key', type: 'string', required: true, description: 'Your solver service API key' },
			{ name: 'site_key', type: 'string', required: true, description: 'Captcha site key from the page HTML' },
			{ name: 'page_url', type: 'string', required: true, description: 'URL of the page with the captcha' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the solved token', default: 'CAPTCHA_TOKEN' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Max wait time for solution', default: '120000' },
		],
		codeExample: `Service: capsolver
Type: RecaptchaV2
Site key: 6Le... (from page source)
Page URL: https://example.com/login
→ Returns captcha token in CAPTCHA_TOKEN`,
		tips: [
			'Find the site key in the page source (data-sitekey attribute)',
			'Token is typically submitted as g-recaptcha-response form field',
			'Tokens expire quickly — use immediately after solving',
		],
		relatedBlocks: ['HttpRequest', 'CloudflareBypass', 'OcrCaptcha'],
		rustCode: `let api_key = self.variables.interpolate(&settings.api_key);
let site_key = self.variables.interpolate(&settings.site_key);
let page_url = self.variables.interpolate(&settings.page_url);
// Submit task to solver service (2captcha, capsolver, etc.)
let task_id = submit_captcha_task(&settings.solver_service, &api_key, &site_key, &page_url, &settings.captcha_type).await?;
// Poll for result with timeout
let result = poll_captcha_result(&settings.solver_service, &api_key, &task_id, settings.timeout_ms).await?;
self.variables.set_user(&settings.output_var, result, settings.capture);`,
	},
	{
		type: 'CloudflareBypass',
		name: 'Cloudflare Bypass',
		category: 'Bypass',
		description: 'Bypasses Cloudflare protection using a FlareSolverr instance. Returns cookies that can be used in subsequent requests.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'URL of the Cloudflare-protected page' },
			{ name: 'flaresolverr_url', type: 'string', required: false, description: 'FlareSolverr API endpoint', default: 'http://localhost:8191/v1' },
			{ name: 'max_timeout_ms', type: 'number', required: false, description: 'Maximum wait time', default: '60000' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store bypass cookies', default: 'CF_COOKIES' },
		],
		codeExample: `URL: https://protected-site.com
FlareSolverr: http://localhost:8191/v1
→ Returns CF clearance cookies`,
		tips: [
			'Requires a running FlareSolverr instance (Docker recommended)',
			'Cookies from bypass should be sent with subsequent HttpRequest blocks',
		],
		relatedBlocks: ['CaptchaSolver', 'HttpRequest'],
		rustCode: `let url = self.variables.interpolate(&settings.url);
let flaresolverr = self.variables.interpolate(&settings.flaresolverr_url);
// POST to FlareSolverr API
let body = serde_json::json!({
    "cmd": "request.get", "url": url,
    "maxTimeout": settings.max_timeout_ms,
});
let resp = reqwest::Client::new()
    .post(&format!("{}/v1", flaresolverr))
    .json(&body).send().await?.json::<Value>().await?;
let solution = &resp["solution"];
let cookies = solution["cookies"].as_str().unwrap_or("");
self.variables.set_user(&settings.output_var, cookies.to_string(), settings.capture);`,
	},
	{
		type: 'LaravelCsrf',
		name: 'Laravel CSRF',
		category: 'Bypass',
		description: 'Fetches a Laravel CSRF token from a page. Extracts both the hidden input token and the XSRF cookie.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'URL of the page with the CSRF token' },
			{ name: 'csrf_selector', type: 'string', required: false, description: 'CSS selector for the hidden input', default: 'input[name="_token"]' },
			{ name: 'cookie_name', type: 'string', required: false, description: 'Name of the CSRF cookie', default: 'XSRF-TOKEN' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the token', default: 'CSRF_TOKEN' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Request timeout', default: '10000' },
		],
		codeExample: `URL: https://laravel-app.com/login
→ Extracts _token and XSRF-TOKEN cookie
→ Use <CSRF_TOKEN> in subsequent POST body`,
		tips: [
			'The CSRF token must be included in form submissions',
			'Token changes on each page load — always fetch fresh',
		],
		relatedBlocks: ['HttpRequest', 'ParseCSS'],
		rustCode: `let url = self.variables.interpolate(&settings.url);
// GET the page via sidecar
let req = SidecarRequest { action: "request".into(), method: Some("GET".into()), url: Some(url), .. };
let resp = sidecar_tx.send(req).await?;
// Extract CSRF token using CSS selector (scraper crate)
let document = scraper::Html::parse_document(&resp.body);
let selector = scraper::Selector::parse(&settings.csrf_selector)?;
let token = document.select(&selector).next()
    .and_then(|el| el.value().attr("value").or(el.value().attr("content")))
    .unwrap_or("").to_string();
self.variables.set_user(&settings.output_var, token, settings.capture);`,
	},
	{
		type: 'OcrCaptcha',
		name: 'OCR Captcha',
		category: 'Bypass',
		description: 'Performs OCR (optical character recognition) on a base64-encoded image to read text captchas locally without a third-party service.',
		parameters: [
			{ name: 'input_var', type: 'string', required: true, description: 'Variable containing base64 image data', default: 'SCREENSHOT_B64' },
			{ name: 'language', type: 'string', required: false, description: 'OCR language', default: 'eng' },
			{ name: 'psm', type: 'number', required: false, description: 'Page segmentation mode (Tesseract)', default: '7' },
			{ name: 'whitelist', type: 'string', required: false, description: 'Allowed characters (e.g. 0123456789 for numbers only)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store OCR text', default: 'OCR_TEXT' },
		],
		codeExample: `Input: SCREENSHOT_B64
Whitelist: 0123456789ABCDEF
→ Reads text from captcha image, only allowing hex chars`,
		tips: [
			'PSM 7 = single text line — best for simple captchas',
			'Whitelist restricts recognized characters for better accuracy',
			'Works best on clean, high-contrast captcha images',
		],
		relatedBlocks: ['Screenshot', 'CaptchaSolver'],
		rustCode: `let input_b64 = self.variables.get(&settings.input_var).unwrap_or_default();
let image_bytes = base64::decode(&input_b64)?;
let temp_path = std::env::temp_dir().join(format!("ocr_{}.png", Uuid::new_v4()));
std::fs::write(&temp_path, &image_bytes)?;
let mut args = rusty_tesseract::Args::default();
args.lang = settings.language.clone();
args.psm = Some(settings.psm as i32);
let image = rusty_tesseract::Image::from_path(&temp_path)?;
let result = rusty_tesseract::image_to_string(&image, &args)?;
std::fs::remove_file(&temp_path).ok();
self.variables.set_user(&settings.output_var, result.trim().to_string(), settings.capture);`,
	},
	{
		type: 'RecaptchaInvisible',
		name: 'reCAPTCHA Invisible',
		category: 'Bypass',
		description: 'Generates a reCAPTCHA invisible token by directly interacting with Google reCAPTCHA API endpoints.',
		parameters: [
			{ name: 'sitekey', type: 'string', required: true, description: 'reCAPTCHA site key' },
			{ name: 'anchor_url', type: 'string', required: true, description: 'Anchor URL from the reCAPTCHA iframe' },
			{ name: 'reload_url', type: 'string', required: true, description: 'Reload URL for token generation' },
			{ name: 'action', type: 'string', required: false, description: 'reCAPTCHA action parameter' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the token', default: 'RECAPTCHA_TOKEN' },
		],
		codeExample: `Sitekey: 6Le...
Anchor URL: (from network tab)
→ Generates reCAPTCHA token without a solver service`,
		tips: [
			'Capture the anchor and reload URLs from the browser network tab',
			'No solver service needed — direct API interaction',
		],
		relatedBlocks: ['CaptchaSolver', 'HttpRequest'],
		rustCode: `// Step 1: GET anchor URL to extract recaptcha-token
let anchor_resp = sidecar.request("GET", &settings.anchor_url, &settings.user_agent).await?;
let token = extract_between(&anchor_resp.body, "recaptcha-token\\" value=\\"", "\\"");
// Step 2: POST reload URL with token parameters
let post_body = format!("v={}&reason=q&c={}&k={}&co={}&hl=en&size={}&cb={}&sa={}",
    settings.v, token, settings.sitekey, settings.co, settings.size, settings.cb, settings.action);
let reload_resp = sidecar.request("POST", &settings.reload_url, &post_body).await?;
let rresp = extract_between(&reload_resp.body, "rresp\\":\\"", "\\"");
self.variables.set_user(&settings.output_var, rresp, settings.capture);`,
	},
	{
		type: 'XacfSensor',
		name: 'XACF Sensor',
		category: 'Sensors',
		description: 'Generates an XACF sensor data payload for Akamai-protected websites.',
		parameters: [
			{ name: 'bundle_id', type: 'string', required: true, description: 'Akamai bundle ID from the target site' },
			{ name: 'version', type: 'string', required: false, description: 'Sensor version', default: '2.1.2' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store sensor data', default: 'XACF_SENSOR' },
		],
		codeExample: `Bundle ID: abc123...
→ Generates sensor payload to submit as _abck cookie`,
		tips: [
			'Bundle ID is found in the Akamai script on the target page',
			'Submit the sensor data in the request to pass Akamai checks',
		],
		relatedBlocks: ['DataDomeSensor', 'HttpRequest'],
		rustCode: `// Generate Akamai Bot Manager sensor data
let bundle_id = self.variables.interpolate(&settings.bundle_id);
let version = self.variables.interpolate(&settings.version);
let sensor = generate_xacf_sensor_data(&bundle_id, &version);
// Sensor includes randomized touch events, accelerometer data,
// screen dimensions, and timing values
self.variables.set_user(&settings.output_var, sensor, settings.capture);`,
	},
	{
		type: 'AkamaiV3Sensor',
		name: 'Akamai V3 Sensor',
		category: 'Sensors',
		description: 'Encrypts or decrypts Akamai v3 sensor data payloads using the seeded PRNG algorithm. Can also extract the cookie_hash from a bm_sz cookie. Algorithm credit: glizzykingdreko (https://github.com/glizzykingdreko/akamai-v3-sensor-data-helper).',
		parameters: [
			{ name: 'mode', type: 'enum', required: true, description: 'Encrypt, Decrypt, or ExtractCookieHash' },
			{ name: 'payload_var', type: 'string', required: true, description: 'Variable containing the payload (for encrypt), sensor_data (for decrypt), or bm_sz cookie (for extract)' },
			{ name: 'file_hash', type: 'string', required: true, description: 'Integer file hash used to seed element swapping PRNG' },
			{ name: 'cookie_hash', type: 'string', required: false, description: 'Integer cookie hash from bm_sz cookie (seeds character substitution)', default: '8888888' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'AKAMAI_SENSOR' },
		],
		codeExample: `Mode: Encrypt
Payload: <SENSOR_PAYLOAD>
File hash: 123456789
Cookie hash: 8888888 (or extracted from bm_sz)
-> Output: 3;0;1;0;{cookie_hash};{file_hash};141659;{encrypted}

Mode: ExtractCookieHash
Payload: <BM_SZ> (bm_sz cookie value)
-> Output: extracted cookie_hash integer`,
		tips: [
			'Use ExtractCookieHash mode first to get the cookie_hash from the bm_sz cookie',
			'File hash is found in the Akamai Bot Manager script on the target page',
			'The encrypted output follows the format: 3;0;1;0;{cookie_hash};{file_hash};141659;{data}',
			'Decrypt mode reverses the process: first undo character substitution, then undo element swapping',
		],
		relatedBlocks: ['XacfSensor', 'DataDomeSensor', 'HttpRequest'],
		rustCode: `// Algorithm credit: glizzykingdreko
// https://github.com/glizzykingdreko/akamai-v3-sensor-data-helper
//
// Encrypt: 1) Swap elements seeded by file_hash, 2) Substitute chars seeded by cookie_hash
// Decrypt: 1) Reverse char substitution, 2) Reverse element swaps
// ExtractCookieHash: decodeURIComponent(bm_sz).split('~')[2] parsed as int`,
	},
	{
		type: 'DataDomeSensor',
		name: 'DataDome Sensor',
		category: 'Sensors',
		description: 'Generates a DataDome sensor payload to bypass DataDome bot protection. Algorithm credit: glizzykingdreko (https://github.com/glizzykingdreko).',
		parameters: [
			{ name: 'site_url', type: 'string', required: true, description: 'Target site URL' },
			{ name: 'cookie_datadome', type: 'string', required: true, description: 'Current DataDome cookie value' },
			{ name: 'user_agent', type: 'string', required: true, description: 'User-Agent string to match' },
			{ name: 'custom_wasm_b64', type: 'string', required: false, description: 'Custom WASM binary (base64)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store sensor data', default: 'DD_SENSOR' },
		],
		codeExample: `Site URL: https://protected-site.com
Cookie: datadome=xxx
→ Generates sensor payload for DataDome validation`,
		tips: [
			'Requires the current datadome cookie from initial page load',
			'User-Agent must match what you use in HttpRequest headers',
		],
		relatedBlocks: ['XacfSensor', 'HttpRequest', 'RandomUserAgent'],
		rustCode: `// Algorithm credit: glizzykingdreko
// https://github.com/glizzykingdreko
let site_url = self.variables.interpolate(&settings.site_url);
let cookie = self.variables.interpolate(&settings.cookie_datadome);
let ua = self.variables.interpolate(&settings.user_agent);
let custom_wasm = if !settings.custom_wasm_b64.is_empty() {
    Some(base64::decode(&settings.custom_wasm_b64)?)
} else { None };
// Generate DataDome interstitial sensor payload
let sensor = datadome::generate_sensor(&site_url, &cookie, &ua, custom_wasm.as_deref())?;
self.variables.set_user(&settings.output_var, sensor, settings.capture);`,
	},
];
