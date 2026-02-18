<script lang="ts">
	import { app } from '$lib/state.svelte';

	let { searchQuery, shouldShowSetting }: {
		searchQuery: string;
		shouldShowSetting: (section: string, label: string) => boolean;
	} = $props();
</script>

{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Layout</div>{/if}

{#if shouldShowSetting('layout', 'Block Palette')}
	<div class="flex items-center justify-between py-1.5">
		<span class="text-[11px] text-muted-foreground">Block Palette</span>
		<!-- svelte-ignore a11y_label_has_associated_control -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={app.showBlockPalette}
				onchange={() => { app.showBlockPalette = !app.showBlockPalette; }}
				class="skeu-checkbox"
			/>
			<span class="text-[11px] text-foreground">{app.showBlockPalette ? 'Visible' : 'Hidden'}</span>
		</label>
	</div>
{/if}

{#if shouldShowSetting('layout', 'Palette Width')}
	<div class="flex items-center justify-between py-1.5">
		<span class="text-[11px] text-muted-foreground">Palette Width</span>
		<div class="flex items-center gap-1.5">
			<input
				type="range"
				min="120"
				max="400"
				step="10"
				bind:value={app.leftPanelWidth}
				class="w-24 accent-[var(--primary)]"
			/>
			<span class="text-[11px] text-foreground font-mono w-10 text-right">{app.leftPanelWidth}px</span>
		</div>
	</div>
{/if}

{#if shouldShowSetting('layout', 'Bottom Panel')}
	<div class="flex items-center justify-between py-1.5">
		<span class="text-[11px] text-muted-foreground">Bottom Panel</span>
		<div class="flex items-center gap-1.5">
			<input
				type="range"
				min="100"
				max="500"
				step="10"
				bind:value={app.bottomPanelHeight}
				class="w-24 accent-[var(--primary)]"
			/>
			<span class="text-[11px] text-foreground font-mono w-10 text-right">{app.bottomPanelHeight}px</span>
		</div>
	</div>
{/if}

{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
