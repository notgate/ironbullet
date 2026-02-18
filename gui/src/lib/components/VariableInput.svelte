<script lang="ts">
	import { app } from '$lib/state.svelte';
	import SkeuSelect from './SkeuSelect.svelte';
	import Variable from '@lucide/svelte/icons/variable';
	import Type from '@lucide/svelte/icons/type';
	import Braces from '@lucide/svelte/icons/braces';

	type InputMode = 'raw' | 'embed' | 'variable';

	let {
		value,
		oninput,
		placeholder = '',
		class: className = '',
	}: {
		value: string;
		oninput: (e: Event) => void;
		placeholder?: string;
		class?: string;
	} = $props();

	let mode = $state<InputMode>('embed');

	// Extract all variables from pipeline blocks
	const availableVars = $derived.by(() => {
		const vars = new Set<string>();

		// Common output variables from different block types
		const extractVars = (blocks: any[]) => {
			for (const block of blocks) {
				const s = block.settings;

				// Output variables
				if (s.output_var) vars.add(s.output_var);
				if (s.response_var) vars.add(s.response_var);

				// Constants
				if (s.constants) {
					for (const c of s.constants) {
						if (c.name) vars.add(c.name);
					}
				}

				// Common data variables
				vars.add('data.SOURCE');
				vars.add('data.HEADERS');
				vars.add('data.COOKIES');
				vars.add('data.RESPONSECODE');
				vars.add('data.ADDRESS');

				// Input variables
				vars.add('input.DATA');
				vars.add('input.USER');
				vars.add('input.PASS');

				// Nested blocks
				if (s.true_blocks) extractVars(s.true_blocks);
				if (s.false_blocks) extractVars(s.false_blocks);
				if (s.blocks) extractVars(s.blocks);
			}
		};

		extractVars(app.pipeline.blocks);

		return Array.from(vars)
			.filter(v => v && v.trim())
			.sort()
			.map(v => ({ value: v, label: v }));
	});

	function cycleMode() {
		if (mode === 'raw') mode = 'embed';
		else if (mode === 'embed') mode = 'variable';
		else mode = 'raw';
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
			type="text"
			{value}
			placeholder={mode === 'embed' ? (placeholder || 'Use <variable> or {{variable}}') : placeholder}
			class={className}
			{oninput}
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
		{getModeLabel()}
	</button>
</div>
