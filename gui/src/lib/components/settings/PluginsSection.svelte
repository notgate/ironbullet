<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';

	let { searchQuery, shouldShowSetting }: {
		searchQuery: string;
		shouldShowSetting: (section: string, label: string) => boolean;
	} = $props();
</script>

{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Plugins</div>{/if}

{#if shouldShowSetting('plugins', 'Plugins Directory')}
	<div class="py-1.5">
		<div class="flex items-center justify-between">
			<div>
				<span class="text-[11px] text-muted-foreground">Plugins directory</span>
				<p class="text-[9px] text-muted-foreground/60">Folder containing .dll plugin files</p>
			</div>
			<div class="flex gap-1">
				<button class="skeu-btn text-[10px]" onclick={() => send('browse_folder', { field: 'plugins' })}>Browse</button>
				<button class="skeu-btn text-[10px]" onclick={() => send('reload_plugins')}>Reload</button>
			</div>
		</div>
		<input
			type="text"
			value={(app.config as any)?.plugins_path || ''}
			class="w-full skeu-input text-[10px] font-mono mt-1"
			placeholder="Not set"
			oninput={(e) => { (app.config as any).plugins_path = (e.target as HTMLInputElement).value; }}
		/>
	</div>
{/if}

{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
