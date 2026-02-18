import type { BlockDoc } from './types';

export const BROWSER_DOCS: BlockDoc[] = [
	{
		type: 'BrowserOpen',
		name: 'Browser Open',
		category: 'Browser',
		description: 'Opens a headless or visible browser instance for automation. Must be called before any other browser blocks.',
		parameters: [
			{ name: 'headless', type: 'boolean', required: false, description: 'Run without visible window', default: 'true' },
			{ name: 'browser_type', type: 'string', required: false, description: 'Browser engine: chromium, firefox, webkit', default: 'chromium' },
			{ name: 'proxy', type: 'string', required: false, description: 'Proxy URL for browser traffic' },
			{ name: 'extra_args', type: 'string', required: false, description: 'Extra command-line arguments' },
		],
		codeExample: `Browser: chromium
Headless: true
→ Opens a headless Chrome instance`,
		tips: [
			'Always pair with browser blocks like NavigateTo, ClickElement',
			'Headless = false is useful for debugging (shows the browser)',
			'Extra args: --disable-gpu, --no-sandbox',
		],
		relatedBlocks: ['NavigateTo', 'ClickElement', 'TypeText'],
		rustCode: `let mut builder = chromiumoxide::BrowserConfig::builder();
if !settings.headless { builder = builder.with_head(); }
if !settings.proxy.is_empty() {
    builder = builder.arg(format!("--proxy-server={}", self.variables.interpolate(&settings.proxy)));
}
for arg in settings.extra_args.split_whitespace() {
    builder = builder.arg(arg);
}
let (browser, mut handler) = Browser::launch(builder.build()?).await?;
tokio::spawn(async move { while handler.next().await.is_some() {} });
self.browser = Some(browser);`,
	},
	{
		type: 'NavigateTo',
		name: 'Navigate To',
		category: 'Browser',
		description: 'Navigates the browser to a URL and waits for the page to load.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'URL to navigate to' },
			{ name: 'wait_until', type: 'string', required: false, description: 'Load state to wait for: load, domcontentloaded, networkidle', default: 'load' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Navigation timeout', default: '30000' },
		],
		codeExample: `URL: https://example.com/login
Wait until: networkidle
→ Navigates and waits for all network activity to stop`,
		tips: [
			'networkidle waits for no network activity for 500ms — most reliable',
			'domcontentloaded is faster but may miss async-loaded content',
		],
		relatedBlocks: ['BrowserOpen', 'ClickElement', 'WaitForElement'],
		rustCode: `let browser = self.browser.as_ref().ok_or("No browser open")?;
let url = self.variables.interpolate(&settings.url);
let page = browser.new_page(&url).await?;
let _ = page.wait_for_navigation().await;
let content = page.content().await.unwrap_or_default();
self.variables.set_data("SOURCE", content);
self.variables.set_data("URL", page.url().await?.to_string());
self.page = Some(page);`,
	},
	{
		type: 'ClickElement',
		name: 'Click Element',
		category: 'Browser',
		description: 'Clicks an element on the page matching the CSS selector.',
		parameters: [
			{ name: 'selector', type: 'string', required: true, description: 'CSS selector for the element to click' },
			{ name: 'wait_for_navigation', type: 'boolean', required: false, description: 'Wait for page navigation after click', default: 'false' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Timeout for finding the element', default: '5000' },
		],
		codeExample: `Selector: button[type="submit"]
Wait for navigation: true
→ Clicks the submit button and waits for page load`,
		tips: [
			'Enable wait_for_navigation for login/form submit buttons',
			'Use WaitForElement first if the element loads dynamically',
		],
		relatedBlocks: ['TypeText', 'WaitForElement', 'NavigateTo'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let selector = self.variables.interpolate(&settings.selector);
let element = page.find_element(&selector).await?;
element.click().await?;
if settings.wait_for_navigation {
    let _ = page.wait_for_navigation().await;
    self.variables.set_data("SOURCE", page.content().await?);
}`,
	},
	{
		type: 'TypeText',
		name: 'Type Text',
		category: 'Browser',
		description: 'Types text into an input field with optional keystroke delay for realistic typing simulation.',
		parameters: [
			{ name: 'selector', type: 'string', required: true, description: 'CSS selector for the input field' },
			{ name: 'text', type: 'string', required: true, description: 'Text to type. Supports <VAR> interpolation' },
			{ name: 'clear_first', type: 'boolean', required: false, description: 'Clear existing text before typing', default: 'true' },
			{ name: 'delay_ms', type: 'number', required: false, description: 'Delay between keystrokes', default: '50' },
		],
		codeExample: `Selector: input[name="email"]
Text: <USER>
Clear first: true
→ Types the username into the email field`,
		tips: [
			'Use a small delay (30-100ms) for realistic typing',
			'Clear first prevents appending to existing text',
		],
		relatedBlocks: ['ClickElement', 'WaitForElement'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let selector = self.variables.interpolate(&settings.selector);
let text = self.variables.interpolate(&settings.text);
let element = page.find_element(&selector).await?;
if settings.clear_first {
    element.click().await?;
    // Ctrl+A then Backspace to clear
    page.execute(DispatchKeyEventParams { key: "a", modifiers: 2 }).await?;
    page.execute(DispatchKeyEventParams { key: "Backspace" }).await?;
}
element.type_str(&text).await?;`,
	},
	{
		type: 'WaitForElement',
		name: 'Wait For Element',
		category: 'Browser',
		description: 'Waits for an element to appear, disappear, or reach a specific state on the page.',
		parameters: [
			{ name: 'selector', type: 'string', required: true, description: 'CSS selector for the element' },
			{ name: 'state', type: 'string', required: false, description: 'State to wait for: visible, hidden, attached, detached', default: 'visible' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Maximum wait time', default: '10000' },
		],
		codeExample: `Selector: .dashboard-content
State: visible
→ Waits for the dashboard to become visible after login`,
		tips: [
			'visible = element exists and is not hidden',
			'Use before interacting with dynamically loaded elements',
		],
		relatedBlocks: ['ClickElement', 'GetElementText', 'NavigateTo'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let selector = self.variables.interpolate(&settings.selector);
let timeout = Duration::from_millis(settings.timeout_ms);
let start = Instant::now();
loop {
    if page.find_element(&selector).await.is_ok() { break; }
    if start.elapsed() > timeout {
        return Err(format!("Timeout waiting for '{}'", selector));
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
}`,
	},
	{
		type: 'GetElementText',
		name: 'Get Element Text',
		category: 'Browser',
		description: 'Reads text content or a specific attribute from a page element.',
		parameters: [
			{ name: 'selector', type: 'string', required: true, description: 'CSS selector for the element' },
			{ name: 'attribute', type: 'string', required: false, description: 'HTML attribute to read. Empty = inner text' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the text', default: 'ELEMENT_TEXT' },
		],
		codeExample: `Selector: span.user-email
Attribute: (empty)
→ Reads the text content of the user email span`,
		tips: [
			'Leave attribute empty to get visible text content',
			'Use "value" attribute for input fields, "href" for links',
		],
		relatedBlocks: ['WaitForElement', 'ExecuteJs', 'Screenshot'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let selector = self.variables.interpolate(&settings.selector);
let element = page.find_element(&selector).await?;
let value = if settings.attribute.is_empty() || settings.attribute == "innerText" {
    element.inner_text().await?.unwrap_or_default()
} else {
    element.attribute(&settings.attribute).await?.unwrap_or_default()
};
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
	{
		type: 'Screenshot',
		name: 'Screenshot',
		category: 'Browser',
		description: 'Takes a screenshot of the page or a specific element. Stores the image as a base64-encoded string.',
		parameters: [
			{ name: 'full_page', type: 'boolean', required: false, description: 'Capture the entire scrollable page', default: 'false' },
			{ name: 'selector', type: 'string', required: false, description: 'CSS selector for a specific element (empty = full viewport)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store base64 image', default: 'SCREENSHOT_B64' },
		],
		codeExample: `Full page: false
Selector: .captcha-image
→ Captures just the captcha image element`,
		tips: [
			'Output is base64-encoded PNG — can be used with OcrCaptcha',
			'Full page captures everything including off-screen content',
		],
		relatedBlocks: ['OcrCaptcha', 'WaitForElement', 'ExecuteJs'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let bytes = if !settings.selector.is_empty() {
    let el = page.find_element(&self.variables.interpolate(&settings.selector)).await?;
    el.screenshot(CaptureScreenshotFormat::Png).await?
} else {
    page.screenshot(CaptureScreenshotParams::builder()
        .format(CaptureScreenshotFormat::Png).build()?).await?
};
let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
self.variables.set_user(&settings.output_var, b64, false);`,
	},
	{
		type: 'ExecuteJs',
		name: 'Execute JS',
		category: 'Browser',
		description: 'Executes JavaScript code in the browser page context. Can interact with the DOM and return values.',
		parameters: [
			{ name: 'code', type: 'string', required: true, description: 'JavaScript code to execute in the page' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the return value', default: 'JS_RESULT' },
		],
		codeExample: `// Get the CSRF token from a meta tag
return document.querySelector('meta[name="csrf-token"]').content;`,
		tips: [
			'Code runs in the page context — full DOM access',
			'Use "return" to pass values back to the pipeline',
			'Unlike the Script block, this runs IN the browser page',
		],
		relatedBlocks: ['Script', 'GetElementText', 'NavigateTo'],
		rustCode: `let page = self.page.as_ref().ok_or("No page open")?;
let code = self.variables.interpolate(&settings.code);
let result = page.evaluate_expression(&code).await?;
let value = match result.value() {
    Some(Value::String(s)) => s.clone(),
    Some(other) => other.to_string(),
    None => String::new(),
};
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
];
