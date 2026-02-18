import type { BlockDoc } from './types';

export const UTILITY_DOCS: BlockDoc[] = [
	{
		type: 'Log',
		name: 'Log',
		category: 'Utilities',
		description: 'Writes a message to the debug log. Supports variable interpolation with <VAR> syntax. Essential for debugging.',
		parameters: [
			{ name: 'message', type: 'string', required: true, description: 'Log message. Use <VAR> for variable values.' },
		],
		codeExample: `Message: Response code: <data.RESPONSECODE>, Token: <TOKEN>
→ Logs "Response code: 200, Token: abc123"`,
		tips: [
			'Invaluable for debugging — add Log blocks to inspect variable values',
			'Use <VAR> syntax to interpolate any variable',
			'Check the Debug tab at the bottom to see log output',
		],
		relatedBlocks: ['SetVariable', 'Webhook'],
		rustCode: `let message = self.variables.interpolate(&settings.message);
self.log.push(LogEntry {
    timestamp_ms: elapsed_ms(),
    block_id: block.id,
    block_label: block.label.clone(),
    message,
});`,
	},
	{
		type: 'SetVariable',
		name: 'Set Variable',
		category: 'Utilities',
		description: 'Creates or updates a variable with a specific value. The value supports variable interpolation with <VAR> syntax.',
		parameters: [
			{ name: 'name', type: 'string', required: true, description: 'Variable name to set' },
			{ name: 'value', type: 'string', required: true, description: 'Value to assign. Use <VAR> for interpolation.' },
			{ name: 'capture', type: 'boolean', required: false, description: 'Mark as captured output', default: 'false' },
		],
		codeExample: `Name: FULL_URL
Value: https://example.com/api/<TOKEN>
→ Creates FULL_URL with the token interpolated`,
		tips: [
			'Variables set here are available to all subsequent blocks',
			'Enable "capture" to include in hit output',
			'Use for building dynamic URLs, combining extracted data',
		],
		relatedBlocks: ['Log', 'Script', 'StringFunction'],
		rustCode: `let value = self.variables.interpolate(&settings.value);
self.variables.set_user(&settings.name, value, settings.capture);`,
	},
	{
		type: 'ClearCookies',
		name: 'Clear Cookies',
		category: 'Utilities',
		description: 'Clears all cookies stored in the current session. No parameters needed. Useful between login attempts or when testing different accounts.',
		parameters: [],
		codeExample: `[No parameters]
→ Clears all session cookies`,
		tips: [
			'Use before a new login attempt to start with a clean cookie state',
			'Cookies accumulate across HttpRequest blocks within a session',
		],
		relatedBlocks: ['HttpRequest', 'ParseCookie', 'CookieContainer'],
		rustCode: `// Clears the session cookie jar maintained by the sidecar process.
// After this block, subsequent HTTP requests start with no cookies.
sidecar_tx.send(SidecarRequest {
    action: "clear_cookies".into(),
    ..Default::default()
}).await?;`,
	},
	{
		type: 'Webhook',
		name: 'Webhook',
		category: 'Utilities',
		description: 'Sends data to an external webhook URL. Commonly used with Discord, Slack, or custom notification endpoints.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'Webhook endpoint URL' },
			{ name: 'method', type: 'string', required: false, description: 'HTTP method', default: 'POST' },
			{ name: 'headers', type: 'array', required: false, description: 'Custom headers' },
			{ name: 'body_template', type: 'string', required: false, description: 'JSON body template with <VAR> interpolation' },
			{ name: 'content_type', type: 'string', required: false, description: 'Content-Type header', default: 'application/json' },
		],
		codeExample: `URL: https://discord.com/api/webhooks/...
Body: {"content": "Hit: <USER>:<PASS>"}
→ Sends a Discord notification on hit`,
		tips: [
			'Use inside a KeyCheck Success branch to notify on hits',
			'Body supports <VAR> interpolation for dynamic content',
			'Discord webhooks expect JSON with a "content" field',
		],
		relatedBlocks: ['HttpRequest', 'KeyCheck', 'Log'],
		rustCode: `let url = self.variables.interpolate(&settings.url);
let body = self.variables.interpolate(&settings.body_template);
let headers = settings.headers.iter()
    .map(|(k, v)| vec![self.variables.interpolate(k), self.variables.interpolate(v)])
    .collect();
// Fire-and-forget HTTP request via sidecar
let req = SidecarRequest {
    action: "request".into(), method: Some(settings.method.clone()),
    url: Some(url), headers: Some(headers), body: Some(body), ..
};
sidecar_tx.send(req).await?;`,
	},
	{
		type: 'WebSocket',
		name: 'WebSocket',
		category: 'Utilities',
		description: 'Manages WebSocket connections: connect, send messages, receive responses, or close the connection.',
		parameters: [
			{ name: 'url', type: 'string', required: true, description: 'WebSocket URL (ws:// or wss://)' },
			{ name: 'action', type: 'enum', required: true, description: 'Operation: connect, send, receive, close', default: 'connect' },
			{ name: 'message', type: 'string', required: false, description: 'Message to send (for send action)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store received data', default: 'WS_RESPONSE' },
			{ name: 'timeout_ms', type: 'number', required: false, description: 'Timeout for receive operation', default: '10000' },
		],
		codeExample: `Action: connect → ws://example.com/socket
Action: send → {"type":"auth","token":"<TOKEN>"}
Action: receive → stores response in WS_RESPONSE`,
		tips: [
			'Always connect before sending or receiving',
			'Close the connection when done to free resources',
			'Use multiple WebSocket blocks in sequence for full conversations',
		],
		relatedBlocks: ['HttpRequest', 'TcpRequest'],
		rustCode: `let url = self.variables.interpolate(&settings.url);
match settings.action.as_str() {
    "connect" => {
        let (ws, _) = tokio_tungstenite::connect_async(&url).await?;
        self.ws_connection = Some(ws);
    }
    "send" => {
        let msg = self.variables.interpolate(&settings.message);
        self.ws_connection.as_mut()?.send(Message::Text(msg)).await?;
    }
    "receive" => {
        let msg = self.ws_connection.as_mut()?.next().await??;
        self.variables.set_user(&settings.output_var, msg.to_string(), false);
    }
    "close" => { self.ws_connection.take(); }
}`,
	},
	{
		type: 'RandomUserAgent',
		name: 'Random User Agent',
		category: 'Utilities',
		description: 'Generates a random, realistic User-Agent string. Can filter by browser and platform, or use a custom list.',
		parameters: [
			{ name: 'mode', type: 'enum', required: true, description: 'Random = generate realistic UA, CustomList = pick from your list', default: 'Random' },
			{ name: 'browser_filter', type: 'array', required: false, description: 'Browsers to include: Chrome, Firefox, Safari, Edge' },
			{ name: 'platform_filter', type: 'array', required: false, description: 'Platforms: Desktop, Mobile' },
			{ name: 'custom_list', type: 'string', required: false, description: 'Custom UA strings, one per line (CustomList mode)' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the User-Agent', default: 'USER_AGENT' },
			{ name: 'match_tls', type: 'boolean', required: false, description: 'Match TLS fingerprint to the generated UA', default: 'false' },
		],
		codeExample: `Mode: Random
Browsers: Chrome, Firefox
Platform: Desktop
→ Generates a realistic desktop Chrome/Firefox User-Agent`,
		tips: [
			'Use <USER_AGENT> in the User-Agent header of HttpRequest',
			'match_tls adjusts TLS fingerprint to match the browser',
			'Custom list mode picks a random line from your list',
		],
		relatedBlocks: ['HttpRequest', 'BrowserOpen'],
		rustCode: `let ua = match settings.mode {
    Mode::Random => {
        BUILTIN_USER_AGENTS.iter()
            .filter(|(_, browser, platform)| {
                (settings.browser_filter.is_empty() || settings.browser_filter.contains(browser))
                && (settings.platform_filter.is_empty() || settings.platform_filter.contains(platform))
            })
            .choose(&mut rand::thread_rng())
            .map(|(ua, _, _)| ua.to_string())
            .unwrap_or_default()
    }
    Mode::CustomList => {
        settings.custom_list.lines().choose(&mut rand::thread_rng())
            .unwrap_or("").to_string()
    }
};
self.variables.set_user(&settings.output_var, ua, settings.capture);`,
	},
	{
		type: 'RandomData',
		name: 'Random Data',
		category: 'Utilities',
		description: 'Generates random data: strings, UUIDs, numbers, emails, names, addresses, phone numbers, and dates.',
		parameters: [
			{ name: 'data_type', type: 'enum', required: true, description: 'Type: String, Uuid, Number, Email, FirstName, LastName, FullName, StreetAddress, City, State, ZipCode, PhoneNumber, Date' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'RANDOM' },
			{ name: 'string_length', type: 'number', required: false, description: 'Length for random strings', default: '16' },
			{ name: 'string_charset', type: 'string', required: false, description: 'Character set: alphanumeric, alpha, numeric, hex, custom', default: 'alphanumeric' },
			{ name: 'number_min', type: 'number', required: false, description: 'Minimum for random numbers', default: '0' },
			{ name: 'number_max', type: 'number', required: false, description: 'Maximum for random numbers', default: '100' },
		],
		codeExample: `Type: Email
→ Generates a random email like john.doe42@example.com

Type: String, Length: 32, Charset: hex
→ Generates a random 32-char hex string`,
		tips: [
			'Email, names, and addresses generate realistic fake data',
			'Custom charset: set string_charset to "custom" and specify chars',
			'UUID generates a random v4 UUID',
		],
		relatedBlocks: ['SetVariable', 'StringFunction'],
		rustCode: `let value = match settings.data_type {
    RandomDataType::String => random_string(settings.string_length, &settings.string_charset),
    RandomDataType::Uuid => Uuid::new_v4().to_string(),
    RandomDataType::Number => random_number(settings.number_min, settings.number_max, settings.number_decimal),
    RandomDataType::Email => format!("{}@{}.com", random_word(), random_word()),
    RandomDataType::FirstName => FIRST_NAMES.choose(&mut rng).unwrap().to_string(),
    RandomDataType::FullName => format!("{} {}", random_first_name(), random_last_name()),
    RandomDataType::PhoneNumber => format!("+1{}", random_digits(10)),
    RandomDataType::Date => random_date(&settings.date_format, &settings.date_min, &settings.date_max),
    // ... City, State, ZipCode, StreetAddress, LastName
};
self.variables.set_user(&settings.output_var, value, settings.capture);`,
	},
	{
		type: 'Plugin',
		name: 'Plugin Block',
		category: 'Utilities',
		description: 'Executes a block provided by an external plugin DLL. Plugin blocks extend ironbullet with custom functionality.',
		parameters: [
			{ name: 'plugin_block_type', type: 'string', required: true, description: 'Block type identifier (PluginName.BlockName)' },
			{ name: 'settings_json', type: 'string', required: false, description: 'JSON settings for the plugin block', default: '{}' },
			{ name: 'output_var', type: 'string', required: false, description: 'Variable to store the result', default: 'PLUGIN_RESULT' },
		],
		codeExample: `Plugin block type: ExamplePlugin.ReverseString
Settings: {"input_var": "data.SOURCE"}
→ Executes the plugin block with the given settings`,
		tips: [
			'Import plugins via File → Import Plugin (.dll)',
			'Plugin settings are rendered as a form when schema is available',
			'Plugin blocks appear in the palette under their declared category',
		],
		relatedBlocks: ['Script', 'SetVariable'],
		rustCode: `let pm = self.plugin_manager.as_ref()?;
let settings_json = self.variables.interpolate(&settings.settings_json);
let vars_snapshot = serde_json::to_string(&self.variables.snapshot())?;

// Call plugin DLL via FFI (plugin_execute export)
let (success, updated_vars, log) = pm.execute_block(
    &settings.plugin_block_type,
    &settings_json,
    &vars_snapshot,
)?;

// Apply variable updates from plugin
for (k, v) in updated_vars {
    self.variables.set_user(&k, v, settings.capture);
}`,
	},
];
