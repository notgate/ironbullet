<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import * as Dialog from '$lib/components/ui/dialog';
	import Download from '@lucide/svelte/icons/download';
	import Check from '@lucide/svelte/icons/check';
	import ArrowUpCircle from '@lucide/svelte/icons/arrow-up-circle';

	let open = $derived(app.showUpdateDialog);

	const RING_CIRCUMFERENCE = 2 * Math.PI * 20;
	let ringOffset = $derived(RING_CIRCUMFERENCE - (app.updateProgress / 100) * RING_CIRCUMFERENCE);

	function installUpdate() {
		if (!app.updateDownloadUrl) return;
		app.updateInstalling = true;
		app.updateProgress = 0;
		app.updateComplete = false;
		send('download_update', { url: app.updateDownloadUrl });
	}

	function dismiss() {
		app.showUpdateDialog = false;
	}
</script>

<Dialog.Root {open} onOpenChange={(v) => { if (!app.updateInstalling) app.showUpdateDialog = v; }}>
	<Dialog.Content class="sm:max-w-[360px] p-0 gap-0 overflow-hidden" showCloseButton={false}>
		<div class="p-5">
			<div class="flex items-start gap-3">
				<div class="p-2 rounded-md bg-accent/10 text-accent shrink-0">
					<ArrowUpCircle size={18} />
				</div>
				<div class="flex-1">
					<Dialog.Title class="text-sm font-medium text-foreground">Update Available</Dialog.Title>
					<Dialog.Description class="text-[11px] text-muted-foreground mt-1">
						<strong>v{app.updateLatestVersion}</strong> is available (you have v{app.updateCurrentVersion}).
						{#if app.updatePublishedAt}
							<br/>Released {new Date(app.updatePublishedAt).toLocaleDateString()}.
						{/if}
					</Dialog.Description>
				</div>
			</div>

			{#if app.updateReleaseNotes}
				<div class="mt-3 p-2 bg-background/60 rounded border border-border text-[10px] text-muted-foreground whitespace-pre-wrap max-h-24 overflow-y-auto">
					{app.updateReleaseNotes}
				</div>
			{/if}

			{#if app.updateInstalling || app.updateComplete}
				<div class="flex justify-center mt-4">
					{#if app.updateComplete}
						<div class="flex flex-col items-center gap-2">
							<div class="w-12 h-12 rounded-full bg-green/10 flex items-center justify-center">
								<Check size={22} class="text-green" />
							</div>
							<span class="text-[10px] text-green">Restart to apply update</span>
						</div>
					{:else}
						<div class="flex flex-col items-center gap-2">
							<div class="relative w-12 h-12 flex items-center justify-center">
								<svg class="w-12 h-12 -rotate-90" viewBox="0 0 44 44">
									<circle
										cx="22" cy="22" r="20"
										fill="none" stroke="currentColor" stroke-width="2.5"
										class="text-border"
									/>
									<circle
										cx="22" cy="22" r="20"
										fill="none" stroke="currentColor" stroke-width="2.5"
										stroke-linecap="round"
										stroke-dasharray={RING_CIRCUMFERENCE}
										stroke-dashoffset={ringOffset}
										class="text-accent transition-all duration-300"
									/>
								</svg>
								<span class="absolute text-[9px] font-mono text-foreground">{app.updateProgress}%</span>
							</div>
							<span class="text-[10px] text-muted-foreground">Downloading...</span>
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<div class="flex items-center gap-2 px-4 py-3 bg-background/50 border-t border-border justify-end">
			{#if app.updateComplete}
				<button class="skeu-btn text-[10px] text-foreground" onclick={dismiss}>Close</button>
			{:else if app.updateInstalling}
				<span class="text-[10px] text-muted-foreground">Installing...</span>
			{:else}
				<button class="skeu-btn text-[10px] text-muted-foreground" onclick={dismiss}>Later</button>
				<button
					class="skeu-btn text-[10px] text-accent flex items-center gap-1.5"
					onclick={installUpdate}
					disabled={!app.updateDownloadUrl}
				>
					<Download size={11} />
					Install Update
				</button>
			{/if}
		</div>
	</Dialog.Content>
</Dialog.Root>
