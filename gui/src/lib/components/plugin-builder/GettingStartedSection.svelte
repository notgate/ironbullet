<h3 class="text-sm font-semibold text-foreground mb-3">Getting Started with Plugins</h3>
<p class="text-[12px] text-muted-foreground mb-3">ironbullet plugins are <strong>Rust DLLs</strong> (.dll on Windows) that export a C-compatible ABI. The host loads your DLL at runtime, queries it for block definitions, and calls your execute function during pipeline runs.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">How It Works</h3>
<ol class="text-[12px] text-foreground/85 space-y-2 pl-5 list-decimal mb-4">
<li><strong>ironbullet scans</strong> the plugins directory for <code>.dll</code> files</li>
<li>For each DLL, it calls <code>plugin_info()</code> to get your plugin's name, version, and block count</li>
<li>It calls <code>plugin_block_info(index)</code> for each block to get labels, categories, settings schema, and defaults</li>
<li>Your blocks appear in the Block Palette alongside built-in blocks</li>
<li>During execution, ironbullet calls <code>plugin_execute(block_index, settings, variables)</code> with the current pipeline state</li>
<li>Your function processes data and returns updated variables</li>
</ol>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Project Structure</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">my-plugin/
  Cargo.toml          # crate-type = ["cdylib"]
  src/
    lib.rs            # All 4 required exports</pre>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Required Exports</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Function</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Purpose</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono text-foreground">plugin_info()</td><td class="px-3 py-1.5 text-foreground/80">Returns plugin metadata (name, version, block count)</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono text-foreground">plugin_block_info(index)</td><td class="px-3 py-1.5 text-foreground/80">Returns block definition for given index</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono text-foreground">plugin_execute(index, settings, vars)</td><td class="px-3 py-1.5 text-foreground/80">Runs block logic, returns updated variables</td></tr>
<tr class="bg-accent/5"><td class="px-3 py-1.5 font-mono text-foreground">plugin_free_string(ptr)</td><td class="px-3 py-1.5 text-foreground/80">Frees a CString allocated by the plugin</td></tr>
</tbody>
</table>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Quick Workflow</h3>
<ol class="text-[12px] text-foreground/85 space-y-1 pl-5 list-decimal">
<li>Configure your plugin name, blocks, and settings in the <strong>Plugin Config</strong> tab</li>
<li>Click <strong>Regenerate</strong> to update the generated code</li>
<li>Copy or download the files</li>
<li>Add your custom logic in the <code>// TODO</code> sections</li>
<li>Build: <kbd>cargo build --release</kbd></li>
<li>Copy the DLL to your plugins directory</li>
<li>Restart Ironbullet or re-scan plugins</li>
</ol>
