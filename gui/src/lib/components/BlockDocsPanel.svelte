<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { BLOCK_DOCS_FULL, GUIDE_SECTIONS, type BlockDoc } from '$lib/blockDocs';
	import { BLOCK_CATALOG } from '$lib/types';
	import X from '@lucide/svelte/icons/x';
	import Search from '@lucide/svelte/icons/search';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Download from '@lucide/svelte/icons/download';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Rocket from '@lucide/svelte/icons/rocket';
	import Puzzle from '@lucide/svelte/icons/puzzle';
	import Play from '@lucide/svelte/icons/play';
	import Shield from '@lucide/svelte/icons/shield';

	let searchQuery = $state('');
	let selectedType = $state<string | null>(app.blockDocsInitialType);
	let selectedGuide = $state<string | null>(null);

	const GUIDE_ICONS: Record<string, typeof Rocket> = {
		'Rocket': Rocket,
		'Puzzle': Puzzle,
		'Play': Play,
		'Shield': Shield,
	};

	const CATEGORY_ORDER = ['Requests', 'Parsing', 'Checks', 'Functions', 'Control', 'Utilities', 'Bypass', 'Sensors', 'Browser'];
	const CATEGORY_COLORS: Record<string, string> = {
		'Requests': '#0078d4',
		'Parsing': '#4ec9b0',
		'Checks': '#d7ba7d',
		'Functions': '#c586c0',
		'Control': '#dcdcaa',
		'Utilities': '#858585',
		'Bypass': '#e5c07b',
		'Browser': '#e06c75',
		'Sensors': '#2dd4bf',
	};

	const filteredDocs = $derived(() => {
		if (!searchQuery) return BLOCK_DOCS_FULL;
		const q = searchQuery.toLowerCase();
		return BLOCK_DOCS_FULL.filter(d =>
			d.name.toLowerCase().includes(q) ||
			d.category.toLowerCase().includes(q) ||
			d.description.toLowerCase().includes(q) ||
			d.type.toLowerCase().includes(q)
		);
	});

	const groupedDocs = $derived(() => {
		const groups = new Map<string, BlockDoc[]>();
		for (const doc of filteredDocs()) {
			if (!groups.has(doc.category)) groups.set(doc.category, []);
			groups.get(doc.category)!.push(doc);
		}
		const sorted = new Map<string, BlockDoc[]>();
		for (const cat of CATEGORY_ORDER) {
			if (groups.has(cat)) sorted.set(cat, groups.get(cat)!);
		}
		for (const [cat, docs] of groups) {
			if (!sorted.has(cat)) sorted.set(cat, docs);
		}
		return sorted;
	});

	const selectedDoc = $derived(() => {
		if (!selectedType) return null;
		return BLOCK_DOCS_FULL.find(d => d.type === selectedType) || null;
	});

	const activeGuide = $derived(() => {
		if (!selectedGuide) return null;
		return GUIDE_SECTIONS.find(g => g.id === selectedGuide) || null;
	});

	function close() {
		app.showBlockDocs = false;
		app.blockDocsInitialType = null;
	}

	function selectBlock(type: string) {
		selectedType = type;
		selectedGuide = null;
	}

	function selectGuideSection(id: string) {
		selectedGuide = id;
		selectedType = null;
	}

	function getBlockColor(type: string): string {
		return BLOCK_CATALOG.find(b => b.type === type)?.color || '#858585';
	}

	function generateFullMarkdown(): string {
		let md = '# reqflow Documentation\n\n';

		// Guide sections
		for (const guide of GUIDE_SECTIONS) {
			md += `## ${guide.title}\n\n`;
			// Strip HTML tags for markdown
			const text = guide.content
				.replace(/<h3[^>]*>(.*?)<\/h3>/g, '### $1\n\n')
				.replace(/<p[^>]*>(.*?)<\/p>/gs, '$1\n\n')
				.replace(/<pre[^>]*>([\s\S]*?)<\/pre>/g, '```\n$1\n```\n\n')
				.replace(/<li[^>]*>(.*?)<\/li>/g, '- $1\n')
				.replace(/<code>(.*?)<\/code>/g, '`$1`')
				.replace(/<strong>(.*?)<\/strong>/g, '**$1**')
				.replace(/<em>(.*?)<\/em>/g, '*$1*')
				.replace(/<kbd>(.*?)<\/kbd>/g, '`$1`')
				.replace(/<table[\s\S]*?<\/table>/g, '')
				.replace(/<[^>]+>/g, '')
				.replace(/&lt;/g, '<')
				.replace(/&gt;/g, '>')
				.replace(/&amp;/g, '&')
				.replace(/\n{3,}/g, '\n\n')
				.trim();
			md += text + '\n\n---\n\n';
		}

		// Block reference
		md += '## Block Reference\n\n';
		for (const doc of BLOCK_DOCS_FULL) {
			md += `### ${doc.name}\n\n`;
			md += `**Category:** ${doc.category}\n\n`;
			md += `${doc.description}\n\n`;

			if (doc.parameters.length > 0) {
				md += '#### Parameters\n\n';
				md += '| Name | Type | Required | Description | Default |\n';
				md += '|------|------|----------|-------------|--------|\n';
				for (const p of doc.parameters) {
					md += `| \`${p.name}\` | ${p.type} | ${p.required ? 'Yes' : 'No'} | ${p.description} | ${p.default || '—'} |\n`;
				}
				md += '\n';
			}

			md += '#### Example\n\n';
			md += '```\n' + doc.codeExample + '\n```\n\n';

			if (doc.rustCode) {
				md += '#### Rust Implementation\n\n';
				md += '```rust\n' + doc.rustCode + '\n```\n\n';
			}

			if (doc.tips.length > 0) {
				md += '#### Tips\n\n';
				for (const tip of doc.tips) {
					md += `- ${tip}\n`;
				}
				md += '\n';
			}

			md += '---\n\n';
		}

		return md;
	}

	function downloadMarkdown() {
		const md = generateFullMarkdown();
		const blob = new Blob([md], { type: 'text/markdown' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = 'reqflow-docs.md';
		a.click();
		URL.revokeObjectURL(url);
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 top-[28px] z-50 flex items-center justify-center" onkeydown={(e) => { if (e.key === 'Escape') close(); }}>
	<!-- Backdrop -->
	<div class="absolute inset-0 bg-black/60 backdrop-blur-sm" onclick={close}></div>

	<!-- Modal -->
	<div class="relative w-[90vw] max-w-[1200px] h-[85vh] bg-surface border border-border rounded-lg shadow-2xl flex flex-col overflow-hidden">
		<!-- Header -->
		<div class="flex items-center gap-3 px-4 py-3 border-b border-border bg-surface shrink-0">
			<BookOpen size={18} class="text-muted-foreground" />
			<h2 class="text-sm font-semibold text-foreground flex-1">Documentation</h2>
			<div class="relative">
				<Search size={12} class="absolute left-2 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none" />
				<input
					type="text"
					placeholder="Search blocks..."
					class="skeu-input text-[11px] w-64"
					style="padding-left: 24px;"
					bind:value={searchQuery}
				/>
			</div>
			<button
				class="p-1 rounded hover:bg-accent/30 text-muted-foreground"
				onclick={downloadMarkdown}
				title="Export as Markdown"
			>
				<Download size={16} />
			</button>
			<button class="p-1 rounded hover:bg-accent/30 text-muted-foreground" onclick={close}>
				<X size={16} />
			</button>
		</div>

		<!-- Body -->
		<div class="flex flex-1 overflow-hidden">
			<!-- Sidebar -->
			<div class="w-56 border-r border-border overflow-y-auto shrink-0 bg-surface/50">
				<!-- Guide sections -->
				<div class="py-1">
					<div class="px-3 py-1 flex items-center gap-1.5">
						<BookOpen size={8} class="text-muted-foreground" />
						<span class="text-[9px] uppercase tracking-wider text-muted-foreground font-semibold">Guide</span>
					</div>
					{#each GUIDE_SECTIONS as guide}
						{@const IconComponent = GUIDE_ICONS[guide.icon]}
						<button
							class="w-full text-left px-3 py-1 text-[11px] hover:bg-accent/30 transition-colors flex items-center gap-2 {selectedGuide === guide.id ? 'bg-accent/40 text-foreground font-medium' : 'text-muted-foreground'}"
							onclick={() => selectGuideSection(guide.id)}
						>
							{#if IconComponent}
								<IconComponent size={11} class="shrink-0" />
							{/if}
							{guide.title}
						</button>
					{/each}
				</div>

				<div class="mx-3 border-t border-border"></div>

				<!-- Block Reference -->
				{#each [...groupedDocs().entries()] as [category, docs]}
					<div class="py-1">
						<div class="px-3 py-1 flex items-center gap-1.5">
							<div class="w-2 h-2 rounded-full" style="background: {CATEGORY_COLORS[category] || '#858585'}"></div>
							<span class="text-[9px] uppercase tracking-wider text-muted-foreground font-semibold">{category}</span>
						</div>
						{#each docs as doc}
							<button
								class="w-full text-left px-3 py-1 text-[11px] hover:bg-accent/30 transition-colors flex items-center gap-2 {selectedType === doc.type ? 'bg-accent/40 text-foreground font-medium' : 'text-muted-foreground'}"
								onclick={() => selectBlock(doc.type)}
							>
								<div class="w-1.5 h-1.5 rounded-full shrink-0" style="background: {getBlockColor(doc.type)}"></div>
								{doc.name}
							</button>
						{/each}
					</div>
				{/each}
			</div>

			<!-- Content area -->
			<div class="flex-1 overflow-y-auto px-6 py-4">
				{#if activeGuide()}
					<!-- Guide content -->
					{@const guide = activeGuide()!}
					<div class="max-w-3xl guide-content">
						<h2 class="text-lg font-semibold text-foreground mb-4">{guide.title}</h2>
						{@html guide.content}
					</div>
				{:else if selectedDoc()}
					<!-- Block detail -->
					{@const doc = selectedDoc()!}
					<div class="space-y-3 max-w-3xl">
						<!-- Block name + category -->
						<div>
							<div class="flex items-center gap-3 mb-1">
								<div class="w-3 h-3 rounded" style="background: {getBlockColor(doc.type)}"></div>
								<h3 class="text-lg font-semibold text-foreground">{doc.name}</h3>
								<span class="text-[9px] uppercase tracking-wider px-2 py-0.5 rounded-full border border-border text-muted-foreground">{doc.category}</span>
							</div>
							<p class="text-[12px] text-muted-foreground leading-relaxed mt-2">{doc.description}</p>
						</div>

						<!-- Parameters -->
						{#if doc.parameters.length > 0}
							<details open class="group">
								<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
									<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
									Parameters
								</summary>
								<div class="mt-2">
									<div class="border border-border rounded overflow-hidden">
										<table class="w-full text-[11px]">
											<thead>
												<tr class="bg-accent/10 border-b border-border">
													<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Name</th>
													<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Type</th>
													<th class="text-left px-3 py-1.5 text-muted-foreground font-medium">Description</th>
													<th class="text-left px-3 py-1.5 text-muted-foreground font-medium w-20">Default</th>
												</tr>
											</thead>
											<tbody>
												{#each doc.parameters as param, i}
													<tr class="{i % 2 === 0 ? 'bg-transparent' : 'bg-accent/5'} border-b border-border/30">
														<td class="px-3 py-1.5 font-mono text-foreground">
															{param.name}
															{#if param.required}<span class="text-red-400 ml-0.5">*</span>{/if}
														</td>
														<td class="px-3 py-1.5 text-muted-foreground">{param.type}</td>
														<td class="px-3 py-1.5 text-foreground/80">{param.description}</td>
														<td class="px-3 py-1.5 font-mono text-muted-foreground text-[10px]">{param.default || '—'}</td>
													</tr>
												{/each}
											</tbody>
										</table>
									</div>
								</div>
							</details>
						{/if}

						<!-- Example -->
						<details open class="group">
							<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
								<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
								Example
							</summary>
							<div class="mt-2">
								<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 overflow-x-auto whitespace-pre-wrap">{doc.codeExample}</pre>
							</div>
						</details>

						<!-- Rust Implementation -->
						{#if doc.rustCode}
							<details class="group">
								<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
									<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
									Rust Implementation
								</summary>
								<div class="mt-2">
									<pre class="bg-[#1e1e1e] border border-border rounded p-3 text-[11px] font-mono text-foreground/90 overflow-x-auto whitespace-pre-wrap">{doc.rustCode}</pre>
								</div>
							</details>
						{/if}

						<!-- Tips -->
						{#if doc.tips.length > 0}
							<details class="group">
								<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
									<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
									Tips
								</summary>
								<div class="mt-2">
									<ul class="space-y-1">
										{#each doc.tips as tip}
											<li class="text-[11px] text-foreground/80 flex items-start gap-2">
												<span class="text-muted-foreground mt-0.5 shrink-0">•</span>
												{tip}
											</li>
										{/each}
									</ul>
								</div>
							</details>
						{/if}

						<!-- Related blocks -->
						{#if doc.relatedBlocks.length > 0}
							<details class="group">
								<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
									<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
									Related Blocks
								</summary>
								<div class="mt-2">
									<div class="flex flex-wrap gap-1.5">
										{#each doc.relatedBlocks as related}
											<button
												class="text-[10px] px-2 py-0.5 rounded border border-border hover:bg-accent/30 text-foreground/70 hover:text-foreground transition-colors"
												onclick={() => selectBlock(related)}
											>
												{BLOCK_DOCS_FULL.find(d => d.type === related)?.name || related}
											</button>
										{/each}
									</div>
								</div>
							</details>
						{/if}
					</div>
				{:else}
					<div class="flex flex-col items-center justify-center h-full text-muted-foreground">
						<BookOpen size={48} class="mb-3 opacity-30" />
						<p class="text-sm">Select a guide or block from the sidebar</p>
						<p class="text-[11px] mt-1 opacity-60">Or use the search bar to find a specific block</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>

<style>
	.guide-content :global(h3) {
		color: var(--foreground);
	}
	.guide-content :global(p) {
		color: var(--muted-foreground);
	}
	.guide-content :global(li) {
		color: var(--foreground);
		opacity: 0.85;
	}
	.guide-content :global(code) {
		background: var(--accent);
		padding: 1px 4px;
		border-radius: 3px;
		font-size: 10px;
		font-family: var(--font-mono, monospace);
	}
	.guide-content :global(pre) {
		color: var(--foreground);
		opacity: 0.9;
	}
	.guide-content :global(pre code) {
		background: none;
		padding: 0;
	}
	.guide-content :global(table) {
		border: 1px solid var(--border);
		border-radius: 6px;
		overflow: hidden;
	}
	.guide-content :global(td) {
		color: var(--foreground);
		opacity: 0.85;
	}
	.guide-content :global(kbd) {
		background: var(--accent);
		padding: 1px 6px;
		border-radius: 3px;
		border: 1px solid var(--border);
		font-size: 10px;
		font-family: var(--font-mono, monospace);
	}
	.guide-content :global(ol) {
		list-style-type: decimal;
	}
	.guide-content :global(ul) {
		list-style-type: disc;
	}
</style>
