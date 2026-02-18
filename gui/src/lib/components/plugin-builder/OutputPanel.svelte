<script lang="ts">
	import Terminal from '@lucide/svelte/icons/terminal';
	import X from '@lucide/svelte/icons/x';

	let {
		outputLines,
		isCompiling,
		lastBuildSuccess,
		outputHeight,
		onClear,
		onClose,
		onResizeStart,
	}: {
		outputLines: Array<{ text: string; type: 'info' | 'error' | 'success' | 'cmd' }>;
		isCompiling: boolean;
		lastBuildSuccess: boolean | null;
		outputHeight: number;
		onClear: () => void;
		onClose: () => void;
		onResizeStart: (e: MouseEvent) => void;
	} = $props();

	let outputEl: HTMLDivElement;

	$effect(() => {
		// Auto-scroll when outputLines changes
		if (outputLines.length && outputEl) {
			requestAnimationFrame(() => { outputEl.scrollTop = outputEl.scrollHeight; });
		}
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="groove-h resize-handle-h h-[3px] shrink-0" onmousedown={onResizeStart}></div>
<div class="bg-[#1a1a1d] shrink-0" style="height: {outputHeight}px;">
	<div class="flex items-center gap-1.5 px-2 py-1 border-b border-border/50 bg-surface/30">
		<Terminal size={10} class="text-muted-foreground" />
		<span class="text-[10px] text-muted-foreground font-medium">Output</span>
		{#if isCompiling}
			<span class="text-[9px] text-yellow-400 animate-pulse">compiling...</span>
		{:else if lastBuildSuccess === true}
			<span class="text-[9px] text-green">build ok</span>
		{:else if lastBuildSuccess === false}
			<span class="text-[9px] text-red-400">build failed</span>
		{/if}
		<div class="flex-1"></div>
		<button class="text-[9px] text-muted-foreground hover:text-foreground" onclick={onClear}>Clear</button>
		<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground" onclick={onClose}>
			<X size={10} />
		</button>
	</div>
	<div class="output-scroll overflow-auto h-[calc(100%-26px)] px-2 py-1 font-mono text-[11px]" bind:this={outputEl}>
		{#each outputLines as line}
			<div class="leading-[18px] {line.type === 'error' ? 'text-red-400' : line.type === 'success' ? 'text-green' : line.type === 'cmd' ? 'text-blue-400' : 'text-foreground/80'}">
				{line.text || '\u00A0'}
			</div>
		{/each}
		{#if outputLines.length === 0}
			<div class="text-muted-foreground/50 text-[10px] mt-2">Click Build to compile your plugin, or Inspect to validate config.</div>
		{/if}
	</div>
</div>

<style>
	.output-scroll {
		user-select: text;
		cursor: text;
	}
	.output-scroll :global(div) {
		user-select: text;
	}
</style>
