<script lang="ts">
	/**
	 * Global intellisense popup — mounted ONCE at the app root.
	 * Reads from intelliPopup store. Always position:fixed directly under
	 * the anchor input regardless of any scroll/overflow/transform ancestors.
	 */
	import { intelliPopup } from '$lib/intellisense-popup.svelte';
	import { onDestroy } from 'svelte';

	let listEl = $state<HTMLUListElement | null>(null);

	// Live RAF positioning loop — runs only while popup is visible
	let posLeft   = $state(0);
	let posTop    = $state(0);
	let popWidth  = $state(240);
	let rafId     = 0;

	$effect(() => {
		if (!intelliPopup.visible) {
			cancelAnimationFrame(rafId);
			return;
		}
		function tick() {
			const rect = intelliPopup.anchorRect;
			if (rect) {
				posLeft  = rect.left;
				posTop   = rect.bottom + 4;
				popWidth = Math.max(rect.width, 240);
			}
			rafId = requestAnimationFrame(tick);
		}
		tick();
		return () => cancelAnimationFrame(rafId);
	});

	// Scroll selected item into view
	$effect(() => {
		const idx = intelliPopup.selectedIndex;
		if (!listEl) return;
		const item = listEl.children[idx] as HTMLElement | undefined;
		item?.scrollIntoView({ block: 'nearest' });
	});

	onDestroy(() => cancelAnimationFrame(rafId));

	function kindColor(kind: string): string {
		const map: Record<string, string> = {
			variable: 'text-sky-400',    input:    'text-violet-400',
			data:     'text-amber-400',  function: 'text-emerald-400',
			keyword:  'text-rose-400',   snippet:  'text-cyan-400',
			ldelim:   'text-orange-400', rdelim:   'text-orange-400',
		};
		return map[kind] ?? 'text-muted-foreground';
	}

	function kindGlyph(kind: string): string {
		const map: Record<string, string> = {
			variable: 'x', input: 'i', data: 'd', function: 'ƒ',
			keyword: 'k', snippet: '~', ldelim: 'L', rdelim: 'R',
		};
		return map[kind] ?? '?';
	}
</script>

{#if intelliPopup.visible && intelliPopup.suggestions.length > 0}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		role="listbox"
		style="position: fixed; left: {posLeft}px; top: {posTop}px; width: {popWidth}px; max-width: 440px;"
		class="z-[99999] rounded-md border border-border bg-popover shadow-2xl overflow-hidden flex flex-col intelli-popup"
		onmousedown={(e) => e.preventDefault()}
	>
		<ul bind:this={listEl} class="overflow-y-auto max-h-[220px] py-0.5 scroll-smooth" style="overscroll-behavior: contain;">
			{#each intelliPopup.suggestions as item, i}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<li
					role="option"
					aria-selected={i === intelliPopup.selectedIndex}
					class="flex items-center gap-2 px-2 py-[3px] cursor-pointer select-none text-[11px]
						{i === intelliPopup.selectedIndex
							? 'bg-accent/70 text-foreground'
							: 'text-foreground/75 hover:bg-accent/25'}"
					onclick={() => intelliPopup.pick(item)}
					onmouseenter={() => intelliPopup.setHover(i)}
				>
					<!-- Kind badge -->
					<span class="shrink-0 w-[18px] h-[18px] flex items-center justify-center rounded text-[8px] font-bold font-mono
						{kindColor(item.kind)} bg-current/10 leading-none">
						{kindGlyph(item.kind)}
					</span>

					<!-- Label + detail -->
					<span class="flex-1 min-w-0 font-mono text-[11px] truncate {kindColor(item.kind)}">{item.label}</span>
					{#if item.detail}
						<span class="shrink-0 text-[9px] text-muted-foreground/50 font-sans pr-1">{item.detail}</span>
					{/if}
				</li>
			{/each}
		</ul>

		<!-- Footer hint bar -->
		<div class="flex items-center gap-3 px-2 py-[2px] border-t border-border/60 bg-background/60
		            text-[9px] text-muted-foreground/40 select-none shrink-0 font-sans">
			<span>↑↓</span>
			<span>Tab / Enter — insert</span>
			<span>Esc — dismiss</span>
		</div>
	</div>
{/if}

<style>
	.intelli-popup {
		animation: intelli-in 110ms cubic-bezier(0.16, 1, 0.3, 1) both;
		transform-origin: top center;
	}

	@keyframes intelli-in {
		from {
			opacity: 0;
			transform: translateY(-6px) scaleY(0.94);
		}
		to {
			opacity: 1;
			transform: translateY(0) scaleY(1);
		}
	}
</style>
