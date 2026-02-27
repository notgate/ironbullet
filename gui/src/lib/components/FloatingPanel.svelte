<script lang="ts">
	import type { PanelId } from '$lib/state/dock.svelte';
	import { dock, PANEL_LABELS } from '$lib/state/dock.svelte';
	import X from '@lucide/svelte/icons/x';
	import Minus from '@lucide/svelte/icons/minus';
	import ArrowDownToLine from '@lucide/svelte/icons/arrow-down-to-line';
	import GripHorizontal from '@lucide/svelte/icons/grip-horizontal';

	let { id, children }: { id: PanelId; children: import('svelte').Snippet } = $props();

	let cfg = $derived(dock.panels.find(p => p.id === id));
	let float = $derived(cfg?.float ?? { x: 200, y: 100, width: 520, height: 380, minimized: false });

	let draggingWindow = false;
	let dragStartX = 0, dragStartY = 0, origX = 0, origY = 0;

	let resizing = false;
	let resizeStartX = 0, resizeStartY = 0, origW = 0, origH = 0;

	function onHeaderMouseDown(e: MouseEvent) {
		if ((e.target as HTMLElement).closest('button')) return;
		draggingWindow = true;
		dragStartX = e.clientX;
		dragStartY = e.clientY;
		origX = float.x;
		origY = float.y;
		window.addEventListener('mousemove', onWindowMouseMove);
		window.addEventListener('mouseup', onWindowMouseUp, { once: true });
	}

	function onWindowMouseMove(e: MouseEvent) {
		if (!draggingWindow) return;
		const dx = e.clientX - dragStartX;
		const dy = e.clientY - dragStartY;
		dock.setFloatPosition(id, Math.max(0, origX + dx), Math.max(28, origY + dy));
	}

	function onWindowMouseUp() {
		draggingWindow = false;
		window.removeEventListener('mousemove', onWindowMouseMove);
	}

	function onResizeMouseDown(e: MouseEvent) {
		e.stopPropagation();
		resizing = true;
		resizeStartX = e.clientX;
		resizeStartY = e.clientY;
		origW = float.width;
		origH = float.height;
		window.addEventListener('mousemove', onResizeMouseMove);
		window.addEventListener('mouseup', onResizeMouseUp, { once: true });
	}

	function onResizeMouseMove(e: MouseEvent) {
		if (!resizing) return;
		const dw = e.clientX - resizeStartX;
		const dh = e.clientY - resizeStartY;
		dock.setFloatSize(id, Math.max(300, origW + dw), Math.max(200, origH + dh));
	}

	function onResizeMouseUp() {
		resizing = false;
		window.removeEventListener('mousemove', onResizeMouseMove);
	}
</script>

{#if cfg?.zone === 'float'}
<div
	class="fixed z-50 bg-surface border border-border rounded-lg shadow-2xl flex flex-col overflow-hidden"
	style="left: {float.x}px; top: {float.y}px; width: {float.width}px; height: {float.minimized ? 'auto' : `${float.height}px`};"
>
	<!-- Title bar -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="flex items-center gap-1.5 px-2 py-1 bg-surface-raised border-b border-border shrink-0 select-none cursor-move"
		onmousedown={onHeaderMouseDown}
	>
		<GripHorizontal size={12} class="text-muted-foreground/50 shrink-0" />
		<span class="text-[11px] font-medium flex-1 truncate">{PANEL_LABELS[id]}</span>
		<button
			class="p-0.5 rounded hover:bg-accent text-muted-foreground hover:text-foreground"
			onclick={() => dock.movePanel(id, 'bottom')}
			title="Dock to bottom"
		><ArrowDownToLine size={11} /></button>
		<button
			class="p-0.5 rounded hover:bg-accent text-muted-foreground hover:text-foreground"
			onclick={() => dock.toggleMinimize(id)}
			title={float.minimized ? 'Restore' : 'Minimize'}
		><Minus size={11} /></button>
		<button
			class="p-0.5 rounded hover:bg-red-500/20 text-muted-foreground hover:text-red-400"
			onclick={() => dock.movePanel(id, 'bottom')}
			title="Close (return to bottom)"
		><X size={11} /></button>
	</div>

	<!-- Content -->
	{#if !float.minimized}
		<div class="flex-1 overflow-hidden min-h-0">
			{@render children()}
		</div>

		<!-- Resize handle -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="absolute bottom-0 right-0 w-4 h-4 cursor-nwse-resize opacity-40 hover:opacity-100 transition-opacity"
			onmousedown={onResizeMouseDown}
			style="background: radial-gradient(circle at 100% 100%, var(--muted-foreground) 30%, transparent 30%);"
		></div>
	{/if}
</div>
{/if}
