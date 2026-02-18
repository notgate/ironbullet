<script lang="ts">
	import BlockRenderer from './BlockRenderer.svelte';
	import type { Block } from '$lib/types';
	import { app, pushUndo } from '$lib/state.svelte';
	import { send } from '$lib/ipc';

	let { blocks, depth = 0, parentId = '', branch = '' }: { blocks: Block[]; depth?: number; parentId?: string; branch?: string } = $props();

	let nestedDragOverIndex = $state<number | null>(null);

	function onNestedDropZoneDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		e.stopPropagation();
		nestedDragOverIndex = index;
	}

	function onNestedDropZoneDragLeave(e: DragEvent) {
		const related = e.relatedTarget as HTMLElement | null;
		const target = e.currentTarget as HTMLElement;
		if (!related || !target.contains(related)) {
			nestedDragOverIndex = null;
		}
	}

	function onNestedDropZoneDrop(e: DragEvent, index: number) {
		e.preventDefault();
		e.stopPropagation();
		nestedDragOverIndex = null;

		if (!parentId || !branch) return;

		const blockId = e.dataTransfer?.getData('application/x-block-id');
		const blockType = e.dataTransfer?.getData('text/plain');

		if (blockId) {
			// Move existing block into this nested container
			pushUndo();
			send('move_block_to_nested', { block_id: blockId, parent_id: parentId, branch, index });
		} else if (blockType) {
			// Add new block from palette into nested container
			pushUndo();
			send('add_block_nested', { block_type: blockType, parent_id: parentId, branch, index });
		}
	}

	function onEmptyDrop(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		nestedDragOverIndex = null;

		if (!parentId || !branch) return;

		const blockId = e.dataTransfer?.getData('application/x-block-id');
		const blockType = e.dataTransfer?.getData('text/plain');

		if (blockId) {
			pushUndo();
			send('move_block_to_nested', { block_id: blockId, parent_id: parentId, branch });
		} else if (blockType) {
			pushUndo();
			send('add_block_nested', { block_type: blockType, parent_id: parentId, branch });
		}
	}

	function onEmptyDragOver(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		nestedDragOverIndex = 0;
	}

	function onEmptyDragLeave() {
		nestedDragOverIndex = null;
	}
</script>

{#if blocks.length > 0}
	<!-- Drop zone before first nested block -->
	{#if parentId}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="drop-zone {nestedDragOverIndex === 0 ? 'active' : ''}"
			ondragover={(e) => onNestedDropZoneDragOver(e, 0)}
			ondragleave={onNestedDropZoneDragLeave}
			ondrop={(e) => onNestedDropZoneDrop(e, 0)}
		></div>
	{/if}

	{#each blocks as block, i (block.id)}
		<BlockRenderer {block} index={i} />

		{#if block.settings.type === 'IfElse' && !app.collapsedBlockIds.has(block.id)}
			<div class="ml-4 border-l-2 border-[#dcdcaa]/30 pl-2 my-0.5" style="margin-left: {depth > 0 ? '0.5rem' : '1rem'}">
				<div class="text-[9px] text-green-400/70 uppercase tracking-wider mb-0.5 pl-1">If True</div>
				<svelte:self blocks={block.settings.true_blocks} depth={depth + 1} parentId={block.id} branch="true" />
				<div class="text-[9px] text-red-400/70 uppercase tracking-wider mb-0.5 mt-1 pl-1">Else</div>
				<svelte:self blocks={block.settings.false_blocks} depth={depth + 1} parentId={block.id} branch="false" />
			</div>
		{/if}

		{#if block.settings.type === 'Loop' && !app.collapsedBlockIds.has(block.id)}
			<div class="ml-4 border-l-2 border-[#dcdcaa]/30 pl-2 my-0.5" style="margin-left: {depth > 0 ? '0.5rem' : '1rem'}">
				<div class="text-[9px] text-blue-400/70 uppercase tracking-wider mb-0.5 pl-1">Loop Body</div>
				<svelte:self blocks={block.settings.blocks} depth={depth + 1} parentId={block.id} branch="body" />
			</div>
		{/if}

		{#if block.settings.type === 'Group' && !app.collapsedBlockIds.has(block.id) && !block.settings.collapsed}
			<div class="ml-4 border-l-2 border-[#858585]/30 pl-2 my-0.5" style="margin-left: {depth > 0 ? '0.5rem' : '1rem'}">
				<svelte:self blocks={block.settings.blocks} depth={depth + 1} parentId={block.id} branch="body" />
			</div>
		{/if}

		<!-- Drop zone after each nested block -->
		{#if parentId}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="drop-zone {nestedDragOverIndex === i + 1 ? 'active' : ''}"
				ondragover={(e) => onNestedDropZoneDragOver(e, i + 1)}
				ondragleave={onNestedDropZoneDragLeave}
				ondrop={(e) => onNestedDropZoneDrop(e, i + 1)}
			></div>
		{/if}
	{/each}
{:else}
	<!-- Empty drop target -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="py-1 px-1 {nestedDragOverIndex === 0 ? 'bg-primary/10 rounded border border-dashed border-primary/40' : ''}"
		ondragover={onEmptyDragOver}
		ondragleave={onEmptyDragLeave}
		ondrop={onEmptyDrop}
	>
		<div class="text-[9px] text-muted-foreground/40 italic pl-1">
			{#if nestedDragOverIndex === 0}
				drop here
			{:else}
				empty
			{/if}
		</div>
	</div>
{/if}
