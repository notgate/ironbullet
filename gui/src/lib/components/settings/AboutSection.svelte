<script lang="ts">
	let { searchQuery }: { searchQuery: string } = $props();
</script>

{#if !searchQuery}
	<div class="space-y-2 py-2">
		<div class="text-[13px] font-semibold text-foreground">Ironbullet</div>
		<div class="text-[11px] text-muted-foreground">Version 0.1.0</div>
		<div class="text-[10px] text-muted-foreground mt-2">
			Pipeline-based HTTP automation toolkit with visual block editor.
		</div>
	</div>
{/if}
