<script lang="ts">
	import BlockRenderer from './BlockRenderer.svelte';
	import BlockList from './BlockList.svelte';
	import { app, pushUndo } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import Plus from '@lucide/svelte/icons/plus';

	let dragOverIndex = $state<number | null>(null);
	let isDraggingOver = $state(false);

	// Rubber band selection
	let listEl: HTMLDivElement | undefined = $state();
	let selecting = $state(false);
	let selStart = $state({ x: 0, y: 0 });
	let selCurr = $state({ x: 0, y: 0 });

	function selRect() {
		const x = Math.min(selStart.x, selCurr.x);
		const y = Math.min(selStart.y, selCurr.y);
		const w = Math.abs(selCurr.x - selStart.x);
		const h = Math.abs(selCurr.y - selStart.y);
		return { x, y, w, h };
	}

	function onListMouseDown(e: MouseEvent) {
		// Only start rubber band on left click, not on blocks or drop zones
		if (e.button !== 0) return;
		const target = e.target as HTMLElement;
		if (target.closest('[data-block-id]') || target.closest('.drop-zone')) return;
		// Don't interfere with drag-and-drop from palette
		if (e.dataTransfer) return;

		selecting = true;
		const rect = listEl!.getBoundingClientRect();
		selStart = { x: e.clientX - rect.left + listEl!.scrollLeft, y: e.clientY - rect.top + listEl!.scrollTop };
		selCurr = { ...selStart };

		if (!e.ctrlKey && !e.metaKey) {
			app.selectedBlockIds = [];
		}

		const onMove = (ev: MouseEvent) => {
			if (!selecting || !listEl) return;
			const r = listEl.getBoundingClientRect();
			selCurr = { x: ev.clientX - r.left + listEl.scrollLeft, y: ev.clientY - r.top + listEl.scrollTop };
			// Find blocks intersecting the selection rectangle
			const sr = selRect();
			const blocks = listEl.querySelectorAll('[data-block-id]');
			const ids: string[] = [];
			blocks.forEach((el) => {
				const br = el.getBoundingClientRect();
				const bx = br.left - r.left + listEl!.scrollLeft;
				const by = br.top - r.top + listEl!.scrollTop;
				const bw = br.width;
				const bh = br.height;
				// AABB intersection
				if (bx < sr.x + sr.w && bx + bw > sr.x && by < sr.y + sr.h && by + bh > sr.y) {
					ids.push(el.getAttribute('data-block-id')!);
				}
			});
			if (e.ctrlKey || e.metaKey) {
				// Additive: merge with existing selection
				const existing = new Set(app.selectedBlockIds);
				ids.forEach(id => existing.add(id));
				app.selectedBlockIds = [...existing];
			} else {
				app.selectedBlockIds = ids;
			}
		};

		const onUp = () => {
			selecting = false;
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		};

		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	function onDragOver(e: DragEvent) {
		e.preventDefault();
		isDraggingOver = true;
	}

	function onDragLeave(e: DragEvent) {
		// Only set false if we're leaving the container entirely
		const related = e.relatedTarget as HTMLElement | null;
		if (!related || !e.currentTarget || !(e.currentTarget as HTMLElement).contains(related)) {
			isDraggingOver = false;
			dragOverIndex = null;
		}
	}

	function onDrop(e: DragEvent) {
		e.preventDefault();
		isDraggingOver = false;
		dragOverIndex = null;
		const fromIndex = e.dataTransfer?.getData('application/x-block-index');
		const blockType = e.dataTransfer?.getData('text/plain');

		if (fromIndex !== undefined && fromIndex !== '') {
			// Reorder: move to end
			const from = parseInt(fromIndex);
			const to = app.pipeline.blocks.length;
			if (from !== to && from !== to - 1) {
				pushUndo();
				send('move_block', { from, to });
			}
		} else if (blockType) {
			// Palette drag: add at end
			pushUndo();
			try {
				const parsed = JSON.parse(blockType);
				if (parsed.type === 'Plugin') {
					send('add_block', { block_type: 'Plugin', plugin_block_type: parsed.plugin_block_type,
						settings_json: parsed.settings_json || '{}', label: parsed.label });
				} else {
					send('add_block', { block_type: blockType });
				}
			} catch {
				send('add_block', { block_type: blockType });
			}
		}
	}

	function onDropZoneDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		e.stopPropagation();
		dragOverIndex = index;
	}

	function onDropZoneDragLeave(e: DragEvent) {
		const related = e.relatedTarget as HTMLElement | null;
		const target = e.currentTarget as HTMLElement;
		if (!related || !target.contains(related)) {
			dragOverIndex = null;
		}
	}

	function onDropZoneDrop(e: DragEvent, index: number) {
		e.preventDefault();
		e.stopPropagation();
		isDraggingOver = false;
		dragOverIndex = null;
		const fromIndex = e.dataTransfer?.getData('application/x-block-index');
		const blockType = e.dataTransfer?.getData('text/plain');

		if (fromIndex !== undefined && fromIndex !== '') {
			const from = parseInt(fromIndex);
			let to = index;
			// When dragging forward, account for the removal shifting indices
			if (from < to) to = to - 1;
			if (from !== to) {
				pushUndo();
				send('move_block', { from, to });
			}
		} else if (blockType) {
			// Palette drag: insert at position
			pushUndo();
			try {
				const parsed = JSON.parse(blockType);
				if (parsed.type === 'Plugin') {
					send('add_block', { block_type: 'Plugin', plugin_block_type: parsed.plugin_block_type,
						settings_json: parsed.settings_json || '{}', label: parsed.label, index });
				} else {
					send('add_block', { block_type: blockType, index });
				}
			} catch {
				send('add_block', { block_type: blockType, index });
			}
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="flex flex-col h-full bg-background"
	ondragover={onDragOver}
	ondragleave={onDragLeave}
	ondrop={onDrop}
>
	<!-- Canvas header -->
	<div class="flex items-center justify-between px-2 py-1 bg-surface panel-raised">
		<span class="text-xs text-muted-foreground">Pipeline Blocks</span>
		<span class="text-[10px] text-muted-foreground">{app.pipeline.blocks.length} block{app.pipeline.blocks.length !== 1 ? 's' : ''}</span>
	</div>

	<!-- Block list -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="flex-1 overflow-y-auto p-1.5 panel-inset relative" bind:this={listEl} onmousedown={onListMouseDown}>
		{#if app.pipeline.blocks.length === 0}
			<div class="flex flex-col items-center justify-center h-full text-muted-foreground text-xs gap-2">
				<Plus size={32} class="opacity-30" />
				<p>Drag blocks from the palette</p>
				<p class="text-[10px]">or double-click to add</p>
			</div>
		{:else}
			<!-- Drop zone before first block -->
			<div
				class="drop-zone {dragOverIndex === 0 ? 'active' : ''}"
				ondragover={(e) => onDropZoneDragOver(e, 0)}
				ondragleave={onDropZoneDragLeave}
				ondrop={(e) => onDropZoneDrop(e, 0)}
			></div>

			{#each app.pipeline.blocks as block, i (block.id)}
				<BlockRenderer {block} index={i} />

				{#if block.settings.type === 'IfElse'}
					<div class="ml-4 border-l-2 border-[#dcdcaa]/30 pl-2 my-0.5">
						<div class="text-[9px] text-green-400/70 uppercase tracking-wider mb-0.5 pl-1">If True</div>
						<BlockList blocks={block.settings.true_blocks} depth={1} parentId={block.id} branch="true" />
						<div class="text-[9px] text-red-400/70 uppercase tracking-wider mb-0.5 mt-1 pl-1">Else</div>
						<BlockList blocks={block.settings.false_blocks} depth={1} parentId={block.id} branch="false" />
					</div>
				{/if}

				{#if block.settings.type === 'Loop'}
					<div class="ml-4 border-l-2 border-[#dcdcaa]/30 pl-2 my-0.5">
						<div class="text-[9px] text-blue-400/70 uppercase tracking-wider mb-0.5 pl-1">Loop Body</div>
						<BlockList blocks={block.settings.blocks} depth={1} parentId={block.id} branch="body" />
					</div>
				{/if}

				{#if block.settings.type === 'Group' && !block.settings.collapsed}
					<div class="ml-4 border-l-2 border-[#858585]/30 pl-2 my-0.5">
						<BlockList blocks={block.settings.blocks} depth={1} parentId={block.id} branch="body" />
					</div>
				{/if}

				<!-- Drop zone after each block -->
				<div
					class="drop-zone {dragOverIndex === i + 1 ? 'active' : ''}"
					ondragover={(e) => onDropZoneDragOver(e, i + 1)}
					ondragleave={onDropZoneDragLeave}
					ondrop={(e) => onDropZoneDrop(e, i + 1)}
				></div>
			{/each}
		{/if}

		<!-- Rubber band selection rectangle -->
		{#if selecting}
			{@const r = selRect()}
			{#if r.w > 3 || r.h > 3}
				<div class="absolute pointer-events-none border border-primary/60 bg-primary/10 rounded-sm" style="left: {r.x}px; top: {r.y}px; width: {r.w}px; height: {r.h}px; z-index: 20;"></div>
			{/if}
		{/if}
	</div>
</div>
