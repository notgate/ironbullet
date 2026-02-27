<script lang="ts">
	import { onMount } from 'svelte';
	import { registerCallbacks } from '$lib/ipc';
	import { app } from '$lib/state.svelte';
	import { dock } from '$lib/state/dock.svelte';
	import type { PanelId } from '$lib/state/dock.svelte';
	import { createKeydownHandler } from '$lib/keyboard';
	import DockZoneTabs from '$lib/components/DockZoneTabs.svelte';
	import FloatingPanel from '$lib/components/FloatingPanel.svelte';
	import TitleBar from '$lib/components/TitleBar.svelte';
	import ConfigTabBar from '$lib/components/ConfigTabBar.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import ContextMenu from '$lib/components/ContextMenu.svelte';
	import BlockPalette from '$lib/components/BlockPalette.svelte';
	import PipelineEditor from '$lib/components/PipelineEditor.svelte';
	import BlockSettingsPanel from '$lib/components/BlockSettingsPanel.svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';
	import VariableInspector from '$lib/components/VariableInspector.svelte';
	import DebugPanel from '$lib/components/DebugPanel.svelte';
	import CodeView from '$lib/components/CodeView.svelte';
	import NetworkViewer from '$lib/components/NetworkViewer.svelte';
	import DataPanel from '$lib/components/DataPanel.svelte';
	import ResponseViewer from '$lib/components/ResponseViewer.svelte';
	import JobMonitor from '$lib/components/JobMonitor.svelte';
	import StartupDialog from '$lib/components/StartupDialog.svelte';
	import UnsavedDialog from '$lib/components/UnsavedDialog.svelte';
	import UpdateDialog from '$lib/components/UpdateDialog.svelte';
	import ChangelogDialog from '$lib/components/ChangelogDialog.svelte';
	import BlockDocsPanel from '$lib/components/BlockDocsPanel.svelte';
	import PluginBuilder from '$lib/components/PluginBuilder.svelte';
	import Onboarding from '$lib/components/Onboarding.svelte';
	import SecurityAlertDialog from '$lib/components/SecurityAlertDialog.svelte';
	import FingerprintDialog from '$lib/components/FingerprintDialog.svelte';
	import HitsDialog from '$lib/components/HitsDialog.svelte';
	import Toast from '$lib/components/Toast.svelte';
	import type { Block } from '$lib/types';

	let bottomPanelCollapsed = $state(false);
	let activeBottomTab = $state<string>('debugger');
	let activeRightTab = $state<string>('');

	// Keep activeBottomTab valid when panels move zones
	$effect(() => {
		const bottomPanels = dock.panelsIn('bottom');
		if (!bottomPanels.find(p => p.id === activeBottomTab) && bottomPanels.length > 0) {
			activeBottomTab = bottomPanels[0].id;
		}
	});
	$effect(() => {
		const rightPanels = dock.panelsIn('right');
		if (!rightPanels.find(p => p.id === activeRightTab) && rightPanels.length > 0) {
			activeRightTab = rightPanels[0].id;
		}
	});

	// Block clipboard for Ctrl+C / Ctrl+V
	let clipboardBlocks: Block[] = $state([]);

	// Keyboard handler
	const onKeydown = createKeydownHandler(
		() => clipboardBlocks,
		(blocks) => { clipboardBlocks = blocks; },
	);

	onMount(() => {
		registerCallbacks();
	});

	// Push font settings to CSS custom properties so portals/popups inherit them
	$effect(() => {
		document.documentElement.style.setProperty('--font-family', `'${app.fontFamily}'`);
		document.documentElement.style.setProperty('--font-size', `${app.fontSize}px`);
		document.documentElement.style.setProperty('--font-scale', `${app.fontSize / 13}`);
		document.documentElement.style.setProperty('--app-zoom', `${app.zoom}`);
	});

	// Resize state
	let isResizingLeft = $state(false);
	let isResizingRight = $state(false);
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

	function startResizeRight(e: MouseEvent) {
		isResizingRight = true;
		const startX = e.clientX;
		const startW = app.rightPanelWidth;
		const onMove = (ev: MouseEvent) => { app.rightPanelWidth = Math.max(260, Math.min(700, startW - (ev.clientX - startX))); };
		const onUp = () => { isResizingRight = false; window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); };
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

	// Right dock panel width (for secondary docked panels)
	let rightDockWidth = $state(300);
	function startResizeRightDock(e: MouseEvent) {
		const startX = e.clientX;
		const startW = rightDockWidth;
		const onMove = (ev: MouseEvent) => { rightDockWidth = Math.max(180, Math.min(600, startW - (ev.clientX - startX))); };
		const onUp = () => { window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); };
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}


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
	<Toolbar
		clipboardBlocks={clipboardBlocks}
		onClipboardChange={(blocks) => { clipboardBlocks = blocks; }}
	/>

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

				<!-- Right panel: Block Settings + optional docked panels -->
				<div class="flex shrink-0">
					{#if app.editingBlockId !== null}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="groove-v resize-handle w-[3px] shrink-0" onmousedown={startResizeRight}></div>
						<BlockSettingsPanel />
					{/if}

					<!-- Secondary right dock zone (panels dragged here) -->
					{#if dock.panelsIn('right').length > 0}
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div class="groove-v resize-handle w-[3px] shrink-0" onmousedown={startResizeRightDock}></div>
						<div style="width: {rightDockWidth}px" class="shrink-0 flex flex-col border-l border-border bg-surface">
							<DockZoneTabs
								zone="right"
								bind:activeTab={activeRightTab}
							>
								{#snippet children(id: PanelId)}
									{#if id === 'debugger'}<DebugPanel />
									{:else if id === 'code'}<CodeView />
									{:else if id === 'data'}<DataPanel />
									{:else if id === 'jobs'}<div class="overflow-y-auto h-full"><JobMonitor /></div>
									{:else if id === 'network'}<div class="overflow-y-auto h-full"><NetworkViewer /></div>
									{:else if id === 'variables'}<div class="overflow-y-auto h-full"><VariableInspector /></div>
									{/if}
								{/snippet}
							</DockZoneTabs>
						</div>
					{/if}
				</div>
			</div>

			{#if !bottomPanelCollapsed}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="groove-h resize-handle-h h-[3px] shrink-0" onmousedown={startResizeBottom}></div>
			{/if}

			<!-- Bottom panel — dockable tabs -->
			{#if dock.panelsIn('bottom').length > 0}
				<div style="height: {bottomPanelCollapsed ? 'auto' : `${app.bottomPanelHeight}px`}" class="shrink-0 flex flex-col">
					<DockZoneTabs
						zone="bottom"
						bind:activeTab={activeBottomTab}
						bind:collapsed={bottomPanelCollapsed}
						onToggleCollapse={() => { bottomPanelCollapsed = !bottomPanelCollapsed; }}
						showDockToRight={true}
					>
						{#snippet children(id: PanelId)}
							{#if id === 'debugger'}<DebugPanel />
							{:else if id === 'code'}<CodeView />
							{:else if id === 'data'}<DataPanel />
							{:else if id === 'jobs'}<div class="overflow-y-auto h-full"><JobMonitor /></div>
							{:else if id === 'network'}<div class="overflow-y-auto h-full"><NetworkViewer /></div>
							{:else if id === 'variables'}<div class="overflow-y-auto h-full"><VariableInspector /></div>
							{/if}
						{/snippet}
					</DockZoneTabs>
				</div>
			{:else}
				<!-- Empty drop zone when all panels are moved away -->
				<div
					class="h-8 shrink-0 flex items-center justify-center border-t border-border text-[10px] text-muted-foreground/50 bg-surface"
					ondragover={(e) => { e.preventDefault(); dock.dragOver = 'bottom'; }}
					ondrop={(e) => { e.preventDefault(); if (dock.dragging) dock.movePanel(dock.dragging, 'bottom'); dock.dragging = null; dock.dragOver = null; }}
					role="region"
					aria-label="Bottom panel drop zone"
				>
					Drop panels here
				</div>
			{/if}
		</div>
	</div>

	<!-- Floating panels (rendered at top level, above everything) -->
	{#each dock.panelsIn('float') as panel (panel.id)}
		<FloatingPanel id={panel.id}>
			{#snippet children()}
				{#if panel.id === 'debugger'}<DebugPanel />
				{:else if panel.id === 'code'}<CodeView />
				{:else if panel.id === 'data'}<DataPanel />
				{:else if panel.id === 'jobs'}<div class="overflow-y-auto h-full"><JobMonitor /></div>
				{:else if panel.id === 'network'}<div class="overflow-y-auto h-full"><NetworkViewer /></div>
				{:else if panel.id === 'variables'}<div class="overflow-y-auto h-full"><VariableInspector /></div>
				{/if}
			{/snippet}
		</FloatingPanel>
	{/each}
	</div>
	{/if}
</div>

<SettingsModal />
<ResponseViewer />
<StartupDialog />
<UnsavedDialog />
<UpdateDialog />
<ChangelogDialog />
{#if app.showBlockDocs}
	<BlockDocsPanel />
{/if}
<Onboarding />
<SecurityAlertDialog />
<FingerprintDialog />
<HitsDialog />
<Toast />

<ContextMenu />
