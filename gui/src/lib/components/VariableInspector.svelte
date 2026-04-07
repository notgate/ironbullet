<script lang="ts">
	import { app } from '$lib/state.svelte';

	let result = $derived(app.debugResult);
	let vars = $derived(result?.variables_after || {});

	let ctxMenu = $state<{ x: number; y: number; text: string } | null>(null);

	function showCtx(e: MouseEvent, text: string) {
		e.preventDefault();
		ctxMenu = { x: e.clientX, y: e.clientY, text };
	}

	async function copyText(text: string) {
		try { await navigator.clipboard.writeText(text); } catch {}
		ctxMenu = null;
	}

	function selectAll() {
		const sel = window.getSelection();
		const range = document.createRange();
		const el = document.querySelector('.var-inspector-list');
		if (sel && el) { range.selectNodeContents(el); sel.removeAllRanges(); sel.addRange(range); }
		ctxMenu = null;
	}
</script>

<div class="flex flex-col h-full">
	<div class="px-2 py-1 panel-raised">
		<span class="text-[10px] uppercase tracking-wider text-muted-foreground">Variables</span>
	</div>
	<div class="flex-1 overflow-y-auto panel-inset var-inspector-list">
		{#if Object.keys(vars).length === 0}
			<div class="flex items-center justify-center h-full text-muted-foreground text-[10px]">
				Run debug to see variables
			</div>
		{:else}
			{#each Object.entries(vars) as [key, value]}
				<div
					class="flex px-2 py-0.5 border-b border-border/30 hover:bg-accent/10 text-[11px]"
					oncontextmenu={(e) => showCtx(e, `${key}=${String(value)}`)}
				>
					<span class="text-primary font-mono w-32 shrink-0 truncate" title={key}>{key}</span>
					<span class="text-foreground font-mono truncate" title={String(value)}>{String(value)}</span>
				</div>
			{/each}
		{/if}
	</div>
</div>

{#if ctxMenu}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-[9999]" onclick={() => ctxMenu = null} oncontextmenu={(e) => { e.preventDefault(); ctxMenu = null; }}>
		<div
			class="fixed bg-popover border border-border rounded shadow-lg py-1 text-xs min-w-[140px] z-[10000]"
			style="left:{ctxMenu.x}px;top:{ctxMenu.y}px"
		>
			<button class="w-full px-3 py-1 text-left hover:bg-accent/20" onclick={() => copyText(ctxMenu?.text || '')}>Copy</button>
			<button class="w-full px-3 py-1 text-left hover:bg-accent/20" onclick={() => copyText(ctxMenu?.text.split('=').slice(1).join('=') || '')}>Copy Value</button>
			<button class="w-full px-3 py-1 text-left hover:bg-accent/20" onclick={() => copyText(ctxMenu?.text.split('=')[0] || '')}>Copy Key</button>
			<div class="border-t border-border/50 my-0.5"></div>
			<button class="w-full px-3 py-1 text-left hover:bg-accent/20" onclick={selectAll}>Select All</button>
		</div>
	</div>
{/if}
