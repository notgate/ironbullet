import type { GuideSection } from './types';

export const GUIDE_SECTIONS: GuideSection[] = [
	{
		id: 'get-started',
		title: 'Get Started',
		icon: 'Rocket',
		content: `
<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">What is ironbullet?</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:16px">
ironbullet is a pipeline-based HTTP automation engine. You build <strong>configs</strong> — sequences of blocks that describe an HTTP workflow — then run them at scale against a list of inputs (wordlists). Each input line is parsed into variables like <code>&lt;USER&gt;</code> and <code>&lt;PASS&gt;</code>, then flows through the pipeline.
</p>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Core Concepts</h3>
<ul style="font-size:12px;line-height:1.8;margin-bottom:16px;padding-left:20px">
<li><strong>Pipeline</strong> — An ordered list of blocks that run sequentially for each input line</li>
<li><strong>Variables</strong> — Data flows between blocks via named variables. Use <code>&lt;VAR&gt;</code> syntax to interpolate. Built-in: <code>&lt;USER&gt;</code>, <code>&lt;PASS&gt;</code>, <code>&lt;data.SOURCE&gt;</code>, <code>&lt;data.RESPONSECODE&gt;</code></li>
<li><strong>Bot Status</strong> — Each line ends with a status: <em>SUCCESS</em>, <em>FAIL</em>, <em>BAN</em>, <em>RETRY</em>, <em>CUSTOM</em>, or <em>NONE</em>. Set by KeyCheck blocks</li>
<li><strong>Captures</strong> — Variables marked as "capture" are saved with hits (e.g., account balance, email, token)</li>
</ul>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Basic Workflow</h3>
<ol style="font-size:12px;line-height:1.8;margin-bottom:16px;padding-left:20px">
<li><strong>Create a config</strong> — File → New Config, or open an existing <code>.yaml</code></li>
<li><strong>Add blocks</strong> — Drag blocks from the palette: typically HttpRequest → Parser → KeyCheck</li>
<li><strong>Configure blocks</strong> — Click a block to edit its settings (URL, headers, parse rules, check conditions)</li>
<li><strong>Load data</strong> — Import a wordlist file with one entry per line (<code>user:pass</code> format)</li>
<li><strong>Run</strong> — Set thread count and proxy list, click Start</li>
</ol>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">First Pipeline Example</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
1. HttpRequest
   URL: https://example.com/api/login
   Method: POST
   Body: username=&lt;USER&gt;&amp;password=&lt;PASS&gt;

2. ParseJSON
   JSONPath: $.token
   Output var: TOKEN

3. KeyCheck
   SUCCESS when data.RESPONSECODE EqualTo "200"
   FAIL (default)
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Variable Interpolation</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:8px">
All string fields in blocks support <code>&lt;VARIABLE_NAME&gt;</code> interpolation:
</p>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
&lt;input.EMAIL&gt;       — Input line fields
&lt;data.SOURCE&gt;       — Response body
&lt;data.RESPONSECODE&gt; — HTTP status code
&lt;TOKEN&gt;             — User-defined variable
&lt;random.uuid&gt;       — Random UUID
&lt;random.email&gt;      — Random email
&lt;random.string.32&gt;  — Random 32-char string
&lt;random.number.1.100&gt; — Random number 1-100
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Keyboard Shortcuts</h3>
<table style="font-size:11px;width:100%;border-collapse:collapse;margin-bottom:16px">
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>Ctrl+S</kbd></td><td style="padding:4px 8px">Save config</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>F1</kbd></td><td style="padding:4px 8px">Open docs</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>F5</kbd></td><td style="padding:4px 8px">Debug current line</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>Ctrl+D</kbd></td><td style="padding:4px 8px">Duplicate block</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:4px 8px"><kbd>Delete</kbd></td><td style="padding:4px 8px">Remove selected block</td></tr>
<tr><td style="padding:4px 8px"><kbd>Ctrl+Z / Y</kbd></td><td style="padding:4px 8px">Undo / Redo</td></tr>
</table>
`,
	},
	{
		id: 'plugin-kit',
		title: 'Plugin Kit',
		icon: 'Puzzle',
		content: `
<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Creating a Plugin</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:16px">
Plugins are Rust shared libraries (<code>.dll</code> / <code>.so</code>) that extend ironbullet with custom block types. A plugin exports a set of C-ABI functions that the engine calls to discover and execute blocks.
</p>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">1. Cargo.toml Setup</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]   # Required: builds a C-compatible shared library

[dependencies]
serde_json = "1"
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">2. ABI Structs</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
#[repr(C)]
pub struct PluginInfo {
    pub name: *const c_char,        // "My Plugin"
    pub version: *const c_char,     // "1.0.0"
    pub description: *const c_char, // "Does something useful"
    pub block_count: u32,           // Number of blocks provided
}

#[repr(C)]
pub struct BlockInfo {
    pub type_name: *const c_char,   // "MyPlugin.MyBlock"
    pub label: *const c_char,       // "My Block"
    pub category: *const c_char,    // "Utilities"
    pub color: *const c_char,       // "#4ec9b0"
    pub settings_schema: *const c_char, // JSON Schema for settings UI
}

#[repr(C)]
pub struct ExecuteResult {
    pub success: bool,
    pub updated_variables_json: *const c_char,
    pub log_message: *const c_char,
    pub error_message: *const c_char,
}
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">3. Required Exports</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
#[no_mangle]
pub extern "C" fn plugin_info() -> *mut PluginInfo { ... }

#[no_mangle]
pub extern "C" fn plugin_block_info(index: u32) -> *mut BlockInfo { ... }

#[no_mangle]
pub extern "C" fn plugin_execute(
    block_type: *const c_char,    // Which block to run
    settings_json: *const c_char, // Block settings as JSON
    variables_json: *const c_char // Current variables as JSON
) -> *mut ExecuteResult { ... }

#[no_mangle]
pub extern "C" fn plugin_free_string(ptr: *mut c_char) { ... }
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">4. Execute Example</h3>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
#[no_mangle]
pub extern "C" fn plugin_execute(
    _block_type: *const c_char,
    settings_json: *const c_char,
    variables_json: *const c_char,
) -> *mut ExecuteResult {
    let settings: HashMap&lt;String, String&gt; = serde_json::from_str(&settings_str).unwrap_or_default();
    let mut vars: HashMap&lt;String, String&gt; = serde_json::from_str(&vars_str).unwrap_or_default();

    // Your custom logic here
    let input = vars.get("data.SOURCE").cloned().unwrap_or_default();
    let reversed: String = input.chars().rev().collect();
    vars.insert("PLUGIN_RESULT".to_string(), reversed);

    Box::into_raw(Box::new(ExecuteResult {
        success: true,
        updated_variables_json: CString::new(serde_json::to_string(&vars).unwrap()).unwrap().into_raw(),
        log_message: CString::new("Processed OK").unwrap().into_raw(),
        error_message: std::ptr::null(),
    }))
}
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">5. Settings JSON Schema</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:8px">
Return a JSON Schema string from <code>BlockInfo.settings_schema</code> to get auto-generated settings UI:
</p>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
{
  "type": "object",
  "properties": {
    "input_var": { "type": "string", "default": "data.SOURCE", "title": "Input Variable" },
    "mode": { "type": "string", "enum": ["fast", "accurate"], "default": "fast", "title": "Mode" }
  }
}
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">6. Build &amp; Load</h3>
<ol style="font-size:12px;line-height:1.8;padding-left:20px">
<li><code>cargo build --release</code> → produces <code>target/release/my_plugin.dll</code></li>
<li>In ironbullet GUI: File → Import Plugin → select the <code>.dll</code></li>
<li>Plugin blocks appear in the block palette under their declared category</li>
</ol>
`,
	},
	{
		id: 'runners',
		title: 'Runners',
		icon: 'Play',
		content: `
<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Architecture</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:16px">
The runner system is an orchestrator that manages concurrent workers processing a shared data pool. Each worker picks a line from the pool, parses it into input variables, executes the full pipeline, and reports the result.
</p>

<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
┌─────────────────────────────────────┐
│           Orchestrator              │
│  ┌──────────┐  ┌──────────────────┐ │
│  │ Data Pool │  │   Proxy Pool    │ │
│  │ (lines)   │  │ (round-robin)   │ │
│  └─────┬─────┘  └────────┬────────┘ │
│        │                 │          │
│  ┌─────▼─────────────────▼────────┐ │
│  │     Worker 1  │  Worker 2  ... │ │
│  │  get line     │  get line      │ │
│  │  get proxy    │  get proxy     │ │
│  │  run pipeline │  run pipeline  │ │
│  │  report stats │  report stats  │ │
│  └────────────────────────────────┘ │
└─────────────────────────────────────┘
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Execution Flow (per line)</h3>
<ol style="font-size:12px;line-height:1.8;margin-bottom:16px;padding-left:20px">
<li><strong>Get line</strong> — Worker pulls the next unprocessed line from the data pool</li>
<li><strong>Parse input</strong> — Line is split by the delimiter (default <code>:</code>) into <code>&lt;USER&gt;</code>, <code>&lt;PASS&gt;</code>, etc.</li>
<li><strong>Get proxy</strong> — Round-robin selection from the proxy pool (if proxies loaded)</li>
<li><strong>Execute pipeline</strong> — Each block runs in sequence, passing variables forward</li>
<li><strong>Check status</strong> — Final bot status determines the outcome</li>
<li><strong>Update stats</strong> — Increment counters (hits, fails, bans, retries, tested, CPM)</li>
</ol>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Bot Statuses</h3>
<table style="font-size:11px;width:100%;border-collapse:collapse;margin-bottom:16px">
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#22c55e">SUCCESS</td><td style="padding:6px 8px">Valid credentials — saved to hits file with captures</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#ef4444">FAIL</td><td style="padding:6px 8px">Invalid credentials — line is discarded</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#f59e0b">BAN</td><td style="padding:6px 8px">IP/account blocked — proxy is temp-banned, line retried with different proxy</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#3b82f6">RETRY</td><td style="padding:6px 8px">Temporary error — line goes back to pool for retry</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600;color:#a855f7">CUSTOM</td><td style="padding:6px 8px">User-defined status — saved separately (e.g., "2FA", "Locked")</td></tr>
<tr><td style="padding:6px 8px;font-weight:600;color:#6b7280">NONE</td><td style="padding:6px 8px">No KeyCheck matched — treated as fail</td></tr>
</table>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Retry Logic</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:8px">
When a line gets RETRY or BAN status, or when a network error occurs:
</p>
<ul style="font-size:12px;line-height:1.8;padding-left:20px;margin-bottom:16px">
<li>Line is placed back in the data pool</li>
<li>For BAN: the proxy is temporarily removed from rotation</li>
<li>Max retry count is configurable (default: 3) — after that, marked as TOCHECK</li>
<li>Retried lines get a fresh proxy assignment</li>
</ul>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Stats Tracking</h3>
<p style="font-size:12px;line-height:1.7">
The runner tracks: <strong>Hits</strong> (success count), <strong>Fails</strong>, <strong>Bans</strong>, <strong>Retries</strong>, <strong>Tested</strong> (total processed), <strong>CPM</strong> (checks per minute), <strong>Progress</strong> (tested / total), and <strong>Elapsed</strong> time. Stats update in real-time in the GUI status bar.
</p>
`,
	},
	{
		id: 'proxies',
		title: 'Proxies',
		icon: 'Shield',
		content: `
<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Proxy Formats</h3>
<p style="font-size:12px;line-height:1.7;margin-bottom:8px">
ironbullet accepts proxy lists in multiple formats, one proxy per line:
</p>
<pre style="font-size:11px;background:#1e1e1e;border:1px solid var(--border);border-radius:6px;padding:12px;margin-bottom:16px;overflow-x:auto">
# Format 1: HOST:PORT (defaults to HTTP)
192.168.1.1:8080

# Format 2: Protocol URL
http://192.168.1.1:8080
https://proxy.example.com:3128
socks5://192.168.1.1:1080

# Format 3: Protocol URL with auth
socks5://user:pass@192.168.1.1:1080

# Format 4: TYPE:HOST:PORT:USER:PASS
http:192.168.1.1:8080:username:password
socks5:10.0.0.1:1080:user:pass
</pre>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Supported Proxy Types</h3>
<table style="font-size:11px;width:100%;border-collapse:collapse;margin-bottom:16px">
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600">HTTP</td><td style="padding:6px 8px">Standard HTTP/1.1 proxy with CONNECT tunneling</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600">HTTPS</td><td style="padding:6px 8px">HTTP proxy over TLS connection</td></tr>
<tr style="border-bottom:1px solid var(--border)"><td style="padding:6px 8px;font-weight:600">SOCKS4</td><td style="padding:6px 8px">SOCKS4 protocol (no authentication support)</td></tr>
<tr><td style="padding:6px 8px;font-weight:600">SOCKS5</td><td style="padding:6px 8px">SOCKS5 protocol with optional username/password auth</td></tr>
</table>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Rotation &amp; Banning</h3>
<ul style="font-size:12px;line-height:1.8;padding-left:20px;margin-bottom:16px">
<li><strong>Round-robin</strong> — Proxies are assigned to workers in order, cycling through the list</li>
<li><strong>Temp ban</strong> — When a proxy gets a BAN result, it's removed from rotation for a configurable duration (default: 30s)</li>
<li><strong>Auto-recovery</strong> — Banned proxies are automatically re-added after the ban period expires</li>
<li><strong>No proxies</strong> — If no proxy list is loaded, all requests go through the direct connection</li>
</ul>

<h3 style="font-size:15px;font-weight:600;margin-bottom:12px">Best Practices</h3>
<ul style="font-size:12px;line-height:1.8;padding-left:20px">
<li>Use <strong>residential proxies</strong> for sites with strong anti-bot protection</li>
<li>Keep the thread count proportional to your proxy count (1-3 threads per proxy)</li>
<li>Test proxies before a run — dead proxies waste time and cause retries</li>
<li>Use SOCKS5 for sites that block datacenter HTTP proxies</li>
<li>Match your User-Agent TLS fingerprint to the proxy type for consistency</li>
</ul>
`,
	},
];
