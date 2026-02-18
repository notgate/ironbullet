<h3 class="text-sm font-semibold text-foreground mb-3">Settings Schema</h3>
<p class="text-[12px] text-muted-foreground mb-3">The <code>settings_schema_json</code> field in <code>BlockInfo</code> defines what the settings panel shows when the user clicks on your block. It uses <strong>JSON Schema</strong> (draft-07 compatible).</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Supported Field Types</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">JSON Schema Type</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">UI Control</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Example</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">string</td><td class="px-3 py-1.5 text-foreground/80">Text input</td><td class="px-3 py-1.5 font-mono text-[10px]">{`{"type":"string","title":"URL"}`}</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">number</td><td class="px-3 py-1.5 text-foreground/80">Number input</td><td class="px-3 py-1.5 font-mono text-[10px]">{`{"type":"number","title":"Timeout"}`}</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">boolean</td><td class="px-3 py-1.5 text-foreground/80">Toggle switch</td><td class="px-3 py-1.5 font-mono text-[10px]">{`{"type":"boolean","title":"Verbose"}`}</td></tr>
<tr class="bg-accent/5"><td class="px-3 py-1.5 font-mono">string + enum</td><td class="px-3 py-1.5 text-foreground/80">Dropdown select</td><td class="px-3 py-1.5 font-mono text-[10px]">{`{"type":"string","enum":["a","b"]}`}</td></tr>
</tbody>
</table>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Full Example</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">{`{
    "type": "object",
    "properties": {
        "input_var": {
            "type": "string",
            "title": "Input Variable",
            "default": "data.SOURCE"
        },
        "timeout_ms": {
            "type": "number",
            "title": "Timeout (ms)",
            "default": 5000
        },
        "mode": {
            "type": "string",
            "title": "Mode",
            "enum": ["fast", "thorough", "balanced"],
            "default": "fast"
        },
        "verbose": {
            "type": "boolean",
            "title": "Verbose Logging",
            "default": false
        }
    }
}`}</pre>
<p class="text-[12px] text-muted-foreground mb-3">The <code>default_settings_json</code> should match the schema defaults:</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">{`{"input_var":"data.SOURCE","timeout_ms":5000,"mode":"fast","verbose":false}`}</pre>
