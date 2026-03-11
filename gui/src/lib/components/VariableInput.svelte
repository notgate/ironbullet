<script lang="ts">
	import { app } from '$lib/state.svelte';
	import SkeuSelect from './SkeuSelect.svelte';
	import type { SuggestionItem } from './Intellisense.svelte';
	import type { FieldContext } from '$lib/intellisense';
	import { buildSuggestions, getQueryAtCursor, applySuggestion } from '$lib/intellisense';
	import { intelliPopup } from '$lib/intellisense-popup.svelte';
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
		context?: FieldContext;
		responseBody?: string;
	} = $props();

	let mode = $state<InputMode>('embed');
	let inputEl = $state<HTMLInputElement | null>(null);

	// Whether THIS input owns the currently visible popup
	let isOwner = $state(false);

	function getRect(): DOMRect | null {
		return inputEl?.getBoundingClientRect() ?? null;
	}

	function refreshSuggestions() {
		if (!inputEl || !app.uiPrefs.intellisenseEnabled) {
			if (isOwner) { intelliPopup.hide(); isOwner = false; }
			return;
		}
		const pos = inputEl.selectionStart ?? 0;
		const trigger = getQueryAtCursor(value ?? '', pos);
		if (!trigger) {
			if (isOwner) { intelliPopup.hide(); isOwner = false; }
			return;
		}
		const sug = buildSuggestions(context, trigger.query, app.pipeline, responseBody);
		if (sug.length === 0) {
			if (isOwner) { intelliPopup.hide(); isOwner = false; }
			return;
		}

		const rect = getRect();
		if (!rect) return;

		if (isOwner && intelliPopup.visible) {
			intelliPopup.update(sug, rect);
		} else {
			isOwner = true;
			intelliPopup.show(sug, rect, handlePick, () => { isOwner = false; });
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!isOwner || !intelliPopup.visible) return;
		if (e.key === 'ArrowDown') { e.preventDefault(); intelliPopup.moveDown(); return; }
		if (e.key === 'ArrowUp')   { e.preventDefault(); intelliPopup.moveUp();   return; }
		if (e.key === 'Enter' || e.key === 'Tab') {
			e.preventDefault();
			intelliPopup.pick();
			return;
		}
		if (e.key === 'Escape') { intelliPopup.hide(); isOwner = false; return; }
	}

	function handlePick(item: SuggestionItem) {
		const pos = inputEl?.selectionStart ?? (value?.length ?? 0);
		const { newValue, newCursor } = applySuggestion(value ?? '', pos, item);

		const synth = new Event('input', { bubbles: true });
		Object.defineProperty(synth, 'target', { value: { value: newValue }, enumerable: true });
		oninput(synth);

		isOwner = false;
		requestAnimationFrame(() => {
			if (inputEl) {
				inputEl.setSelectionRange(newCursor, newCursor);
				inputEl.focus();
			}
		});
	}

	function handleBlur() {
		// Small delay so mousedown on popup item fires before blur closes it
		setTimeout(() => {
			if (isOwner) { intelliPopup.hide(); isOwner = false; }
		}, 150);
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
		if (isOwner) { intelliPopup.hide(); isOwner = false; }
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
			placeholder={mode === 'embed' ? (placeholder || 'Use <variable> or type for suggestions...') : placeholder}
			class={className}
			oninput={(e) => { oninput(e); refreshSuggestions(); }}
			onkeydown={handleKeydown}
			onblur={handleBlur}
			onclick={refreshSuggestions}
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
