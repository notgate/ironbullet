<h3 class="text-sm font-semibold text-foreground mb-3">Dependencies</h3>
<p class="text-[12px] text-muted-foreground mb-3">Your plugin is a standard Rust crate. You can add any crate from <a href="https://crates.io" class="text-blue-400 underline">crates.io</a> to your <code>Cargo.toml</code>.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Required Dependencies</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">[dependencies]
serde = {'{'} version = "1", features = ["derive"] {'}'}
serde_json = "1"</pre>
<p class="text-[12px] text-muted-foreground mb-3"><code>serde</code> and <code>serde_json</code> are mandatory -- they handle the JSON encoding/decoding for settings and variables.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Common Useful Crates</h3>
<table class="w-full text-[11px] border border-border rounded overflow-hidden mb-3">
<thead><tr class="bg-accent/10 border-b border-border">
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Crate</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Purpose</th>
<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Cargo.toml</th>
</tr></thead>
<tbody>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">regex</td><td class="px-3 py-1.5 text-foreground/80">Regular expressions</td><td class="px-3 py-1.5 font-mono text-[10px]">regex = "1"</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">scraper</td><td class="px-3 py-1.5 text-foreground/80">HTML/CSS parsing</td><td class="px-3 py-1.5 font-mono text-[10px]">scraper = "0.20"</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">sha2 / md5</td><td class="px-3 py-1.5 text-foreground/80">Hashing</td><td class="px-3 py-1.5 font-mono text-[10px]">sha2 = "0.10"</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">base64</td><td class="px-3 py-1.5 text-foreground/80">Base64 encode/decode</td><td class="px-3 py-1.5 font-mono text-[10px]">base64 = "0.22"</td></tr>
<tr class="border-b border-border/30"><td class="px-3 py-1.5 font-mono">rand</td><td class="px-3 py-1.5 text-foreground/80">Random generation</td><td class="px-3 py-1.5 font-mono text-[10px]">rand = "0.8"</td></tr>
<tr class="border-b border-border/30 bg-accent/5"><td class="px-3 py-1.5 font-mono">url</td><td class="px-3 py-1.5 text-foreground/80">URL parsing</td><td class="px-3 py-1.5 font-mono text-[10px]">url = "2"</td></tr>
<tr><td class="px-3 py-1.5 font-mono">chrono</td><td class="px-3 py-1.5 text-foreground/80">Date/time</td><td class="px-3 py-1.5 font-mono text-[10px]">chrono = "0.4"</td></tr>
</tbody>
</table>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Important: crate-type</h3>
<p class="text-[12px] text-muted-foreground mb-3">Your <code>Cargo.toml</code> <strong>must</strong> include <code>crate-type = ["cdylib"]</code> under <code>[lib]</code>. This tells Cargo to produce a C-compatible dynamic library (.dll) instead of a Rust-only .rlib.</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">[lib]
crate-type = ["cdylib"]</pre>
<p class="text-[12px] text-muted-foreground mb-3"><strong>Note:</strong> Async crates (tokio, reqwest) work but your <code>plugin_execute</code> is called synchronously. If you need async, create a tokio runtime inside your execute function:</p>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">let rt = tokio::runtime::Runtime::new().unwrap();
let result = rt.block_on(async {'{'}
    // async code here
{'}'});</pre>
