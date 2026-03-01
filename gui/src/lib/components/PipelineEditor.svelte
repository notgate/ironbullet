<script lang="ts">
	import BlockRenderer from './BlockRenderer.svelte';
	import BlockList from './BlockList.svelte';
	import PipelineMinimap from './PipelineMinimap.svelte';
	import { app, pushUndo, collapseAllBlocks, expandAllBlocks, blockMatchesSearch } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import Plus from '@lucide/svelte/icons/plus';
	import Search from '@lucide/svelte/icons/search';
	import X from '@lucide/svelte/icons/x';
	import ChevronsDownUp from '@lucide/svelte/icons/chevrons-down-up';
	import ChevronsUpDown from '@lucide/svelte/icons/chevrons-up-down';
	import Map from '@lucide/svelte/icons/map';
	import Scan from '@lucide/svelte/icons/scan';
	import Save from '@lucide/svelte/icons/save';

	let dragOverIndex = $state<number | null>(null);
	let isDraggingOver = $state(false);

	// Rubber band selection
	let listEl: HTMLDivElement | undefined = $state();
	let selecting = $state(false);
	let selStart = $state({ x: 0, y: 0 });
	let selCurr = $state({ x: 0, y: 0 });

	// Search bar ref
	let searchInputEl: HTMLInputElement | undefined = $state();

	// Expose focus method for keyboard shortcut
	export function focusSearch() {
		app.pipelineSearchFocused = true;
		setTimeout(() => searchInputEl?.focus(), 0);
	}

	// Count matching blocks for search
	let searchMatchCount = $derived(
		app.pipelineSearchQuery
			? app.pipeline.blocks.filter(b => blockMatchesSearch(b, app.pipelineSearchQuery)).length
			: 0
	);

	function clearSearch() {
		app.pipelineSearchQuery = '';
		app.pipelineSearchFocused = false;
	}

	function onSearchKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { clearSearch(); }
	}

	function selRect() {
		const x = Math.min(selStart.x, selCurr.x);
		const y = Math.min(selStart.y, selCurr.y);
		const w = Math.abs(selCurr.x - selStart.x);
		const h = Math.abs(selCurr.y - selStart.y);
		return { x, y, w, h };
	}

	function onListMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		const target = e.target as HTMLElement;
		if (target.closest('[data-block-id]') || target.closest('.drop-zone')) return;
		if (e.dataTransfer) return;

		selecting = true;
		const rect = listEl!.getBoundingClientRect();
		const z = app.zoom || 1;
		selStart = { x: (e.clientX - rect.left) / z + listEl!.scrollLeft, y: (e.clientY - rect.top) / z + listEl!.scrollTop };
		selCurr = { ...selStart };

		if (!e.ctrlKey && !e.metaKey) {
			app.selectedBlockIds = [];
		}

		const onMove = (ev: MouseEvent) => {
			if (!selecting || !listEl) return;
			const r = listEl.getBoundingClientRect();
			const z = app.zoom || 1;
			selCurr = { x: (ev.clientX - r.left) / z + listEl.scrollLeft, y: (ev.clientY - r.top) / z + listEl.scrollTop };
			const sr = selRect();
			const blocks = listEl.querySelectorAll('[data-block-id]');
			const ids: string[] = [];
			blocks.forEach((el) => {
				const br = el.getBoundingClientRect();
				const bx = (br.left - r.left) / z + listEl!.scrollLeft;
				const by = (br.top - r.top) / z + listEl!.scrollTop;
				const bw = br.width / z;
				const bh = br.height / z;
				if (bx < sr.x + sr.w && bx + bw > sr.x && by < sr.y + sr.h && by + bh > sr.y) {
					ids.push(el.getAttribute('data-block-id')!);
				}
			});
			if (e.ctrlKey || e.metaKey) {
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

		// Multi-select drag
		const selectedIds = e.dataTransfer?.getData('application/x-block-ids');
		if (selectedIds) {
			const ids: string[] = JSON.parse(selectedIds);
			pushUndo();
			send('move_blocks_to', { ids, to: app.pipeline.blocks.length });
			return;
		}

		const fromIndex = e.dataTransfer?.getData('application/x-block-index');
		const blockType = e.dataTransfer?.getData('text/plain');

		if (fromIndex !== undefined && fromIndex !== '') {
			const from = parseInt(fromIndex);
			const to = app.pipeline.blocks.length;
			if (from !== to && from !== to - 1) {
				pushUndo();
				send('move_block', { from, to });
			}
		} else if (blockType) {
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

		// Multi-select drag
		const selectedIds = e.dataTransfer?.getData('application/x-block-ids');
		if (selectedIds) {
			const ids: string[] = JSON.parse(selectedIds);
			pushUndo();
			send('move_blocks_to', { ids, to: index });
			return;
		}

		const fromIndex = e.dataTransfer?.getData('application/x-block-index');
		const blockType = e.dataTransfer?.getData('text/plain');

		if (fromIndex !== undefined && fromIndex !== '') {
			const from = parseInt(fromIndex);
			let to = index;
			if (from < to) to = to - 1;
			if (from !== to) {
				pushUndo();
				send('move_block', { from, to });
			}
		} else if (blockType) {
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
	<div class="flex items-center px-2 py-1 bg-surface panel-raised gap-1">
		<span class="text-xs text-muted-foreground shrink-0">Pipeline Blocks</span>

		<div class="flex-1"></div>

		<!-- Search bar (inline, toggleable) -->
		{#if app.pipelineSearchFocused || app.pipelineSearchQuery}
			<div class="flex items-center gap-1 max-w-[200px]">
				<Search size={10} class="text-muted-foreground shrink-0" />
				<input
					bind:this={searchInputEl}
					type="text"
					placeholder="Search blocks..."
					class="flex-1 bg-transparent border-none outline-none text-[11px] text-foreground placeholder:text-muted-foreground/50 min-w-0"
					bind:value={app.pipelineSearchQuery}
					onkeydown={onSearchKeydown}
					onblur={() => { if (!app.pipelineSearchQuery) app.pipelineSearchFocused = false; }}
				/>
				{#if app.pipelineSearchQuery}
					<span class="text-[9px] text-muted-foreground shrink-0">{searchMatchCount}</span>
				{/if}
				<button class="p-0.5 rounded hover:bg-secondary text-muted-foreground" onclick={clearSearch}><X size={10} /></button>
			</div>
		{/if}

		<div class="flex items-center gap-0.5">
			<button class="p-0.5 rounded hover:bg-secondary text-muted-foreground {app.pipelineSearchFocused || app.pipelineSearchQuery ? 'text-primary' : ''}" onclick={focusSearch} title="Search (Ctrl+F)"><Search size={11} /></button>
			<button class="p-0.5 rounded hover:bg-secondary text-muted-foreground" onclick={collapseAllBlocks} title="Collapse All"><ChevronsDownUp size={11} /></button>
			<button class="p-0.5 rounded hover:bg-secondary text-muted-foreground" onclick={expandAllBlocks} title="Expand All"><ChevronsUpDown size={11} /></button>
			<button class="p-0.5 rounded hover:bg-secondary {app.showMinimap ? 'text-primary' : 'text-muted-foreground'}" onclick={() => { app.showMinimap = !app.showMinimap; }} title="Toggle Minimap"><Map size={11} /></button>
			<button class="p-0.5 rounded hover:bg-secondary {app.previewMode ? 'text-primary' : 'text-muted-foreground'}" onclick={() => { app.previewMode = !app.previewMode; }} title="Preview Variables"><Scan size={11} /></button>
			<button class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-green" onclick={() => send('save_pipeline', {})} title="Save Config (Ctrl+S)"><Save size={11} /></button>
		</div>

		<span class="text-[10px] text-muted-foreground shrink-0">{app.pipeline.blocks.length} block{app.pipeline.blocks.length !== 1 ? 's' : ''}</span>
	</div>

	<!-- Block list + minimap container -->
	<div class="flex-1 flex overflow-hidden relative">
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="flex-1 overflow-y-auto p-1.5 panel-inset relative" bind:this={listEl} onmousedown={onListMouseDown}>
			{#if app.pipeline.blocks.length === 0}
				<div class="flex flex-col items-center justify-center h-full text-muted-foreground text-xs gap-2">
					<Plus size={32} class="opacity-30" />
					<p>Drag blocks from the palette</p>
					<p class="text-[10px]">or double-click to add</p>
				</div>
			{:else}
				<div
					class="drop-zone {dragOverIndex === 0 ? 'active' : ''}"
					ondragover={(e) => onDropZoneDragOver(e, 0)}
					ondragleave={onDropZoneDragLeave}
					ondrop={(e) => onDropZoneDrop(e, 0)}
				></div>

				{#each app.pipeline.blocks as block, i (block.id)}
					<BlockRenderer {block} index={i} />

					{#if block.settings.type === 'IfElse' && !app.collapsedBlockIds.has(block.id)}
						<div class="ml-4 border-l-2 border-[#dcdcaa]/30 pl-2 my-0.5">
							<div class="text-[9px] text-green-400/70 uppercase tracking-wider mb-0.5 pl-1">If True</div>
							<BlockList blocks={block.settings.true_blocks} depth={1} parentId={block.id} branch="true" />
							<div class="text-[9px] text-red-400/70 uppercase tracking-wider mb-0.5 mt-1 pl-1">Else</div>
							<BlockList blocks={block.settings.false_blocks} depth={1} parentId={block.id} branch="false" />
						</div>
					{/if}

					{#if block.settings.type === 'Loop' && !app.collapsedBlockIds.has(block.id)}
						<div class="ml-4 border-l-2 border-[#dcdcaa]/30 pl-2 my-0.5">
							<div class="text-[9px] text-blue-400/70 uppercase tracking-wider mb-0.5 pl-1">Loop Body</div>
							<BlockList blocks={block.settings.blocks} depth={1} parentId={block.id} branch="body" />
						</div>
					{/if}

					{#if block.settings.type === 'Group' && !app.collapsedBlockIds.has(block.id) && !block.settings.collapsed}
						<div class="ml-4 border-l-2 border-[#858585]/30 pl-2 my-0.5">
							<BlockList blocks={block.settings.blocks} depth={1} parentId={block.id} branch="body" />
						</div>
					{/if}

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

		<!-- Minimap overlay -->
		{#if app.showMinimap && app.pipeline.blocks.length > 3}
			<PipelineMinimap blocks={app.pipeline.blocks} scrollContainer={listEl} />
		{/if}
	</div>
</div>
