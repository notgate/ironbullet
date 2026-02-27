<script lang="ts">
	import { DropdownMenu } from 'bits-ui';
	import { send } from '$lib/ipc';
	import { app, undo, redo, pushUndo, zoomIn, zoomOut, zoomReset, selectAllBlocks, collapseAllBlocks, expandAllBlocks } from '$lib/state.svelte';
	import Undo2 from '@lucide/svelte/icons/undo-2';
	import Redo2 from '@lucide/svelte/icons/redo-2';
	import Play from '@lucide/svelte/icons/play';
	import Square from '@lucide/svelte/icons/square';
	import Bug from '@lucide/svelte/icons/bug';

	import FolderOpen from '@lucide/svelte/icons/folder-open';

	import type { Block } from '$lib/types';

	interface Props {
		clipboardBlocks: Block[];
		onClipboardChange: (blocks: Block[]) => void;
	}

	let { clipboardBlocks, onClipboardChange }: Props = $props();
</script>

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
				<DropdownMenu.Item class="menu-item" onSelect={() => { if (app.selectedBlockIds.length > 0) { onClipboardChange(JSON.parse(JSON.stringify(app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id))))); } }} disabled={app.selectedBlockIds.length === 0}>
					Copy <span class="menu-shortcut">Ctrl+C</span>
				</DropdownMenu.Item>
				<DropdownMenu.Item class="menu-item" onSelect={() => { if (app.selectedBlockIds.length > 0) { onClipboardChange(JSON.parse(JSON.stringify(app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id))))); pushUndo(); send('remove_blocks', { ids: [...app.selectedBlockIds] }); app.selectedBlockIds = []; } }} disabled={app.selectedBlockIds.length === 0}>
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
				<DropdownMenu.Item class="menu-item" onSelect={() => { app.showMinimap = !app.showMinimap; }}>
					{app.showMinimap ? 'Hide' : 'Show'} Minimap
				</DropdownMenu.Item>
				<DropdownMenu.Separator class="menu-sep" />
				<DropdownMenu.Item class="menu-item" onSelect={() => { app.pipelineSearchFocused = true; }}>
					Find Block <span class="menu-shortcut">Ctrl+F</span>
				</DropdownMenu.Item>
				<DropdownMenu.Item class="menu-item" onSelect={() => collapseAllBlocks()}>
					Collapse All Blocks
				</DropdownMenu.Item>
				<DropdownMenu.Item class="menu-item" onSelect={() => expandAllBlocks()}>
					Expand All Blocks
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
				<DropdownMenu.Item class="menu-item" onSelect={() => { app.showChangelog = true; }}>
					Changelog
				</DropdownMenu.Item>
				<DropdownMenu.Item class="menu-item" onSelect={() => { app.showPluginBuilder = true; }}>
					Plugin Builder
				</DropdownMenu.Item>
				<DropdownMenu.Separator class="menu-sep" />
				<DropdownMenu.Item class="menu-item" onSelect={() => { app.showFingerprint = true; }}>
					Site Fingerprint
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Portal>
	</DropdownMenu.Root>

	<!-- Run menu -->
	<DropdownMenu.Root>
		<DropdownMenu.Trigger class="toolbar-trigger">Run</DropdownMenu.Trigger>
		<DropdownMenu.Portal>
			<DropdownMenu.Content class="menu-content" sideOffset={2} align="start">
				<DropdownMenu.Item class="menu-item" onSelect={() => send('debug_pipeline', { pipeline: JSON.parse(JSON.stringify(app.pipeline)) })}>
					Debug Run <span class="menu-shortcut">F5</span>
				</DropdownMenu.Item>
				<DropdownMenu.Separator class="menu-sep" />
				<DropdownMenu.Item class="menu-item" onSelect={() => { send('start_runner', { threads: app.pipeline.runner_settings.threads, wordlist_path: app.wordlistPath, proxy_path: app.proxyPath, pipeline: JSON.parse(JSON.stringify(app.pipeline)) }); app.isRunning = true; }}>
					Start Runner
				</DropdownMenu.Item>
				<DropdownMenu.Item class="menu-item" onSelect={() => { send('stop_runner'); app.isRunning = false; app.isPaused = false; }} disabled={!app.isRunning}>
					Stop Runner
				</DropdownMenu.Item>
			</DropdownMenu.Content>
		</DropdownMenu.Portal>
	</DropdownMenu.Root>

	<!-- Hits menu -->
	<DropdownMenu.Root>
		<DropdownMenu.Trigger class="toolbar-trigger flex items-center gap-1 {app.hits.length > 0 ? 'text-primary' : ''}">
			Hits{#if app.hits.length > 0}<span class="text-[9px] bg-primary/15 px-1 rounded">{app.hits.length}</span>{/if}
		</DropdownMenu.Trigger>
		<DropdownMenu.Portal>
			<DropdownMenu.Content class="menu-content" sideOffset={2} align="start">
				<DropdownMenu.Item class="menu-item" onSelect={() => { app.showHitsDialog = true; }}>
					View Hits Database
				</DropdownMenu.Item>
				<DropdownMenu.Separator class="menu-sep" />
				<DropdownMenu.Item class="menu-item" onSelect={() => { app.bottomTab = 'data'; }}>
					Show Data Panel
				</DropdownMenu.Item>
				<DropdownMenu.Separator class="menu-sep" />
				<DropdownMenu.Item class="menu-item text-muted-foreground" onSelect={() => { app.hits = []; }} disabled={app.hits.length === 0}>
					Clear All Hits
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
				onclick={() => { send('start_runner', { threads: app.pipeline.runner_settings.threads, wordlist_path: app.wordlistPath, proxy_path: app.proxyPath, pipeline: JSON.parse(JSON.stringify(app.pipeline)) }); app.isRunning = true; }}
				title="Start Runner"
			>
				<Play size={12} />
			</button>
		{/if}
		<button
			class="p-0.5 rounded transition-colors text-muted-foreground hover:text-foreground hover:bg-secondary"
			onclick={() => send('debug_pipeline', { pipeline: JSON.parse(JSON.stringify(app.pipeline)) })}
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
