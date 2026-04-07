<script lang="ts">
	import { app } from '$lib/state.svelte';

	let result = $derived(app.debugResult);
	let vars = $derived(result?.variables_after || {});
</script>

<div class="flex flex-col h-full">
	<div class="px-2 py-1 panel-raised">
		<span class="text-[10px] uppercase tracking-wider text-muted-foreground">Variables</span>
	</div>
	<div class="flex-1 overflow-y-auto panel-inset">
		{#if Object.keys(vars).length === 0}
			<div class="flex items-center justify-center h-full text-muted-foreground text-[10px]">
				Run debug to see variables
			</div>
		{:else}
			{#each Object.entries(vars) as [key, value]}
				<div class="flex px-2 py-0.5 border-b border-border/30 hover:bg-accent/10 text-[11px]">
					<span class="text-primary font-mono w-32 shrink-0 truncate" title={key}>{key}</span>
					<span class="text-foreground font-mono truncate" title={String(value)}>{String(value)}</span>
				</div>
			{/each}
		{/if}
	</div>
</div>
