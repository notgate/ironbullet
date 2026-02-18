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
