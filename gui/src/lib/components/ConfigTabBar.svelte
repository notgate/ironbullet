<script lang="ts">
	import { app, switchTab, createNewTab, requestCloseTab, reorderTabs } from '$lib/state.svelte';
	import Plus from '@lucide/svelte/icons/plus';
	import X from '@lucide/svelte/icons/x';
	import ChevronLeft from '@lucide/svelte/icons/chevron-left';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';

	let dragOverIdx = $state<number | null>(null);
	let dragFromIdx = $state<number | null>(null);
	let newTabId = $state<string | null>(null);
	let closingTabId = $state<string | null>(null);
	let scrollContainer: HTMLDivElement | undefined = $state();
	let canScrollLeft = $state(false);
	let canScrollRight = $state(false);

	function updateScrollState() {
		if (!scrollContainer) return;
		canScrollLeft = scrollContainer.scrollLeft > 0;
		canScrollRight = scrollContainer.scrollLeft + scrollContainer.clientWidth < scrollContainer.scrollWidth - 1;
	}

	function scrollToEnd() {
		if (!scrollContainer) return;
		requestAnimationFrame(() => {
			scrollContainer!.scrollTo({ left: scrollContainer!.scrollWidth, behavior: 'smooth' });
			updateScrollState();
		});
	}

	function scrollToActiveTab() {
		if (!scrollContainer) return;
		requestAnimationFrame(() => {
			const activeEl = scrollContainer!.querySelector('.config-tab.active') as HTMLElement | null;
			if (activeEl) {
				activeEl.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: 'smooth' });
			}
			updateScrollState();
		});
	}

	function handleWheel(e: WheelEvent) {
		if (!scrollContainer) return;
		e.preventDefault();
		scrollContainer.scrollLeft += e.deltaY !== 0 ? e.deltaY : e.deltaX;
		updateScrollState();
	}

	function scrollLeft() {
		if (!scrollContainer) return;
		scrollContainer.scrollBy({ left: -150, behavior: 'smooth' });
		setTimeout(updateScrollState, 200);
	}

	function scrollRight() {
		if (!scrollContainer) return;
		scrollContainer.scrollBy({ left: 150, behavior: 'smooth' });
		setTimeout(updateScrollState, 200);
	}

	function handleNewTab() {
		createNewTab();
		newTabId = app.activeTabId;
		setTimeout(() => { newTabId = null; }, 250);
		scrollToEnd();
	}

	function handleCloseTab(e: MouseEvent, tabId: string) {
		e.stopPropagation();
		closingTabId = tabId;
		setTimeout(() => {
			requestCloseTab(tabId);
			closingTabId = null;
			updateScrollState();
		}, 150);
	}

	function handleTabClick(tabId: string) {
		switchTab(tabId);
		requestAnimationFrame(() => scrollToActiveTab());
	}

	function handleDragStart(e: DragEvent, idx: number) {
		dragFromIdx = idx;
		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'move';
			e.dataTransfer.setData('text/plain', String(idx));
		}
	}

	function handleDragOver(e: DragEvent, idx: number) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		dragOverIdx = idx;
	}

	function handleDragLeave() {
		dragOverIdx = null;
	}

	function handleDrop(e: DragEvent, toIdx: number) {
		e.preventDefault();
		if (dragFromIdx !== null && dragFromIdx !== toIdx) {
			reorderTabs(dragFromIdx, toIdx);
		}
		dragFromIdx = null;
		dragOverIdx = null;
	}

	function handleDragEnd() {
		dragFromIdx = null;
		dragOverIdx = null;
	}

	$effect(() => {
		// Re-check scroll state when tabs change
		app.configTabs.length;
		requestAnimationFrame(updateScrollState);
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="config-tab-bar">
	{#if canScrollLeft}
		<button class="scroll-btn scroll-btn-left" onclick={scrollLeft}>
			<ChevronLeft size={10} />
		</button>
	{/if}
	<div
		class="config-tabs-scroll"
		bind:this={scrollContainer}
		onwheel={handleWheel}
		onscroll={updateScrollState}
	>
		{#each app.configTabs as tab, i (tab.id)}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="config-tab"
				class:active={tab.id === app.activeTabId}
				class:tab-enter={tab.id === newTabId}
				class:tab-exit={tab.id === closingTabId}
				class:drag-over={dragOverIdx === i && dragFromIdx !== i}
				class:dragging={dragFromIdx === i}
				draggable="true"
				onclick={() => handleTabClick(tab.id)}
				ondragstart={(e) => handleDragStart(e, i)}
				ondragover={(e) => handleDragOver(e, i)}
				ondragleave={handleDragLeave}
				ondrop={(e) => handleDrop(e, i)}
				ondragend={handleDragEnd}
				title={tab.filePath || tab.name}
			>
				<span class="config-tab-name">
					{#if tab.isDirty || (!tab.filePath && tab.id === app.activeTabId && JSON.stringify(app.pipeline) !== tab.savedSnapshot)}
						<span class="dirty-dot"></span>
					{/if}
					{tab.id === app.activeTabId ? (tab.filePath ? tab.name : app.pipeline.name) : tab.name}
				</span>
				{#if app.configTabs.length > 1}
					<button
						class="config-tab-close"
						onclick={(e) => handleCloseTab(e, tab.id)}
						title="Close tab"
					>
						<X size={9} />
					</button>
				{/if}
			</div>
		{/each}
		<button class="config-tab-new" onclick={handleNewTab} title="New Config (Ctrl+T)">
			<Plus size={11} />
		</button>
	</div>
	{#if canScrollRight}
		<button class="scroll-btn scroll-btn-right" onclick={scrollRight}>
			<ChevronRight size={10} />
		</button>
	{/if}
</div>

<style>
	.config-tab-bar {
		display: flex;
		align-items: stretch;
		background: var(--background);
		border-bottom: 1px solid var(--border-dark);
		height: 28px;
		min-height: 28px;
		padding-left: 2px;
		user-select: none;
		overflow: hidden;
	}

	.config-tabs-scroll {
		display: flex;
		align-items: stretch;
		overflow-x: auto;
		overflow-y: hidden;
		flex: 1;
		min-width: 0;
		scrollbar-width: none;
	}
	.config-tabs-scroll::-webkit-scrollbar {
		display: none;
	}

	.config-tab {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 0 8px;
		min-width: 80px;
		max-width: 180px;
		font-size: calc(10px * var(--font-scale, 1));
		color: var(--muted-foreground);
		cursor: pointer;
		position: relative;
		border-right: 1px solid var(--border-dark);
		transition: background 0.12s, color 0.12s, max-width 0.2s ease, min-width 0.2s ease, padding 0.2s ease, opacity 0.15s ease;
		flex-shrink: 0;
	}

	.config-tab:hover {
		color: var(--foreground);
		background: rgba(255, 255, 255, 0.03);
	}

	.config-tab.active {
		color: var(--foreground);
		background: var(--surface);
		border-bottom: 1px solid var(--surface);
		margin-bottom: -1px;
	}
	.config-tab.active::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 1px;
		background: var(--primary);
	}

	.config-tab.drag-over {
		border-left: 2px solid var(--primary);
	}

	.config-tab.dragging {
		opacity: 0.4;
	}

	/* New tab slide-in animation */
	.config-tab.tab-enter {
		animation: tabSlideIn 0.25s cubic-bezier(0.16, 1, 0.3, 1) forwards;
	}

	/* Close tab shrink animation */
	.config-tab.tab-exit {
		animation: tabShrinkOut 0.15s ease forwards;
		pointer-events: none;
	}

	@keyframes tabSlideIn {
		0% {
			max-width: 0;
			min-width: 0;
			padding: 0 0;
			opacity: 0;
		}
		100% {
			max-width: 180px;
			min-width: 80px;
			padding: 0 8px;
			opacity: 1;
		}
	}

	@keyframes tabShrinkOut {
		0% {
			max-width: 180px;
			min-width: 80px;
			padding: 0 8px;
			opacity: 1;
		}
		100% {
			max-width: 0;
			min-width: 0;
			padding: 0 0;
			opacity: 0;
			border-right-width: 0;
		}
	}

	.config-tab-name {
		display: flex;
		align-items: center;
		gap: 4px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
		min-width: 0;
	}

	.dirty-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--primary);
		flex-shrink: 0;
	}

	.config-tab-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 14px;
		height: 14px;
		border-radius: 2px;
		border: none;
		background: transparent;
		color: var(--muted-foreground);
		cursor: pointer;
		padding: 0;
		flex-shrink: 0;
		opacity: 0;
		transition: opacity 0.1s, background 0.1s, color 0.1s;
	}
	.config-tab:hover .config-tab-close,
	.config-tab.active .config-tab-close {
		opacity: 1;
	}
	.config-tab-close:hover {
		background: rgba(255, 255, 255, 0.1);
		color: var(--foreground);
	}

	.config-tab-new {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		flex-shrink: 0;
		border: none;
		background: transparent;
		color: var(--muted-foreground);
		cursor: pointer;
		transition: color 0.1s, background 0.1s;
		padding: 0;
	}
	.config-tab-new:hover {
		color: var(--foreground);
		background: rgba(255, 255, 255, 0.04);
	}

	.scroll-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		flex-shrink: 0;
		border: none;
		background: var(--background);
		color: var(--muted-foreground);
		cursor: pointer;
		padding: 0;
		z-index: 1;
		border-right: 1px solid var(--border-dark);
		transition: color 0.1s, background 0.1s;
	}
	.scroll-btn-right {
		border-right: none;
		border-left: 1px solid var(--border-dark);
	}
	.scroll-btn:hover {
		color: var(--foreground);
		background: rgba(255, 255, 255, 0.06);
	}
</style>
