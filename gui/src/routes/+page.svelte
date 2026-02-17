<script lang="ts">
	import { onMount } from 'svelte';
	import { DropdownMenu, Tabs } from 'bits-ui';
	import { registerCallbacks, send } from '$lib/ipc';
	import { app, undo, redo, pushUndo, zoomIn, zoomOut, zoomReset, selectAllBlocks, createNewTab } from '$lib/state.svelte';
	import TitleBar from '$lib/components/TitleBar.svelte';
	import ConfigTabBar from '$lib/components/ConfigTabBar.svelte';
	import BlockPalette from '$lib/components/BlockPalette.svelte';
	import PipelineEditor from '$lib/components/PipelineEditor.svelte';
	import BlockSettingsPanel from '$lib/components/BlockSettingsPanel.svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';
	import VariableInspector from '$lib/components/VariableInspector.svelte';
	import RunnerPanel from '$lib/components/RunnerPanel.svelte';
	import DebugPanel from '$lib/components/DebugPanel.svelte';
	import CodeView from '$lib/components/CodeView.svelte';
	import NetworkViewer from '$lib/components/NetworkViewer.svelte';
	import DataPanel from '$lib/components/DataPanel.svelte';
	import ResponseViewer from '$lib/components/ResponseViewer.svelte';
	import JobMonitor from '$lib/components/JobMonitor.svelte';
	import StartupDialog from '$lib/components/StartupDialog.svelte';
	import UnsavedDialog from '$lib/components/UnsavedDialog.svelte';
	import BlockDocsPanel from '$lib/components/BlockDocsPanel.svelte';
	import PluginBuilder from '$lib/components/PluginBuilder.svelte';
	import Onboarding from '$lib/components/Onboarding.svelte';
	import SecurityAlertDialog from '$lib/components/SecurityAlertDialog.svelte';
	import Toast from '$lib/components/Toast.svelte';
	import Undo2 from '@lucide/svelte/icons/undo-2';
	import Redo2 from '@lucide/svelte/icons/redo-2';
	import Play from '@lucide/svelte/icons/play';
	import Square from '@lucide/svelte/icons/square';
	import Bug from '@lucide/svelte/icons/bug';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import FolderOpen from '@lucide/svelte/icons/folder-open';

	import type { Block } from '$lib/types';

	let bottomPanelCollapsed = $state(false);

	// Block clipboard for Ctrl+C / Ctrl+V
	let clipboardBlocks: Block[] = [];

	onMount(() => {
		registerCallbacks();
	});

	// Push font settings to CSS custom properties so portals/popups inherit them
	$effect(() => {
		document.documentElement.style.setProperty('--font-family', `'${app.fontFamily}'`);
		document.documentElement.style.setProperty('--font-size', `${app.fontSize}px`);
		document.documentElement.style.setProperty('--app-zoom', `${app.zoom}`);
	});

	// Check if focus is in a text-editable element (input, textarea, contenteditable, Monaco)
	function isEditableFocused(): boolean {
		const el = document.activeElement;
		if (!el) return false;
		const tag = el.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA') return true;
		if ((el as HTMLElement).isContentEditable) return true;
		// Monaco editor uses a textarea inside .monaco-editor
		if (el.closest?.('.monaco-editor')) return true;
		return false;
	}

	// Keyboard shortcuts
	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && app.contextMenu) { app.contextMenu = null; return; }

		const editable = isEditableFocused();

		// Global shortcuts that always work (even in text inputs)
		if (e.ctrlKey && e.key === 's') { e.preventDefault(); send('save_pipeline', {}); return; }
		if (e.ctrlKey && e.key === 'o') { e.preventDefault(); send('load_pipeline'); return; }
		if (e.key === 'F1') { e.preventDefault(); app.showBlockDocs = true; app.blockDocsInitialType = null; return; }
		if (e.key === 'F5') { e.preventDefault(); send('debug_pipeline'); return; }
		if (e.ctrlKey && (e.key === '=' || e.key === '+')) { e.preventDefault(); zoomIn(); return; }
		if (e.ctrlKey && e.key === '-') { e.preventDefault(); zoomOut(); return; }
		if (e.ctrlKey && e.key === '0') { e.preventDefault(); zoomReset(); return; }
		if (e.ctrlKey && e.key === 't') { e.preventDefault(); createNewTab(); return; }

		// Block-editing shortcuts — only when NOT focused on a text input
		if (editable) return;

		if (e.ctrlKey && e.key === 'z') { e.preventDefault(); undo(); }
		else if (e.ctrlKey && e.key === 'y') { e.preventDefault(); redo(); }
		else if (e.key === 'Delete' && app.selectedBlockIds.length > 0) {
			pushUndo();
			send('remove_blocks', { ids: [...app.selectedBlockIds] });
			if (app.editingBlockId && app.selectedBlockIds.includes(app.editingBlockId)) app.editingBlockId = null;
			app.selectedBlockIds = [];
		}
		else if (e.ctrlKey && e.key === 'a') {
			e.preventDefault();
			selectAllBlocks();
		}
		else if (e.ctrlKey && e.key === 'c' && app.selectedBlockIds.length > 0) {
			e.preventDefault();
			const selected = app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id));
			clipboardBlocks = JSON.parse(JSON.stringify(selected));
		}
		else if (e.ctrlKey && e.key === 'x' && app.selectedBlockIds.length > 0) {
			// Cut: copy to clipboard then remove from pipeline
			e.preventDefault();
			const selected = app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id));
			clipboardBlocks = JSON.parse(JSON.stringify(selected));
			pushUndo();
			send('remove_blocks', { ids: [...app.selectedBlockIds] });
			if (app.editingBlockId && app.selectedBlockIds.includes(app.editingBlockId)) app.editingBlockId = null;
			app.selectedBlockIds = [];
		}
		else if (e.ctrlKey && e.key === 'v' && clipboardBlocks.length > 0) {
			e.preventDefault();
			pushUndo();
			send('paste_blocks', { blocks: JSON.parse(JSON.stringify(clipboardBlocks)) });
		}
		else if (e.ctrlKey && e.key === 'd' && app.selectedBlockIds.length > 0) {
			e.preventDefault();
			const selected = app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id));
			pushUndo();
			send('paste_blocks', { blocks: JSON.parse(JSON.stringify(selected)) });
		}
		else if (e.key === 'F2' && app.selectedBlockIds.length === 1) {
			e.preventDefault();
			app.renamingBlockId = app.selectedBlockIds[0];
		}
	}

	// Context menu helpers
	function ctxAction(fn: () => void) {
		fn();
		app.contextMenu = null;
	}

	function closeContextMenu() {
		app.contextMenu = null;
	}

	// Resize state
	let isResizingLeft = $state(false);
	let isResizingBottom = $state(false);

	function startResizeLeft(e: MouseEvent) {
		isResizingLeft = true;
		const startX = e.clientX;
		const startW = app.leftPanelWidth;
		const onMove = (ev: MouseEvent) => { app.leftPanelWidth = Math.max(120, Math.min(400, startW + ev.clientX - startX)); };
		const onUp = () => { isResizingLeft = false; window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); };
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	function startResizeBottom(e: MouseEvent) {
		isResizingBottom = true;
		const startY = e.clientY;
		const startH = app.bottomPanelHeight;
		const onMove = (ev: MouseEvent) => { app.bottomPanelHeight = Math.max(100, Math.min(500, startH - (ev.clientY - startY))); };
		const onUp = () => { isResizingBottom = false; window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); };
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	let multiSelected = $derived(app.selectedBlockIds.length > 1);
</script>

<svelte:window onkeydown={onKeydown} />

<div class="h-screen flex flex-col overflow-hidden">
	<TitleBar />
	<!-- Spacer for fixed title bar -->
	<div class="h-7 shrink-0"></div>

	<!-- Plugin Builder (replaces entire workspace) -->
	{#if app.showPluginBuilder}
		<div class="flex-1 overflow-hidden">
			<PluginBuilder />
		</div>
	{:else}
	<!-- Zoom wrapper: everything below title bar -->
	<div class="flex-1 flex flex-col overflow-hidden app-zoom-container" style="--app-zoom: {app.zoom}">

	<!-- Toolbar -->
	<div class="flex items-center bg-surface h-7 shrink-0 px-1 gap-0.5 border-b border-border">
		<!-- File menu -->
		<DropdownMenu.Root>
			<DropdownMenu.Trigger class="toolbar-trigger">File</DropdownMenu.Trigger>
			<DropdownMenu.Portal>
				<DropdownMenu.Content class="menu-content" sideOffset={2} align="start">
					<DropdownMenu.Item class="menu-item" onSelect={() => send('load_pipeline')}>
						Open Config <span class="menu-shortcut">Ctrl+O</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => send('save_pipeline', {})}>
						Save Config <span class="menu-shortcut">Ctrl+S</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => send('save_pipeline', { force_dialog: true })}>
						Save Config As...
					</DropdownMenu.Item>
					<DropdownMenu.Separator class="menu-sep" />
					<DropdownMenu.Item class="menu-item" onSelect={() => { send('generate_code', { pipeline: app.pipeline }); app.activeTab = 'code'; }}>
						Export as Rust Code
					</DropdownMenu.Item>
					<DropdownMenu.Separator class="menu-sep" />
					<DropdownMenu.Item class="menu-item" onSelect={() => send('import_config')}>
						Import .SVB / .OPK
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => send('import_plugin')}>
						Import Plugin (.dll)
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => { app.showPluginBuilder = true; }}>
						Plugin Builder
					</DropdownMenu.Item>
					<DropdownMenu.Separator class="menu-sep" />
					<DropdownMenu.Item class="menu-item" onSelect={() => { app.showSettings = true; }}>
						Settings
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Portal>
		</DropdownMenu.Root>

		<!-- Edit menu -->
		<DropdownMenu.Root>
			<DropdownMenu.Trigger class="toolbar-trigger">Edit</DropdownMenu.Trigger>
			<DropdownMenu.Portal>
				<DropdownMenu.Content class="menu-content" sideOffset={2} align="start">
					<DropdownMenu.Item class="menu-item" onSelect={() => undo()} disabled={!app.canUndo}>
						Undo <span class="menu-shortcut">Ctrl+Z</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => redo()} disabled={!app.canRedo}>
						Redo <span class="menu-shortcut">Ctrl+Y</span>
					</DropdownMenu.Item>
					<DropdownMenu.Separator class="menu-sep" />
					<DropdownMenu.Item class="menu-item" onSelect={() => { if (app.selectedBlockIds.length > 0) { clipboardBlocks = JSON.parse(JSON.stringify(app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id)))); } }} disabled={app.selectedBlockIds.length === 0}>
						Copy <span class="menu-shortcut">Ctrl+C</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => { if (app.selectedBlockIds.length > 0) { clipboardBlocks = JSON.parse(JSON.stringify(app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id)))); pushUndo(); send('remove_blocks', { ids: [...app.selectedBlockIds] }); app.selectedBlockIds = []; } }} disabled={app.selectedBlockIds.length === 0}>
						Cut <span class="menu-shortcut">Ctrl+X</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => { if (clipboardBlocks.length > 0) { pushUndo(); send('paste_blocks', { blocks: JSON.parse(JSON.stringify(clipboardBlocks)) }); } }} disabled={clipboardBlocks.length === 0}>
						Paste <span class="menu-shortcut">Ctrl+V</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => { if (app.selectedBlockIds.length > 0) { pushUndo(); send('paste_blocks', { blocks: JSON.parse(JSON.stringify(app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id)))) }); } }} disabled={app.selectedBlockIds.length === 0}>
						Duplicate <span class="menu-shortcut">Ctrl+D</span>
					</DropdownMenu.Item>
					<DropdownMenu.Separator class="menu-sep" />
					<DropdownMenu.Item class="menu-item" onSelect={() => selectAllBlocks()}>
						Select All <span class="menu-shortcut">Ctrl+A</span>
					</DropdownMenu.Item>
					<DropdownMenu.Separator class="menu-sep" />
					<DropdownMenu.Item class="menu-item" onSelect={() => { if (app.selectedBlockIds.length > 0) { pushUndo(); send('remove_blocks', { ids: [...app.selectedBlockIds] }); app.selectedBlockIds = []; } }} disabled={app.selectedBlockIds.length === 0}>
						Delete {app.selectedBlockIds.length > 1 ? `${app.selectedBlockIds.length} Blocks` : 'Block'} <span class="menu-shortcut">Del</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => { if (app.selectedBlockIds.length === 1) app.renamingBlockId = app.selectedBlockIds[0]; }} disabled={app.selectedBlockIds.length !== 1}>
						Rename Block <span class="menu-shortcut">F2</span>
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Portal>
		</DropdownMenu.Root>

		<!-- View menu -->
		<DropdownMenu.Root>
			<DropdownMenu.Trigger class="toolbar-trigger">View</DropdownMenu.Trigger>
			<DropdownMenu.Portal>
				<DropdownMenu.Content class="menu-content" sideOffset={2} align="start">
					<DropdownMenu.Item class="menu-item" onSelect={() => { app.showBlockPalette = !app.showBlockPalette; }}>
						{app.showBlockPalette ? 'Hide' : 'Show'} Block Palette
					</DropdownMenu.Item>
					<DropdownMenu.Separator class="menu-sep" />
					<DropdownMenu.Item class="menu-item" onSelect={() => zoomIn()}>
						Zoom In <span class="menu-shortcut">Ctrl+=</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => zoomOut()}>
						Zoom Out <span class="menu-shortcut">Ctrl+-</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => zoomReset()}>
						Reset Zoom <span class="menu-shortcut">Ctrl+0</span>
					</DropdownMenu.Item>
					<DropdownMenu.Separator class="menu-sep" />
					<DropdownMenu.Item class="menu-item" onSelect={() => { app.showBlockDocs = true; app.blockDocsInitialType = null; }}>
						Documentation <span class="menu-shortcut">F1</span>
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => { app.showPluginBuilder = true; }}>
						Plugin Builder
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Portal>
		</DropdownMenu.Root>

		<!-- Run menu -->
		<DropdownMenu.Root>
			<DropdownMenu.Trigger class="toolbar-trigger">Run</DropdownMenu.Trigger>
			<DropdownMenu.Portal>
				<DropdownMenu.Content class="menu-content" sideOffset={2} align="start">
					<DropdownMenu.Item class="menu-item" onSelect={() => send('debug_pipeline')}>
						Debug Run <span class="menu-shortcut">F5</span>
					</DropdownMenu.Item>
					<DropdownMenu.Separator class="menu-sep" />
					<DropdownMenu.Item class="menu-item" onSelect={() => { send('start_runner', { threads: app.pipeline.runner_settings.threads, wordlist_path: app.wordlistPath, proxy_path: app.proxyPath }); app.isRunning = true; }}>
						Start Runner
					</DropdownMenu.Item>
					<DropdownMenu.Item class="menu-item" onSelect={() => { send('stop_runner'); app.isRunning = false; app.isPaused = false; }} disabled={!app.isRunning}>
						Stop Runner
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Portal>
		</DropdownMenu.Root>

		<!-- Collections dropdown -->
		{#if app.collectionsPath}
			<DropdownMenu.Root onOpenChange={(open) => { if (open) send('list_collections'); }}>
				<DropdownMenu.Trigger class="toolbar-trigger flex items-center gap-1">
					<FolderOpen size={10} /> Collections
				</DropdownMenu.Trigger>
				<DropdownMenu.Portal>
					<DropdownMenu.Content class="menu-content" sideOffset={2} align="start">
						{#if app.collectionConfigs.length === 0}
							<DropdownMenu.Item class="menu-item" disabled>No configs found</DropdownMenu.Item>
						{:else}
							{#each app.collectionConfigs as cfg}
								<DropdownMenu.Item class="menu-item" onSelect={() => send('load_pipeline', { path: cfg.path })}>
									{cfg.name}
								</DropdownMenu.Item>
							{/each}
						{/if}
						<DropdownMenu.Separator class="menu-sep" />
						<DropdownMenu.Item class="menu-item text-muted-foreground" onSelect={() => send('browse_folder', { field: 'collections' })}>
							Change folder...
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Portal>
			</DropdownMenu.Root>
		{/if}

		<div class="flex-1"></div>

		<!-- Quick actions: Run/Debug -->
		<div class="flex items-center gap-0.5 mr-1">
			{#if app.isRunning}
				<button
					class="p-0.5 rounded transition-colors text-red hover:bg-secondary"
					onclick={() => { send('stop_runner'); app.isRunning = false; app.isPaused = false; }}
					title="Stop Runner"
				>
					<Square size={12} />
				</button>
			{:else}
				<button
					class="p-0.5 rounded transition-colors text-green hover:bg-secondary"
					onclick={() => { send('start_runner', { threads: app.pipeline.runner_settings.threads, wordlist_path: app.wordlistPath, proxy_path: app.proxyPath }); app.isRunning = true; }}
					title="Start Runner"
				>
					<Play size={12} />
				</button>
			{/if}
			<button
				class="p-0.5 rounded transition-colors text-muted-foreground hover:text-foreground hover:bg-secondary"
				onclick={() => send('debug_pipeline')}
				title="Debug Run (F5)"
			>
				<Bug size={12} />
			</button>
		</div>

		<!-- Divider -->
		<div class="w-px h-3.5 bg-border mx-0.5"></div>

		<!-- Quick undo/redo -->
		<div class="flex items-center gap-0.5 mr-1">
			<button
				class="p-0.5 rounded transition-colors {app.canUndo ? 'text-muted-foreground hover:text-foreground hover:bg-secondary' : 'text-muted-foreground/30 cursor-default'}"
				onclick={() => undo()}
				disabled={!app.canUndo}
				title="Undo (Ctrl+Z)"
			>
				<Undo2 size={12} />
			</button>
			<button
				class="p-0.5 rounded transition-colors {app.canRedo ? 'text-muted-foreground hover:text-foreground hover:bg-secondary' : 'text-muted-foreground/30 cursor-default'}"
				onclick={() => redo()}
				disabled={!app.canRedo}
				title="Redo (Ctrl+Y)"
			>
				<Redo2 size={12} />
			</button>
		</div>

		<span class="text-[10px] text-muted-foreground mr-1">{Math.round(app.zoom * 100)}%</span>
	</div>

	<!-- Config tab bar -->
	<ConfigTabBar />

	<!-- Main content -->
	<div class="flex-1 flex overflow-hidden">
		<!-- Left sidebar: Block Palette -->
		{#if app.showBlockPalette}
			<div style="width: {app.leftPanelWidth}px" class="shrink-0">
				<BlockPalette />
			</div>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="groove-v resize-handle w-[3px] shrink-0" onmousedown={startResizeLeft}></div>
		{/if}

		<!-- Center: Pipeline Editor + Bottom Panel -->
		<div class="flex-1 flex flex-col min-w-0">
			<div class="flex-1 min-h-0 flex">
				<div class="flex-1 min-w-0 relative">
					<PipelineEditor />
				</div>
				<BlockSettingsPanel />
			</div>

			{#if !bottomPanelCollapsed}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="groove-h resize-handle-h h-[3px] shrink-0" onmousedown={startResizeBottom}></div>
			{/if}

			<!-- Bottom panel with Tabs -->
			<div style="height: {bottomPanelCollapsed ? 'auto' : `${app.bottomPanelHeight}px`}" class="shrink-0 flex flex-col">
				<Tabs.Root bind:value={app.bottomTab} class="flex flex-col h-full">
					<div class="bg-surface shrink-0 px-1 flex items-center border-b border-border-dark">
						<Tabs.List class="flex gap-0 flex-1">
							<Tabs.Trigger value="debugger" class="tab-trigger">Debugger</Tabs.Trigger>
							<Tabs.Trigger value="runner" class="tab-trigger">Runner</Tabs.Trigger>
							<Tabs.Trigger value="code" class="tab-trigger">Code View</Tabs.Trigger>
							<Tabs.Trigger value="data" class="tab-trigger">Data / Proxy</Tabs.Trigger>
							<Tabs.Trigger value="jobs" class="tab-trigger">Jobs</Tabs.Trigger>
							<Tabs.Trigger value="network" class="tab-trigger">Network</Tabs.Trigger>
							<Tabs.Trigger value="variables" class="tab-trigger">Variables</Tabs.Trigger>
						</Tabs.List>
						<button
							class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors"
							onclick={() => { bottomPanelCollapsed = !bottomPanelCollapsed; }}
							title={bottomPanelCollapsed ? 'Expand panel' : 'Collapse panel'}
						>
							{#if bottomPanelCollapsed}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
						</button>
					</div>

					{#if !bottomPanelCollapsed}
						<Tabs.Content value="debugger" class="flex-1 overflow-hidden"><DebugPanel /></Tabs.Content>
						<Tabs.Content value="runner" class="flex-1 overflow-hidden"><RunnerPanel /></Tabs.Content>
						<Tabs.Content value="code" class="flex-1 overflow-hidden"><CodeView /></Tabs.Content>
						<Tabs.Content value="data" class="flex-1 overflow-hidden"><DataPanel /></Tabs.Content>
						<Tabs.Content value="jobs" class="flex-1 overflow-hidden"><JobMonitor /></Tabs.Content>
						<Tabs.Content value="network" class="flex-1 overflow-hidden"><NetworkViewer /></Tabs.Content>
						<Tabs.Content value="variables" class="flex-1 overflow-hidden"><VariableInspector /></Tabs.Content>
					{/if}
				</Tabs.Root>
			</div>
		</div>
	</div>
	</div>
	{/if}
</div>

<SettingsModal />
<ResponseViewer />
<StartupDialog />
<UnsavedDialog />
{#if app.showBlockDocs}
	<BlockDocsPanel />
{/if}
<Onboarding />
<SecurityAlertDialog />
<Toast />

<!-- Block context menu (manual, no bits-ui Trigger interference with drag) -->
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
			<button class="menu-item menu-item-danger w-full text-left" onclick={() => ctxAction(() => { pushUndo(); send('remove_block', { block_id: ctx.blockId }); app.selectedBlockIds = app.selectedBlockIds.filter(id => id !== ctx.blockId); if (app.editingBlockId === ctx.blockId) app.editingBlockId = null; })}>
				Delete <span class="menu-shortcut">Del</span>
			</button>
		{:else}
			<!-- Multi-select context menu -->
			<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { pushUndo(); send('toggle_blocks', { ids: [...app.selectedBlockIds], disabled: true }); })}>
				Disable {selCount} Blocks
			</button>
			<button class="menu-item w-full text-left" onclick={() => ctxAction(() => { pushUndo(); send('toggle_blocks', { ids: [...app.selectedBlockIds], disabled: false }); })}>
				Enable {selCount} Blocks
			</button>
			<div class="menu-sep"></div>
			<button class="menu-item menu-item-danger w-full text-left" onclick={() => ctxAction(() => { pushUndo(); send('remove_blocks', { ids: [...app.selectedBlockIds] }); app.selectedBlockIds = []; })}>
				Delete {selCount} Blocks <span class="menu-shortcut">Del</span>
			</button>
		{/if}
	</div>
{/if}
