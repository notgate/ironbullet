<script lang="ts">
	import { app, zoomIn, zoomOut, zoomReset } from '$lib/state.svelte';
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';
	import Minus from '@lucide/svelte/icons/minus';
	import Plus from '@lucide/svelte/icons/plus';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';

	let { searchQuery, shouldShowSetting }: {
		searchQuery: string;
		shouldShowSetting: (section: string, label: string) => boolean;
	} = $props();

	const FONTS = [
		{ value: 'Segoe UI', label: 'Segoe UI' },
		{ value: 'Inter', label: 'Inter' },
		{ value: 'Cascadia Code', label: 'Cascadia Code' },
		{ value: 'Consolas', label: 'Consolas' },
		{ value: 'system-ui', label: 'System Default' },
	];
</script>

{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2">Display</div>{/if}

{#if shouldShowSetting('display', 'UI Scale')}
	<div class="flex items-center justify-between py-1.5">
		<span class="text-[11px] text-muted-foreground">UI Scale</span>
		<div class="flex items-center gap-1.5">
			<button class="skeu-btn text-[10px] w-6 h-6 flex items-center justify-center p-0" onclick={() => zoomOut()} title="Zoom Out">
				<Minus size={10} />
			</button>
			<span class="text-[11px] text-foreground font-mono w-10 text-center">{Math.round(app.zoom * 100)}%</span>
			<button class="skeu-btn text-[10px] w-6 h-6 flex items-center justify-center p-0" onclick={() => zoomIn()} title="Zoom In">
				<Plus size={10} />
			</button>
			<button class="skeu-btn text-[10px] w-6 h-6 flex items-center justify-center p-0 ml-1" onclick={() => zoomReset()} title="Reset">
				<RotateCcw size={10} />
			</button>
		</div>
	</div>
{/if}

{#if shouldShowSetting('display', 'Font Family')}
	<div class="flex items-center justify-between py-1.5">
		<span class="text-[11px] text-muted-foreground">Font Family</span>
		<SkeuSelect
			value={app.fontFamily}
			onValueChange={(v) => { app.fontFamily = v; }}
			options={FONTS}
			class="text-[11px] w-[160px]"
		/>
	</div>
{/if}

{#if shouldShowSetting('display', 'Font Size')}
	<div class="flex items-center justify-between py-1.5">
		<span class="text-[11px] text-muted-foreground">Font Size</span>
		<div class="flex items-center gap-1.5">
			<input
				type="range"
				min="10"
				max="18"
				step="1"
				bind:value={app.fontSize}
				class="w-24 accent-[var(--primary)]"
			/>
			<span class="text-[11px] text-foreground font-mono w-8 text-right">{app.fontSize}px</span>
		</div>
	</div>
{/if}

{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
