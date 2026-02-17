// TypeScript types mirroring Rust pipeline types

export interface Pipeline {
	id: string;
	name: string;
	author: string;
	created: string;
	modified: string;
	blocks: Block[];
	startup_blocks: Block[];
	data_settings: DataSettings;
	proxy_settings: ProxySettings;
	browser_settings: BrowserSettings;
	runner_settings: RunnerSettings;
	output_settings: OutputSettings;
}

export interface Block {
	id: string;
	block_type: BlockType;
	label: string;
	disabled: boolean;
	safe_mode: boolean;
	settings: BlockSettings;
}

export type BlockType =
	| 'HttpRequest' | 'ParseLR' | 'ParseRegex' | 'ParseJSON' | 'ParseCSS' | 'ParseXPath' | 'ParseCookie'
	| 'KeyCheck' | 'StringFunction' | 'ListFunction' | 'CryptoFunction' | 'ConversionFunction'
	| 'IfElse' | 'Loop' | 'Delay' | 'Script' | 'Log' | 'SetVariable' | 'ClearCookies' | 'Webhook' | 'WebSocket'
	| 'TcpRequest' | 'UdpRequest' | 'FtpRequest' | 'SshRequest' | 'ImapRequest' | 'SmtpRequest' | 'PopRequest'
	| 'DateFunction' | 'CaseSwitch' | 'CookieContainer'
	| 'CaptchaSolver' | 'CloudflareBypass' | 'LaravelCsrf'
	| 'BrowserOpen' | 'NavigateTo' | 'ClickElement' | 'TypeText' | 'WaitForElement' | 'GetElementText' | 'Screenshot' | 'ExecuteJs'
	| 'RandomUserAgent' | 'OcrCaptcha' | 'RecaptchaInvisible' | 'XacfSensor'
	| 'RandomData' | 'DataDomeSensor' | 'Plugin' | 'Group';

export type BlockSettings =
	| { type: 'HttpRequest' } & HttpRequestSettings
	| { type: 'ParseLR' } & ParseLRSettings
	| { type: 'ParseRegex' } & ParseRegexSettings
	| { type: 'ParseJSON' } & ParseJSONSettings
	| { type: 'ParseCSS' } & ParseCSSSettings
	| { type: 'ParseXPath' } & ParseXPathSettings
	| { type: 'ParseCookie' } & ParseCookieSettings
	| { type: 'KeyCheck' } & KeyCheckSettings
	| { type: 'StringFunction' } & StringFunctionSettings
	| { type: 'ListFunction' } & ListFunctionSettings
	| { type: 'CryptoFunction' } & CryptoFunctionSettings
	| { type: 'ConversionFunction' } & ConversionFunctionSettings
	| { type: 'IfElse' } & IfElseSettings
	| { type: 'Loop' } & LoopSettings
	| { type: 'Delay' } & DelaySettings
	| { type: 'Script' } & ScriptSettings
	| { type: 'Log' } & LogSettings
	| { type: 'SetVariable' } & SetVariableSettings
	| { type: 'ClearCookies' }
	| { type: 'Webhook' } & WebhookSettings
	| { type: 'WebSocket' } & WebSocketSettings
	| { type: 'TcpRequest' } & TcpRequestSettings
	| { type: 'UdpRequest' } & UdpRequestSettings
	| { type: 'FtpRequest' } & FtpRequestSettings
	| { type: 'SshRequest' } & SshRequestSettings
	| { type: 'ImapRequest' } & ImapRequestSettings
	| { type: 'SmtpRequest' } & SmtpRequestSettings
	| { type: 'PopRequest' } & PopRequestSettings
	| { type: 'DateFunction' } & DateFunctionSettings
	| { type: 'CaseSwitch' } & CaseSwitchSettings
	| { type: 'CookieContainer' } & CookieContainerSettings
	| { type: 'CaptchaSolver' } & CaptchaSolverSettings
	| { type: 'CloudflareBypass' } & CloudflareBypassSettings
	| { type: 'LaravelCsrf' } & LaravelCsrfSettings
	| { type: 'BrowserOpen' } & BrowserOpenSettings
	| { type: 'NavigateTo' } & NavigateToSettings
	| { type: 'ClickElement' } & ClickElementSettings
	| { type: 'TypeText' } & TypeTextSettings
	| { type: 'WaitForElement' } & WaitForElementSettings
	| { type: 'GetElementText' } & GetElementTextSettings
	| { type: 'Screenshot' } & ScreenshotSettings
	| { type: 'ExecuteJs' } & ExecuteJsSettings
	| { type: 'RandomUserAgent' } & RandomUserAgentSettings
	| { type: 'OcrCaptcha' } & OcrCaptchaSettings
	| { type: 'RecaptchaInvisible' } & RecaptchaInvisibleSettings
	| { type: 'XacfSensor' } & XacfSensorSettings
	| { type: 'RandomData' } & RandomDataSettings
	| { type: 'DataDomeSensor' } & DataDomeSensorSettings
	| { type: 'Plugin' } & PluginBlockSettings
	| { type: 'Group' } & GroupSettings;

export interface HttpRequestSettings {
	method: string;
	url: string;
	headers: [string, string][];
	body: string;
	body_type: 'None' | 'Standard' | 'Raw' | 'Multipart' | 'BasicAuth';
	content_type: string;
	follow_redirects: boolean;
	max_redirects: number;
	timeout_ms: number;
	auto_redirect: boolean;
	basic_auth: [string, string] | null;
	http_version: string;
	response_var: string;
	custom_cookies: string;
}

export interface ParseLRSettings {
	input_var: string;
	left: string;
	right: string;
	output_var: string;
	capture: boolean;
	recursive: boolean;
	case_insensitive: boolean;
}

export interface ParseRegexSettings {
	input_var: string;
	pattern: string;
	output_format: string;
	output_var: string;
	capture: boolean;
	multi_line: boolean;
}

export interface ParseJSONSettings {
	input_var: string;
	json_path: string;
	output_var: string;
	capture: boolean;
}

export interface ParseCSSSettings {
	input_var: string;
	selector: string;
	attribute: string;
	output_var: string;
	capture: boolean;
	index: number;
}

export interface ParseXPathSettings {
	input_var: string;
	xpath: string;
	output_var: string;
	capture: boolean;
}

export interface ParseCookieSettings {
	input_var: string;
	cookie_name: string;
	output_var: string;
	capture: boolean;
}

export interface KeyCheckSettings {
	keychains: Keychain[];
}

export interface Keychain {
	result: BotStatus;
	conditions: KeyCondition[];
}

export interface KeyCondition {
	source: string;
	comparison: Comparison;
	value: string;
}

export type BotStatus = 'None' | 'Success' | 'Fail' | 'Ban' | 'Retry' | 'Error' | 'Custom';
export type Comparison = 'Contains' | 'NotContains' | 'EqualTo' | 'NotEqualTo' | 'MatchesRegex' | 'GreaterThan' | 'LessThan' | 'Exists' | 'NotExists';

export interface StringFunctionSettings {
	function_type: string;
	input_var: string;
	output_var: string;
	capture: boolean;
	param1: string;
	param2: string;
}

export interface ListFunctionSettings {
	function_type: string;
	input_var: string;
	output_var: string;
	capture: boolean;
	param1: string;
}

export interface CryptoFunctionSettings {
	function_type: string;
	input_var: string;
	output_var: string;
	capture: boolean;
	key: string;
}

export interface ConversionFunctionSettings {
	input_var: string;
	output_var: string;
	capture: boolean;
	from_type: string;
	to_type: string;
}

// ── Date Function ──

export interface DateFunctionSettings {
	function_type: string;
	input_var: string;
	output_var: string;
	format: string;
	amount: number;
	unit: string;
	capture: boolean;
}

// ── Case / Switch ──

export interface CaseSwitchSettings {
	input_var: string;
	cases: CaseBranch[];
	default_value: string;
	output_var: string;
	capture: boolean;
}

export interface CaseBranch {
	match_value: string;
	result_value: string;
}

// ── Cookie Container ──

export interface CookieContainerSettings {
	source: string;
	source_type: string;
	domain: string;
	output_var: string;
	capture: boolean;
	save_netscape: boolean;
}

// ── Webhook / WebSocket ──

export interface WebhookSettings {
	url: string;
	method: string;
	headers: [string, string][];
	body_template: string;
	content_type: string;
	custom_cookies: string;
}

export interface WebSocketSettings {
	url: string;
	action: string;
	message: string;
	output_var: string;
	timeout_ms: number;
}

// ── Protocol Requests ──

export interface TcpRequestSettings {
	host: string;
	port: number;
	data: string;
	output_var: string;
	timeout_ms: number;
	use_tls: boolean;
	capture: boolean;
}

export interface UdpRequestSettings {
	host: string;
	port: number;
	data: string;
	output_var: string;
	timeout_ms: number;
	capture: boolean;
}

export interface FtpRequestSettings {
	host: string;
	port: number;
	username: string;
	password: string;
	command: string;
	output_var: string;
	timeout_ms: number;
	capture: boolean;
}

export interface SshRequestSettings {
	host: string;
	port: number;
	username: string;
	password: string;
	command: string;
	output_var: string;
	timeout_ms: number;
	capture: boolean;
}

export interface ImapRequestSettings {
	host: string;
	port: number;
	username: string;
	password: string;
	use_tls: boolean;
	command: string;
	output_var: string;
	timeout_ms: number;
	capture: boolean;
}

export interface SmtpRequestSettings {
	host: string;
	port: number;
	username: string;
	password: string;
	use_tls: boolean;
	command: string;
	output_var: string;
	timeout_ms: number;
	capture: boolean;
}

export interface PopRequestSettings {
	host: string;
	port: number;
	username: string;
	password: string;
	use_tls: boolean;
	command: string;
	output_var: string;
	timeout_ms: number;
	capture: boolean;
}

// ── Bypass / Anti-bot ──

export interface CaptchaSolverSettings {
	solver_service: string;
	api_key: string;
	site_key: string;
	page_url: string;
	captcha_type: string;
	output_var: string;
	timeout_ms: number;
	capture: boolean;
}

export interface CloudflareBypassSettings {
	url: string;
	flaresolverr_url: string;
	max_timeout_ms: number;
	output_var: string;
	capture: boolean;
}

export interface LaravelCsrfSettings {
	url: string;
	csrf_selector: string;
	cookie_name: string;
	output_var: string;
	timeout_ms: number;
	capture: boolean;
}

// ── Random User Agent ──

export interface RandomUserAgentSettings {
	mode: 'Random' | 'CustomList';
	browser_filter: string[];
	platform_filter: string[];
	custom_list: string;
	output_var: string;
	capture: boolean;
	match_tls: boolean;
}

// ── OCR Captcha ──

export interface OcrCaptchaSettings {
	input_var: string;
	language: string;
	psm: number;
	whitelist: string;
	output_var: string;
	capture: boolean;
}

// ── reCAPTCHA Invisible ──

export interface RecaptchaInvisibleSettings {
	ar: string;
	sitekey: string;
	co: string;
	hi: string;
	v: string;
	size: string;
	action: string;
	cb: string;
	anchor_url: string;
	reload_url: string;
	user_agent: string;
	output_var: string;
	capture: boolean;
}

// ── XACF Sensor ──

export interface XacfSensorSettings {
	bundle_id: string;
	version: string;
	output_var: string;
	capture: boolean;
}

// ── Random Data ──

export type RandomDataType = 'String' | 'Uuid' | 'Number' | 'Email' | 'FirstName' | 'LastName' | 'FullName' | 'StreetAddress' | 'City' | 'State' | 'ZipCode' | 'PhoneNumber' | 'Date';

export interface RandomDataSettings {
	data_type: RandomDataType;
	output_var: string;
	capture: boolean;
	string_length: number;
	string_charset: string;
	custom_chars: string;
	number_min: number;
	number_max: number;
	number_decimal: boolean;
	date_format: string;
	date_min: string;
	date_max: string;
}

// ── DataDome Sensor ──

export interface DataDomeSensorSettings {
	site_url: string;
	cookie_datadome: string;
	user_agent: string;
	custom_wasm_b64: string;
	output_var: string;
	capture: boolean;
}

// ── Plugin Block ──

export interface PluginBlockSettings {
	plugin_block_type: string;
	settings_json: string;
	output_var: string;
	capture: boolean;
}

// ── Group ──

export interface GroupSettings {
	blocks: Block[];
	collapsed: boolean;
}

// ── Browser Automation ──

export interface BrowserOpenSettings {
	headless: boolean;
	browser_type: string;
	proxy: string;
	extra_args: string;
}

export interface NavigateToSettings {
	url: string;
	wait_until: string;
	timeout_ms: number;
	custom_cookies: string;
}

export interface ClickElementSettings {
	selector: string;
	wait_for_navigation: boolean;
	timeout_ms: number;
}

export interface TypeTextSettings {
	selector: string;
	text: string;
	clear_first: boolean;
	delay_ms: number;
}

export interface WaitForElementSettings {
	selector: string;
	state: string;
	timeout_ms: number;
}

export interface GetElementTextSettings {
	selector: string;
	attribute: string;
	output_var: string;
	capture: boolean;
}

export interface ScreenshotSettings {
	full_page: boolean;
	selector: string;
	output_var: string;
}

export interface ExecuteJsSettings {
	code: string;
	output_var: string;
	capture: boolean;
}

export interface IfElseSettings {
	condition: KeyCondition;
	true_blocks: Block[];
	false_blocks: Block[];
}

export interface LoopSettings {
	loop_type: 'ForEach' | 'Repeat';
	list_var: string;
	item_var: string;
	count: number;
	blocks: Block[];
}

export interface DelaySettings {
	min_ms: number;
	max_ms: number;
}

export interface ScriptSettings {
	code: string;
	output_var: string;
	capture: boolean;
}

export interface LogSettings {
	message: string;
}

export interface SetVariableSettings {
	name: string;
	value: string;
	capture: boolean;
}

export interface DataSettings {
	wordlist_type: string;
	separator: string;
	slices: string[];
}

export interface ProxySettings {
	proxy_mode: 'None' | 'Rotate' | 'Sticky' | 'CpmLimited';
	proxy_sources: ProxySource[];
	ban_duration_secs: number;
	max_retries_before_ban: number;
	cpm_per_proxy: number;
	proxy_groups: ProxyGroup[];
	active_group: string;
}

export interface ProxyGroup {
	name: string;
	mode: 'None' | 'Rotate' | 'Sticky' | 'CpmLimited';
	sources: ProxySource[];
	cpm_per_proxy: number;
}

export interface ProxySource {
	source_type: 'File' | 'Url' | 'Inline';
	value: string;
	refresh_interval_secs: number;
}

export interface BrowserSettings {
	browser: string;
	ja3: string | null;
	http2_fingerprint: string | null;
	user_agent: string | null;
}

export interface RunnerSettings {
	threads: number;
	skip: number;
	take: number;
	continue_statuses: BotStatus[];
	custom_status_name: string;
	max_retries: number;
	concurrent_per_proxy: number;
	start_threads_gradually: boolean;
	gradual_delay_ms: number;
	automatic_thread_count: boolean;
	lower_threads_on_retry: boolean;
	retry_thread_reduction_pct: number;
	pause_on_ratelimit: boolean;
	only_proxyless: boolean;
}

export interface OutputSettings {
	save_to_file: boolean;
	save_to_database: boolean;
	include_response: boolean;
	output_directory: string;
	output_format: string;
	database_path: string;
	output_format_type: 'Txt' | 'Csv' | 'Json';
	capture_filters: CaptureFilter[];
}

export interface CaptureFilter {
	variable_name: string;
	filter_type: CaptureFilterType;
	value: string;
	negate: boolean;
}

export type CaptureFilterType = 'Contains' | 'Equals' | 'StartsWith' | 'EndsWith' | 'MatchesRegex' | 'MinLength' | 'MaxLength' | 'NotEmpty';

export interface RunnerStats {
	total: number;
	processed: number;
	hits: number;
	fails: number;
	bans: number;
	retries: number;
	errors: number;
	cpm: number;
	active_threads: number;
	elapsed_secs: number;
}

export interface BlockResult {
	block_id: string;
	block_label: string;
	block_type: BlockType;
	success: boolean;
	timing_ms: number;
	variables_after: Record<string, string>;
	log_message: string;
	request?: RequestInfo;
	response?: ResponseInfo;
}

export interface RequestInfo {
	method: string;
	url: string;
	headers: [string, string][];
	body: string;
}

export interface ResponseInfo {
	status_code: number;
	headers: Record<string, string>;
	body: string;
	final_url: string;
	cookies: Record<string, string>;
	timing_ms: number;
}

export interface NetworkEntry {
	block_id: string;
	block_label: string;
	method: string;
	url: string;
	status_code: number;
	timing_ms: number;
	response_size: number;
	cookies_set: [string, string][];
	cookies_sent: [string, string][];
}

// Config tab for multi-config management
export interface ConfigTab {
	id: string;
	name: string;
	filePath: string | null;
	pipeline: Pipeline;
	isDirty: boolean;
	savedSnapshot: string; // JSON snapshot for dirty comparison
}

// Block metadata for palette
export interface BlockMeta {
	type: BlockType;
	label: string;
	category: string;
	color: string;
	icon: string;
}

// Plugin metadata
export interface PluginBlockMeta {
	block_type_name: string;
	label: string;
	category: string;
	color: string;
	icon: string;
	settings_schema_json: string;
	default_settings_json: string;
	plugin_name: string;
	block_index: number;
}

export interface PluginMeta {
	name: string;
	version: string;
	author: string;
	description: string;
	dll_path: string;
}

export const BLOCK_CATALOG: BlockMeta[] = [
	{ type: 'HttpRequest', label: 'HTTP Request', category: 'Requests', color: '#0078d4', icon: 'globe' },
	{ type: 'ParseLR', label: 'Parse LR', category: 'Parsing', color: '#4ec9b0', icon: 'scissors' },
	{ type: 'ParseRegex', label: 'Parse Regex', category: 'Parsing', color: '#4ec9b0', icon: 'regex' },
	{ type: 'ParseJSON', label: 'Parse JSON', category: 'Parsing', color: '#4ec9b0', icon: 'braces' },
	{ type: 'ParseCSS', label: 'Parse CSS', category: 'Parsing', color: '#4ec9b0', icon: 'code' },
	{ type: 'ParseXPath', label: 'Parse XPath', category: 'Parsing', color: '#4ec9b0', icon: 'file-code' },
	{ type: 'ParseCookie', label: 'Parse Cookie', category: 'Parsing', color: '#4ec9b0', icon: 'cookie' },
	{ type: 'KeyCheck', label: 'Key Check', category: 'Checks', color: '#d7ba7d', icon: 'shield-check' },
	{ type: 'StringFunction', label: 'String Function', category: 'Functions', color: '#c586c0', icon: 'type' },
	{ type: 'ListFunction', label: 'List Function', category: 'Functions', color: '#c586c0', icon: 'list' },
	{ type: 'CryptoFunction', label: 'Crypto Function', category: 'Functions', color: '#c586c0', icon: 'lock' },
	{ type: 'ConversionFunction', label: 'Conversion', category: 'Functions', color: '#c586c0', icon: 'arrow-right-left' },
	{ type: 'DateFunction', label: 'Date Function', category: 'Functions', color: '#c586c0', icon: 'calendar' },
	{ type: 'CookieContainer', label: 'Cookie Container', category: 'Functions', color: '#c586c0', icon: 'cookie' },
	{ type: 'IfElse', label: 'If / Else', category: 'Control', color: '#dcdcaa', icon: 'git-branch' },
	{ type: 'Loop', label: 'Loop', category: 'Control', color: '#dcdcaa', icon: 'repeat' },
	{ type: 'Delay', label: 'Delay', category: 'Control', color: '#dcdcaa', icon: 'clock' },
	{ type: 'CaseSwitch', label: 'Case / Switch', category: 'Control', color: '#dcdcaa', icon: 'list-tree' },
	{ type: 'Script', label: 'Script', category: 'Control', color: '#dcdcaa', icon: 'terminal' },
	{ type: 'Log', label: 'Log', category: 'Utilities', color: '#858585', icon: 'file-text' },
	{ type: 'SetVariable', label: 'Set Variable', category: 'Utilities', color: '#858585', icon: 'variable' },
	{ type: 'ClearCookies', label: 'Clear Cookies', category: 'Utilities', color: '#858585', icon: 'cookie' },
	{ type: 'Webhook', label: 'Webhook', category: 'Utilities', color: '#858585', icon: 'globe' },
	{ type: 'WebSocket', label: 'WebSocket', category: 'Utilities', color: '#858585', icon: 'globe' },
	// Protocol requests
	{ type: 'TcpRequest', label: 'TCP Request', category: 'Requests', color: '#0078d4', icon: 'cable' },
	{ type: 'UdpRequest', label: 'UDP Request', category: 'Requests', color: '#0078d4', icon: 'radio' },
	{ type: 'FtpRequest', label: 'FTP Request', category: 'Requests', color: '#0078d4', icon: 'hard-drive-download' },
	{ type: 'SshRequest', label: 'SSH Request', category: 'Requests', color: '#0078d4', icon: 'terminal' },
	{ type: 'ImapRequest', label: 'IMAP Request', category: 'Requests', color: '#0078d4', icon: 'mail' },
	{ type: 'SmtpRequest', label: 'SMTP Request', category: 'Requests', color: '#0078d4', icon: 'send' },
	{ type: 'PopRequest', label: 'POP Request', category: 'Requests', color: '#0078d4', icon: 'inbox' },
	// Bypass / Anti-bot
	{ type: 'CaptchaSolver', label: 'Captcha Solver', category: 'Bypass', color: '#e5c07b', icon: 'shield' },
	{ type: 'CloudflareBypass', label: 'Cloudflare Bypass', category: 'Bypass', color: '#e5c07b', icon: 'cloud' },
{ type: 'LaravelCsrf', label: 'Laravel CSRF', category: 'Bypass', color: '#e5c07b', icon: 'key' },
	// New blocks
	{ type: 'RandomUserAgent', label: 'Random User Agent', category: 'Utilities', color: '#858585', icon: 'user' },
	{ type: 'OcrCaptcha', label: 'OCR Captcha', category: 'Bypass', color: '#e5c07b', icon: 'scan-eye' },
	{ type: 'RecaptchaInvisible', label: 'reCAPTCHA Invisible', category: 'Bypass', color: '#e5c07b', icon: 'shield-check' },
	{ type: 'XacfSensor', label: 'XACF Sensor', category: 'Sensors', color: '#2dd4bf', icon: 'cpu' },
	{ type: 'DataDomeSensor', label: 'DataDome Sensor', category: 'Sensors', color: '#2dd4bf', icon: 'cpu' },
	{ type: 'RandomData', label: 'Random Data', category: 'Utilities', color: '#858585', icon: 'dices' },
	{ type: 'Plugin', label: 'Plugin Block', category: 'Utilities', color: '#858585', icon: 'plug' },
	{ type: 'Group', label: 'Group', category: 'Control', color: '#dcdcaa', icon: 'folder' },
	// Browser automation
	{ type: 'BrowserOpen', label: 'Browser Open', category: 'Browser', color: '#e06c75', icon: 'monitor' },
	{ type: 'NavigateTo', label: 'Navigate To', category: 'Browser', color: '#e06c75', icon: 'globe' },
	{ type: 'ClickElement', label: 'Click Element', category: 'Browser', color: '#e06c75', icon: 'mouse-pointer-click' },
	{ type: 'TypeText', label: 'Type Text', category: 'Browser', color: '#e06c75', icon: 'keyboard' },
	{ type: 'WaitForElement', label: 'Wait For Element', category: 'Browser', color: '#e06c75', icon: 'hourglass' },
	{ type: 'GetElementText', label: 'Get Element Text', category: 'Browser', color: '#e06c75', icon: 'scan-text' },
	{ type: 'Screenshot', label: 'Screenshot', category: 'Browser', color: '#e06c75', icon: 'camera' },
	{ type: 'ExecuteJs', label: 'Execute JS', category: 'Browser', color: '#e06c75', icon: 'terminal' },
];

export function getBlockCategory(type: BlockType): string {
	return BLOCK_CATALOG.find(b => b.type === type)?.category || 'Utilities';
}

export function getBlockColor(type: BlockType): string {
	return BLOCK_CATALOG.find(b => b.type === type)?.color || '#858585';
}

export function getBlockCssClass(type: BlockType): string {
	const cat = getBlockCategory(type);
	switch (cat) {
		case 'Requests': return 'block-request';
		case 'Parsing': return 'block-parse';
		case 'Checks': return 'block-check';
		case 'Functions': return 'block-function';
		case 'Control': return 'block-control';
		case 'Utilities': return 'block-utility';
		case 'Bypass': return 'block-bypass';
		case 'Browser': return 'block-browser';
		case 'Sensors': return 'block-sensor';
		default: return 'block-utility';
	}
}

// ── Job System ──

export type JobState = 'Queued' | 'Waiting' | 'Running' | 'Paused' | 'Completed' | 'Stopped';

export type DataSourceType = 'File' | 'Url' | 'Inline' | 'Range' | 'Combinations';

export interface DataSource {
	source_type: DataSourceType;
	value: string;
}

export type StartCondition =
	| { Immediate: null }
	| { Delayed: { delay_secs: number } }
	| { Scheduled: { at: string } };

export type HitOutput =
	| { FileSystem: { directory: string; format: 'Txt' | 'Csv' | 'Json' } }
	| { DiscordWebhook: { webhook_url: string; template: string } }
	| { TelegramBot: { bot_token: string; chat_id: string; template: string } }
	| { CustomWebhook: { url: string; method: string; body_template: string } };

export interface Job {
	id: string;
	name: string;
	config_path: string | null;
	pipeline: Pipeline;
	data_source: DataSource;
	proxy_source: { settings: ProxySettings };
	state: JobState;
	start_condition: StartCondition;
	hit_outputs: HitOutput[];
	thread_count: number;
	created: string;
	started: string | null;
	completed: string | null;
	stats: RunnerStats;
}
