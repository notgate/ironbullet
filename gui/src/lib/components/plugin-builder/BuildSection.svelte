<h3 class="text-sm font-semibold text-foreground mb-3">Build & Load</h3>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">1. Build Your Plugin</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">cd my-plugin
cargo build --release</pre>
<p class="text-[12px] text-muted-foreground mb-3">The compiled DLL will be at:<br><code>target/release/my_plugin.dll</code></p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">2. Copy to Plugins Directory</h3>
<p class="text-[12px] text-muted-foreground mb-3">Copy the DLL to your configured plugins directory. You can set this path in <strong>Settings &gt; Plugins Path</strong>.</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3"># Default location (next to ironbullet.exe)
cp target/release/my_plugin.dll ./plugins/</pre>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">3. Load in ironbullet</h3>
<p class="text-[12px] text-muted-foreground mb-3">Plugins are loaded at startup. To reload after changes:</p>
<ul class="text-[12px] text-foreground/85 space-y-1 pl-5 list-disc mb-3">
<li>Restart Ironbullet, or</li>
<li>Use <strong>File &gt; Import Plugin (.dll)</strong> to load a specific DLL</li>
</ul>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">4. Verify</h3>
<p class="text-[12px] text-muted-foreground mb-3">After loading, your blocks should appear in the Block Palette under their configured category. Add one to a pipeline and check the Debug panel to verify execution.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Troubleshooting</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Issue</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Cause & Fix</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 text-foreground">DLL not found</td><td class="px-3 py-1.5 text-foreground/80">Check plugins directory path in Settings.</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 text-foreground">Missing export error</td><td class="px-3 py-1.5 text-foreground/80">Ensure all 4 functions are <code>#[no_mangle] pub extern "C"</code> and crate-type is <code>cdylib</code>.</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 text-foreground">Crash on load</td><td class="px-3 py-1.5 text-foreground/80">Check for null pointer returns.</td></tr>
<tr class="bg-accent/5"><td class="px-3 py-1.5 text-foreground">Settings not showing</td><td class="px-3 py-1.5 text-foreground/80">Validate your JSON Schema string.</td></tr>
</tbody>
</table>
