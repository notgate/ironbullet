<script lang="ts">
	import { getAvailableVariables } from '$lib/state.svelte';

	let {
		value = '',
		onValueChange,
		blockId = '',
		placeholder = '',
		class: className = '',
	}: {
		value?: string;
		onValueChange: (v: string) => void;
		blockId?: string;
		placeholder?: string;
		class?: string;
	} = $props();

	let inputEl: HTMLInputElement | undefined = $state();
	let showDropdown = $state(false);
	let cursorPos = $state(0);
	let filterText = $state('');

	let allVars = $derived(getAvailableVariables(blockId));
	let filteredVars = $derived(
		filterText
			? allVars.filter(v => v.toLowerCase().includes(filterText.toLowerCase()))
			: allVars
	);

	let selectedIdx = $state(0);

	function onInput(e: Event) {
		const input = e.target as HTMLInputElement;
		const val = input.value;
		onValueChange(val);
		cursorPos = input.selectionStart || 0;

		// Check if cursor is inside a <...> token
		const before = val.substring(0, cursorPos);
		const openBracket = before.lastIndexOf('<');
		const closeBracket = before.lastIndexOf('>');

		if (openBracket > closeBracket) {
			// We're inside an open <...> token
			filterText = before.substring(openBracket + 1);
			showDropdown = true;
			selectedIdx = 0;
		} else {
			showDropdown = false;
			filterText = '';
		}
	}

	function insertVar(varName: string) {
		const before = value.substring(0, cursorPos);
		const after = value.substring(cursorPos);
		const openBracket = before.lastIndexOf('<');
		const newValue = before.substring(0, openBracket) + '<' + varName + '>' + after;
		onValueChange(newValue);
		showDropdown = false;
		filterText = '';
		setTimeout(() => inputEl?.focus(), 0);
	}

	function onKeydown(e: KeyboardEvent) {
		if (!showDropdown || filteredVars.length === 0) return;

		if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIdx = Math.min(selectedIdx + 1, filteredVars.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIdx = Math.max(selectedIdx - 1, 0);
		} else if (e.key === 'Enter' || e.key === 'Tab') {
			if (showDropdown && filteredVars.length > 0) {
				e.preventDefault();
				insertVar(filteredVars[selectedIdx]);
			}
		} else if (e.key === 'Escape') {
			showDropdown = false;
		}
	}

	function onBlur() {
		// Delay to allow click on dropdown items
		setTimeout(() => { showDropdown = false; }, 150);
	}
</script>

<div class="relative">
	<input
		bind:this={inputEl}
		type="text"
		{value}
		{placeholder}
		class="{className}"
		oninput={onInput}
		onkeydown={onKeydown}
		onblur={onBlur}
		onfocus={(e) => {
			const input = e.target as HTMLInputElement;
			cursorPos = input.selectionStart || 0;
		}}
	/>

	{#if showDropdown && filteredVars.length > 0}
		<div class="absolute left-0 right-0 top-full mt-0.5 bg-popover border border-border rounded shadow-lg z-50 max-h-[140px] overflow-y-auto" style="zoom: var(--app-zoom, 1);">
			{#each filteredVars as v, i}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="px-2 py-0.5 text-[10px] font-mono cursor-pointer {i === selectedIdx ? 'bg-accent text-accent-foreground' : 'text-foreground hover:bg-accent/30'}"
					onmousedown={() => insertVar(v)}
				>
					&lt;{v}&gt;
				</div>
			{/each}
		</div>
	{/if}
</div>
