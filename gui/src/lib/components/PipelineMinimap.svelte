<script lang="ts">
	import { onMount } from 'svelte';
	import { getBlockColor, type Block } from '$lib/types';
	import { app } from '$lib/state.svelte';

	let { blocks, scrollContainer }: { blocks: Block[]; scrollContainer?: HTMLDivElement } = $props();

	let minimapEl: HTMLDivElement | undefined = $state();
	let viewportTop = $state(0);
	let viewportHeight = $state(100);
	let totalHeight = $state(1);

	const MINIMAP_HEIGHT = 120;
	const BLOCK_HEIGHT = 3;
	const GAP = 1;

	function updateViewport() {
		if (!scrollContainer) return;
		const sh = scrollContainer.scrollHeight || 1;
		const ch = scrollContainer.clientHeight;
		const st = scrollContainer.scrollTop;
		totalHeight = sh;
		viewportTop = (st / sh) * MINIMAP_HEIGHT;
		viewportHeight = Math.max(8, (ch / sh) * MINIMAP_HEIGHT);
	}

	$effect(() => {
		if (!scrollContainer) return;
		updateViewport();
		scrollContainer.addEventListener('scroll', updateViewport);
		const ro = new ResizeObserver(updateViewport);
		ro.observe(scrollContainer);
		return () => {
			scrollContainer!.removeEventListener('scroll', updateViewport);
			ro.disconnect();
		};
	});

	// Also recalc when blocks change
	$effect(() => {
		void blocks.length;
		setTimeout(updateViewport, 50);
	});

	function onMinimapClick(e: MouseEvent) {
		if (!scrollContainer || !minimapEl) return;
		const rect = minimapEl.getBoundingClientRect();
		const y = e.clientY - rect.top;
		const ratio = y / MINIMAP_HEIGHT;
		scrollContainer.scrollTop = ratio * scrollContainer.scrollHeight - scrollContainer.clientHeight / 2;
	}

	function getNestedCount(block: Block): number {
		const s = block.settings as any;
		let count = 0;
		if (s.true_blocks) count += s.true_blocks.length;
		if (s.false_blocks) count += s.false_blocks.length;
		if (s.blocks) count += s.blocks.length;
		return count;
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	bind:this={minimapEl}
	class="absolute top-1 right-1 w-[50px] bg-surface/80 border border-border/50 rounded overflow-hidden cursor-pointer z-10 backdrop-blur-sm"
	style="height: {MINIMAP_HEIGHT}px"
	onclick={onMinimapClick}
>
	<!-- Block bars -->
	<div class="p-0.5">
		{#each blocks as block, i}
			{@const color = getBlockColor(block.block_type)}
			{@const nested = getNestedCount(block)}
			<div
				class="rounded-sm mb-px {block.disabled ? 'opacity-30' : ''}"
				style="background: {color}; height: {BLOCK_HEIGHT}px; {app.selectedBlockIds.includes(block.id) ? 'box-shadow: 0 0 0 1px var(--primary);' : ''}"
				title="{block.label}"
			></div>
			{#if nested > 0 && !app.collapsedBlockIds.has(block.id)}
				{#each Array(Math.min(nested, 5)) as _, j}
					<div
						class="rounded-sm mb-px ml-1 opacity-60"
						style="background: {color}; height: {BLOCK_HEIGHT - 1}px;"
					></div>
				{/each}
			{/if}
		{/each}
	</div>

	<!-- Viewport indicator -->
	<div
		class="absolute left-0 right-0 border border-primary/60 bg-primary/10 rounded-sm pointer-events-none"
		style="top: {viewportTop}px; height: {viewportHeight}px;"
	></div>
</div>
