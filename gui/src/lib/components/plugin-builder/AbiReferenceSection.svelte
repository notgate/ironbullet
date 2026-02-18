<h3 class="text-sm font-semibold text-foreground mb-3">ABI Reference</h3>
<p class="text-[12px] text-muted-foreground mb-3">All communication between ironbullet and plugins uses C-compatible <code>#[repr(C)]</code> structs with raw <code>*const c_char</code> pointers. This ensures ABI stability across different Rust compiler versions.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">PluginInfo</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">#[repr(C)]
pub struct PluginInfo {'{'}
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
    pub block_count: u32,
{'}'}</pre>
<p class="text-[12px] text-muted-foreground mb-3"><strong>Lifetime:</strong> The returned pointer must be valid for the entire DLL lifetime. Use <code>OnceLock</code> + <code>leak_cstring()</code> for static allocation.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">BlockInfo</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">#[repr(C)]
pub struct BlockInfo {'{'}
    pub block_type_name: *const c_char,
    pub label: *const c_char,
    pub category: *const c_char,
    pub color: *const c_char,
    pub icon: *const c_char,
    pub settings_schema_json: *const c_char,
    pub default_settings_json: *const c_char,
{'}'}</pre>
<p class="text-[12px] text-muted-foreground mb-3"><strong>block_type_name</strong> must be <code>"PluginName.BlockName"</code> format. ironbullet uses this to route execution calls to the correct plugin and block.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">ExecuteResult</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">#[repr(C)]
pub struct ExecuteResult {'{'}
    pub success: bool,
    pub updated_variables_json: *const c_char,
    pub log_message: *const c_char,
    pub error_message: *const c_char,
{'}'}</pre>
<p class="text-[12px] text-muted-foreground mb-3"><strong>Memory:</strong> These strings are allocated by the plugin and freed by ironbullet via <code>plugin_free_string()</code>. Use <code>CString::new(s).unwrap().into_raw()</code> to allocate.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Memory Safety Rules</h3>
<ul class="text-[12px] text-foreground/85 space-y-1 pl-5 list-disc">
<li><strong>Static data</strong> (PluginInfo, BlockInfo): Use <code>OnceLock</code> + <code>leak_cstring()</code> -- these are never freed</li>
<li><strong>Result data</strong> (ExecuteResult): Use <code>CString::new().into_raw()</code> -- freed by host via <code>plugin_free_string</code></li>
<li><strong>Box&lt;ExecuteResult&gt;</strong>: Allocate with <code>Box::into_raw()</code> -- host reads then drops</li>
<li>Never return stack pointers or references to temporaries</li>
</ul>
