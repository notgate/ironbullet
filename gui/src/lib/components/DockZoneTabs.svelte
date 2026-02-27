<!--
  DockZoneTabs — renders a group of panels as draggable tabs.
  Tabs can be dragged to another zone or floated.
  This component is used for bottom, right, and left dock zones.
-->
<script lang="ts">
	import type { PanelId, DockZone } from '$lib/state/dock.svelte';
	import { dock, PANEL_LABELS } from '$lib/state/dock.svelte';
	import { send } from '$lib/ipc';
	import ExternalLink from '@lucide/svelte/icons/external-link';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import MoveRight from '@lucide/svelte/icons/move-right';
	import MoveLeft from '@lucide/svelte/icons/move-left';
	import ArrowDown from '@lucide/svelte/icons/arrow-down';

	let {
		zone,
		activeTab = $bindable(),
		collapsed = $bindable(false),
		onToggleCollapse,
		children,
		showDockToRight = false,
		showDockToLeft = false,
	}: {
		zone: DockZone;
		activeTab: string;
		collapsed?: boolean;
		onToggleCollapse?: () => void;
		children: import('svelte').Snippet<[PanelId]>;
		showDockToRight?: boolean;
		showDockToLeft?: boolean;
	} = $props();

	let panels = $derived(dock.panelsIn(zone));
	let dragOverId = $state<PanelId | null>(null);
	let dragOverZone = $state<DockZone | null>(null);
	/** Track last reorder target to prevent oscillating swaps on continuous dragover */
	let lastReorderTarget = $state<PanelId | null>(null);

	function onDragStart(e: DragEvent, id: PanelId) {
		dock.dragging = id;
		e.dataTransfer?.setData('text/plain', id);
	}

	function onDragEnd() {
		dock.dragging = null;
		dragOverId = null;
		dragOverZone = null;
		lastReorderTarget = null;
	}

	function onDragOverTab(e: DragEvent, id: PanelId) {
		e.preventDefault();
		dragOverId = id;
		// Only reorder when the target actually changes — prevents oscillating swaps
		if (dock.dragging && dock.dragging !== id && lastReorderTarget !== id) {
			lastReorderTarget = id;
			dock.reorderPanel(dock.dragging, id);
		}
	}

	function onDragOverZone(e: DragEvent, z: DockZone) {
		e.preventDefault();
		dragOverZone = z;
		dock.dragOver = z;
	}

	function onDropZone(_e: DragEvent, z: DockZone) {
		if (dock.dragging) {
			dock.movePanel(dock.dragging, z);
			dock.dragging = null;
		}
		dragOverZone = null;
		dock.dragOver = null;
	}

	function onDragLeaveZone() {
		dragOverZone = null;
		dock.dragOver = null;
	}

	/** Float the active panel as a native OS window */
	function floatActivePanel() {
		if (!activeTab) return;
		send('float_panel_native', { id: activeTab });
	}
</script>

<div
	class="flex flex-col h-full"
	ondragover={(e) => onDragOverZone(e, zone)}
	ondrop={(e) => onDropZone(e, zone)}
	ondragleave={onDragLeaveZone}
	role="tabpanel"
>
	<!-- Tab bar -->
	<div
		class="bg-surface shrink-0 px-1 flex items-center border-b border-border-dark gap-0"
		class:ring-1={dragOverZone === zone}
		class:ring-primary={dragOverZone === zone}
	>
		<!-- Draggable tabs — labels only, no inline action buttons -->
		<div class="flex gap-0 flex-1 overflow-x-auto no-scrollbar">
			{#each panels as panel (panel.id)}
				<div
					draggable={true}
					ondragstart={(e) => onDragStart(e, panel.id)}
					ondragend={onDragEnd}
					ondragover={(e) => onDragOverTab(e, panel.id)}
					role="tab"
					aria-selected={activeTab === panel.id}
					tabindex="0"
					class="tab-trigger flex items-center gap-1 relative select-none cursor-grab"
					class:active={activeTab === panel.id}
					class:ring-1={dragOverId === panel.id && dock.dragging !== panel.id}
					class:ring-primary={dragOverId === panel.id && dock.dragging !== panel.id}
					onclick={() => { if (collapsed) collapsed = false; activeTab = panel.id; }}
					onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { activeTab = panel.id; e.preventDefault(); } }}
				>
					{PANEL_LABELS[panel.id]}
				</div>
			{/each}
		</div>

		<!-- Panel actions for the ACTIVE tab — positioned at the right end of the tab bar -->
		<!-- Separated from tabs so users don't misclick action buttons when selecting tabs -->
		{#if activeTab && panels.find(p => p.id === activeTab)}
			<div class="flex items-center gap-0.5 shrink-0 border-l border-border/40 ml-1 pl-1">
				<!-- Float as native OS window -->
				<button
					class="p-0.5 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
					onclick={floatActivePanel}
					title="Open as native window"
				>
					<ExternalLink size={10} />
				</button>

				<!-- Dock to right (from bottom or left zone) -->
				{#if showDockToRight && (zone === 'bottom' || zone === 'left')}
					<button
						class="p-0.5 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
						onclick={() => dock.movePanel(activeTab as PanelId, 'right')}
						title="Dock to right panel"
					>
						<MoveRight size={10} />
					</button>
				{/if}

				<!-- Dock to left (from bottom zone) -->
				{#if showDockToLeft && zone === 'bottom'}
					<button
						class="p-0.5 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
						onclick={() => dock.movePanel(activeTab as PanelId, 'left')}
						title="Dock to left panel"
					>
						<MoveLeft size={10} />
					</button>
				{/if}

				<!-- Dock to bottom (from right or left zone) -->
				{#if zone === 'right' || zone === 'left'}
					<button
						class="p-0.5 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
						onclick={() => dock.movePanel(activeTab as PanelId, 'bottom')}
						title="Dock to bottom"
					>
						<ArrowDown size={10} />
					</button>
				{/if}
			</div>
		{/if}

		<!-- Collapse toggle (for bottom panel) -->
		{#if onToggleCollapse}
			<button
				class="p-0.5 ml-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors shrink-0"
				onclick={onToggleCollapse}
				title={collapsed ? 'Expand panel' : 'Collapse panel'}
			>
				{#if collapsed}<ChevronUp size={12} />{:else}<ChevronDown size={12} />{/if}
			</button>
		{/if}
	</div>

	<!-- Panel content -->
	{#if !collapsed}
		{#each panels as panel (panel.id)}
			<div class="flex-1 overflow-hidden min-h-0" class:hidden={activeTab !== panel.id}>
				{@render children(panel.id)}
			</div>
		{/each}
	{/if}
</div>

<style>
	.no-scrollbar::-webkit-scrollbar { display: none; }
	.no-scrollbar { scrollbar-width: none; }
</style>
