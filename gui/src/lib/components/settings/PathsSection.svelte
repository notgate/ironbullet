<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';

	let { searchQuery, shouldShowSetting }: {
		searchQuery: string;
		shouldShowSetting: (section: string, label: string) => boolean;
	} = $props();
</script>

{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Paths</div>{/if}

{#if shouldShowSetting('paths', 'Collections Folder')}
	<div class="py-1.5">
		<div class="flex items-center justify-between">
			<div>
				<span class="text-[11px] text-muted-foreground">Collections folder</span>
				<p class="text-[9px] text-muted-foreground/60">Folder containing .rfx configs for quick access</p>
			</div>
			<button class="skeu-btn text-[10px]" onclick={() => send('browse_folder', { field: 'collections' })}>Browse</button>
		</div>
		<input
			type="text"
			bind:value={app.collectionsPath}
			class="w-full skeu-input text-[10px] font-mono mt-1"
			placeholder="Not set"
		/>
	</div>
{/if}

{#if shouldShowSetting('paths', 'Wordlist Directory')}
	<div class="py-1.5">
		<div class="flex items-center justify-between">
			<div>
				<span class="text-[11px] text-muted-foreground">Default wordlist directory</span>
				<p class="text-[9px] text-muted-foreground/60">Start browsing wordlists from this folder</p>
			</div>
			<button class="skeu-btn text-[10px]" onclick={() => send('browse_folder', { field: 'wordlist_dir' })}>Browse</button>
		</div>
		<input
			type="text"
			bind:value={app.defaultWordlistPath}
			class="w-full skeu-input text-[10px] font-mono mt-1"
			placeholder="Not set"
		/>
	</div>
{/if}

{#if shouldShowSetting('paths', 'Proxy Directory')}
	<div class="py-1.5">
		<div class="flex items-center justify-between">
			<div>
				<span class="text-[11px] text-muted-foreground">Default proxy directory</span>
				<p class="text-[9px] text-muted-foreground/60">Start browsing proxy files from this folder</p>
			</div>
			<button class="skeu-btn text-[10px]" onclick={() => send('browse_folder', { field: 'proxy_dir' })}>Browse</button>
		</div>
		<input
			type="text"
			bind:value={app.defaultProxyPath}
			class="w-full skeu-input text-[10px] font-mono mt-1"
			placeholder="Not set"
		/>
	</div>
{/if}

{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
