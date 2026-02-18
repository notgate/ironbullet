<script lang="ts">
	import { CATEGORIES } from './types';
</script>

<h3 class="text-sm font-semibold text-foreground mb-3">Block Definition</h3>
<p class="text-[12px] text-muted-foreground mb-3">Each block your plugin provides is described by a <code>BlockInfo</code> struct returned from <code>plugin_block_info(index)</code>. The index is 0-based and must be less than the <code>block_count</code> you reported in <code>plugin_info()</code>.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Type Name Convention</h3>
<p class="text-[12px] text-muted-foreground mb-3">The <code>block_type_name</code> must follow the <code>PluginName.BlockName</code> convention. This is how ironbullet identifies and routes execution to your block. Example: <code>"MyPlugin.ReverseString"</code>.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Categories</h3>
<p class="text-[12px] text-muted-foreground mb-2">Choose a category that matches your block's purpose:</p>
<div class="flex flex-wrap gap-1.5 mb-3">
{#each CATEGORIES as cat}
<span class="text-[10px] px-2 py-0.5 rounded border border-border text-foreground/70">{cat}</span>
{/each}
</div>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Icon Names</h3>
<p class="text-[12px] text-muted-foreground mb-3">Icons use <a href="https://lucide.dev/icons/" class="text-blue-400 underline">Lucide icon names</a> (lowercase, hyphenated). Examples: <code>puzzle</code>, <code>repeat</code>, <code>globe</code>, <code>database</code>, <code>key</code>, <code>shield</code>.</p>

<h3 class="text-sm font-semibold text-foreground mb-2 mt-4">Example: Multiple Blocks</h3>
<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 mb-3">// Plugin with 2 blocks
fn get_plugin_info() -> &'static PluginInfo {'{'}
    PLUGIN_INFO.get_or_init(|| PluginInfo {'{'}
        name: leak_cstring("StringTools"),
        block_count: 2,
        // ...
    {'}'})
{'}'}

fn plugin_block_info(index: u32) -> *const BlockInfo {'{'}
    match index {'{'}
        0 => get_reverse_block(),
        1 => get_uppercase_block(),
        _ => std::ptr::null(),
    {'}'}
{'}'}</pre>
