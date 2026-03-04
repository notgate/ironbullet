export interface ChangelogEntry {
	version: string;
	date: string;
	highlights?: string;
	sections: Array<{
		title: string;
		items: string[];
	}>;
}

export const CHANGELOG: ChangelogEntry[] = [
	{
		version: '0.3.5',
		date: '2026-03-04',
		highlights: 'WreqTLS browser emulation backend, UI preferences panel, bottom tab visibility controls, skip-confirmation settings, v0.3.5 release.',
		sections: [
			{
				title: 'New Features',
				items: [
					'WreqTLS backend — third TLS client option on every HTTP Request block. Powered by wreq + BoringSSL with 100+ device emulation profiles (Chrome 100–137, Edge 101–134, Firefox 109–139, Safari 15–18, OkHttp 3–5, Opera 116–119). Accurate JA3/JA4 and HTTP/2 SETTINGS frame emulation with no external sidecar required.',
					'WreqTLS cookie persistence — wreq client is reused across HTTP blocks within a single pipeline run so multi-step login flows correctly share cookies. Rebuilt automatically if the emulation profile changes mid-pipeline.',
					'Browser emulation dropdown in HTTP block settings — select any supported profile when WreqTLS is chosen. Verify fingerprints at tls.peet.ws.',
					'UI & Panels settings section — new settings page with controls for dialog suppression, panel visibility, and density.',
					'Skip unsaved dialog — option to close tabs immediately without the "save changes?" prompt.',
					'Skip app close confirm — option to exit the app immediately even with unsaved tabs.',
					'Bottom panel tab visibility — show/hide individual tabs (Debugger, Code View, Data/Proxy, Jobs, Network, Variables, Inspector) from settings. State persists across restarts.',
					'Compact mode setting — reduces spacing in block list and panels.',
					'Reset panel layout button in settings.',
				],
			},
			{
				title: 'Bug Fixes',
				items: [
					'Config tab save isolation (rework) — openInNewTab now calls syncPipelineToBackend immediately so the backend pipeline_path is set on file open, not just on tab switch. All save callsites now use savePipeline() which always passes the active tab filePath explicitly.',
					'Hits dialog flickering — stable $state decoupled from $derived; live feed avoids intermediate @const bindings that caused re-evaluation on every stats push.',
					'Live results panel removed from job datagrid — moved into Hits dialog as a collapsible live feed section.',
				],
			},
		],
	},
	{
		version: '0.2.3',
		date: '2026-03-02',
		highlights: 'Critical sidecar regression fix, TLS fingerprinting controls, AzureTLS session isolation, RustTLS cookie persistence, Chrome Browser Capture reliability, Date-to-Unix function, startup dependency check.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Critical regression fix — reqflow-sidecar was calling ApplyJa3/ApplyHTTP2 on every HTTP request when a pipeline-level JA3 or HTTP2 fingerprint was configured. These functions rebuild internal TLS state and reset HTTP/2 connection pooling; re-applying unchanged values caused massive latency and up to 52% timeout errors. Fingerprints are now tracked per-session and only re-applied when the value actually changes.',
					'Startup alert dialog title — "Malicious Script Detected" was shown when Chrome or reqflow-sidecar was missing on first launch. The dialog now shows "Missing Dependencies" with an amber accent when all issues are dependency warnings, and reserves the red security alert for actual malicious script detections.',
					'AzureTLS session isolation — cookie jars now reset between credentials. Previously one blocked/captcha\'d account shared its session state with all subsequent checks on the same worker thread, causing false Error results to cascade. Each credential now gets a fresh azuretls session.',
					'RustTLS cookie persistence — reqwest client is now reused across all HTTP blocks within a single pipeline execution so multi-step login flows (GET page → POST credentials → check response) correctly share cookies, matching azuretls session behavior.',
					'Chrome Browser Capture freeze — Chrome is now launched with --no-first-run, --no-default-browser-check, --no-sandbox, --disable-sync and an isolated temporary user-data-dir so it never shows login prompts or first-run setup that blocked the CDP handshake.',
					'Proxy group save dialog (#7) — save_pipeline now uses the existing pipeline_path before opening a file chooser; proxy group changes no longer trigger the save dialog on unsaved configs.',
					'Proxy groups not persisting (#8) — proxy group CRUD auto-saves to the existing file path correctly.',
					'FTP race condition (#11) — full multi-line banner consumed before USER command; stops on 5xx responses.',
					'Chrome not-found error now fires immediately with an install link instead of hanging 20 seconds.',
					'Startup integrity check — missing Chrome and missing reqflow-sidecar are reported in the dependency dialog on first load.',
				],
			},
			{
				title: 'New Features',
				items: [
					'Per-block Browser Profile selector (AzureTLS) — override the TLS + HTTP/2 fingerprint per HTTP Request block: Chrome, Firefox, Safari, Edge. Overrides pipeline-level browser setting for that block only.',
					'Per-block JA3 Override (AzureTLS) — set a custom JA3 fingerprint string on a per-block basis. Verify your fingerprint at tls.peet.ws.',
					'Per-block HTTP/2 Fingerprint Override (AzureTLS) — custom HTTP/2 SETTINGS frame fingerprint per block, independent of pipeline-level setting.',
					'Date to Unix (seconds) — new Date Function variant that parses a date or datetime string using a configurable strftime format and outputs a Unix timestamp in seconds.',
					'Date to Unix (ms) — same as above but outputs Unix milliseconds.',
					'Error logging (#10) — when output saving is enabled, errored accounts are written to ConfigName_Error.txt with the error message under the _error capture key.',
					'RustTLS default (#9) — new HTTP Request blocks default to RustTLS instead of AzureTLS.',
				],
			},
			{
				title: 'TLS & Fingerprinting Notes',
				items: [
					'RustTLS (reqwest + rustls) does NOT support JA3 fingerprinting — cipher suite ordering is fixed by the rustls library and cannot be customized per-request. Use AzureTLS for sites that check TLS fingerprints.',
					'AzureTLS (azuretls via Go sidecar) supports full JA3, HTTP/2 SETTINGS frame, and browser profile TLS imitation. Browser profile sets cipher order, extensions, and elliptic curves to match real browser ClientHello.',
					'Per-block overrides let you mix profiles: use Chrome fingerprint for fingerprint-checked endpoints and RustTLS for everything else in the same config.',
				],
			},
		],
	},
	{
		version: '0.2.0',
		date: '2026-02-18',
		highlights: 'Major feature update: 9 new function blocks, enhanced variable input system, improved help documentation, and bug fixes.',
		sections: [
			{
				title: 'New Function Blocks',
				items: [
					'ByteArray — Binary data manipulation (hex, base64, UTF-8 encoding/decoding)',
					'Constants — Define named constants for pipeline reuse',
					'Dictionary — Key-value operations (get, set, remove, exists, keys, values)',
					'FloatFunction — Floating-point math (round, ceil, floor, abs, arithmetic)',
					'IntegerFunction — Integer operations (add, subtract, multiply, divide, modulo, power)',
					'TimeFunction — Advanced time operations (timezone conversion, DST handling, duration calc)',
					'GenerateGUID — UUID generation (v1 timestamp, v4 random, v5 hash-based)',
					'PhoneCountry — Extract country code from phone numbers (supports multiple formats)',
					'LambdaParser — Parse text using lambda/arrow function expressions',
				],
			},
			{
				title: 'Enhanced Input System',
				items: [
					'Variable Input mode switcher on ALL text inputs across all block settings',
					'Three input modes: RAW (literal), EMBED (interpolation), VAR (dropdown selection)',
					'Support for both <variable> and {{variable}} interpolation syntax',
					'Auto-populated variable dropdown with all pipeline variables',
					'Color-coded mode badges: Amber (RAW), Blue (EMBED), Green (VAR)',
					'Click mode badge to cycle through input modes',
					'Available in HTTP requests, parsers, functions, controls, and all other blocks',
				],
			},
			{
				title: 'Improved Documentation',
				items: [
					'Collapsible help sections in Debug Mode and Jobs guides',
					'Professional technical documentation without emojis',
					'Step-by-step workflows with code examples',
					'Comprehensive pre-job validation checklists',
					'Detailed performance tuning guidelines',
					'Visual state flow diagrams for job lifecycle',
				],
			},
			{
				title: 'UI Enhancements',
				items: [
					'Hits Database panel in Data/Proxy tab',
					'Live results display with auto-refresh toggle',
					'Filter, copy, and delete individual hits',
					'Remove duplicates and clear all functionality',
					'Removed redundant Runner tab for cleaner interface',
					'Hit counter and status indicators',
				],
			},
			{
				title: 'Bug Fixes',
				items: [
					'Fixed sidecar binary name mismatch (ironbullet-sidecar.exe → reqflow-sidecar.exe)',
					'Fixed GenerateGUID V1 to use proper timestamp-based UUID generation',
					'Fixed VariableInput component visibility with standard Tailwind colors',
					'All block parameters now properly passed to Rust code view in debug mode',
					'TimeFunction timezone conversion now works correctly',
					'PhoneCountry output formats properly implemented',
				],
			},
			{
				title: 'Code Generation',
				items: [
					'Full Rust code generation for all 9 new function blocks',
					'Proper parameter usage in generated code',
					'Debug terminal shows exact execution logic',
					'V1 UUID uses timestamp-based generation',
					'V5 UUID uses namespace and name parameters',
					'All function types properly matched in code output',
				],
			},
		],
	},
	{
		version: '0.1.0',
		date: '2026-02-17',
		highlights: 'Initial public release of Ironbullet — visual pipeline builder for HTTP automation.',
		sections: [
			{
				title: 'Pipeline Editor',
				items: [
					'Visual block-based pipeline editor with drag-and-drop',
					'50+ block types: HTTP, parsing, crypto, browser, protocols, bypass',
					'Block search/filter with Ctrl+F and match highlighting',
					'Collapse/expand container blocks (IfElse, Loop, Group)',
					'Pipeline minimap with viewport tracking and click-to-navigate',
					'Multi-select blocks with rubber band selection and Ctrl+Click',
					'Multi-select drag reorder and right-click to group blocks',
					'Block templates — save and reuse block selections',
					'Block diff indicators — orange dot on modified blocks since last save',
					'Variable preview mode — see resolved variable references inline',
					'Inline variable autocomplete when typing <VAR> in inputs',
					'Undo/redo with full block history',
				],
			},
			{
				title: 'Runner & Debugging',
				items: [
					'Multi-threaded runner with configurable thread count',
					'Debug mode with step-through execution and response viewer',
					'Network log with request/response inspection',
					'Proxy support with ban detection and rotation',
					'Proxy health checking',
					'Job manager for queuing multiple runs',
				],
			},
			{
				title: 'Config Management',
				items: [
					'Multi-tab config editing with unsaved change tracking',
					'Save/load .json pipeline configs',
					'Import OpenBullet .svb, .opk, and .loliScript formats',
					'Collections folder for quick access to saved configs',
					'Recent configs list on startup',
					'Security scanner for imported configs',
				],
			},
			{
				title: 'Code Generation & Plugins',
				items: [
					'Export pipeline as standalone Rust code',
					'Plugin system with .dll hot-loading',
					'Built-in Plugin Builder with code generation and compilation',
					'Akamai v3 sensor generation block',
				],
			},
			{
				title: 'UI & Settings',
				items: [
					'Dark theme with skeuomorphic controls',
					'Scalable font size affecting all UI components',
					'Configurable zoom level',
					'Adjustable panel widths and heights',
					'Custom window chrome with native drag support',
					'Auto-updater with in-app download and install',
					'Block documentation panel with guides (F1)',
					'Keyboard shortcuts for all common operations',
				],
			},
		],
	},
];
