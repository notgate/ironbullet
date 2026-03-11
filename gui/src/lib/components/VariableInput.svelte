<script lang="ts">
	import { app } from '$lib/state.svelte';
	import SkeuSelect from './SkeuSelect.svelte';
	import Intellisense from './Intellisense.svelte';
	import type { SuggestionItem } from './Intellisense.svelte';
	import type { FieldContext } from '$lib/intellisense';
	import { buildSuggestions, getQueryAtCursor, applySuggestion } from '$lib/intellisense';
	import Variable from '@lucide/svelte/icons/variable';
	import Type from '@lucide/svelte/icons/type';
	import Braces from '@lucide/svelte/icons/braces';

	type InputMode = 'raw' | 'embed' | 'variable';

	let {
		value,
		oninput,
		placeholder = '',
		class: className = '',
		context = 'variable' as FieldContext,
		responseBody = undefined as string | undefined,
	}: {
		value: string;
		oninput: (e: Event) => void;
		placeholder?: string;
		class?: string;
		/** Hint to the intellisense engine about what kind of field this is */
		context?: FieldContext;
		/** Response body from the viewer — enables LR context suggestions */
		responseBody?: string;
	} = $props();

	let mode = $state<InputMode>('embed');

	// ── Intellisense state ─────────────────────────────────────────────────
	let inputEl = $state<HTMLInputElement | null>(null);
	let intellisenseEl = $state<InstanceType<typeof Intellisense> | null>(null);
	let isense_visible = $state(false);
	let isense_suggestions = $state<SuggestionItem[]>([]);
	let cursorPos = $state(0);

	function refreshSuggestions() {
		if (!inputEl) return;
		const pos = inputEl.selectionStart ?? 0;
		cursorPos = pos;
		const trigger = getQueryAtCursor(value ?? '', pos);
		if (!trigger) {
			isense_visible = false;
			return;
		}
		const sug = buildSuggestions(context, trigger.query, app.pipeline, responseBody);
		if (sug.length === 0) {
			isense_visible = false;
			return;
		}
		isense_suggestions = sug;
		isense_visible = true;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!isense_visible || !intellisenseEl) return;
		if (e.key === 'ArrowDown') { e.preventDefault(); intellisenseEl.moveDown(); return; }
		if (e.key === 'ArrowUp')   { e.preventDefault(); intellisenseEl.moveUp();   return; }
		if (e.key === 'Enter' || e.key === 'Tab') {
			// Only intercept Tab/Enter when intellisense is open
			e.preventDefault();
			intellisenseEl.confirm();
			return;
		}
		if (e.key === 'Escape') { isense_visible = false; return; }
	}

	function handlePick(item: SuggestionItem) {
		const pos = inputEl?.selectionStart ?? (value?.length ?? 0);
		const { newValue, newCursor } = applySuggestion(value ?? '', pos, item);

		// Fire synthetic oninput with the new value
		const synth = new Event('input', { bubbles: true });
		Object.defineProperty(synth, 'target', { value: { value: newValue }, enumerable: true });
		oninput(synth);

		// Restore cursor after Svelte re-renders
		isense_visible = false;
		requestAnimationFrame(() => {
			if (inputEl) {
				inputEl.setSelectionRange(newCursor, newCursor);
				inputEl.focus();
			}
		});
	}

	function handleBlur() {
		// Small delay so click-to-pick fires before blur closes the popup
		setTimeout(() => { isense_visible = false; }, 120);
	}

	// ── Variable selector (VAR mode) ───────────────────────────────────────
	const availableVars = $derived.by(() => {
		const vars = new Set<string>();
		const extractVars = (blocks: any[]) => {
			for (const block of blocks) {
				const s = block.settings;
				if (s.output_var) vars.add(s.output_var);
				if (s.response_var) vars.add(s.response_var);
				if (s.constants) for (const c of s.constants) if (c.name) vars.add(c.name);
				vars.add('data.SOURCE'); vars.add('data.HEADERS'); vars.add('data.COOKIES');
				vars.add('data.RESPONSECODE'); vars.add('data.ADDRESS');
				vars.add('input.DATA'); vars.add('input.USER'); vars.add('input.PASS');
				if (s.true_blocks) extractVars(s.true_blocks);
				if (s.false_blocks) extractVars(s.false_blocks);
				if (s.blocks) extractVars(s.blocks);
			}
		};
		extractVars(app.pipeline.blocks);
		return Array.from(vars).filter(v => v?.trim()).sort().map(v => ({ value: v, label: v }));
	});

	function cycleMode() {
		if (mode === 'raw') mode = 'embed';
		else if (mode === 'embed') mode = 'variable';
		else mode = 'raw';
		isense_visible = false;
	}

	function getModeIcon() {
		if (mode === 'raw') return Type;
		if (mode === 'embed') return Braces;
		return Variable;
	}
	function getModeTitle() {
		if (mode === 'raw') return 'Raw text (no interpolation)';
		if (mode === 'embed') return 'Embed mode (use <variable> or {{variable}} syntax)';
		return 'Variable selector';
	}
	function getModeLabel() {
		if (mode === 'raw') return 'RAW';
		if (mode === 'embed') return 'EMBED';
		return 'VAR';
	}
</script>

<div class="relative flex items-center gap-1">
	{#if mode === 'variable'}
		<SkeuSelect
			value={value}
			onValueChange={(v) => {
				const event = new Event('input', { bubbles: true });
				Object.defineProperty(event, 'target', { value: { value: v }, enumerable: true });
				oninput(event);
			}}
			options={availableVars}
			placeholder={placeholder || 'Select variable...'}
			class={className}
		/>
	{:else}
		<input
			bind:this={inputEl}
			type="text"
			{value}
			placeholder={mode === 'embed' ? (placeholder || 'Use <variable> or type to search suggestions...') : placeholder}
			class={className}
			oninput={(e) => { oninput(e); refreshSuggestions(); }}
			onkeydown={handleKeydown}
			onblur={handleBlur}
			onclick={refreshSuggestions}
		/>

		<Intellisense
			bind:this={intellisenseEl}
			suggestions={isense_suggestions}
			anchorEl={inputEl}
			visible={isense_visible}
			onpick={handlePick}
			onclose={() => { isense_visible = false; }}
		/>
	{/if}

	<button
		type="button"
		class="shrink-0 px-1.5 py-0.5 rounded text-[9px] font-mono uppercase tracking-wider border transition-colors {
			mode === 'raw' ? 'bg-amber-500/10 border-amber-500/30 text-amber-500 hover:bg-amber-500/20' :
			mode === 'embed' ? 'bg-blue-500/10 border-blue-500/30 text-blue-500 hover:bg-blue-500/20' :
			'bg-emerald-500/10 border-emerald-500/30 text-emerald-500 hover:bg-emerald-500/20'
		}"
		onclick={cycleMode}
		title={getModeTitle()}
	>
		<svelte:component this={getModeIcon()} size={9} class="inline-block mr-0.5" />{getModeLabel()}
	</button>
</div>
