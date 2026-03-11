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
	let posStyle      = $state('');

	$effect(() => { if (suggestions) selectedIndex = 0; });

	$effect(() => {
		if (!listEl) return;
		const item = listEl.children[selectedIndex] as HTMLElement | undefined;
		item?.scrollIntoView({ block: 'nearest' });
	});

	$effect(() => {
		if (!anchorEl || !visible) return;
		const rect = anchorEl.getBoundingClientRect();
		posStyle = `position:fixed;left:${rect.left}px;top:${rect.bottom + 2}px;min-width:${Math.max(rect.width, 220)}px;max-width:400px;`;
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
		style={posStyle}
		class="z-[9999] rounded border border-border bg-popover shadow-xl overflow-hidden flex flex-col"
		onmousedown={(e) => e.preventDefault()}
	>
		<ul bind:this={listEl} class="overflow-y-auto max-h-[220px] py-0.5">
			{#each suggestions as item, i}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<li
					role="option"
					aria-selected={i === selectedIndex}
					class="flex items-center gap-2 px-2 py-1 cursor-pointer select-none text-[11px] transition-colors
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
			<span>Tab / Enter to insert</span>
			<span>Esc to dismiss</span>
		</div>
	</div>
{/if}
