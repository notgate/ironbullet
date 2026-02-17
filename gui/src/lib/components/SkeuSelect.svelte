<script lang="ts">
	import * as Select from '$lib/components/ui/select';

	let {
		value,
		onValueChange,
		options,
		class: className = '',
		placeholder = 'Select...',
	}: {
		value: string;
		onValueChange: (v: string) => void;
		options: { value: string; label: string }[];
		class?: string;
		placeholder?: string;
	} = $props();
</script>

<Select.Root type="single" {value} onValueChange={(v) => { if (v) onValueChange(v); }}>
	<Select.Trigger class="skeu-select-trigger {className}">
		{options.find((o) => o.value === value)?.label || placeholder}
	</Select.Trigger>
	<Select.Content class="skeu-select-content">
		{#each options as opt}
			<Select.Item value={opt.value} label={opt.label} class="skeu-select-item" />
		{/each}
	</Select.Content>
</Select.Root>
