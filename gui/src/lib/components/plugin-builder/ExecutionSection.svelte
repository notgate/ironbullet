<h3 class="text-sm font-semibold text-foreground mb-3">Execution Flow</h3>
<p class="text-[12px] text-muted-foreground mb-3">When ironbullet executes a pipeline and encounters one of your plugin blocks, it calls <code>plugin_execute()</code> with the block index, current settings, and the full variable map.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Input Parameters</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">pub extern "C" fn plugin_execute(
    block_index: u32,
    settings_json: *const c_char,
    variables_json: *const c_char,
) -> *const ExecuteResult</pre>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Variables Map</h3>
<p class="text-[12px] text-muted-foreground mb-2">The variables JSON is a flat <code>HashMap&lt;String, String&gt;</code> containing all pipeline state:</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">{'{'}
    "data.USER": "john@example.com",
    "data.PASS": "secret123",
    "data.SOURCE": "&lt;html&gt;...&lt;/html&gt;",
    "data.RESPONSECODE": "200",
    "data.SOURCE.HEADERS": "{'{'}...{'}'}",
    "data.SOURCE.COOKIES": "{'{'}...{'}'}",
    "CAPTURE_email": "john@example.com"
{'}'}</pre>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Common Variable Names</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Variable</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Contains</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">data.SOURCE</td><td class="px-3 py-1.5 text-foreground/80">Last HTTP response body</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">data.RESPONSECODE</td><td class="px-3 py-1.5 text-foreground/80">HTTP status code</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">data.USER / data.PASS</td><td class="px-3 py-1.5 text-foreground/80">Current data line fields</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">data.SOURCE.HEADERS</td><td class="px-3 py-1.5 text-foreground/80">Response headers (JSON)</td></tr>
<tr><td class="px-3 py-1.5 font-mono">data.SOURCE.COOKIES</td><td class="px-3 py-1.5 text-foreground/80">Response cookies (JSON)</td></tr>
</tbody>
</table>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Return Pattern</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">// Return your modified variables
let mut vars = parse_variables(variables_json);

// Read input
let source = vars.get("data.SOURCE").cloned().unwrap_or_default();

// Do your processing
let result = my_transform(&source);

// Write output
vars.insert("PLUGIN_RESULT".to_string(), result);
vars.insert("CAPTURE_myfield".to_string(), "captured_value".to_string());

// Return success
make_result(true, &vars, "Processed OK", "")</pre>
<p class="text-[12px] text-muted-foreground mb-2"><strong>Captures:</strong> Any variable prefixed with <code>CAPTURE_</code> will be saved to the output file when the pipeline finishes with a "hit" status.</p>
