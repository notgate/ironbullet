<script lang="ts">
	/**
	 * Intellisense popup — Monaco-style suggestion list anchored to an input.
	 *
	 * Props:
	 *   suggestions  - filtered suggestion list to display
	 *   anchorEl     - the input element to position below
	 *   visible      - whether to show the popup
	 *   onpick       - called when user selects a suggestion
	 *   onclose      - called when popup should close (Escape / blur)
	 */

	export interface SuggestionItem {
		label: string;
		insertText: string;
		kind: SuggestionKind;
		detail?: string;
		documentation?: string;
	}

	export type SuggestionKind =
		| 'variable' | 'input' | 'data' | 'function'
		| 'keyword'  | 'snippet' | 'ldelim' | 'rdelim';

	let {
		suggestions = [] as SuggestionItem[],
		anchorEl    = null as HTMLElement | null,
		visible     = false,
		onpick      = (_item: SuggestionItem) => {},
		onclose     = () => {},
	}: {
		suggestions?: SuggestionItem[];
		anchorEl?:    HTMLElement | null;
		visible?:     boolean;
		onpick?:      (item: SuggestionItem) => void;
		onclose?:     () => void;
	} = $props();

	let selectedIndex = $state(0);
	let listEl        = $state<HTMLElement | null>(null);

	// Live position — updated every RAF frame while visible so it never drifts
	// even inside scrolling containers or when the window resizes.
	let posLeft  = $state(0);
	let posTop   = $state(0);
	let posWidth = $state(220);

	let rafId = 0;

	function updatePos() {
		if (!anchorEl) return;
		const rect = anchorEl.getBoundingClientRect();
		posLeft  = rect.left;
		posTop   = rect.bottom + 4;
		posWidth = Math.max(rect.width, 240);
	}

	$effect(() => {
		if (!visible || !anchorEl) {
			cancelAnimationFrame(rafId);
			return;
		}
		function tick() {
			updatePos();
			rafId = requestAnimationFrame(tick);
		}
		tick();
		return () => cancelAnimationFrame(rafId);
	});

	$effect(() => { if (suggestions) selectedIndex = 0; });

	$effect(() => {
		if (!listEl) return;
		const item = listEl.children[selectedIndex] as HTMLElement | undefined;
		item?.scrollIntoView({ block: 'nearest' });
	});

	export function moveUp()   { selectedIndex = (selectedIndex - 1 + suggestions.length) % suggestions.length; }
	export function moveDown() { selectedIndex = (selectedIndex + 1) % suggestions.length; }
	export function confirm()  { if (suggestions[selectedIndex]) onpick(suggestions[selectedIndex]); }
	export function close()    { onclose(); }

	function kindColor(kind: SuggestionKind): string {
		const map: Record<SuggestionKind, string> = {
			variable: 'text-sky-400',    input:    'text-violet-400',
			data:     'text-amber-400',  function: 'text-emerald-400',
			keyword:  'text-rose-400',   snippet:  'text-cyan-400',
			ldelim:   'text-orange-400', rdelim:   'text-orange-400',
		};
		return map[kind] ?? 'text-muted-foreground';
	}

	function kindGlyph(kind: SuggestionKind): string {
		const map: Record<SuggestionKind, string> = {
			variable: 'x', input: 'i', data: 'd', function: 'f',
			keyword: 'k', snippet: 's', ldelim: 'L', rdelim: 'R',
		};
		return map[kind] ?? '?';
	}
</script>

{#if visible && suggestions.length > 0}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		role="listbox"
		style="position: fixed; left: {posLeft}px; top: {posTop}px; min-width: {posWidth}px; max-width: 420px;"
		class="z-[9999] rounded border border-border bg-popover shadow-2xl overflow-hidden flex flex-col
		       intellisense-popup"
		onmousedown={(e) => e.preventDefault()}
	>
		<ul bind:this={listEl} class="overflow-y-auto max-h-[240px] py-0.5">
			{#each suggestions as item, i}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<li
					role="option"
					aria-selected={i === selectedIndex}
					class="flex items-center gap-2 px-2 py-1 cursor-pointer select-none text-[11px] transition-colors duration-75
						{i === selectedIndex ? 'bg-accent/60 text-foreground' : 'text-foreground/80 hover:bg-accent/30'}"
					onclick={() => { selectedIndex = i; onpick(item); }}
					onmouseenter={() => { selectedIndex = i; }}
				>
					<span class="shrink-0 w-5 h-5 flex items-center justify-center rounded text-[9px] font-bold font-mono
						{kindColor(item.kind)} bg-current/10">
						{kindGlyph(item.kind)}
					</span>
					<span class="flex-1 min-w-0 font-mono truncate {kindColor(item.kind)}">{item.label}</span>
					{#if item.detail}
						<span class="shrink-0 text-[10px] text-muted-foreground/60 font-sans">{item.detail}</span>
					{/if}
				</li>
			{/each}
		</ul>
		<div class="flex items-center gap-3 px-2 py-0.5 border-t border-border bg-background/80 text-[9px] text-muted-foreground/50 select-none shrink-0">
			<span>↑↓ navigate</span>
			<span>Tab / Enter to insert</span>
			<span>Esc to dismiss</span>
		</div>
	</div>
{/if}

<style>
	.intellisense-popup {
		animation: isense-in 120ms cubic-bezier(0.16, 1, 0.3, 1) both;
		transform-origin: top left;
	}

	@keyframes isense-in {
		from {
			opacity: 0;
			transform: translateY(-4px) scaleY(0.96);
		}
		to {
			opacity: 1;
			transform: translateY(0) scaleY(1);
		}
	}
</style>
