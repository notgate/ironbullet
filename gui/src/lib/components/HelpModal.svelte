<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import X from '@lucide/svelte/icons/x';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';

	let {
		open = $bindable(false),
		title,
		sections,
	}: {
		open?: boolean;
		title: string;
		sections: { heading: string; content: string }[];
	} = $props();

	let expandedSections = $state<Set<number>>(new Set([0])); // First section expanded by default

	function toggleSection(index: number) {
		const newExpanded = new Set(expandedSections);
		if (newExpanded.has(index)) {
			newExpanded.delete(index);
		} else {
			newExpanded.add(index);
		}
		expandedSections = newExpanded;
	}
</script>

<Dialog.Root {open} onOpenChange={(v) => { open = v; }}>
	<Dialog.Content class="bg-surface border-border max-w-[700px] max-h-[85vh] p-0 gap-0 overflow-hidden flex flex-col">
		<!-- Header -->
		<div class="px-4 py-3 border-b border-border shrink-0 panel-raised flex items-center justify-between">
			<h2 class="text-sm font-semibold text-foreground">{title}</h2>
			<button
				class="p-1 rounded hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors"
				onclick={() => { open = false; }}
			>
				<X size={14} />
			</button>
		</div>

		<!-- Content with Accordion -->
		<div class="flex-1 overflow-y-auto p-3 panel-inset">
			{#each sections as section, i}
				<div class="border-b border-border/30 last:border-0">
					<button
						class="w-full px-3 py-2.5 flex items-center justify-between hover:bg-accent/10 transition-colors group"
						onclick={() => toggleSection(i)}
					>
						<h3 class="text-xs font-semibold text-primary uppercase tracking-wide group-hover:text-primary/80 transition-colors">
							{section.heading}
						</h3>
						<ChevronDown
							size={14}
							class="text-muted-foreground transition-transform duration-200 {expandedSections.has(i) ? 'rotate-180' : ''}"
						/>
					</button>

					{#if expandedSections.has(i)}
						<div class="px-3 pb-3 pt-1 animate-in fade-in slide-in-from-top-1 duration-200">
							<div class="text-[11px] text-foreground/90 leading-relaxed space-y-2">
								{#each section.content.split('\n\n') as paragraph}
									{#if paragraph.trim().startsWith('•') || paragraph.trim().match(/^\d+\./)}
										<!-- Bullet points or numbered lists -->
										<ul class="space-y-1 pl-1">
											{#each paragraph.split('\n') as line}
												{#if line.trim()}
													<li class="flex gap-2">
														<span class="text-primary/70 shrink-0">{line.trim().match(/^[•\-\d.]+/)?.[0] || '•'}</span>
														<span class="flex-1">{line.trim().replace(/^[•\-\d.]+\s*/, '')}</span>
													</li>
												{/if}
											{/each}
										</ul>
									{:else}
										<!-- Regular paragraph -->
										<p class="whitespace-pre-line">{paragraph}</p>
									{/if}
								{/each}
							</div>
						</div>
					{/if}
				</div>
			{/each}
		</div>

		<!-- Footer -->
		<div class="px-4 py-2.5 border-t border-border shrink-0 panel-raised flex items-center justify-between">
			<span class="text-[10px] text-muted-foreground">Click sections to expand/collapse</span>
			<button
				class="skeu-btn text-xs"
				onclick={() => { open = false; }}
			>
				Close
			</button>
		</div>
	</Dialog.Content>
</Dialog.Root>
