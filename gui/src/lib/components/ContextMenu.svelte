<script lang="ts">
	import { send } from '$lib/ipc';
	import { app, pushUndo, saveBlockTemplate, toggleBlockCollapse } from '$lib/state.svelte';

	let templateName = $state('');
	let showTemplateInput = $state(false);

	function ctxAction(fn: () => void) {
		fn();
		app.contextMenu = null;
		showTemplateInput = false;
	}

	function closeContextMenu() {
		app.contextMenu = null;
		showTemplateInput = false;
	}

	function handleSaveTemplate() {
		if (!templateName.trim()) return;
		const selectedBlocks = app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id));
		if (selectedBlocks.length === 0) {
			const block = app.pipeline.blocks.find(b => b.id === app.contextMenu?.blockId);
			if (block) saveBlockTemplate(templateName.trim(), [block]);
		} else {
			saveBlockTemplate(templateName.trim(), selectedBlocks);
		}
		templateName = '';
		showTemplateInput = false;
		app.contextMenu = null;
	}

	function isContainerBlock(blockId: string): boolean {
		const block = app.pipeline.blocks.find(b => b.id === blockId);
		if (!block) return false;
		const t = block.settings.type;
		return t === 'IfElse' || t === 'Loop' || t === 'Group';
	}
</script>

{#if app.contextMenu}
	{@const ctx = app.contextMenu}
	{@const idx = ctx.blockIndex}
	{@const blockCount = app.pipeline.blocks.length}
	{@const selCount = app.selectedBlockIds.length}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="fixed inset-0 top-[28px] z-40" onclick={closeContextMenu} oncontextmenu={(e) => { e.preventDefault(); closeContextMenu(); }}></div>
	<div class="menu-content fixed z-50" style="left: {ctx.x}px; top: {ctx.y}px;">
		{#if selCount <= 1}
			<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { app.editingBlockId = ctx.blockId; })}>
				Edit Settings
			</button>
			<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { app.renamingBlockId = ctx.blockId; })}>
				Rename <span class="menu-shortcut">F2</span>
			</button>
			<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { const b = app.pipeline.blocks.find(b => b.id === ctx.blockId); if (b) { pushUndo(); send('add_block', { block_type: b.block_type, index: idx + 1 }); } })}>
				Duplicate
			</button>
			{#if isContainerBlock(ctx.blockId)}
				<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { toggleBlockCollapse(ctx.blockId); })}>
					{app.collapsedBlockIds.has(ctx.blockId) ? 'Expand' : 'Collapse'}
				</button>
			{/if}
			<div class="menu-sep"></div>
			<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { const b = app.pipeline.blocks.find(b => b.id === ctx.blockId); if (b) { pushUndo(); b.disabled = !b.disabled; send('update_block', b); } })}>
				{app.pipeline.blocks.find(b => b.id === ctx.blockId)?.disabled ? 'Enable' : 'Disable'}
			</button>
			<div class="menu-sep"></div>
			<button class="menu-item w-full text-left" disabled={idx === 0} onclick={() => ctxAction(() => { pushUndo(); send('move_block', { from: idx, to: idx - 1 }); })}>
				Move Up
			</button>
			<button class="menu-item w-full text-left" disabled={idx >= blockCount - 1} onclick={() => ctxAction(() => { pushUndo(); send('move_block', { from: idx, to: idx + 1 }); })}>
				Move Down
			</button>
			<div class="menu-sep"></div>
			{#if showTemplateInput}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="px-2 py-1 flex items-center gap-1" onclick={(e) => e.stopPropagation()}>
					<!-- svelte-ignore a11y_autofocus -->
					<input
						type="text"
						class="skeu-input text-[10px] flex-1 min-w-0"
						placeholder="Template name..."
						bind:value={templateName}
						onkeydown={(e) => { if (e.key === 'Enter') handleSaveTemplate(); if (e.key === 'Escape') { showTemplateInput = false; } }}
						autofocus
					/>
					<button class="skeu-btn text-[9px] px-1.5" onclick={handleSaveTemplate}>Save</button>
				</div>
			{:else}
				<button class="menu-item w-full text-left" onclick={() => { showTemplateInput = true; }}>
					Save as Template
				</button>
			{/if}
			<div class="menu-sep"></div>
			<button class="menu-item menu-item-danger w-full text-left" onclick={() => ctxAction(() => { pushUndo(); send('remove_block', { block_id: ctx.blockId }); app.selectedBlockIds = app.selectedBlockIds.filter(id => id !== ctx.blockId); if (app.editingBlockId === ctx.blockId) app.editingBlockId = null; })}>
				Delete <span class="menu-shortcut">Del</span>
			</button>
		{:else}
			<!-- Multi-select context menu -->
			<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { pushUndo(); send('group_blocks', { ids: [...app.selectedBlockIds] }); })}>
				Group {selCount} Blocks
			</button>
			<div class="menu-sep"></div>
			<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { pushUndo(); send('toggle_blocks', { ids: [...app.selectedBlockIds], disabled: true }); })}>
				Disable {selCount} Blocks
			</button>
			<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { pushUndo(); send('toggle_blocks', { ids: [...app.selectedBlockIds], disabled: false }); })}>
				Enable {selCount} Blocks
			</button>
			<div class="menu-sep"></div>
			{#if showTemplateInput}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="px-2 py-1 flex items-center gap-1" onclick={(e) => e.stopPropagation()}>
					<!-- svelte-ignore a11y_autofocus -->
					<input
						type="text"
						class="skeu-input text-[10px] flex-1 min-w-0"
						placeholder="Template name..."
						bind:value={templateName}
						onkeydown={(e) => { if (e.key === 'Enter') handleSaveTemplate(); if (e.key === 'Escape') { showTemplateInput = false; } }}
						autofocus
					/>
					<button class="skeu-btn text-[9px] px-1.5" onclick={handleSaveTemplate}>Save</button>
				</div>
			{:else}
				<button class="menu-item w-full text-left" onclick={() => { showTemplateInput = true; }}>
					Save {selCount} Blocks as Template
				</button>
			{/if}
			<div class="menu-sep"></div>
			<button class="menu-item menu-item-danger w-full text-left" onclick={() => ctxAction(() => { pushUndo(); send('remove_blocks', { ids: [...app.selectedBlockIds] }); app.selectedBlockIds = []; })}>
				Delete {selCount} Blocks <span class="menu-shortcut">Del</span>
			</button>
		{/if}
	</div>
{/if}
