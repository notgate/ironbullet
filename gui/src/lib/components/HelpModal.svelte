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

	let expandedSections = $state<Set<number>>(new Set([0]));

	function toggleSection(index: number) {
		const next = new Set(expandedSections);
		if (next.has(index)) { next.delete(index); } else { next.add(index); }
		expandedSections = next;
	}

	type Span = { text: string; isCode: boolean };

	function splitWithCode(text: string): Span[] {
		const parts = text.split(/(`[^`\n]+`)/);
		return parts.map(p =>
			p.startsWith('`') && p.endsWith('`') && p.length > 2
				? { text: p.slice(1, -1), isCode: true }
				: { text: p, isCode: false }
		);
	}

	function renderLine(line: string): Span[] {
		return splitWithCode(line);
	}
</script>

<Dialog.Root {open} onOpenChange={(v) => { open = v; }}>
	<Dialog.Content class="bg-surface border-border max-w-[700px] max-h-[85vh] p-0 gap-0 overflow-hidden flex flex-col" showCloseButton={false}>
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

		<!-- Accordion content -->
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
						<ChevronDown size={14} class="text-muted-foreground transition-transform duration-200 {expandedSections.has(i) ? 'rotate-180' : ''}" />
					</button>

					{#if expandedSections.has(i)}
						<div class="px-3 pb-3 pt-1 animate-in fade-in slide-in-from-top-1 duration-200">
							<div class="text-[11px] text-foreground/90 leading-relaxed space-y-1.5">
								{#each section.content.split('\n') as line}
									{#if line.trim() === ''}
										<div class="h-1"></div>
									{:else if line.trim().startsWith('•') || line.trim().startsWith('-')}
										<div class="flex gap-2 pl-1">
											<span class="text-primary/60 shrink-0 mt-px">•</span>
											<span class="flex-1">
												{#each renderLine(line.trim().replace(/^[•\-]\s*/, '')) as span}
													{#if span.isCode}<code class="font-mono text-[10px] bg-secondary px-1 py-0.5 rounded text-primary border border-border/30">{span.text}</code>{:else}{span.text}{/if}
												{/each}
											</span>
										</div>
									{:else if line.trim().match(/^\d+\./)}
										<div class="flex gap-2 pl-1">
											<span class="text-primary/80 font-medium shrink-0 w-4">{line.trim().match(/^(\d+)\./)?.[1]}.</span>
											<span class="flex-1">
												{#each renderLine(line.trim().replace(/^\d+\.\s*/, '')) as span}
													{#if span.isCode}<code class="font-mono text-[10px] bg-secondary px-1 py-0.5 rounded text-primary border border-border/30">{span.text}</code>{:else}{span.text}{/if}
												{/each}
											</span>
										</div>
									{:else}
										<p class="pl-1">
											{#each renderLine(line) as span}
												{#if span.isCode}<code class="font-mono text-[10px] bg-secondary px-1 py-0.5 rounded text-primary border border-border/30">{span.text}</code>{:else}{span.text}{/if}
											{/each}
										</p>
									{/if}
								{/each}
							</div>
						</div>
					{/if}
				</div>
			{/each}
		</div>

		<!-- Footer — hint text only, no duplicate close button -->
		<div class="px-4 py-2 border-t border-border shrink-0 panel-raised">
			<span class="text-[10px] text-muted-foreground">Click section headers to expand / collapse</span>
		</div>
	</Dialog.Content>
</Dialog.Root>
