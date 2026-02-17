<script lang="ts">
	import { app, getEditingBlock, pushUndo } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import type { Block, KeyCondition, Keychain } from '$lib/types';
	import SkeuSelect from './SkeuSelect.svelte';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import X from '@lucide/svelte/icons/x';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash2 from '@lucide/svelte/icons/trash-2';

	let block = $derived(getEditingBlock());
	let open = $derived(app.editingBlockId !== null);

	function close() {
		app.editingBlockId = null;
	}

	function updateBlock(updates: Partial<Block>) {
		if (!block) return;
		pushUndo();
		const updated = { ...block, ...updates };
		send('update_block', updated);
	}

	function updateSettings(key: string, value: unknown) {
		if (!block) return;
		pushUndo();
		const updated = { ...block, settings: { ...block.settings, [key]: value } };
		send('update_block', updated);
	}

	// --- Variable embed detection ---
	const varPattern = /<[a-zA-Z_][a-zA-Z0-9_.]*>/;
	function hasVars(val: string | undefined): boolean {
		return !!val && varPattern.test(val);
	}

	// --- Shared input class strings ---
	const inputCls = "w-full skeu-input font-mono mt-0.5";
	const labelCls = "text-[10px] uppercase tracking-wider text-muted-foreground";
	const hintCls = "text-[9px] text-muted-foreground/60 mt-0.5";

	function fieldHint(field: string): string {
		if (!block) return '';
		const docs = BLOCK_DOCS[block.settings.type];
		return docs?.fields[field] || '';
	}
	const smallInputCls = "skeu-input text-[10px] font-mono";

	// --- Shared option arrays ---
	const COMPARISON_OPTIONS = [
		{value:'Contains',label:'Contains'},{value:'NotContains',label:'Not Contains'},
		{value:'EqualTo',label:'Equals'},{value:'NotEqualTo',label:'Not Equals'},
		{value:'MatchesRegex',label:'Regex'},{value:'GreaterThan',label:'Greater'},
		{value:'LessThan',label:'Less'},{value:'Exists',label:'Exists'},{value:'NotExists',label:'Not Exists'},
	];

	const CONVERSION_TYPE_OPTIONS = [
		{value:'String',label:'String'},{value:'Int',label:'Integer'},{value:'Float',label:'Float'},
		{value:'Bool',label:'Boolean'},{value:'Hex',label:'Hex'},{value:'Base64',label:'Base64'},
	];

	// --- HTTP Request: header helpers ---
	function headersToRaw(headers: [string, string][]): string {
		return headers.map(([k, v]) => `${k}: ${v}`).join('\n');
	}

	function rawToHeaders(raw: string): [string, string][] {
		return raw.split('\n')
			.map(line => line.trim())
			.filter(line => line.length > 0)
			.map(line => {
				const idx = line.indexOf(':');
				if (idx === -1) return [line, ''] as [string, string];
				return [line.slice(0, idx).trim(), line.slice(idx + 1).trim()] as [string, string];
			});
	}

	let httpTab = $state<'headers' | 'body' | 'cookies' | 'options'>('headers');
	let rawHeaders = $state('');
	let headersViewMode = $state<'raw' | 'kv'>('raw');

	// Sync raw headers from block when block changes
	$effect(() => {
		if (block && block.settings.type === 'HttpRequest') {
			rawHeaders = headersToRaw(block.settings.headers);
		}
	});

	function commitRawHeaders() {
		if (!block || block.settings.type !== 'HttpRequest') return;
		updateSettings('headers', rawToHeaders(rawHeaders));
	}

	function addHeader() {
		if (!block || block.settings.type !== 'HttpRequest') return;
		const headers: [string, string][] = [...block.settings.headers, ['', '']];
		updateSettings('headers', headers);
	}

	function removeHeader(idx: number) {
		if (!block || block.settings.type !== 'HttpRequest') return;
		const headers = block.settings.headers.filter((_: [string, string], i: number) => i !== idx);
		updateSettings('headers', headers);
	}

	function updateHeaderKey(idx: number, key: string) {
		if (!block || block.settings.type !== 'HttpRequest') return;
		const headers: [string, string][] = [...block.settings.headers];
		headers[idx] = [key, headers[idx][1]];
		updateSettings('headers', headers);
	}

	function updateHeaderValue(idx: number, value: string) {
		if (!block || block.settings.type !== 'HttpRequest') return;
		const headers: [string, string][] = [...block.settings.headers];
		headers[idx] = [headers[idx][0], value];
		updateSettings('headers', headers);
	}

	// --- Copy as curl/PowerShell ---
	let copyNotice = $state('');

	function generateCurl(): string {
		if (!block || block.settings.type !== 'HttpRequest') return '';
		const s = block.settings;
		const parts = ['curl'];
		if (s.method !== 'GET') parts.push(`-X ${s.method}`);
		for (const [k, v] of s.headers) {
			parts.push(`-H '${k}: ${v}'`);
		}
		if (s.body && s.body_type !== 'None') {
			parts.push(`-d '${s.body.replace(/'/g, "'\\''")}'`);
		}
		if (!s.follow_redirects) parts.push('-L');
		parts.push(`'${s.url}'`);
		return parts.join(' \\\n  ');
	}

	function generatePowershell(): string {
		if (!block || block.settings.type !== 'HttpRequest') return '';
		const s = block.settings;
		const parts = ['Invoke-WebRequest'];
		parts.push(`-Method ${s.method}`);
		parts.push(`-Uri "${s.url}"`);
		if (s.headers.length > 0) {
			const hdrs = s.headers.map(([k, v]: [string, string]) => `"${k}"="${v}"`).join('; ');
			parts.push(`-Headers @{${hdrs}}`);
		}
		if (s.body && s.body_type !== 'None') {
			parts.push(`-Body "${s.body.replace(/"/g, '`"')}"`);
		}
		if (s.content_type) {
			parts.push(`-ContentType "${s.content_type}"`);
		}
		return parts.join(' `\n  ');
	}

	function copyAsCurl() {
		navigator.clipboard.writeText(generateCurl());
		copyNotice = 'Copied as curl!';
		setTimeout(() => { copyNotice = ''; }, 1500);
	}

	function copyAsPowershell() {
		navigator.clipboard.writeText(generatePowershell());
		copyNotice = 'Copied as PowerShell!';
		setTimeout(() => { copyNotice = ''; }, 1500);
	}

	// --- KeyCheck helpers ---
	function addKeychain() {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains, {
			result: 'Success',
			conditions: [{ source: 'data.RESPONSECODE', comparison: 'EqualTo', value: '200' }]
		}];
		updateSettings('keychains', keychains);
	}

	function removeKeychain(ki: number) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains = block.settings.keychains.filter((_: Keychain, i: number) => i !== ki);
		updateSettings('keychains', keychains);
	}

	function addCondition(ki: number) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains];
		const conditions: KeyCondition[] = [...keychains[ki].conditions, { source: '', comparison: 'Contains', value: '' }];
		keychains[ki] = { ...keychains[ki], conditions };
		updateSettings('keychains', keychains);
	}

	function removeCondition(ki: number, ci: number) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains];
		const conditions = keychains[ki].conditions.filter((_: KeyCondition, i: number) => i !== ci);
		keychains[ki] = { ...keychains[ki], conditions };
		updateSettings('keychains', keychains);
	}

	function updateKeychainResult(ki: number, value: string) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains];
		keychains[ki] = { ...keychains[ki], result: value as any };
		updateSettings('keychains', keychains);
	}

	function updateConditionField(ki: number, ci: number, field: keyof KeyCondition, value: string) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains];
		const conditions: KeyCondition[] = [...keychains[ki].conditions];
		conditions[ci] = { ...conditions[ci], [field]: value };
		keychains[ki] = { ...keychains[ki], conditions };
		updateSettings('keychains', keychains);
	}

	// --- String Function param labels ---
	const STRING_FUNCTIONS = [
		{ value: 'Replace', label: 'Replace', p1: 'Find', p2: 'Replace with' },
		{ value: 'Substring', label: 'Substring', p1: 'Start index', p2: 'Length' },
		{ value: 'Trim', label: 'Trim', p1: '', p2: '' },
		{ value: 'ToUpper', label: 'To Upper', p1: '', p2: '' },
		{ value: 'ToLower', label: 'To Lower', p1: '', p2: '' },
		{ value: 'URLEncode', label: 'URL Encode', p1: '', p2: '' },
		{ value: 'URLDecode', label: 'URL Decode', p1: '', p2: '' },
		{ value: 'Base64Encode', label: 'Base64 Encode', p1: '', p2: '' },
		{ value: 'Base64Decode', label: 'Base64 Decode', p1: '', p2: '' },
		{ value: 'HTMLEncode', label: 'HTML Encode', p1: '', p2: '' },
		{ value: 'HTMLDecode', label: 'HTML Decode', p1: '', p2: '' },
		{ value: 'Split', label: 'Split', p1: 'Separator', p2: '' },
		{ value: 'RandomString', label: 'Random String', p1: 'Length', p2: 'Charset (abc123...)' },
	];

	function getStringFuncMeta(ft: string) {
		return STRING_FUNCTIONS.find(f => f.value === ft) || STRING_FUNCTIONS[0];
	}

	// --- List Function options ---
	const LIST_FUNCTIONS = [
		{ value: 'Join', label: 'Join', param: 'Separator' },
		{ value: 'Sort', label: 'Sort', param: '' },
		{ value: 'Shuffle', label: 'Shuffle', param: '' },
		{ value: 'Add', label: 'Add Item', param: 'Item' },
		{ value: 'Remove', label: 'Remove Item', param: 'Item' },
		{ value: 'Deduplicate', label: 'Deduplicate', param: '' },
		{ value: 'RandomItem', label: 'Random Item', param: '' },
		{ value: 'Length', label: 'Length', param: '' },
	];

	function getListFuncMeta(ft: string) {
		return LIST_FUNCTIONS.find(f => f.value === ft) || LIST_FUNCTIONS[0];
	}

	// --- Crypto Function options ---
	const CRYPTO_FUNCTIONS = [
		{ value: 'MD5', label: 'MD5', needsKey: false },
		{ value: 'SHA1', label: 'SHA-1', needsKey: false },
		{ value: 'SHA256', label: 'SHA-256', needsKey: false },
		{ value: 'SHA512', label: 'SHA-512', needsKey: false },
		{ value: 'SHA384', label: 'SHA-384', needsKey: false },
		{ value: 'CRC32', label: 'CRC32', needsKey: false },
		{ value: 'HMACSHA256', label: 'HMAC-SHA256', needsKey: true },
		{ value: 'HMACSHA512', label: 'HMAC-SHA512', needsKey: true },
		{ value: 'HMACMD5', label: 'HMAC-MD5', needsKey: true },
		{ value: 'BCryptHash', label: 'BCrypt Hash', needsKey: false },
		{ value: 'BCryptVerify', label: 'BCrypt Verify', needsKey: true },
		{ value: 'Base64Encode', label: 'Base64 Encode', needsKey: false },
		{ value: 'Base64Decode', label: 'Base64 Decode', needsKey: false },
		{ value: 'AESEncrypt', label: 'AES Encrypt', needsKey: true },
		{ value: 'AESDecrypt', label: 'AES Decrypt', needsKey: true },
	];

	function getCryptoFuncMeta(ft: string) {
		return CRYPTO_FUNCTIONS.find(f => f.value === ft) || CRYPTO_FUNCTIONS[0];
	}

	const HTTP_VERSION_OPTIONS = [
		{ value: 'HTTP/1.1', label: 'HTTP/1.1' },
		{ value: 'HTTP/2', label: 'HTTP/2' },
		{ value: 'HTTP/3', label: 'HTTP/3' },
	];

	// --- Block documentation / field hints ---
	const BLOCK_DOCS: Record<string, { summary: string; fields: Record<string, string> }> = {
		HttpRequest: {
			summary: 'Sends an HTTP request to a target URL and stores the response.',
			fields: {
				url: 'Target URL. Supports <VAR> interpolation.',
				method: 'HTTP method for the request.',
				headers: 'Custom HTTP headers sent with the request.',
				body: 'Request body content (for POST, PUT, PATCH).',
				follow_redirects: 'Automatically follow 3xx redirects.',
				timeout_ms: 'Max time to wait for a response in milliseconds.',
				http_version: 'HTTP protocol version to use.',
				response_var: 'Variable name to store the response body.',
			}
		},
		ParseLR: {
			summary: 'Extracts text between left and right delimiters from a string.',
			fields: {
				input_var: 'Source variable containing the text to parse.',
				left: 'String that appears before the target text.',
				right: 'String that appears after the target text.',
				output_var: 'Variable name to store the extracted value.',
				recursive: 'Find all matches instead of just the first.',
				capture: 'Save the result to the hit output.',
			}
		},
		ParseJSON: {
			summary: 'Extracts a value from JSON using a dot-notation path.',
			fields: {
				input_var: 'Variable containing JSON string.',
				json_path: 'Dot-notation path (e.g. user.token, items[0].id).',
				output_var: 'Variable name to store the extracted value.',
			}
		},
		ParseRegex: {
			summary: 'Extracts text using a regular expression pattern.',
			fields: {
				input_var: 'Variable containing the text to search.',
				pattern: 'Regular expression pattern with capture groups.',
				output_format: 'Format string using $1, $2, etc. for groups.',
				output_var: 'Variable to store the result.',
			}
		},
		ParseCSS: {
			summary: 'Extracts text or attributes from HTML using CSS selectors.',
			fields: {
				selector: 'CSS selector (e.g. div.class > a, #id).',
				attribute: 'HTML attribute to extract. Empty = text content.',
				index: 'Element index (0-based). -1 extracts all matches.',
			}
		},
		ParseXPath: {
			summary: 'Extracts data from HTML/XML using XPath expressions.',
			fields: { xpath: 'XPath expression (e.g. //div[@class="result"]/text()).' }
		},
		ParseCookie: {
			summary: 'Extracts a specific cookie value from the response cookies.',
			fields: { cookie_name: 'Name of the cookie to extract.' }
		},
		KeyCheck: {
			summary: 'Checks conditions against variables to set the bot status (Success, Fail, Ban, etc.).',
			fields: {
				keychains: 'Each keychain defines conditions that, when ALL match, set a result status.',
				source: 'Variable to check (e.g. data.SOURCE, data.RESPONSECODE).',
				comparison: 'How to compare the source against the value.',
				value: 'Expected value to compare against.',
			}
		},
		StringFunction: {
			summary: 'Performs a string operation (replace, trim, encode, etc.) on a variable.',
			fields: {
				function_type: 'String operation to perform.',
				input_var: 'Variable containing the input string.',
			}
		},
		ListFunction: {
			summary: 'Performs a list operation (join, sort, add, remove, etc.).',
			fields: { input_var: 'Variable containing the list (prefix with @).' }
		},
		CryptoFunction: {
			summary: 'Hashes or encrypts data using the selected algorithm.',
			fields: { key: 'Secret key required for HMAC and AES operations.' }
		},
		ConversionFunction: {
			summary: 'Converts a variable between types (string, int, hex, base64, etc.).',
			fields: {}
		},
		IfElse: {
			summary: 'Branches execution based on a condition. Blocks in the True branch run when the condition matches.',
			fields: { condition: 'Source variable, comparison operator, and expected value.' }
		},
		Loop: {
			summary: 'Repeats nested blocks for each item in a list or a fixed number of times.',
			fields: {
				list_var: 'Variable containing the list to iterate (prefix with @).',
				item_var: 'Variable name for the current item in each iteration.',
				count: 'Number of times to repeat the loop body.',
			}
		},
		Delay: {
			summary: 'Pauses execution for a random duration between min and max milliseconds.',
			fields: {}
		},
		Script: {
			summary: 'Executes custom JavaScript code with access to pipeline variables.',
			fields: { code: 'JavaScript code to execute. Use return to output a value.' }
		},
		Log: {
			summary: 'Writes a message to the debug log. Useful for inspecting variable values.',
			fields: { message: 'Log text. Use <VAR> for variable values.' }
		},
		SetVariable: {
			summary: 'Creates or updates a variable with a specific value.',
			fields: { name: 'Variable name to set.', value: 'Value to assign.' }
		},
		ClearCookies: {
			summary: 'Clears all cookies stored in the current session. No parameters needed.',
			fields: {}
		},
		Webhook: {
			summary: 'Sends data to an external webhook URL (e.g. Discord, Slack).',
			fields: {
				url: 'Webhook endpoint URL.',
				body_template: 'JSON body template. Use <VAR> for variable values.',
			}
		},
		WebSocket: {
			summary: 'Manages WebSocket connections: connect, send, receive, or close.',
			fields: { action: 'WebSocket operation to perform.' }
		},
		TcpRequest: {
			summary: 'Sends raw data over a TCP connection and reads the response.',
			fields: { use_tls: 'Encrypt the connection with TLS/SSL.' }
		},
		UdpRequest: {
			summary: 'Sends a UDP datagram and optionally reads a response.',
			fields: {}
		},
		FtpRequest: { summary: 'Connects to an FTP server and executes a command.', fields: {} },
		SshRequest: { summary: 'Connects to an SSH server and executes a remote command.', fields: {} },
		ImapRequest: { summary: 'Connects to an IMAP mail server to check or fetch email.', fields: {} },
		SmtpRequest: { summary: 'Connects to an SMTP server to send email.', fields: {} },
		PopRequest: { summary: 'Connects to a POP3 mail server to retrieve email.', fields: {} },
		CaptchaSolver: {
			summary: 'Solves a captcha using a third-party solver service API.',
			fields: {
				solver_service: 'Captcha solving service provider.',
				captcha_type: 'Type of captcha on the target page.',
				api_key: 'Your API key for the solver service.',
				site_key: 'The captcha site key from the target page HTML.',
				page_url: 'URL of the page containing the captcha.',
			}
		},
		CloudflareBypass: {
			summary: 'Bypasses Cloudflare protection using a FlareSolverr instance.',
			fields: {
				url: 'URL of the Cloudflare-protected page.',
				flaresolverr_url: 'Local FlareSolverr API endpoint.',
			}
		},
		LaravelCsrf: {
			summary: 'Fetches a Laravel CSRF token from a page for form submissions.',
			fields: {
				csrf_selector: 'CSS selector for the hidden CSRF input field.',
				cookie_name: 'Name of the CSRF cookie.',
			}
		},
		BrowserOpen: {
			summary: 'Opens a headless or visible browser instance for automation.',
			fields: {
				browser_type: 'Browser engine to use.',
				headless: 'Run without visible browser window.',
				extra_args: 'Additional command-line flags for the browser.',
			}
		},
		NavigateTo: {
			summary: 'Navigates the browser to a URL and waits for the page to load.',
			fields: { wait_until: 'When to consider the page fully loaded.' }
		},
		ClickElement: {
			summary: 'Clicks an element on the page matching the CSS selector.',
			fields: { wait_for_navigation: 'Wait for a page navigation after clicking.' }
		},
		TypeText: {
			summary: 'Types text into an input field with optional keystroke delay.',
			fields: {
				clear_first: 'Clear the field before typing new text.',
				delay_ms: 'Delay between keystrokes in milliseconds.',
			}
		},
		WaitForElement: {
			summary: 'Waits for an element to appear, disappear, or change state.',
			fields: { state: 'Element state to wait for.' }
		},
		GetElementText: {
			summary: 'Reads text content or an attribute from a page element.',
			fields: { attribute: 'HTML attribute to read. Empty = inner text.' }
		},
		Screenshot: {
			summary: 'Takes a screenshot of the page or a specific element.',
			fields: { full_page: 'Capture the entire scrollable page.' }
		},
		ExecuteJs: {
			summary: 'Executes JavaScript code in the browser page context.',
			fields: { code: 'JS code to run. Use return to capture a value.' }
		},
		DateFunction: {
			summary: 'Performs date/time operations: get current time, format, parse, or add/subtract time.',
			fields: {
				function_type: 'Date operation to perform.',
				input_var: 'Variable containing the date string to process.',
				format: 'Date format string using strftime syntax (e.g. %Y-%m-%d %H:%M:%S).',
				amount: 'Number of time units to add or subtract.',
				unit: 'Time unit for add/subtract operations.',
			}
		},
		CaseSwitch: {
			summary: 'Maps an input value to a result using case matching. Like a switch/case statement.',
			fields: {
				input_var: 'Variable to match against each case value.',
				cases: 'List of value → result mappings checked in order.',
				default_value: 'Result when no case matches the input.',
				output_var: 'Variable to store the matched result.',
			}
		},
		CookieContainer: {
			summary: 'Reads cookies from a file (Netscape format) or raw text, optionally filters by domain, and stores them in a variable. Based on OpenBullet Cookie Edition.',
			fields: {
				source: 'File path (if source_type=file) or raw cookie text.',
				source_type: 'Whether to read from a "file" or use "text" directly.',
				domain: 'Domain filter — only include cookies matching this domain.',
				output_var: 'Variable to store cookies as name=value; pairs.',
				save_netscape: 'Also store in Netscape format as {output_var}_NETSCAPE.',
			}
		},
		Group: {
			summary: 'Organizational container that groups blocks together. Child blocks execute sequentially. Toggle collapsed to show/hide contents.',
			fields: {
				collapsed: 'Whether the group is visually collapsed in the editor.',
			}
		},
	};
</script>

{#if open && block}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="w-[360px] shrink-0 bg-surface border-l border-border flex flex-col slide-in-right"
		onclick={(e) => e.stopPropagation()}
	>
		<!-- Header -->
		<div class="px-3 py-2 border-b border-border shrink-0 panel-raised flex items-center justify-between">
			<div class="flex items-center gap-2 min-w-0">
				<span class="text-sm font-medium text-foreground truncate">{block.label}</span>
				<span class="text-[9px] uppercase tracking-wider text-muted-foreground bg-background px-1.5 py-px rounded border border-border shrink-0">{block.block_type === 'Plugin' && block.settings?.plugin_block_type ? block.settings.plugin_block_type : block.block_type}</span>
			</div>
			<div class="flex items-center gap-1 shrink-0 ml-2">
				<button
					class="flex items-center gap-0.5 px-1.5 py-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors"
					onclick={() => { app.blockDocsInitialType = block.settings.type; app.showBlockDocs = true; }}
					title="Documentation (F1)"
				>
					<span class="text-[10px] font-semibold">?</span>
					<span class="text-[9px]">Docs</span>
				</button>
				<button
					class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-foreground"
					onclick={close}
					title="Close"
				>
					<X size={14} />
				</button>
			</div>
		</div>

		<!-- Block options: disabled / safe mode -->
		<div class="flex gap-3 px-3 py-1.5 border-b border-border bg-surface shrink-0">
			<label class="flex items-center gap-1 text-[10px] text-muted-foreground">
				<input type="checkbox" checked={block.disabled} onchange={() => updateBlock({ disabled: !block!.disabled })} class="skeu-checkbox" />
				Disabled
			</label>
			<label class="flex items-center gap-1 text-[10px] text-muted-foreground">
				<input type="checkbox" checked={block.safe_mode} onchange={() => updateBlock({ safe_mode: !block!.safe_mode })} class="skeu-checkbox" />
				Safe Mode
			</label>
		</div>

		{#snippet embedBadge(val: string | undefined)}
			{#if hasVars(val)}
				<span class="absolute top-0.5 right-1 text-[8px] uppercase tracking-wider font-semibold text-primary/80 bg-primary/10 px-1 py-px rounded select-none pointer-events-none z-10">embed</span>
			{/if}
		{/snippet}

		<!-- Settings body -->
		<div class="flex-1 overflow-y-auto p-2 space-y-1.5 panel-inset">

			<!-- Block documentation summary -->
			{#if BLOCK_DOCS[block.settings.type]}
				<p class="text-[10px] text-muted-foreground/70 italic leading-snug pb-1 border-b border-border/50 mb-1">{BLOCK_DOCS[block.settings.type].summary}</p>
			{/if}

			<!-- ===================== HTTP REQUEST ===================== -->
			{#if block.settings.type === 'HttpRequest'}
				<!-- URL bar -->
				<div class="flex gap-1.5">
					<SkeuSelect
						value={block.settings.method}
						onValueChange={(v) => updateSettings('method', v)}
						options={[{value:'GET',label:'GET'},{value:'POST',label:'POST'},{value:'PUT',label:'PUT'},{value:'DELETE',label:'DELETE'},{value:'PATCH',label:'PATCH'},{value:'HEAD',label:'HEAD'},{value:'OPTIONS',label:'OPTIONS'}]}
						class="text-[11px] w-20"
					/>
					<div class="relative flex-1">
						<input type="text" value={block.settings.url} placeholder="https://example.com/api/endpoint"
							class="w-full skeu-input text-[11px] font-mono"
							oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.url)}
					</div>
				</div>

				<!-- Tabbed sub-panel -->
				<div class="flex border-b border-border">
					{#each ['headers', 'body', 'cookies', 'options'] as tab}
						<button
							class="px-2 py-0.5 text-[11px] capitalize {httpTab === tab ? 'text-foreground font-medium' : 'text-muted-foreground hover:text-foreground'} transition-colors"
							onclick={() => { httpTab = tab as typeof httpTab; }}
						>{tab}</button>
					{/each}
				</div>

				{#if httpTab === 'headers'}
					<!-- View mode toggle -->
					<div class="flex items-center gap-1 mb-1">
						<button
							class="text-[10px] px-1.5 py-0.5 rounded {headersViewMode === 'raw' ? 'bg-secondary text-foreground' : 'text-muted-foreground hover:text-foreground'}"
							onclick={() => { headersViewMode = 'raw'; }}
						>Raw</button>
						<button
							class="text-[10px] px-1.5 py-0.5 rounded {headersViewMode === 'kv' ? 'bg-secondary text-foreground' : 'text-muted-foreground hover:text-foreground'}"
							onclick={() => { headersViewMode = 'kv'; if (block && block.settings.type === 'HttpRequest') commitRawHeaders(); }}
						>Key-Value</button>
					</div>

					{#if headersViewMode === 'raw'}
						<div class="relative">
							<textarea
								class="w-full skeu-input text-[11px] font-mono min-h-[120px] resize-y"
								placeholder="Content-Type: application/json&#10;Accept: */*&#10;Authorization: Bearer <token>"
								bind:value={rawHeaders}
								onblur={commitRawHeaders}
							></textarea>
							{@render embedBadge(rawHeaders)}
						</div>
						<p class="text-[10px] text-muted-foreground">One header per line: <code class="text-foreground/70">Name: Value</code></p>
					{:else}
						<!-- Key-Value pairs -->
						<div class="space-y-1">
							{#each block.settings.headers as header, hi}
								<div class="flex gap-1 items-center">
									<input type="text" value={header[0]} placeholder="Header name"
										class="flex-1 skeu-input text-[10px] font-mono"
										oninput={(e) => updateHeaderKey(hi, (e.target as HTMLInputElement).value)} />
									<input type="text" value={header[1]} placeholder="Value"
										class="flex-1 skeu-input text-[10px] font-mono"
										oninput={(e) => updateHeaderValue(hi, (e.target as HTMLInputElement).value)} />
									<button class="p-0.5 text-muted-foreground hover:text-red shrink-0" onclick={() => removeHeader(hi)} title="Remove">
										<Trash2 size={10} />
									</button>
								</div>
							{/each}
						</div>
						<button class="flex items-center gap-1 text-[10px] text-primary hover:underline mt-1" onclick={addHeader}>
							<Plus size={10} /> Add Header
						</button>
					{/if}
				{:else if httpTab === 'body'}
					<div class="flex items-center gap-2 mb-1">
						<span class={labelCls}>Mode</span>
						<SkeuSelect value={block.settings.body_type}
							onValueChange={(v) => updateSettings('body_type', v)}
							options={[{value:'None',label:'None'},{value:'Standard',label:'Form'},{value:'Raw',label:'Raw'},{value:'Multipart',label:'Multipart'}]}
							class="text-[10px]"
						/>
					</div>
					{#if block.settings.body_type !== 'None'}
						<div class="relative">
							<textarea value={block.settings.body} placeholder="Request body..."
								class="w-full skeu-input text-[11px] font-mono min-h-[100px] resize-y"
								oninput={(e) => updateSettings('body', (e.target as HTMLTextAreaElement).value)}></textarea>
							{@render embedBadge(block.settings.body)}
						</div>
					{/if}
				{:else if httpTab === 'cookies'}
					<p class={hintCls}>Custom cookies to send with the request. One per line: <code class="text-foreground/70">name=value</code></p>
					<div class="relative">
						<textarea
							value={block.settings.custom_cookies || ''}
							placeholder="session_id=abc123&#10;csrf_token=xyz789&#10;auth=<COOKIES>"
							class="w-full skeu-input text-[11px] font-mono min-h-[100px] resize-y mt-1"
							oninput={(e) => updateSettings('custom_cookies', (e.target as HTMLTextAreaElement).value)}
						></textarea>
						{@render embedBadge(block.settings.custom_cookies)}
					</div>
					<p class="text-[9px] text-muted-foreground mt-1">Cookies are joined with <code class="text-foreground/70">; </code> and sent as the <code class="text-foreground/70">Cookie</code> header. Supports <code class="text-foreground/70">&lt;VARIABLE&gt;</code> interpolation.</p>
				{:else if httpTab === 'options'}
					<div class="space-y-1.5">
						<label class="flex items-center gap-2 text-[11px] text-foreground">
							<input type="checkbox" checked={block.settings.follow_redirects} onchange={() => updateSettings('follow_redirects', !block!.settings.follow_redirects)} class="skeu-checkbox" />
							Follow redirects (auto-follow 3xx)
						</label>
						<div class="flex items-center gap-2">
							<span class="text-[11px] text-muted-foreground">Timeout:</span>
							<input type="number" value={block.settings.timeout_ms}
								class="w-20 skeu-input text-[11px]"
								oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} />
							<span class="text-[10px] text-muted-foreground">ms</span>
						</div>
						<div class="flex items-center gap-2">
							<span class="text-[11px] text-muted-foreground">HTTP Version:</span>
							<SkeuSelect
								value={block.settings.http_version || 'HTTP/1.1'}
								onValueChange={(v) => updateSettings('http_version', v)}
								options={HTTP_VERSION_OPTIONS}
								class="text-[10px]"
							/>
						</div>
						<div class="my-1.5 groove-h h-px"></div>
						<div>
							<label class={labelCls}>Response variable</label>
							<input type="text" value={block.settings.response_var || 'SOURCE'} placeholder="SOURCE"
								class={inputCls}
								oninput={(e) => updateSettings('response_var', (e.target as HTMLInputElement).value)} />
							<p class="text-[9px] text-muted-foreground mt-0.5">Body → <code class="text-foreground/70">data.{'{'}var{'}'}</code> &nbsp; Headers → <code class="text-foreground/70">data.{'{'}var{'}'}.HEADERS</code> &nbsp; Cookies → <code class="text-foreground/70">data.{'{'}var{'}'}.COOKIES</code></p>
						</div>
					</div>
				{/if}

				<!-- Copy as curl/PowerShell -->
				<div class="my-1.5 groove-h h-px"></div>
				<div class="flex gap-1.5">
					<button class="skeu-btn text-[10px] flex-1" onclick={copyAsCurl}>Copy as curl</button>
					<button class="skeu-btn text-[10px] flex-1" onclick={copyAsPowershell}>Copy as PowerShell</button>
				</div>
				{#if copyNotice}
					<div class="text-[9px] text-green text-center">{copyNotice}</div>
				{/if}

			<!-- ===================== PARSE LR ===================== -->
			{:else if block.settings.type === 'ParseLR'}
				<div class="space-y-1.5">
					<div class="relative">
						<label class={labelCls}>Input variable</label>
						<input type="text" value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.input_var)}
					</div>
					<div class="relative">
						<label class={labelCls}>Left delimiter</label>
						<input type="text" value={block.settings.left} class={inputCls}
							oninput={(e) => updateSettings('left', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.left)}
					</div>
					<div class="relative">
						<label class={labelCls}>Right delimiter</label>
						<input type="text" value={block.settings.right} class={inputCls}
							oninput={(e) => updateSettings('right', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.right)}
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
					<label class="flex items-center gap-2 text-[11px] text-foreground">
						<input type="checkbox" checked={block.settings.recursive} onchange={() => updateSettings('recursive', !block!.settings.recursive)} class="skeu-checkbox" />
						Recursive
					</label>
					<label class="flex items-center gap-2 text-[11px] text-foreground">
						<input type="checkbox" checked={block.settings.case_insensitive} onchange={() => updateSettings('case_insensitive', !block!.settings.case_insensitive)} class="skeu-checkbox" />
						Case insensitive
					</label>
				</div>

			<!-- ===================== PARSE JSON ===================== -->
			{:else if block.settings.type === 'ParseJSON'}
				<div class="space-y-1.5">
					<div class="relative">
						<label class={labelCls}>Input variable</label>
						<input type="text" value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.input_var)}
					</div>
					<div class="relative">
						<label class={labelCls}>JSON Path</label>
						<input type="text" value={block.settings.json_path} placeholder="e.g. user.token" class={inputCls}
							oninput={(e) => updateSettings('json_path', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.json_path)}
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== PARSE REGEX ===================== -->
			{:else if block.settings.type === 'ParseRegex'}
				<div class="space-y-1.5">
					<div class="relative">
						<label class={labelCls}>Input variable</label>
						<input type="text" value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.input_var)}
					</div>
					<div class="relative">
						<label class={labelCls}>Pattern</label>
						<input type="text" value={block.settings.pattern} class={inputCls}
							oninput={(e) => updateSettings('pattern', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.pattern)}
					</div>
					<div>
						<label class={labelCls}>Output format</label>
						<input type="text" value={block.settings.output_format} placeholder="$1" class={inputCls}
							oninput={(e) => updateSettings('output_format', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
					<label class="flex items-center gap-2 text-[11px] text-foreground">
						<input type="checkbox" checked={block.settings.multi_line} onchange={() => updateSettings('multi_line', !block!.settings.multi_line)} class="skeu-checkbox" />
						Multi-line mode
					</label>
				</div>

			<!-- ===================== PARSE CSS ===================== -->
			{:else if block.settings.type === 'ParseCSS'}
				<div class="space-y-1.5">
					<div class="relative">
						<label class={labelCls}>Input variable</label>
						<input type="text" value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.input_var)}
					</div>
					<div class="relative">
						<label class={labelCls}>CSS Selector</label>
						<input type="text" value={block.settings.selector} placeholder="div.content > a" class={inputCls}
							oninput={(e) => updateSettings('selector', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.selector)}
					</div>
					<div>
						<label class={labelCls}>Attribute (empty = text content)</label>
						<input type="text" value={block.settings.attribute} placeholder="href" class={inputCls}
							oninput={(e) => updateSettings('attribute', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Element index (0-based, -1 = all)</label>
						<input type="number" value={block.settings.index}
							class="w-20 skeu-input text-[11px] mt-0.5"
							oninput={(e) => updateSettings('index', parseInt((e.target as HTMLInputElement).value))} />
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== PARSE XPATH ===================== -->
			{:else if block.settings.type === 'ParseXPath'}
				<div class="space-y-1.5">
					<div class="relative">
						<label class={labelCls}>Input variable</label>
						<input type="text" value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.input_var)}
					</div>
					<div class="relative">
						<label class={labelCls}>XPath expression</label>
						<input type="text" value={block.settings.xpath} placeholder="//div[@class='result']/text()" class={inputCls}
							oninput={(e) => updateSettings('xpath', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.xpath)}
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== PARSE COOKIE ===================== -->
			{:else if block.settings.type === 'ParseCookie'}
				<div class="space-y-1.5">
					<div class="relative">
						<label class={labelCls}>Input variable</label>
						<input type="text" value={block.settings.input_var} placeholder="data.SOURCE.COOKIES" class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.input_var)}
					</div>
					<div class="relative">
						<label class={labelCls}>Cookie name</label>
						<input type="text" value={block.settings.cookie_name} placeholder="session_id" class={inputCls}
							oninput={(e) => updateSettings('cookie_name', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.cookie_name)}
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== KEYCHECK ===================== -->
			{:else if block.settings.type === 'KeyCheck'}
				<div class="space-y-1.5">
					<div class="flex items-center justify-between">
						<span class={labelCls}>Keychains</span>
						<button class="text-[10px] text-primary hover:underline" onclick={addKeychain}>+ Add Keychain</button>
					</div>
					{#each block.settings.keychains as keychain, ki}
						<div class="bg-background rounded p-2 border border-border overflow-hidden">
							<div class="flex items-center gap-2 mb-1.5">
								<span class="text-[10px] text-muted-foreground">Result:</span>
								<SkeuSelect value={keychain.result}
									onValueChange={(v) => updateKeychainResult(ki, v)}
									options={[{value:'Success',label:'SUCCESS'},{value:'Fail',label:'FAIL'},{value:'Ban',label:'BAN'},{value:'Retry',label:'RETRY'},{value:'Custom',label:'CUSTOM'}]}
									class="text-[10px]"
								/>
								<div class="flex-1"></div>
								<button class="text-[10px] text-red hover:underline" onclick={() => removeKeychain(ki)}>Remove</button>
							</div>
							{#each keychain.conditions as cond, ci}
								<div class="flex gap-1 mb-1 items-center min-w-0">
									<div class="relative flex-1 min-w-0">
										<input type="text" value={cond.source} placeholder="data.SOURCE" class="w-full {smallInputCls}"
											oninput={(e) => updateConditionField(ki, ci, 'source', (e.target as HTMLInputElement).value)} />
										{@render embedBadge(cond.source)}
									</div>
									<SkeuSelect value={cond.comparison}
										onValueChange={(v) => updateConditionField(ki, ci, 'comparison', v)}
										options={COMPARISON_OPTIONS}
										class="text-[10px] shrink-0"
									/>
									<div class="relative flex-1 min-w-0">
										<input type="text" value={cond.value} placeholder="value" class="w-full {smallInputCls}"
											oninput={(e) => updateConditionField(ki, ci, 'value', (e.target as HTMLInputElement).value)} />
										{@render embedBadge(cond.value)}
									</div>
									<button class="text-muted-foreground hover:text-red px-0.5 shrink-0" onclick={() => removeCondition(ki, ci)}>x</button>
								</div>
							{/each}
							<button class="text-[10px] text-primary hover:underline mt-1" onclick={() => addCondition(ki)}>+ Add Condition</button>
						</div>
					{/each}
				</div>

			<!-- ===================== STRING FUNCTION ===================== -->
			{:else if block.settings.type === 'StringFunction'}
				{@const meta = getStringFuncMeta(block.settings.function_type)}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Function</label>
						<SkeuSelect value={block.settings.function_type}
							onValueChange={(v) => updateSettings('function_type', v)}
							options={STRING_FUNCTIONS.map(f => ({value: f.value, label: f.label}))}
							class="w-full mt-0.5"
						/>
					</div>
					<div>
						<label class={labelCls}>Input variable</label>
						<input type="text" value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
					</div>
					{#if meta.p1}
						<div>
							<label class={labelCls}>{meta.p1}</label>
							<input type="text" value={block.settings.param1} class={inputCls}
								oninput={(e) => updateSettings('param1', (e.target as HTMLInputElement).value)} />
						</div>
					{/if}
					{#if meta.p2}
						<div>
							<label class={labelCls}>{meta.p2}</label>
							<input type="text" value={block.settings.param2} class={inputCls}
								oninput={(e) => updateSettings('param2', (e.target as HTMLInputElement).value)} />
						</div>
					{/if}
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== LIST FUNCTION ===================== -->
			{:else if block.settings.type === 'ListFunction'}
				{@const meta = getListFuncMeta(block.settings.function_type)}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Function</label>
						<SkeuSelect value={block.settings.function_type}
							onValueChange={(v) => updateSettings('function_type', v)}
							options={LIST_FUNCTIONS.map(f => ({value: f.value, label: f.label}))}
							class="w-full mt-0.5"
						/>
					</div>
					<div>
						<label class={labelCls}>Input variable (list)</label>
						<input type="text" value={block.settings.input_var} placeholder="@myList" class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
					</div>
					{#if meta.param}
						<div>
							<label class={labelCls}>{meta.param}</label>
							<input type="text" value={block.settings.param1} class={inputCls}
								oninput={(e) => updateSettings('param1', (e.target as HTMLInputElement).value)} />
						</div>
					{/if}
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== CRYPTO FUNCTION ===================== -->
			{:else if block.settings.type === 'CryptoFunction'}
				{@const meta = getCryptoFuncMeta(block.settings.function_type)}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Algorithm</label>
						<SkeuSelect value={block.settings.function_type}
							onValueChange={(v) => updateSettings('function_type', v)}
							options={CRYPTO_FUNCTIONS.map(f => ({value: f.value, label: f.label}))}
							class="w-full mt-0.5"
						/>
					</div>
					<div>
						<label class={labelCls}>Input variable</label>
						<input type="text" value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
					</div>
					{#if meta.needsKey}
						<div>
							<label class={labelCls}>Key / Secret</label>
							<input type="text" value={block.settings.key} class={inputCls}
								oninput={(e) => updateSettings('key', (e.target as HTMLInputElement).value)} />
						</div>
					{/if}
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== CONVERSION FUNCTION ===================== -->
			{:else if block.settings.type === 'ConversionFunction'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Input variable</label>
						<input type="text" value={block.settings.input_var} class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="flex gap-2 items-end">
						<div class="flex-1">
							<label class={labelCls}>From type</label>
							<SkeuSelect value={block.settings.from_type}
								onValueChange={(v) => updateSettings('from_type', v)}
								options={CONVERSION_TYPE_OPTIONS}
								class="w-full mt-0.5"
							/>
						</div>
						<div class="flex items-center pb-1 text-muted-foreground">
							<ArrowRight size={14} />
						</div>
						<div class="flex-1">
							<label class={labelCls}>To type</label>
							<SkeuSelect value={block.settings.to_type}
								onValueChange={(v) => updateSettings('to_type', v)}
								options={CONVERSION_TYPE_OPTIONS}
								class="w-full mt-0.5"
							/>
						</div>
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== IF/ELSE ===================== -->
			{:else if block.settings.type === 'IfElse'}
				<div class="space-y-1.5">
					<span class={labelCls}>Condition</span>
					<div class="flex gap-1">
						<input type="text" value={block.settings.condition.source} placeholder="data.SOURCE" class="flex-1 {smallInputCls}"
							oninput={(e) => updateSettings('condition', { ...block!.settings.condition, source: (e.target as HTMLInputElement).value })} />
						<SkeuSelect value={block.settings.condition.comparison}
							onValueChange={(v) => updateSettings('condition', { ...block!.settings.condition, comparison: v })}
							options={COMPARISON_OPTIONS}
							class="text-[10px]"
						/>
						<input type="text" value={block.settings.condition.value} placeholder="value" class="flex-1 {smallInputCls}"
							oninput={(e) => updateSettings('condition', { ...block!.settings.condition, value: (e.target as HTMLInputElement).value })} />
					</div>
					<div class="border border-border rounded p-2 bg-background">
						<span class="text-[10px] text-green-400 uppercase">True branch</span>
						<p class="text-[10px] text-muted-foreground mt-1">
							{block.settings.true_blocks.length} block{block.settings.true_blocks.length !== 1 ? 's' : ''}
						</p>
					</div>
					<div class="border border-border rounded p-2 bg-background">
						<span class="text-[10px] text-red uppercase">False branch</span>
						<p class="text-[10px] text-muted-foreground mt-1">
							{block.settings.false_blocks.length} block{block.settings.false_blocks.length !== 1 ? 's' : ''}
						</p>
					</div>
				</div>

			<!-- ===================== LOOP ===================== -->
			{:else if block.settings.type === 'Loop'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Loop type</label>
						<SkeuSelect value={block.settings.loop_type}
							onValueChange={(v) => updateSettings('loop_type', v)}
							options={[{value:'ForEach',label:'For Each'},{value:'Repeat',label:'Repeat N times'}]}
							class="w-full mt-0.5"
						/>
					</div>
					{#if block.settings.loop_type === 'ForEach'}
						<div>
							<label class={labelCls}>List variable</label>
							<input type="text" value={block.settings.list_var} placeholder="@myList" class={inputCls}
								oninput={(e) => updateSettings('list_var', (e.target as HTMLInputElement).value)} />
						</div>
						<div>
							<label class={labelCls}>Item variable name</label>
							<input type="text" value={block.settings.item_var} placeholder="@item" class={inputCls}
								oninput={(e) => updateSettings('item_var', (e.target as HTMLInputElement).value)} />
						</div>
					{:else}
						<div>
							<label class={labelCls}>Repeat count</label>
							<input type="number" value={block.settings.count} min="1"
								class="w-24 skeu-input text-[11px] mt-0.5"
								oninput={(e) => updateSettings('count', parseInt((e.target as HTMLInputElement).value))} />
						</div>
					{/if}
					<div class="border border-border rounded p-2 bg-background">
						<span class="text-[10px] text-yellow-400 uppercase">Loop body</span>
						<p class="text-[10px] text-muted-foreground mt-1">
							{block.settings.blocks.length} block{block.settings.blocks.length !== 1 ? 's' : ''}
						</p>
					</div>
				</div>

			<!-- ===================== DELAY ===================== -->
			{:else if block.settings.type === 'Delay'}
				<div class="flex gap-2">
					<div>
						<label class={labelCls}>Min (ms)</label>
						<input type="number" value={block.settings.min_ms} class="w-24 skeu-input text-[11px] mt-0.5"
							oninput={(e) => updateSettings('min_ms', parseInt((e.target as HTMLInputElement).value))} />
					</div>
					<div>
						<label class={labelCls}>Max (ms)</label>
						<input type="number" value={block.settings.max_ms} class="w-24 skeu-input text-[11px] mt-0.5"
							oninput={(e) => updateSettings('max_ms', parseInt((e.target as HTMLInputElement).value))} />
					</div>
				</div>

			<!-- ===================== SCRIPT ===================== -->
			{:else if block.settings.type === 'Script'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Script code</label>
						<textarea value={block.settings.code} placeholder="// JavaScript code here..."
							class="w-full skeu-input text-[11px] font-mono min-h-[120px] resize-y mt-0.5"
							oninput={(e) => updateSettings('code', (e.target as HTMLTextAreaElement).value)}></textarea>
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== LOG ===================== -->
			{:else if block.settings.type === 'Log'}
				<div class="relative">
					<label class={labelCls}>Message</label>
					<input type="text" value={block.settings.message} placeholder="Log message with <variables>" class={inputCls}
						oninput={(e) => updateSettings('message', (e.target as HTMLInputElement).value)} />
					{@render embedBadge(block.settings.message)}
				</div>

			<!-- ===================== SET VARIABLE ===================== -->
			{:else if block.settings.type === 'SetVariable'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Variable name</label>
						<input type="text" value={block.settings.name} class={inputCls}
							oninput={(e) => updateSettings('name', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="relative">
						<label class={labelCls}>Value</label>
						<input type="text" value={block.settings.value} class={inputCls}
							oninput={(e) => updateSettings('value', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.value)}
					</div>
					<label class="flex items-center gap-2 text-[11px] text-foreground">
						<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
						Capture (save to hit output)
					</label>
				</div>

			<!-- ===================== CLEAR COOKIES ===================== -->
			{:else if block.settings.type === 'ClearCookies'}
				<div class="text-[11px] text-muted-foreground">
					Clears all cookies in the current session. No additional settings required.
				</div>

			<!-- ===================== WEBHOOK ===================== -->
			{:else if block.settings.type === 'Webhook'}
				<div class="space-y-1.5">
					<div class="flex gap-1.5">
						<SkeuSelect
							value={block.settings.method}
							onValueChange={(v) => updateSettings('method', v)}
							options={[{value:'POST',label:'POST'},{value:'GET',label:'GET'},{value:'PUT',label:'PUT'},{value:'PATCH',label:'PATCH'}]}
							class="text-[11px] w-20"
						/>
						<div class="relative flex-1">
							<input type="text" value={block.settings.url} placeholder="https://discord.com/api/webhooks/..."
								class="w-full skeu-input text-[11px] font-mono"
								oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
							{@render embedBadge(block.settings.url)}
						</div>
					</div>
					<div>
						<label class={labelCls}>Content-Type</label>
						<input type="text" value={block.settings.content_type} class={inputCls}
							oninput={(e) => updateSettings('content_type', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="relative">
						<label class={labelCls}>Body template</label>
						<textarea value={block.settings.body_template} placeholder={'{"content": "Hit: <USER>:<PASS>"}'}
							class="w-full skeu-input text-[11px] font-mono min-h-[80px] resize-y mt-0.5"
							oninput={(e) => updateSettings('body_template', (e.target as HTMLTextAreaElement).value)}></textarea>
						{@render embedBadge(block.settings.body_template)}
						<p class="text-[9px] text-muted-foreground mt-0.5">Use <code class="text-foreground/70">&lt;VAR&gt;</code> for variable interpolation</p>
					</div>
					<div class="relative">
						<label class={labelCls}>Custom cookies</label>
						<textarea value={block.settings.custom_cookies || ''} placeholder="session_id=abc123&#10;auth=<TOKEN>"
							class="w-full skeu-input text-[11px] font-mono min-h-[50px] resize-y mt-0.5"
							oninput={(e) => updateSettings('custom_cookies', (e.target as HTMLTextAreaElement).value)}></textarea>
						{@render embedBadge(block.settings.custom_cookies)}
						<p class="text-[9px] text-muted-foreground mt-0.5">One per line: <code class="text-foreground/70">name=value</code>. Sent as <code class="text-foreground/70">Cookie</code> header.</p>
					</div>
				</div>

			<!-- ===================== WEBSOCKET ===================== -->
			{:else if block.settings.type === 'WebSocket'}
				<div class="space-y-1.5">
					<div class="relative">
						<label class={labelCls}>URL</label>
						<input type="text" value={block.settings.url} placeholder="wss://example.com/ws" class={inputCls}
							oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
						{@render embedBadge(block.settings.url)}
					</div>
					<div>
						<label class={labelCls}>Action</label>
						<SkeuSelect value={block.settings.action}
							onValueChange={(v) => updateSettings('action', v)}
							options={[{value:'connect',label:'Connect'},{value:'send',label:'Send Message'},{value:'receive',label:'Receive'},{value:'close',label:'Close'}]}
							class="w-full mt-0.5"
						/>
					</div>
					{#if block.settings.action === 'send'}
						<div class="relative">
							<label class={labelCls}>Message</label>
							<textarea value={block.settings.message} placeholder="Message payload..."
								class="w-full skeu-input text-[11px] font-mono min-h-[60px] resize-y mt-0.5"
								oninput={(e) => updateSettings('message', (e.target as HTMLTextAreaElement).value)}></textarea>
							{@render embedBadge(block.settings.message)}
						</div>
					{/if}
					{#if block.settings.action === 'receive'}
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
						</div>
					{/if}
					<div class="flex items-center gap-2">
						<span class="text-[11px] text-muted-foreground">Timeout:</span>
						<input type="number" value={block.settings.timeout_ms}
							class="w-20 skeu-input text-[11px]"
							oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} />
						<span class="text-[10px] text-muted-foreground">ms</span>
					</div>
				</div>

			<!-- ===================== TCP REQUEST ===================== -->
			{:else if block.settings.type === 'TcpRequest'}
				<div class="space-y-1.5">
					<div class="flex gap-2">
						<div class="flex-1"><label class={labelCls}>Host</label><input type="text" value={block.settings.host} placeholder="example.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
						<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
					</div>
					<div><label class={labelCls}>Data to send</label><textarea value={block.settings.data} placeholder="Raw data..." class="w-full skeu-input text-[11px] font-mono min-h-[60px] resize-y mt-0.5" oninput={(e) => updateSettings('data', (e.target as HTMLTextAreaElement).value)}></textarea></div>
					<label class="flex items-center gap-2 text-[11px] text-foreground"><input type="checkbox" checked={block.settings.use_tls} onchange={() => updateSettings('use_tls', !block!.settings.use_tls)} class="skeu-checkbox" /> Use TLS</label>
					<div class="flex gap-2">
						<div class="flex-1"><label class={labelCls}>Output var</label><input type="text" value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
					</div>
					<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
				</div>

			<!-- ===================== UDP REQUEST ===================== -->
			{:else if block.settings.type === 'UdpRequest'}
				<div class="space-y-1.5">
					<div class="flex gap-2">
						<div class="flex-1"><label class={labelCls}>Host</label><input type="text" value={block.settings.host} placeholder="example.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
						<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
					</div>
					<div><label class={labelCls}>Data to send</label><textarea value={block.settings.data} placeholder="Raw data..." class="w-full skeu-input text-[11px] font-mono min-h-[60px] resize-y mt-0.5" oninput={(e) => updateSettings('data', (e.target as HTMLTextAreaElement).value)}></textarea></div>
					<div class="flex gap-2">
						<div class="flex-1"><label class={labelCls}>Output var</label><input type="text" value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
					</div>
					<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
				</div>

			<!-- ===================== FTP / SSH / IMAP / SMTP / POP ===================== -->
			{:else if block.settings.type === 'FtpRequest' || block.settings.type === 'SshRequest' || block.settings.type === 'ImapRequest' || block.settings.type === 'SmtpRequest' || block.settings.type === 'PopRequest'}
				<div class="space-y-1.5">
					<div class="flex gap-2">
						<div class="flex-1"><label class={labelCls}>Host</label><input type="text" value={block.settings.host} placeholder="mail.example.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
						<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
					</div>
					<div class="flex gap-2">
						<div class="flex-1"><label class={labelCls}>Username</label><input type="text" value={block.settings.username} placeholder="input.USER" class={inputCls} oninput={(e) => updateSettings('username', (e.target as HTMLInputElement).value)} /></div>
						<div class="flex-1"><label class={labelCls}>Password</label><input type="text" value={block.settings.password} placeholder="input.PASS" class={inputCls} oninput={(e) => updateSettings('password', (e.target as HTMLInputElement).value)} /></div>
					</div>
					<div><label class={labelCls}>Command</label><input type="text" value={block.settings.command} class={inputCls} oninput={(e) => updateSettings('command', (e.target as HTMLInputElement).value)} /></div>
					{'use_tls' in block.settings ? '' : ''}
					{#if 'use_tls' in block.settings}
						<label class="flex items-center gap-2 text-[11px] text-foreground"><input type="checkbox" checked={block.settings.use_tls} onchange={() => updateSettings('use_tls', !(block!.settings as any).use_tls)} class="skeu-checkbox" /> Use TLS</label>
					{/if}
					<div class="flex gap-2">
						<div class="flex-1"><label class={labelCls}>Output var</label><input type="text" value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
					</div>
					<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
				</div>

			<!-- ===================== CAPTCHA SOLVER ===================== -->
			{:else if block.settings.type === 'CaptchaSolver'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Solver service</label>
						<SkeuSelect value={block.settings.solver_service}
							onValueChange={(v) => updateSettings('solver_service', v)}
							options={[{value:'capsolver',label:'CapSolver'},{value:'2captcha',label:'2Captcha'},{value:'anticaptcha',label:'Anti-Captcha'},{value:'capmonster',label:'CapMonster'},{value:'deathbycaptcha',label:'DeathByCaptcha'}]}
							class="w-full mt-0.5"
						/>
					</div>
					<div>
						<label class={labelCls}>Captcha type</label>
						<SkeuSelect value={block.settings.captcha_type}
							onValueChange={(v) => updateSettings('captcha_type', v)}
							options={[{value:'RecaptchaV2',label:'reCAPTCHA v2'},{value:'HCaptcha',label:'hCaptcha'},{value:'FunCaptcha',label:'FunCaptcha'},{value:'ImageCaptcha',label:'Image Captcha'},{value:'Turnstile',label:'CF Turnstile'}]}
							class="w-full mt-0.5"
						/>
					</div>
					<div>
						<label class={labelCls}>API key</label>
						<input type="text" value={block.settings.api_key} placeholder="Your solver API key" class={inputCls}
							oninput={(e) => updateSettings('api_key', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Site key</label>
						<input type="text" value={block.settings.site_key} placeholder="Target site captcha key" class={inputCls}
							oninput={(e) => updateSettings('site_key', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Page URL</label>
						<input type="text" value={block.settings.page_url} placeholder="https://example.com/login" class={inputCls}
							oninput={(e) => updateSettings('page_url', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
					<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
				</div>

			<!-- ===================== CLOUDFLARE BYPASS ===================== -->
			{:else if block.settings.type === 'CloudflareBypass'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Target URL</label>
						<input type="text" value={block.settings.url} placeholder="https://protected-site.com" class={inputCls}
							oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>FlareSolverr endpoint</label>
						<input type="text" value={block.settings.flaresolverr_url} placeholder="http://localhost:8191/v1" class={inputCls}
							oninput={(e) => updateSettings('flaresolverr_url', (e.target as HTMLInputElement).value)} />
						<p class="text-[9px] text-muted-foreground mt-0.5">Local FlareSolverr instance URL</p>
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var (cookies)</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
					<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.max_timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('max_timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
				</div>

			<!-- ===================== LARAVEL CSRF ===================== -->
			{:else if block.settings.type === 'LaravelCsrf'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Page URL</label>
						<input type="text" value={block.settings.url} placeholder="https://example.com/login" class={inputCls}
							oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
						<p class="text-[9px] text-muted-foreground mt-0.5">URL to fetch CSRF token from</p>
					</div>
					<div>
						<label class={labelCls}>CSRF selector</label>
						<input type="text" value={block.settings.csrf_selector} placeholder={'input[name="_token"]'} class={inputCls}
							oninput={(e) => updateSettings('csrf_selector', (e.target as HTMLInputElement).value)} />
						<p class="text-[9px] text-muted-foreground mt-0.5">CSS selector for the hidden CSRF input</p>
					</div>
					<div>
						<label class={labelCls}>Cookie name</label>
						<input type="text" value={block.settings.cookie_name} placeholder="XSRF-TOKEN" class={inputCls}
							oninput={(e) => updateSettings('cookie_name', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
					<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
				</div>

			<!-- ===================== BROWSER OPEN ===================== -->
			{:else if block.settings.type === 'BrowserOpen'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Browser</label>
						<SkeuSelect value={block.settings.browser_type}
							onValueChange={(v) => updateSettings('browser_type', v)}
							options={[{value:'chromium',label:'Chromium'},{value:'firefox',label:'Firefox'},{value:'webkit',label:'WebKit'}]}
							class="w-full mt-0.5"
						/>
					</div>
					<label class="flex items-center gap-2 text-[11px] text-foreground">
						<input type="checkbox" checked={block.settings.headless} onchange={() => updateSettings('headless', !block!.settings.headless)} class="skeu-checkbox" />
						Headless mode
					</label>
					<div>
						<label class={labelCls}>Proxy (optional)</label>
						<input type="text" value={block.settings.proxy} placeholder="http://user:pass@host:port" class={inputCls}
							oninput={(e) => updateSettings('proxy', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Extra args</label>
						<input type="text" value={block.settings.extra_args} placeholder="--disable-gpu --no-sandbox" class={inputCls}
							oninput={(e) => updateSettings('extra_args', (e.target as HTMLInputElement).value)} />
						<p class="text-[9px] text-muted-foreground mt-0.5">Space-separated browser launch flags</p>
					</div>
				</div>

			<!-- ===================== NAVIGATE TO ===================== -->
			{:else if block.settings.type === 'NavigateTo'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>URL</label>
						<input type="text" value={block.settings.url} placeholder="https://example.com" class={inputCls}
							oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Wait until</label>
						<SkeuSelect value={block.settings.wait_until}
							onValueChange={(v) => updateSettings('wait_until', v)}
							options={[{value:'load',label:'Page Load'},{value:'domcontentloaded',label:'DOM Ready'},{value:'networkidle',label:'Network Idle'}]}
							class="w-full mt-0.5"
						/>
					</div>
					<div class="flex items-center gap-2">
						<span class="text-[11px] text-muted-foreground">Timeout:</span>
						<input type="number" value={block.settings.timeout_ms}
							class="w-20 skeu-input text-[11px]"
							oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} />
						<span class="text-[10px] text-muted-foreground">ms</span>
					</div>
					<div>
						<label class={labelCls}>Custom cookies</label>
						<textarea value={block.settings.custom_cookies || ''} placeholder={"session=abc123\nauth=<TOKEN>"}
							class="w-full skeu-input text-[11px] font-mono min-h-[50px] resize-y mt-0.5"
							oninput={(e) => updateSettings('custom_cookies', (e.target as HTMLTextAreaElement).value)}></textarea>
						<p class="text-[9px] text-muted-foreground mt-0.5">One per line: name=value. Injected via CDP before navigation.</p>
					</div>
				</div>

			<!-- ===================== CLICK ELEMENT ===================== -->
			{:else if block.settings.type === 'ClickElement'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Selector</label>
						<input type="text" value={block.settings.selector} placeholder="#login-btn, .submit, button[type='submit']" class={inputCls}
							oninput={(e) => updateSettings('selector', (e.target as HTMLInputElement).value)} />
					</div>
					<label class="flex items-center gap-2 text-[11px] text-foreground">
						<input type="checkbox" checked={block.settings.wait_for_navigation} onchange={() => updateSettings('wait_for_navigation', !block!.settings.wait_for_navigation)} class="skeu-checkbox" />
						Wait for navigation after click
					</label>
					<div class="flex items-center gap-2">
						<span class="text-[11px] text-muted-foreground">Timeout:</span>
						<input type="number" value={block.settings.timeout_ms}
							class="w-20 skeu-input text-[11px]"
							oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} />
						<span class="text-[10px] text-muted-foreground">ms</span>
					</div>
				</div>

			<!-- ===================== TYPE TEXT ===================== -->
			{:else if block.settings.type === 'TypeText'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Selector</label>
						<input type="text" value={block.settings.selector} placeholder="#username, input[name='email']" class={inputCls}
							oninput={(e) => updateSettings('selector', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Text</label>
						<input type="text" value={block.settings.text} placeholder="Text to type (supports <variables>)" class={inputCls}
							oninput={(e) => updateSettings('text', (e.target as HTMLInputElement).value)} />
					</div>
					<label class="flex items-center gap-2 text-[11px] text-foreground">
						<input type="checkbox" checked={block.settings.clear_first} onchange={() => updateSettings('clear_first', !block!.settings.clear_first)} class="skeu-checkbox" />
						Clear field before typing
					</label>
					<div class="flex items-center gap-2">
						<span class="text-[11px] text-muted-foreground">Key delay:</span>
						<input type="number" value={block.settings.delay_ms}
							class="w-20 skeu-input text-[11px]"
							oninput={(e) => updateSettings('delay_ms', parseInt((e.target as HTMLInputElement).value))} />
						<span class="text-[10px] text-muted-foreground">ms</span>
					</div>
				</div>

			<!-- ===================== WAIT FOR ELEMENT ===================== -->
			{:else if block.settings.type === 'WaitForElement'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Selector</label>
						<input type="text" value={block.settings.selector} placeholder="#result, .loaded" class={inputCls}
							oninput={(e) => updateSettings('selector', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Wait for state</label>
						<SkeuSelect value={block.settings.state}
							onValueChange={(v) => updateSettings('state', v)}
							options={[{value:'visible',label:'Visible'},{value:'hidden',label:'Hidden'},{value:'attached',label:'Attached'},{value:'detached',label:'Detached'}]}
							class="w-full mt-0.5"
						/>
					</div>
					<div class="flex items-center gap-2">
						<span class="text-[11px] text-muted-foreground">Timeout:</span>
						<input type="number" value={block.settings.timeout_ms}
							class="w-20 skeu-input text-[11px]"
							oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} />
						<span class="text-[10px] text-muted-foreground">ms</span>
					</div>
				</div>

			<!-- ===================== GET ELEMENT TEXT ===================== -->
			{:else if block.settings.type === 'GetElementText'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Selector</label>
						<input type="text" value={block.settings.selector} placeholder="h1.title, #message" class={inputCls}
							oninput={(e) => updateSettings('selector', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Attribute (empty = text content)</label>
						<input type="text" value={block.settings.attribute} placeholder="href, src, value" class={inputCls}
							oninput={(e) => updateSettings('attribute', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== SCREENSHOT ===================== -->
			{:else if block.settings.type === 'Screenshot'}
				<div class="space-y-1.5">
					<label class="flex items-center gap-2 text-[11px] text-foreground">
						<input type="checkbox" checked={block.settings.full_page} onchange={() => updateSettings('full_page', !block!.settings.full_page)} class="skeu-checkbox" />
						Full page screenshot
					</label>
					<div>
						<label class={labelCls}>Selector (optional, for element screenshot)</label>
						<input type="text" value={block.settings.selector} placeholder="#element" class={inputCls}
							oninput={(e) => updateSettings('selector', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Output var (base64)</label>
						<input type="text" value={block.settings.output_var} class={inputCls}
							oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
					</div>
				</div>

			<!-- ===================== EXECUTE JS ===================== -->
			{:else if block.settings.type === 'ExecuteJs'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>JavaScript code</label>
						<textarea value={block.settings.code} placeholder="// Runs in browser context&#10;return document.title;"
							class="w-full skeu-input text-[11px] font-mono min-h-[120px] resize-y mt-0.5"
							oninput={(e) => updateSettings('code', (e.target as HTMLTextAreaElement).value)}></textarea>
						<p class="text-[9px] text-muted-foreground mt-0.5">Executes in the browser page context. Use <code class="text-foreground/70">return</code> to capture a value.</p>
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== DATE FUNCTION ===================== -->
			{:else if block.settings.type === 'DateFunction'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Function</label>
						{#if fieldHint('function_type')}<p class={hintCls}>{fieldHint('function_type')}</p>{/if}
						<SkeuSelect value={block.settings.function_type}
							options={[
								{value:'Now',label:'Now (current time)'},{value:'FormatDate',label:'Format Date'},
								{value:'ParseDate',label:'Parse Date'},{value:'AddTime',label:'Add Time'},
								{value:'SubtractTime',label:'Subtract Time'},{value:'UnixTimestamp',label:'Unix Timestamp'},
								{value:'UnixToDate',label:'Unix to Date'},
							]}
							onChange={(v) => updateSettings('function_type', v)} />
					</div>
					{#if !['Now','UnixTimestamp'].includes(block.settings.function_type)}
						<div>
							<label class={labelCls}>Input var</label>
							{#if fieldHint('input_var')}<p class={hintCls}>{fieldHint('input_var')}</p>{/if}
							<input type="text" value={block.settings.input_var} class={inputCls} placeholder="data.SOURCE"
								oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						</div>
					{/if}
					<div>
						<label class={labelCls}>Format</label>
						{#if fieldHint('format')}<p class={hintCls}>{fieldHint('format')}</p>{/if}
						<input type="text" value={block.settings.format} class={inputCls} placeholder="%Y-%m-%d %H:%M:%S"
							oninput={(e) => updateSettings('format', (e.target as HTMLInputElement).value)} />
					</div>
					{#if ['AddTime','SubtractTime'].includes(block.settings.function_type)}
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Amount</label>
								{#if fieldHint('amount')}<p class={hintCls}>{fieldHint('amount')}</p>{/if}
								<input type="number" value={block.settings.amount} class={inputCls}
									oninput={(e) => updateSettings('amount', parseInt((e.target as HTMLInputElement).value) || 0)} />
							</div>
							<div class="flex-1">
								<label class={labelCls}>Unit</label>
								{#if fieldHint('unit')}<p class={hintCls}>{fieldHint('unit')}</p>{/if}
								<SkeuSelect value={block.settings.unit}
									options={[{value:'seconds',label:'Seconds'},{value:'minutes',label:'Minutes'},{value:'hours',label:'Hours'},{value:'days',label:'Days'}]}
									onChange={(v) => updateSettings('unit', v)} />
							</div>
						</div>
					{/if}
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== CASE / SWITCH ===================== -->
			{:else if block.settings.type === 'CaseSwitch'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Input variable</label>
						{#if fieldHint('input_var')}<p class={hintCls}>{fieldHint('input_var')}</p>{/if}
						<input type="text" value={block.settings.input_var} class={inputCls} placeholder="data.RESPONSECODE"
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Cases</label>
						{#if fieldHint('cases')}<p class={hintCls}>{fieldHint('cases')}</p>{/if}
						{#each block.settings.cases as c, ci}
							<div class="flex gap-1 items-center mt-1">
								<input type="text" value={c.match_value} class="{smallInputCls} flex-1" placeholder="Match value"
									oninput={(e) => {
										const cases = [...block!.settings.cases];
										cases[ci] = { ...cases[ci], match_value: (e.target as HTMLInputElement).value };
										updateSettings('cases', cases);
									}} />
								<ArrowRight size={10} class="text-muted-foreground shrink-0" />
								<input type="text" value={c.result_value} class="{smallInputCls} flex-1" placeholder="Result value"
									oninput={(e) => {
										const cases = [...block!.settings.cases];
										cases[ci] = { ...cases[ci], result_value: (e.target as HTMLInputElement).value };
										updateSettings('cases', cases);
									}} />
								<button class="p-0.5 rounded hover:bg-destructive/20 text-muted-foreground shrink-0"
									onclick={() => {
										const cases = block!.settings.cases.filter((_: unknown, i: number) => i !== ci);
										updateSettings('cases', cases);
									}}>
									<Trash2 size={10} />
								</button>
							</div>
						{/each}
						<button class="skeu-btn text-[10px] mt-1 flex items-center gap-1"
							onclick={() => updateSettings('cases', [...block!.settings.cases, { match_value: '', result_value: '' }])}>
							<Plus size={10} /> Add Case
						</button>
					</div>
					<div>
						<label class={labelCls}>Default value</label>
						{#if fieldHint('default_value')}<p class={hintCls}>{fieldHint('default_value')}</p>{/if}
						<input type="text" value={block.settings.default_value} class={inputCls} placeholder="FAIL"
							oninput={(e) => updateSettings('default_value', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
				</div>

			<!-- ===================== COOKIE CONTAINER ===================== -->
			{:else if block.settings.type === 'CookieContainer'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Source type</label>
						{#if fieldHint('source_type')}<p class={hintCls}>{fieldHint('source_type')}</p>{/if}
						<SkeuSelect
							value={block.settings.source_type}
							onValueChange={(v) => updateSettings('source_type', v)}
							options={[{value:'text',label:'Raw text'},{value:'file',label:'File path'}]}
							class="w-full mt-0.5"
						/>
					</div>
					<div>
						<label class={labelCls}>{block.settings.source_type === 'file' ? 'File path' : 'Cookie text'}</label>
						{#if fieldHint('source')}<p class={hintCls}>{fieldHint('source')}</p>{/if}
						{#if block.settings.source_type === 'file'}
							<input type="text" value={block.settings.source} class={inputCls} placeholder="C:\cookies.txt or <FILE_PATH>"
								oninput={(e) => updateSettings('source', (e.target as HTMLInputElement).value)} />
						{:else}
							<textarea value={block.settings.source}
								placeholder={".example.com\tTRUE\t/\tFALSE\t0\tsession\tabc123\nname=value"}
								class="w-full skeu-input text-[11px] font-mono min-h-[100px] resize-y mt-0.5"
								oninput={(e) => updateSettings('source', (e.target as HTMLTextAreaElement).value)}></textarea>
						{/if}
						<p class="text-[9px] text-muted-foreground mt-0.5">Accepts Netscape format (tab-separated) or simple <code class="text-foreground/70">name=value</code> lines.</p>
					</div>
					<div>
						<label class={labelCls}>Domain filter</label>
						{#if fieldHint('domain')}<p class={hintCls}>{fieldHint('domain')}</p>{/if}
						<input type="text" value={block.settings.domain} class={inputCls} placeholder=".example.com (leave empty for all)"
							oninput={(e) => updateSettings('domain', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="flex gap-2">
						<div class="flex-1">
							<label class={labelCls}>Output var</label>
							{#if fieldHint('output_var')}<p class={hintCls}>{fieldHint('output_var')}</p>{/if}
							<input type="text" value={block.settings.output_var} class={inputCls}
								oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-1 text-xs text-foreground pt-4">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							CAP
						</label>
					</div>
					<label class="flex items-center gap-2 text-[11px] text-foreground">
						<input type="checkbox" checked={block.settings.save_netscape} onchange={() => updateSettings('save_netscape', !block!.settings.save_netscape)} class="skeu-checkbox" />
						Also save in Netscape format
					</label>
					{#if fieldHint('save_netscape')}<p class={hintCls}>{fieldHint('save_netscape')}</p>{/if}
				</div>

			<!-- ===================== RANDOM USER AGENT ===================== -->
			{:else if block.settings.type === 'RandomUserAgent'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Mode</label>
						<SkeuSelect
							value={block.settings.mode}
							onValueChange={(v) => updateSettings('mode', v)}
							options={[{value:'Random',label:'Random (built-in list)'},{value:'CustomList',label:'Custom List'}]}
							class="text-[11px] w-full"
						/>
					</div>
					{#if block.settings.mode === 'Random'}
						<div>
							<label class={labelCls}>Browser filter</label>
							<div class="flex gap-2 flex-wrap">
								{#each ['Chrome', 'Firefox', 'Safari', 'Edge'] as browser}
									<label class="flex items-center gap-1 text-[10px]">
										<input type="checkbox"
											checked={block.settings.browser_filter.includes(browser)}
											onchange={() => {
												const filters = [...block.settings.browser_filter];
												const idx = filters.indexOf(browser);
												if (idx >= 0) filters.splice(idx, 1); else filters.push(browser);
												updateSettings('browser_filter', filters);
											}}
											class="skeu-checkbox" />
										{browser}
									</label>
								{/each}
							</div>
						</div>
						<div>
							<label class={labelCls}>Platform filter</label>
							<div class="flex gap-2 flex-wrap">
								{#each ['Desktop', 'Mobile', 'Tablet'] as platform}
									<label class="flex items-center gap-1 text-[10px]">
										<input type="checkbox"
											checked={block.settings.platform_filter.includes(platform)}
											onchange={() => {
												const filters = [...block.settings.platform_filter];
												const idx = filters.indexOf(platform);
												if (idx >= 0) filters.splice(idx, 1); else filters.push(platform);
												updateSettings('platform_filter', filters);
											}}
											class="skeu-checkbox" />
										{platform}
									</label>
								{/each}
							</div>
						</div>
					{:else}
						<div>
							<label class={labelCls}>Custom UA list (one per line)</label>
							<textarea value={block.settings.custom_list}
								class="w-full skeu-input text-[11px] font-mono min-h-[80px] resize-y mt-0.5"
								oninput={(e) => updateSettings('custom_list', (e.target as HTMLTextAreaElement).value)}
							></textarea>
						</div>
					{/if}
					<label class="flex items-center gap-2 text-[11px]">
						<input type="checkbox" checked={block.settings.match_tls} class="skeu-checkbox"
							onchange={() => updateSettings('match_tls', !block.settings.match_tls)} />
						Match TLS fingerprint (JA3 + HTTP/2)
					</label>
					{#if block.settings.match_tls}
						<p class={hintCls}>Automatically selects matching TLS profile for the chosen browser</p>
					{/if}
					<div>
						<label class={labelCls}>Output variable</label>
						<input type="text" value={block.settings.output_var} class={inputCls}
							oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
					</div>
					<label class="flex items-center gap-2 text-[11px]">
						<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
							onchange={() => updateSettings('capture', !block.settings.capture)} />
						Capture
					</label>
				</div>

			<!-- ===================== OCR CAPTCHA ===================== -->
			{:else if block.settings.type === 'OcrCaptcha'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Input variable (base64 image)</label>
						<input type="text" value={block.settings.input_var} class={inputCls}
							oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Language</label>
						<input type="text" value={block.settings.language} class={inputCls} placeholder="eng"
							oninput={(e) => updateSettings('language', (e.target as HTMLInputElement).value)} />
						{#if fieldHint('language')}<p class={hintCls}>{fieldHint('language')}</p>{/if}
					</div>
					<div>
						<label class={labelCls}>Page Segmentation Mode (PSM)</label>
						<input type="number" value={block.settings.psm} min="0" max="13" class={inputCls}
							oninput={(e) => updateSettings('psm', parseInt((e.target as HTMLInputElement).value) || 7)} />
					</div>
					<div>
						<label class={labelCls}>Whitelist (allowed chars)</label>
						<input type="text" value={block.settings.whitelist} class={inputCls} placeholder="0123456789ABCDEFabcdef"
							oninput={(e) => updateSettings('whitelist', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Output variable</label>
						<input type="text" value={block.settings.output_var} class={inputCls}
							oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
					</div>
					<label class="flex items-center gap-2 text-[11px]">
						<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
							onchange={() => updateSettings('capture', !block.settings.capture)} />
						Capture
					</label>
				</div>

			<!-- ===================== RECAPTCHA INVISIBLE ===================== -->
			{:else if block.settings.type === 'RecaptchaInvisible'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Site Key</label>
						<input type="text" value={block.settings.sitekey} class={inputCls}
							oninput={(e) => updateSettings('sitekey', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Anchor URL</label>
						<input type="text" value={block.settings.anchor_url} class={inputCls}
							oninput={(e) => updateSettings('anchor_url', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Reload URL</label>
						<input type="text" value={block.settings.reload_url} class={inputCls}
							oninput={(e) => updateSettings('reload_url', (e.target as HTMLInputElement).value)} />
					</div>
					<div class="grid grid-cols-2 gap-1">
						<div>
							<label class={labelCls}>co (base64 origin)</label>
							<input type="text" value={block.settings.co} class={inputCls}
								oninput={(e) => updateSettings('co', (e.target as HTMLInputElement).value)} />
						</div>
						<div>
							<label class={labelCls}>v (JS version)</label>
							<input type="text" value={block.settings.v} class={inputCls}
								oninput={(e) => updateSettings('v', (e.target as HTMLInputElement).value)} />
						</div>
					</div>
					<div class="grid grid-cols-2 gap-1">
						<div>
							<label class={labelCls}>ar</label>
							<input type="text" value={block.settings.ar} class={inputCls}
								oninput={(e) => updateSettings('ar', (e.target as HTMLInputElement).value)} />
						</div>
						<div>
							<label class={labelCls}>hi</label>
							<input type="text" value={block.settings.hi} class={inputCls}
								oninput={(e) => updateSettings('hi', (e.target as HTMLInputElement).value)} />
						</div>
					</div>
					<div class="grid grid-cols-2 gap-1">
						<div>
							<label class={labelCls}>size</label>
							<input type="text" value={block.settings.size} class={inputCls}
								oninput={(e) => updateSettings('size', (e.target as HTMLInputElement).value)} />
						</div>
						<div>
							<label class={labelCls}>action</label>
							<input type="text" value={block.settings.action} class={inputCls}
								oninput={(e) => updateSettings('action', (e.target as HTMLInputElement).value)} />
						</div>
					</div>
					<div>
						<label class={labelCls}>cb</label>
						<input type="text" value={block.settings.cb} class={inputCls}
							oninput={(e) => updateSettings('cb', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>User Agent</label>
						<input type="text" value={block.settings.user_agent} class={inputCls}
							oninput={(e) => updateSettings('user_agent', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Output variable</label>
						<input type="text" value={block.settings.output_var} class={inputCls}
							oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
					</div>
					<label class="flex items-center gap-2 text-[11px]">
						<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
							onchange={() => updateSettings('capture', !block.settings.capture)} />
						Capture
					</label>
				</div>

			<!-- ===================== XACF SENSOR ===================== -->
			{:else if block.settings.type === 'XacfSensor'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Bundle ID</label>
						<input type="text" value={block.settings.bundle_id} class={inputCls} placeholder="com.example.app"
							oninput={(e) => updateSettings('bundle_id', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Version</label>
						<input type="text" value={block.settings.version} class={inputCls} placeholder="2.1.2"
							oninput={(e) => updateSettings('version', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Output variable</label>
						<input type="text" value={block.settings.output_var} class={inputCls}
							oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
					</div>
					<label class="flex items-center gap-2 text-[11px]">
						<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
							onchange={() => updateSettings('capture', !block.settings.capture)} />
						Capture
					</label>
				</div>

			<!-- ===================== RANDOM DATA ===================== -->
			{:else if block.settings.type === 'RandomData'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Data type</label>
						<SkeuSelect
							value={block.settings.data_type}
							onValueChange={(v) => updateSettings('data_type', v)}
							options={[
								{value:'String',label:'Random String'},{value:'Uuid',label:'UUID'},{value:'Number',label:'Number'},
								{value:'Email',label:'Email'},{value:'FirstName',label:'First Name'},{value:'LastName',label:'Last Name'},
								{value:'FullName',label:'Full Name'},{value:'StreetAddress',label:'Street Address'},
								{value:'City',label:'City'},{value:'State',label:'State'},{value:'ZipCode',label:'Zip Code'},
								{value:'PhoneNumber',label:'Phone Number'},{value:'Date',label:'Date'},
							]}
							class="text-[11px] w-full"
						/>
					</div>
					{#if block.settings.data_type === 'String'}
						<div>
							<label class={labelCls}>Length</label>
							<input type="number" value={block.settings.string_length} class={inputCls} min="1" max="1024"
								oninput={(e) => updateSettings('string_length', parseInt((e.target as HTMLInputElement).value) || 16)} />
						</div>
						<div>
							<label class={labelCls}>Charset</label>
							<SkeuSelect
								value={block.settings.string_charset}
								onValueChange={(v) => updateSettings('string_charset', v)}
								options={[
									{value:'alphanumeric',label:'Alphanumeric'},{value:'alpha',label:'Alpha only'},
									{value:'hex',label:'Hex'},{value:'numeric',label:'Numeric'},{value:'custom',label:'Custom'},
								]}
								class="text-[11px] w-full"
							/>
						</div>
						{#if block.settings.string_charset === 'custom'}
							<div>
								<label class={labelCls}>Custom characters</label>
								<input type="text" value={block.settings.custom_chars} class={inputCls}
									oninput={(e) => updateSettings('custom_chars', (e.target as HTMLInputElement).value)} />
							</div>
						{/if}
					{:else if block.settings.data_type === 'Number'}
						<div class="grid grid-cols-2 gap-2">
							<div>
								<label class={labelCls}>Min</label>
								<input type="number" value={block.settings.number_min} class={inputCls}
									oninput={(e) => updateSettings('number_min', parseInt((e.target as HTMLInputElement).value) || 0)} />
							</div>
							<div>
								<label class={labelCls}>Max</label>
								<input type="number" value={block.settings.number_max} class={inputCls}
									oninput={(e) => updateSettings('number_max', parseInt((e.target as HTMLInputElement).value) || 100)} />
							</div>
						</div>
						<label class="flex items-center gap-2 text-[11px]">
							<input type="checkbox" checked={block.settings.number_decimal} class="skeu-checkbox"
								onchange={() => updateSettings('number_decimal', !block.settings.number_decimal)} />
							Decimal
						</label>
					{:else if block.settings.data_type === 'Date'}
						<div>
							<label class={labelCls}>Date format</label>
							<input type="text" value={block.settings.date_format} class={inputCls} placeholder="%Y-%m-%d"
								oninput={(e) => updateSettings('date_format', (e.target as HTMLInputElement).value)} />
						</div>
						<div class="grid grid-cols-2 gap-2">
							<div>
								<label class={labelCls}>Min date</label>
								<input type="text" value={block.settings.date_min} class={inputCls} placeholder="1990-01-01"
									oninput={(e) => updateSettings('date_min', (e.target as HTMLInputElement).value)} />
							</div>
							<div>
								<label class={labelCls}>Max date</label>
								<input type="text" value={block.settings.date_max} class={inputCls} placeholder="2025-12-31"
									oninput={(e) => updateSettings('date_max', (e.target as HTMLInputElement).value)} />
							</div>
						</div>
					{/if}
					<div>
						<label class={labelCls}>Output variable</label>
						<input type="text" value={block.settings.output_var} class={inputCls}
							oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
					</div>
					<label class="flex items-center gap-2 text-[11px]">
						<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
							onchange={() => updateSettings('capture', !block.settings.capture)} />
						Capture
					</label>
				</div>

			<!-- ===================== DATADOME SENSOR ===================== -->
			{:else if block.settings.type === 'DataDomeSensor'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Site URL</label>
						<input type="text" value={block.settings.site_url} class={inputCls} placeholder="https://example.com"
							oninput={(e) => updateSettings('site_url', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>DataDome cookie</label>
						<input type="text" value={block.settings.cookie_datadome} class={inputCls} placeholder="<data.COOKIES>"
							oninput={(e) => updateSettings('cookie_datadome', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>User Agent</label>
						<input type="text" value={block.settings.user_agent} class={inputCls} placeholder="<UA>"
							oninput={(e) => updateSettings('user_agent', (e.target as HTMLInputElement).value)} />
					</div>
					<div>
						<label class={labelCls}>Custom WASM (base64, optional)</label>
						<textarea value={block.settings.custom_wasm_b64}
							class="w-full skeu-input text-[10px] font-mono min-h-[40px] resize-y mt-0.5"
							placeholder="Leave empty for default sensor"
							oninput={(e) => updateSettings('custom_wasm_b64', (e.target as HTMLTextAreaElement).value)}
						></textarea>
					</div>
					<div>
						<label class={labelCls}>Output variable</label>
						<input type="text" value={block.settings.output_var} class={inputCls}
							oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
					</div>
					<label class="flex items-center gap-2 text-[11px]">
						<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
							onchange={() => updateSettings('capture', !block.settings.capture)} />
						Capture
					</label>
				</div>

			<!-- ===================== PLUGIN BLOCK ===================== -->
			{:else if block.settings.type === 'Plugin'}
				<div class="space-y-1.5">
					<div>
						<label class={labelCls}>Plugin type</label>
						<div class="text-[11px] font-mono text-muted-foreground px-2 py-1 bg-background rounded border border-border">
							{block.settings.plugin_block_type || 'Not set'}
						</div>
					</div>
					{#if (() => { const meta = app.pluginBlocks.find(pb => pb.block_type_name === block.settings.plugin_block_type); if (!meta?.settings_schema_json) return false; try { const s = JSON.parse(meta.settings_schema_json); return s?.properties && Object.keys(s.properties).length > 0; } catch { return false; } })()}
						{@const pluginSchema = (() => { const meta = app.pluginBlocks.find(pb => pb.block_type_name === block.settings.plugin_block_type); try { return JSON.parse(meta!.settings_schema_json); } catch { return {}; } })()}
						{@const pluginSettings = (() => { try { return JSON.parse(block.settings.settings_json || '{}'); } catch { return {}; } })()}
						{#each Object.entries(pluginSchema.properties || {}) as [propName, propDef]}
							{@const pd = propDef as any}
							<div>
								<label class={labelCls}>{pd.title || propName}</label>
								{#if pd.enum}
									<select class={inputCls} value={pluginSettings[propName] ?? pd.default ?? ''}
										onchange={(e) => {
											const updated = { ...pluginSettings, [propName]: (e.target as HTMLSelectElement).value };
											updateSettings('settings_json', JSON.stringify(updated));
										}}>
										{#each pd.enum as opt}
											<option value={opt}>{opt}</option>
										{/each}
									</select>
								{:else if pd.type === 'boolean'}
									<label class="flex items-center gap-2 text-[11px]">
										<input type="checkbox" class="skeu-checkbox"
											checked={pluginSettings[propName] ?? pd.default ?? false}
											onchange={() => {
												const updated = { ...pluginSettings, [propName]: !(pluginSettings[propName] ?? pd.default ?? false) };
												updateSettings('settings_json', JSON.stringify(updated));
											}} />
									</label>
								{:else if pd.type === 'number' || pd.type === 'integer'}
									<input type="number" class={inputCls}
										value={pluginSettings[propName] ?? pd.default ?? 0}
										oninput={(e) => {
											const updated = { ...pluginSettings, [propName]: parseFloat((e.target as HTMLInputElement).value) };
											updateSettings('settings_json', JSON.stringify(updated));
										}} />
								{:else}
									<div class="relative">
										<input type="text" class={inputCls}
											value={pluginSettings[propName] ?? pd.default ?? ''}
											oninput={(e) => {
												const updated = { ...pluginSettings, [propName]: (e.target as HTMLInputElement).value };
												updateSettings('settings_json', JSON.stringify(updated));
											}} />
										{@render embedBadge(pluginSettings[propName] ?? pd.default ?? '')}
									</div>
								{/if}
							</div>
						{/each}
					{:else}
						{@const kvEntries = (() => { try { return Object.entries(JSON.parse(block.settings.settings_json || '{}')); } catch { return []; } })() as [string, any][]}
						<div>
							<label class={labelCls}>Settings</label>
							<div class="space-y-1">
								{#each kvEntries as [k, v], i}
									<div class="flex items-center gap-1">
										<input type="text" value={k} class={inputCls + ' flex-1'}
											placeholder="key"
											oninput={(e) => {
												const newKey = (e.target as HTMLInputElement).value;
												const obj: Record<string, any> = {};
												kvEntries.forEach(([ek, ev], j) => { obj[j === i ? newKey : ek] = ev; });
												updateSettings('settings_json', JSON.stringify(obj));
											}} />
										<div class="relative flex-1">
										<input type="text" value={typeof v === 'string' ? v : JSON.stringify(v)} class={inputCls + ' w-full'}
											placeholder="value"
											oninput={(e) => {
												const obj: Record<string, any> = {};
												kvEntries.forEach(([ek, ev], j) => { obj[ek] = j === i ? (e.target as HTMLInputElement).value : ev; });
												updateSettings('settings_json', JSON.stringify(obj));
											}} />
										{@render embedBadge(typeof v === 'string' ? v : JSON.stringify(v))}
									</div>
										<button class="p-0.5 text-muted-foreground hover:text-destructive shrink-0" title="Remove field"
											onclick={() => {
												const obj: Record<string, any> = {};
												kvEntries.forEach(([ek, ev], j) => { if (j !== i) obj[ek] = ev; });
												updateSettings('settings_json', JSON.stringify(obj));
											}}>
											<Trash2 size={11} />
										</button>
									</div>
								{/each}
								<button class="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground mt-0.5"
									onclick={() => {
										const obj: Record<string, any> = {};
										kvEntries.forEach(([ek, ev]) => { obj[ek] = ev; });
										obj[''] = '';
										updateSettings('settings_json', JSON.stringify(obj));
									}}>
									<Plus size={10} /> Add field
								</button>
							</div>
						</div>
					{/if}
					<div>
						<label class={labelCls}>Output variable</label>
						<input type="text" value={block.settings.output_var} class={inputCls}
							oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
					</div>
					<label class="flex items-center gap-2 text-[11px]">
						<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
							onchange={() => updateSettings('capture', !block.settings.capture)} />
						Capture
					</label>
				</div>

			{:else if block.settings.type === 'Group'}
				<div class="space-y-1.5">
					<label class="flex items-center gap-2 text-[11px]">
						<input type="checkbox" checked={block.settings.collapsed} class="skeu-checkbox"
							onchange={() => updateSettings('collapsed', !block.settings.collapsed)} />
						Collapsed
					</label>
					<div class="text-[10px] text-muted-foreground">
						{block.settings.blocks?.length || 0} block{(block.settings.blocks?.length || 0) !== 1 ? 's' : ''} in group. Drag blocks into the group container below.
					</div>
				</div>

			{:else}
				<div class="text-[11px] text-muted-foreground">
					Settings editor for {block.block_type} blocks
				</div>
			{/if}
		</div>
	</div>
{/if}
