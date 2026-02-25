<script lang="ts">
	import { onMount } from 'svelte';
	import { Tabs } from 'bits-ui';
	import { registerCallbacks } from '$lib/ipc';
	import { app } from '$lib/state.svelte';
	import { createKeydownHandler } from '$lib/keyboard';
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
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';

	import type { Block } from '$lib/types';

	let bottomPanelCollapsed = $state(false);

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
				{#if app.editingBlockId !== null}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="groove-v resize-handle w-[3px] shrink-0" onmousedown={startResizeRight}></div>
				{/if}
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
