<script lang="ts">
	import { app } from '$lib/state.svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import ShieldAlert from '@lucide/svelte/icons/shield-alert';
	import CircleAlert from '@lucide/svelte/icons/circle-alert';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';

	let open = $derived(app.securityIssues.length > 0);
	let criticalCount = $derived(app.securityIssues.filter(i => i.severity === 'Critical').length);
	let warningCount = $derived(app.securityIssues.filter(i => i.severity === 'Warning').length);

	function onOpenChange(v: boolean) {
		if (!v) {
			app.securityIssues = [];
		}
	}

	function handleClose() {
		app.securityIssues = [];
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content class="sm:max-w-[560px] p-0 gap-0 overflow-hidden max-h-[80vh] flex flex-col" showCloseButton={false}>
		<div class="p-5 border-b border-border shrink-0">
			<div class="flex items-start gap-3">
				<div class="p-2 rounded-md bg-red/10 text-red shrink-0">
					<ShieldAlert size={20} />
				</div>
				<div>
					<Dialog.Title class="text-sm font-medium text-foreground">Malicious Script Detected</Dialog.Title>
					<Dialog.Description class="text-[11px] text-muted-foreground mt-1">
						This config contains potentially dangerous code.
						{#if criticalCount > 0}
							<span class="text-red font-medium">{criticalCount} critical</span>{warningCount > 0 ? ` and ${warningCount} warning` : ''} issue{app.securityIssues.length > 1 ? 's' : ''} found.
						{:else}
							{warningCount} warning{warningCount > 1 ? 's' : ''} found.
						{/if}
						Review the flagged items below before running.
					</Dialog.Description>
				</div>
			</div>
		</div>

		<div class="overflow-y-auto flex-1 p-4 space-y-3 min-h-0">
			{#each app.securityIssues as issue, idx}
				<div class="rounded-md border {issue.severity === 'Critical' ? 'border-red/30 bg-red/5' : 'border-yellow/30 bg-yellow/5'} overflow-hidden">
					<div class="px-3 py-2 flex items-start gap-2">
						{#if issue.severity === 'Critical'}
							<CircleAlert size={14} class="text-red shrink-0 mt-0.5" />
						{:else}
							<TriangleAlert size={14} class="text-yellow shrink-0 mt-0.5" />
						{/if}
						<div class="min-w-0">
							<div class="text-[11px] font-medium {issue.severity === 'Critical' ? 'text-red' : 'text-yellow'}">
								{issue.title}
							</div>
							<div class="text-[10px] text-muted-foreground mt-0.5">
								{issue.description}
							</div>
						</div>
					</div>
					{#if issue.code_snippet}
						<div class="border-t {issue.severity === 'Critical' ? 'border-red/20' : 'border-yellow/20'}">
							<pre class="text-[10px] text-foreground/80 px-3 py-2 overflow-x-auto font-mono whitespace-pre-wrap break-all leading-relaxed bg-background/40">{issue.code_snippet}</pre>
						</div>
					{/if}
				</div>
			{/each}
		</div>

		<div class="flex items-center gap-2 px-4 py-3 bg-background/50 border-t border-border justify-end shrink-0">
			<span class="text-[10px] text-muted-foreground mr-auto">Config was loaded — review flagged blocks before running.</span>
			<button class="skeu-btn text-[10px] text-foreground" onclick={handleClose}>Close</button>
		</div>
	</Dialog.Content>
</Dialog.Root>
