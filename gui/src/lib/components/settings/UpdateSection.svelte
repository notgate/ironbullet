<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import { toast } from '$lib/toast.svelte';
	import Check from '@lucide/svelte/icons/check';
	import Download from '@lucide/svelte/icons/download';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';

	let { searchQuery }: { searchQuery: string } = $props();

	const RING_CIRCUMFERENCE = 2 * Math.PI * 18;
	let ringOffset = $derived(RING_CIRCUMFERENCE - (app.updateProgress / 100) * RING_CIRCUMFERENCE);

	function checkForUpdates() {
		app.updateChecking = true;
		app.updateAvailable = false;
		app.updateComplete = false;
		send('check_for_updates');
	}

	function installUpdate() {
		if (!app.updateDownloadUrl) return;
		app.updateInstalling = true;
		app.updateProgress = 0;
		app.updateComplete = false;
		send('download_update', { url: app.updateDownloadUrl });
	}
</script>

{#if !searchQuery}
	<div class="space-y-4 py-2">
		<div class="text-[12px] font-semibold text-foreground tracking-tight">Updates</div>

		<div class="flex items-center gap-3">
			<div class="text-[11px] text-muted-foreground flex-1">
				Current version: <span class="text-foreground font-mono">{app.updateCurrentVersion}</span>
			</div>
			<button
				class="skeu-btn text-[10px] flex items-center gap-1.5"
				onclick={checkForUpdates}
				disabled={app.updateChecking}
			>
				<RefreshCw size={11} class={app.updateChecking ? 'animate-spin' : ''} />
				{app.updateChecking ? 'Checking...' : 'Check for Updates'}
			</button>
		</div>

		{#if app.updateAvailable}
			<div class="bg-background/80 border border-border rounded-md p-3 space-y-3">
				<div class="flex items-start justify-between">
					<div>
						<div class="text-[11px] font-medium text-foreground">
							v{app.updateLatestVersion} available
						</div>
						{#if app.updatePublishedAt}
							<div class="text-[10px] text-muted-foreground mt-0.5">
								Released {new Date(app.updatePublishedAt).toLocaleDateString()}
							</div>
						{/if}
					</div>

					{#if app.updateComplete}
						<div class="w-10 h-10 rounded-full bg-green/10 flex items-center justify-center">
							<Check size={18} class="text-green" />
						</div>
					{:else if app.updateInstalling}
						<div class="relative w-10 h-10 flex items-center justify-center">
							<svg class="w-10 h-10 -rotate-90" viewBox="0 0 40 40">
								<circle cx="20" cy="20" r="18" fill="none" stroke="currentColor" stroke-width="2.5" class="text-border" />
								<circle
									cx="20" cy="20" r="18" fill="none" stroke="currentColor" stroke-width="2.5"
									stroke-linecap="round"
									stroke-dasharray={RING_CIRCUMFERENCE}
									stroke-dashoffset={ringOffset}
									class="text-accent transition-all duration-300"
								/>
							</svg>
							<span class="absolute text-[8px] font-mono text-foreground">{app.updateProgress}%</span>
						</div>
					{:else}
						<button
							class="skeu-btn text-[10px] flex items-center gap-1.5 text-accent"
							onclick={installUpdate}
							disabled={!app.updateDownloadUrl}
						>
							<Download size={11} />
							Install Update
						</button>
					{/if}
				</div>

				{#if app.updateReleaseNotes}
					<div class="text-[10px] text-muted-foreground border-t border-border pt-2 whitespace-pre-wrap max-h-24 overflow-y-auto">
						{app.updateReleaseNotes}
					</div>
				{/if}

				{#if app.updateComplete}
					<div class="text-[10px] text-green flex items-center gap-1.5 border-t border-border pt-2">
						<Check size={11} />
						Update installed. Restart Ironbullet to apply.
					</div>
				{/if}
			</div>
		{/if}
	</div>
{/if}
