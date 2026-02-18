<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { CHANGELOG, type ChangelogEntry } from '$lib/changelog';
	import * as Dialog from '$lib/components/ui/dialog';
	import ScrollText from '@lucide/svelte/icons/scroll-text';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Sparkles from '@lucide/svelte/icons/sparkles';

	let open = $derived(app.showChangelog);
	let expandedVersions = $state<Set<string>>(new Set([CHANGELOG[0]?.version ?? '']));

	function toggleVersion(version: string) {
		const next = new Set(expandedVersions);
		if (next.has(version)) next.delete(version);
		else next.add(version);
		expandedVersions = next;
	}

	function isExpanded(version: string): boolean {
		return expandedVersions.has(version);
	}
</script>

<Dialog.Root {open} onOpenChange={(v) => { app.showChangelog = v; }}>
	<Dialog.Content class="sm:max-w-[520px] p-0 gap-0 overflow-hidden max-h-[80vh] flex flex-col" showCloseButton={false}>
		<!-- Header -->
		<div class="flex items-center gap-2.5 px-4 py-3 border-b border-border-dark panel-raised shrink-0">
			<div class="p-1.5 rounded-md bg-primary/10 text-primary">
				<ScrollText size={15} />
			</div>
			<div>
				<Dialog.Title class="text-sm font-medium text-foreground">Changelog</Dialog.Title>
				<Dialog.Description class="text-[10px] text-muted-foreground">What's new in Ironbullet</Dialog.Description>
			</div>
		</div>

		<!-- Changelog entries -->
		<div class="flex-1 overflow-y-auto p-3 space-y-1.5">
			{#each CHANGELOG as entry, i (entry.version)}
				{@const expanded = isExpanded(entry.version)}
				{@const isLatest = i === 0}
				<div class="rounded-md border {isLatest ? 'border-primary/30 bg-primary/[0.03]' : 'border-border bg-background/40'}">
					<!-- Version header (clickable expander) -->
					<button
						class="w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-secondary/30 transition-colors rounded-md"
						onclick={() => toggleVersion(entry.version)}
					>
						<ChevronRight
							size={12}
							class="text-muted-foreground shrink-0 transition-transform duration-150"
							style="transform: rotate({expanded ? '90deg' : '0deg'})"
						/>
						<span class="text-[12px] font-semibold text-foreground">v{entry.version}</span>
						{#if isLatest}
							<span class="flex items-center gap-0.5 text-[9px] text-primary bg-primary/10 px-1.5 py-0.5 rounded-full font-medium">
								<Sparkles size={8} /> Latest
							</span>
						{/if}
						<span class="text-[10px] text-muted-foreground ml-auto">{entry.date}</span>
					</button>

					<!-- Expanded content -->
					{#if expanded}
						<div class="px-3 pb-3 space-y-2.5">
							{#if entry.highlights}
								<p class="text-[11px] text-foreground/80 pl-5 leading-relaxed">{entry.highlights}</p>
							{/if}

							{#each entry.sections as section}
								<div class="pl-5">
									<div class="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider mb-1">{section.title}</div>
									<ul class="space-y-0.5">
										{#each section.items as item}
											<li class="text-[10px] text-foreground/75 flex gap-1.5 leading-relaxed">
												<span class="text-muted-foreground/60 shrink-0 mt-0.5">&bull;</span>
												<span>{item}</span>
											</li>
										{/each}
									</ul>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>

		<!-- Footer -->
		<div class="flex items-center justify-end px-4 py-2.5 bg-background/50 border-t border-border shrink-0">
			<button class="skeu-btn text-[10px] text-foreground" onclick={() => { app.showChangelog = false; }}>Close</button>
		</div>
	</Dialog.Content>
</Dialog.Root>
