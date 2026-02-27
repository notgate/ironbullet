<!--
  DockZoneTabs — renders a group of panels as draggable tabs.
  Tabs can be dragged to another zone or floated.
  This component is used for both bottom and right dock zones.
-->
<script lang="ts">
	import type { PanelId, DockZone } from '$lib/state/dock.svelte';
	import { dock, PANEL_LABELS } from '$lib/state/dock.svelte';
	import ExternalLink from '@lucide/svelte/icons/external-link';
	import { Tabs } from 'bits-ui';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import MoveRight from '@lucide/svelte/icons/move-right';

	let {
		zone,
		activeTab = $bindable(),
		collapsed = $bindable(false),
		onToggleCollapse,
		children,
		showDockToRight = false,
	}: {
		zone: DockZone;
		activeTab: string;
		collapsed?: boolean;
		onToggleCollapse?: () => void;
		children: import('svelte').Snippet<[PanelId]>;
		showDockToRight?: boolean;
	} = $props();

	let panels = $derived(dock.panelsIn(zone));
	let dragOverId = $state<PanelId | null>(null);
	let dragOverZone = $state<DockZone | null>(null);

	function onDragStart(e: DragEvent, id: PanelId) {
		dock.dragging = id;
		e.dataTransfer?.setData('text/plain', id);
	}

	function onDragEnd() {
		dock.dragging = null;
		dragOverId = null;
		dragOverZone = null;
	}

	function onDragOverTab(e: DragEvent, id: PanelId) {
		e.preventDefault();
		dragOverId = id;
		if (dock.dragging && dock.dragging !== id) {
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
		<!-- Draggable tabs -->
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
					class="tab-trigger flex items-center gap-1 group relative select-none cursor-grab"
					class:active={activeTab === panel.id}
					class:ring-1={dragOverId === panel.id && dock.dragging !== panel.id}
					class:ring-primary={dragOverId === panel.id && dock.dragging !== panel.id}
					onclick={() => { if (collapsed) collapsed = false; activeTab = panel.id; }}
					onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { activeTab = panel.id; e.preventDefault(); } }}
				>
					{PANEL_LABELS[panel.id]}
					<!-- Float button (shows on hover) -->
					<button
						class="opacity-0 group-hover:opacity-70 hover:!opacity-100 p-0.5 rounded hover:bg-accent transition-opacity"
						onclick={(e) => { e.stopPropagation(); dock.movePanel(panel.id, 'float'); }}
						title="Float panel"
						draggable={false}
					>
						<ExternalLink size={9} />
					</button>
					{#if showDockToRight && zone === 'bottom'}
						<button
							class="opacity-0 group-hover:opacity-70 hover:!opacity-100 p-0.5 rounded hover:bg-accent transition-opacity"
							onclick={(e) => { e.stopPropagation(); dock.movePanel(panel.id, 'right'); }}
							title="Dock to right panel"
							draggable={false}
						>
							<MoveRight size={9} />
						</button>
					{/if}
					{#if zone === 'right'}
						<button
							class="opacity-0 group-hover:opacity-70 hover:!opacity-100 p-0.5 rounded hover:bg-accent transition-opacity"
							onclick={(e) => { e.stopPropagation(); dock.movePanel(panel.id, 'bottom'); }}
							title="Dock to bottom"
							draggable={false}
						>⬇</button>
					{/if}
				</div>
			{/each}
		</div>

		<!-- Collapse toggle (for bottom panel) -->
		{#if onToggleCollapse}
			<button
				class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors shrink-0"
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
