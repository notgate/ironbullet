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
		version: '0.6.1',
		date: '2026-04-14',
		highlights: 'Hotfix release for proxy mode regressions, settings persistence, startup stability, and packaged sidecar assets.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Proxy mode regression fixed (issue #67): Sticky and No Proxies mode selections now round-trip correctly instead of collapsing back to Rotate.',
					'Single-tab settings changes now mark the workspace dirty immediately (issue #68): tab state tracking now detects config edits even when only one tab is open.',
					'Proxy settings persistence hardened (issue #12): proxy-related settings changes are cloned and synchronized correctly before save, so saved settings survive reloads.',
					'Startup crash protection added (issue #69): runner initialization now tolerates missing or legacy config fields instead of panicking during launch.',
				],
			},
			{
				title: 'Release Assets',
				items: [
					'Windows and Linux release archives include the reqflow sidecar alongside the main IronBullet binary.',
					'Linux launcher text updated to point users to the v0.6.1 AppImage when WebKitGTK is unavailable.',
				],
			},
		],
	},
	{
		version: '0.6.0',
		date: '2026-04-07',
		highlights: 'Mega update — error requeue, sticky proxy fix, custom user inputs, right-click context menu, full stat numbers.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Errored credentials requeued for final pass (issue #64): Accounts that exhaust max_retries due to transient network/proxy errors are stashed and replayed once after the main pool drains with fresh proxies.',
					'Sticky proxy mode auto-elevation (issues #58, #59): Selecting a Sticky proxy group in the Job dialog or loading a Saved Config now correctly activates Sticky mode instead of silently ignoring it.',
					'Proxy settings preserved per saved config (issue #63): Loading an .rfx file no longer overwrites its proxy_mode and active_group.',
					'Stats now show full numbers with commas (e.g. 1,234 instead of 1K) for accuracy (issue #65).',
				],
			},
			{
				title: 'New Features',
				items: [
					'Custom user input variables (issue #62): Configs can define custom_inputs with name, description, type, and default value. The Job dialog shows input fields and values are injected into the globals namespace.',
					'Right-click context menu (issue #65): Copy, Paste, Select All on response viewer, variable inspector, debug panel, and all text content areas.',
				],
			},
		],
	},
	{
		version: '0.5.5',
		date: '2026-04-06',
		highlights: 'Bug fixes: startup restore regression (#57), SOCKS5 in Saved Config jobs (#58), Sticky mode group override (#59).',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Startup restore regression (issue #57): After v0.5.4, the dedup check caused the startup-restored tab (empty content, correct filePath) to be reused when opening the same file from Collections. Fixed: AppState::new() now loads the last-opened pipeline from disk so the startup tab has real content.',
					'SOCKS5 proxies ignored in Saved Config jobs (issue #58): Saved Config jobs loaded proxy_mode and proxy_sources from the .rfx file, which may predate in-session changes. Fixed: create_job now inherits proxy_mode and proxy_sources from the active in-memory pipeline.',
					'Sticky mode group override ignored (issue #59): Auto-elevating proxy_mode from None hardcoded Rotate instead of using the group\'s own mode. Fixed: create_job reads ProxyGroup.mode and uses it, so Sticky groups activate in Sticky mode.',
				],
			},
		],
	},
	{
		version: '0.5.4',
		date: '2026-04-06',
		highlights: 'Bug fixes: duplicate tabs, SOCKS5 proxy groups in saved configs, job dialog proxy controls, output filenames, and WebView2 OOM crash.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Duplicate tabs (issue #57): Opening the same config file twice now switches to the existing tab instead of creating a duplicate.',
					'SOCKS5 proxy groups ignored in Saved Config jobs (issue #58): Global proxy groups were not restored when loading a config via config_path. Fixed create_job to apply proxy_groups from GuiConfig the same way load_pipeline does.',
					'Job dialog proxy group override ignored when pipeline proxy_mode=None (issue #59): Selecting a proxy group in the New Job dialog had no effect if the pipeline had no proxy mode set. Fixed create_job to auto-elevate proxy_mode to Rotate when overriding with a group.',
					'Output file named "New_Config_1" for Current Tab jobs (issue #60): When the active pipeline had a default name, output files were named after the default. Fixed create_job to rename from the file stem using pipeline_path when available.',
					'WebView2 Out of Memory crash with SOCKS5 proxies (issue #61): The jobs_list IPC broadcast (every 500ms) was serializing up to 500 ResultEntry structs with full block_results into each message, causing multi-MB JSON payloads that exhausted WebView2 memory. recent_results is now excluded from jobs_list and only fetched on demand by the debug log dialog.',
				],
			},
		],
	},
	{
		version: '0.5.0',
		date: '2026-04-01',
		highlights: 'Major bugfix release — SOCKS5 proxy support, file dialogs respect settings, autosave recovery.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
				'SOCKS5 proxies completely broken (issue #46): ProxyPool.load_from_string() was not parsing proxy lines correctly. Added proper load_from_string() method that accepts Option<ProxyType> and fixed the proxy loading flow to handle each source with its own default type. SOCKS5, SOCKS4, HTTP, HTTPS, and Shadowsocks proxies all work now.',
				'Job output files saved as "New_Config_1" (issue #45): When loading a saved config, the pipeline name was not derived from the config filename if it matched default patterns. Fixed handlers_job.rs to detect "New Config" or "New Config N" patterns and replace with actual filename stem.',
				'Unsaved session dialog on every launch (issue #47): The autosave recovery file was not deleted after successful restoration. Fixed handlers_config.rs to delete autosave after load_pipeline() succeeds.',
				'Job creation dialog cannot find configs (issue #48): The configs list was using setupDirsPaths.configs instead of the saved configsPath from settings. Fixed JobMonitor.svelte to prefer configsPath with fallback to setupDirsPaths.',
				'File dialogs ignore default paths from Settings (issue #50): Several file dialogs were not setting the starting directory from GuiConfig. Fixed save_code, import_config, import_plugin, and save_plugin_files to use their respective default paths from settings.',
				'Proxy groups stored per-project and overwrite global settings (issue #52): Proxy groups were being saved inside .rfx config files, so loading an old config would restore deleted groups. Fixed RfxConfig::from_pipeline() to clear proxy_groups before saving, and load_pipeline() to always use global groups from GuiConfig instead of merging.',
			],
		},
		],
	},
	{
		version: '0.4.9',
		date: '2026-04-01',
		highlights: 'Black screen fix for Shadowsocks users — UI no longer freezes during proxy resolution.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Black screen with Shadowsocks proxy (issue #44): The liveness probe port_is_alive() was called while holding the std::sync::Mutex lock inside resolve(). With multiple SS proxies, each 100ms TCP connect blocked the tokio runtime, starving the WebView and causing a black screen with high CPU. Fixed by moving the port_is_alive() call outside the mutex lock scope — clone the cached value first, drop the lock, then probe liveness.',
				],
			},
		],
	},
	{
		version: '0.4.8',
		date: '2026-03-29',
		highlights: 'Job start fix: jobs no longer stay Queued indefinitely when Run is clicked.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Job stays Queued when clicking Run (issue #42): In v0.4.7, the Shadowsocks pool resolve() function called Handle::block_on(spawn_blocking(...)) to wait for the local SOCKS5 listener to bind. Handle::block_on panics when invoked from within an async context. Since resolve() is called from start_job inside a tokio::spawn async task, the panic killed the job start silently, leaving the job in Queued state with no error surfaced. Fixed by replacing Handle::block_on with tokio::task::block_in_place, which is designed for calling blocking code from within async tasks.',
				],
			},
		],
	},
	{
		version: '0.4.7',
		date: '2026-03-29',
		highlights: 'Job start stall fix for Shadowsocks users, BAN retry — accounts no longer lost to proxy bans.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Job fails to start when Shadowsocks proxies are configured (issue #41): The v0.4.6 port-readiness wait used std::thread::sleep inside a tokio async worker thread. On systems with a small CPU core count, blocking every async worker while waiting for the SS listener to bind prevented the SS server task from executing, stalling job start indefinitely. Fixed by offloading the wait to tokio\'s spawn_blocking thread pool, which is separate from the async worker pool.',
					'Accounts lost to proxy bans (issue #41): When a check returned BAN status the credential was silently dropped. A BAN means the target site blocked the proxy IP, not that the account is invalid. Banned accounts are now re-queued for retry on a different proxy (subject to the job\'s max retry setting). Once retries are exhausted the credential is counted as an error rather than disappearing from the totals. Done/Total now correctly accounts for all credentials.',
				],
			},
		],
	},
	{
		version: '0.4.6',
		date: '2026-03-29',
		highlights: 'Shadowsocks proxy routing fix, gray screen fix for remaining NVIDIA/Wayland configurations.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Shadowsocks proxies not routing traffic (issue #40): The local SOCKS5 listener was being spawned asynchronously but the port URL was returned immediately before the listener was bound. Subsequent requests received ECONNREFUSED, causing continuous BAN results and zero hits. Fixed by polling the loopback port until it accepts connections (up to 10 s) before returning the socks5:// URL. Typical bind latency is under 10 ms.',
					'Shadowsocks ban tracking broken: When a Shadowsocks proxy was banned, the ban key was stored as the original ss:// URI but next_proxy() looked up bans using the socks5://127.0.0.1:<port> form, causing a permanent key mismatch and bans that never applied. Both paths now use the socks5:// tunnel URL as the canonical ban key.',
					'Gray screen on Linux still occurring after v0.4.5 fix (issue #37): Added WEBKIT_FORCE_SANDBOX=0 to the set of startup environment variables. On certain NVIDIA driver versions and kernel security configurations the WebKit sandbox helper process fails to start silently, leaving the WebView gray. Disabling the sandbox resolves this case. The variable is set both from the Rust binary and from start.sh so it is present before GTK initialisation regardless of launch method.',
				],
			},
		],
	},
	{
		version: '0.4.5',
		date: '2026-03-17',
		highlights: 'Linux NVIDIA/Wayland gray screen fix, correct progress calculation (errors excluded), live results feed, Shadowsocks proxy support.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Gray screen on Linux with NVIDIA GPU under Wayland (GNOME 46, Ubuntu 24.04): Added GDK_BACKEND=x11 to force XWayland at startup, bypassing the broken NVIDIA/EGL Wayland renderer in WebKitGTK. Previously set env vars (WEBKIT_DISABLE_COMPOSITING_MODE, WEBKIT_DISABLE_DMABUF_RENDERER) retained for VMs and other edge cases.',
					'Progress bar excluded errors but completed jobs showed less than 100%: Completed and stopped jobs now correctly snap to 100%. During a run the bar is capped at 99% (verified outcomes only — hits + fails) and transitions to 100% on job completion.',
					'Done/Total counter was counting errors as completed work: now shows verified/total (hits + fails only). Hover the counter for a full breakdown including error count.',
					'Live results feed in Hits Database was always empty: update_job_stats was snapshotting recent_results from the ring buffer and then immediately clearing them before saving to the job. One-line bug — removed the Vec::new() clear. The live feed in Hits Database now correctly streams all check results (hits, fails, bans, retries, errors) in real-time.',
				],
			},
			{
				title: 'New Features',
				items: [
					'Shadowsocks proxy support: ss:// URIs are now accepted in proxy lists. Both cleartext format (ss://method:password@host:port) and SIP002 base64-encoded format (ss://BASE64@host:port#label) are supported. The #fragment label (including emoji labels from proxy providers) is stripped automatically. Each unique SS server spins up a local SOCKS5 tunnel on first use and reuses it for the duration of the run — no per-request overhead.',
				],
			},
			{
				title: 'Release Assets',
				items: [
					'Linux zip now includes reqflow-sidecar and start.sh launcher alongside the main binary.',
					'Windows zip now includes reqflow-sidecar.exe alongside the main binary.',
					'Linux AppImage rebuilt with bundled webkit2gtk (95MB, no system dependencies required).',
				],
			},
		],
	},
	{
		version: '0.4.2',
		date: '2026-03-13',
		highlights: 'Bug fix: sidecar spawn failure (error 123) on Windows for users with long install paths or UNC path contexts.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Sidecar spawn error 123 on Windows — "The filename, directory name, or volume label syntax is incorrect". Affected browser capture (Site Inspector) and request engine for users installed in paths where Windows returns \\\\?\\-prefixed UNC paths from current_exe() (long path mode, certain installers). Fixed by applying dunce::canonicalize to all sidecar path resolution branches.',
					'Sidecar spawn could fail with error 123 when CWD was a UNC/network path and sidecar had no resolvable parent directory. Now falls back to the exe directory instead of "." as working directory.',
				],
			},
		],
	},
	{
		version: '0.4.0',
		date: '2026-03-09',
		highlights: 'Site Inspector MITM proxy with full HTTPS interception, BrowserOpen block, headless Chrome pipeline support.',
		sections: [
			{
				title: 'New Features',
				items: [
					'Site Inspector — full HTTPS MITM proxy via Go sidecar. Intercepts and displays all requests/responses including encrypted HTTPS traffic. Ephemeral CA generated on startup; per-hostname certs signed on demand. Request list with domain grouping, virtual scroll, zoom, copy URL, export (JSON/HAR/txt), and target domain filter.',
					'BrowserOpen block — launch headless or headed Chrome within a pipeline. Supports NavigateTo, WaitForElement, ExecuteJs, and BrowserClose blocks. Chrome spawned and connected via CDP (no stderr parsing). Cookies automatically flow into the pipeline\'s internal jar for use by subsequent HTTP blocks.',
					'Proxy auto-inheritance in BrowserOpen — if no proxy is configured on the block, the runner\'s active session proxy is automatically applied to Chrome.',
				],
			},
			{
				title: 'Bug Fixes',
				items: [
					'BrowserOpen headless mode was inverted — browser always launched headed regardless of the headless setting.',
					'Browser launch hang on Windows — Chrome does not write the DevTools WebSocket URL to stderr on Windows; switched to polling /json/version endpoint instead of reading stderr.',
					'NavigateTo timeout ignored — page navigation could hang indefinitely; now enforces the configured timeout.',
					'Browser task deadlock — spawn_blocking executor disconnect caused CDP channels to die immediately after launch; fixed by running Browser::launch directly on the main async runtime.',
					'Proxy groups disappear on new config — update_pipeline now fully replaces proxy groups instead of merging, preventing deleted groups from reappearing after restart.',
					'New tab inherits proxy groups from current pipeline instead of starting empty, preventing group wipe on first sync.',
				],
			},
		],
	},
	{
		version: '0.3.6',
		date: '2026-03-06',
		highlights: 'Critical KeyCheck bug fix, proxy rotation fix, proxy auth parser fix, request latency fix, output format variables, View Debug Log.',
		sections: [
			{
				title: 'Bug Fixes',
				items: [
					'Critical: KeyCheck conditions using "data.VARNAME" now correctly resolve variables set by ParseJSON, JwtToken, and other blocks that store into user_vars. Previously this caused all-error runs with 0 hits when conditions referenced parsed output variables.',
					'Proxy rotation not working — runner now correctly loads proxies from pipeline proxy groups and proxy sources. Previously proxies were only loaded from a per-job flat file, causing all runs to be proxyless.',
					'Proxy auth parser — user:pass@host:port format (no scheme prefix) was silently dropped, causing proxyless requests instead of an error.',
					'Request latency 5–6s — AzureTLS sidecar had no TCP connect timeout (zero-value net.Dialer). Added 10s connect timeout and per-request context deadline via session.Do() wrapper.',
					'Done/Total counter inflated by retries — stats.processed was incremented on every attempt including retries. Fixed: only increment on first attempt (retry_count == 0).',
					'Output format variables {response}, {headers}, {status}, {proxy} now correctly substituted when writing hits to file.',
					'Proxy groups not persisting — ProxiesSection.svelte now syncs Svelte state to Rust before saving.',
					'Parse block "Invalid JSON" on empty response body — now returns empty output instead of erroring.',
					'Browser capture reliability — CDP listeners now attached before navigation; response body fetches run as independent tasks to avoid blocking the event loop.',
					'ClearProxy — replaced SetProxy("") (errors on empty string in azuretls) with session.ClearProxy().',
					'Sidecar proxy error diagnostics — proxyconnect errors now correctly attributed to proxy layer; enriched error messages for context deadline exceeded, no route to host, i/o timeout.',
				],
			},
			{
				title: 'New Features',
				items: [
					'View Debug Log — right-click any job row to open a full debug popup with a live result feed (status, data line, timestamp) and a per-block execution trace accordion (Request, Response, Variables tabs per block). Auto-refreshes every 2 seconds while the job runs.',
				],
			},
		],
	},
	{
		version: '0.3.5',
		date: '2026-03-04',
		highlights: 'WreqTLS browser emulation backend, JWT Token block, Header Spoof block, UI preferences panel, bottom tab visibility controls.',
		sections: [
			{
				title: 'New Features',
				items: [
					'WreqTLS backend — third TLS client option on every HTTP Request block. Powered by wreq + BoringSSL with 100+ device emulation profiles (Chrome 100–137, Edge 101–134, Firefox 109–139, Safari 15–18, OkHttp 3–5, Opera 116–119). Accurate JA3/JA4 and HTTP/2 SETTINGS frame emulation with no external sidecar required.',
					'WreqTLS cookie persistence — wreq client is reused across HTTP blocks within a single pipeline run so multi-step login flows correctly share cookies. Rebuilt automatically if the emulation profile or proxy changes mid-pipeline.',
					'Browser emulation dropdown in HTTP block settings — select any supported profile when WreqTLS is chosen. Verify fingerprints at tls.peet.ws.',
					'JWT Token block — sign and verify JSON Web Tokens directly in your pipeline. Sign mode produces a signed JWT (HS256/HS384/HS512) from a JSON claims template with variable interpolation, automatic iat injection, and optional exp via Expires In. Decode mode verifies the signature, checks expiry, and extracts all claims as CLAIM_<KEY> variables for downstream use.',
					'Header Spoof block — inject proxy detection bypass headers (X-Forwarded-For, X-Real-IP, CF-Connecting-IP, True-Client-IP, X-Forwarded-Proto, X-Forwarded-Host) with four IP strategies: random public IPv4, fixed rotation list, extract from current proxy, or manual value. Headers are automatically picked up by the next HTTP Request block without any manual header wiring.',
					'UI & Panels settings section — new settings page with controls for dialog suppression, panel visibility, and density.',
					'Skip unsaved dialog — option to close tabs immediately without the "save changes?" prompt.',
					'Skip app close confirm — option to exit the app immediately even with unsaved tabs.',
					'Bottom panel tab visibility — show/hide individual tabs (Debugger, Code View, Data/Proxy, Jobs, Network, Variables, Inspector) from settings. State persists across restarts.',
					'Compact mode setting — reduces spacing in block list and panels.',
					'Reset panel layout button in settings.',
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
