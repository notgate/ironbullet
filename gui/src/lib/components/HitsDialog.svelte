<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import * as Dialog from '$lib/components/ui/dialog';
	import { DropdownMenu } from 'bits-ui';
	import X from '@lucide/svelte/icons/x';
	import Search from '@lucide/svelte/icons/search';
	import Download from '@lucide/svelte/icons/download';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Copy from '@lucide/svelte/icons/copy';
	import ArrowUpDown from '@lucide/svelte/icons/arrow-up-down';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import CheckCircle from '@lucide/svelte/icons/check-circle';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';

	type HitRecord = { data_line: string; captures: Record<string, string>; proxy: string | null; received_at: string };
	type SortKey = 'time_asc' | 'time_desc' | 'data_asc' | 'data_desc' | 'captures_desc';

	let open = $derived(app.showHitsDialog);
	function onOpenChange(v: boolean) { if (!v) app.showHitsDialog = false; }

	let search = $state('');
	let sortKey = $state<SortKey>('time_desc');

	// ── Per-job filter ────────────────────────────────────────────────────
	/** '' = All Jobs; otherwise a job UUID */
	let selectedJobId = $state('');

	// When dialog opens: reset search/sort; apply pre-selected job if one was signalled
	$effect(() => {
		if (app.showHitsDialog) {
			search = '';
			sortKey = 'time_desc';
			if (app.hitsDbJobId) {
				selectedJobId = app.hitsDbJobId;
				app.hitsDbJobId = null; // consume the signal
			} else {
				selectedJobId = '';
			}
		}
	});

	// If selected job is deleted while dialog is open, fall back to All Jobs
	$effect(() => {
		if (selectedJobId && !app.jobs.find((j: any) => j.id === selectedJobId)) {
			selectedJobId = '';
		}
	});

	/** The raw (unfiltered) hit list for the current job selection */
	const baseHits = $derived<HitRecord[]>(
		selectedJobId
			? ((app.jobHitsDb[selectedJobId] ?? []) as HitRecord[])
			: (app.hits as HitRecord[])
	);

	const totalForJob = $derived(baseHits.length);

	const SORT_LABELS: Record<SortKey, string> = {
		time_desc: 'Newest first',
		time_asc: 'Oldest first',
		data_asc: 'Data A→Z',
		data_desc: 'Data Z→A',
		captures_desc: 'Most captures',
	};

	function matchesSearch(h: HitRecord): boolean {
		if (!search.trim()) return true;
		const q = search.toLowerCase();
		return h.data_line.toLowerCase().includes(q) ||
			(h.proxy ?? '').toLowerCase().includes(q) ||
			Object.entries(h.captures).some(([k, v]) =>
				k.toLowerCase().includes(q) || v.toLowerCase().includes(q)
			);
	}

	function sortFn(a: HitRecord, b: HitRecord): number {
		switch (sortKey) {
			case 'time_asc': return a.received_at.localeCompare(b.received_at);
			case 'time_desc': return b.received_at.localeCompare(a.received_at);
			case 'data_asc': return a.data_line.localeCompare(b.data_line);
			case 'data_desc': return b.data_line.localeCompare(a.data_line);
			case 'captures_desc': return Object.keys(b.captures).length - Object.keys(a.captures).length;
		}
	}

	let displayed = $derived(
		baseHits.filter(h => matchesSearch(h)).sort((a, b) => sortFn(a, b))
	);

	function refreshJobHits() {
		if (selectedJobId) {
			send('get_job_hits', { id: selectedJobId });
		}
	}

	function exportTxt() {
		const lines = displayed.map(h => {
			const caps = Object.entries(h.captures).map(([k, v]) => `${k}=${v}`).join(' | ');
			return caps ? `${h.data_line} | ${caps}` : h.data_line;
		});
		download('hits.txt', lines.join('\n'), 'text/plain');
	}

	function exportCsv() {
		const rows = [['Data', 'Captures', 'Proxy', 'Received']];
		for (const h of displayed) {
			rows.push([
				csvEsc(h.data_line),
				csvEsc(Object.entries(h.captures).map(([k, v]) => `${k}=${v}`).join('; ')),
				csvEsc(h.proxy ?? ''),
				csvEsc(h.received_at),
			]);
		}
		download('hits.csv', rows.map(r => r.join(',')).join('\n'), 'text/csv');
	}

	function csvEsc(s: string): string {
		if (s.includes(',') || s.includes('"') || s.includes('\n')) return `"${s.replace(/"/g, '""')}"`;
		return s;
	}

	function download(name: string, content: string, mime: string) {
		const url = URL.createObjectURL(new Blob([content], { type: mime }));
		const a = document.createElement('a');
		a.href = url; a.download = name; a.click();
		URL.revokeObjectURL(url);
	}

	function copyAll() {
		navigator.clipboard.writeText(displayed.map(h => h.data_line).join('\n'));
	}

	function copyRow(h: HitRecord) {
		const caps = Object.entries(h.captures).map(([k, v]) => `${k}=${v}`).join(' | ');
		navigator.clipboard.writeText(caps ? `${h.data_line} | ${caps}` : h.data_line);
	}

	function deleteRow(h: HitRecord) {
		if (selectedJobId) {
			const db = app.jobHitsDb;
			if (db[selectedJobId]) {
				app.jobHitsDb = { ...db, [selectedJobId]: db[selectedJobId].filter(r => r !== h) };
			}
		} else {
			app.hits = app.hits.filter(r => r !== h);
		}
	}

	function clearAll() {
		if (selectedJobId) {
			app.jobHitsDb = { ...app.jobHitsDb, [selectedJobId]: [] };
		} else {
			app.hits = [];
		}
	}

	/** Name of the selected job for display / export filename */
	const selectedJobName = $derived(
		selectedJobId
			? (app.jobs.find((j: any) => j.id === selectedJobId) as any)?.name ?? 'job'
			: 'all-jobs'
	);
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content class="max-w-[900px] w-[90vw] p-0 gap-0 overflow-hidden flex flex-col" style="max-height: 80vh;" showCloseButton={false}>
		<!-- Header -->
		<div class="flex items-center gap-3 px-4 py-2.5 border-b border-border bg-surface shrink-0">
			<CheckCircle size={14} class="text-primary" />
			<Dialog.Title class="text-sm font-semibold text-foreground">Hits Database</Dialog.Title>
			<span class="text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded">
				{totalForJob} hits
			</span>
			<div class="flex-1"></div>

			<!-- Export dropdown -->
			<DropdownMenu.Root>
				<DropdownMenu.Trigger class="skeu-btn flex items-center gap-1 text-[10px]">
					<Download size={10} />Export<ChevronDown size={8} />
				</DropdownMenu.Trigger>
				<DropdownMenu.Portal>
					<DropdownMenu.Content class="menu-content" sideOffset={4}>
						<DropdownMenu.Item class="menu-item" onSelect={exportTxt}>Export as .txt</DropdownMenu.Item>
						<DropdownMenu.Item class="menu-item" onSelect={exportCsv}>Export as .csv</DropdownMenu.Item>
						<DropdownMenu.Separator class="menu-sep" />
						<DropdownMenu.Item class="menu-item" onSelect={copyAll}>Copy all data lines</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Portal>
			</DropdownMenu.Root>

			<button class="skeu-btn text-[10px] text-red flex items-center gap-1" onclick={clearAll} title={selectedJobId ? 'Clear hits for this job' : 'Clear all hits'}>
				<Trash2 size={10} />Clear
			</button>
			<button class="p-1 rounded hover:bg-accent/20 text-muted-foreground" onclick={() => app.showHitsDialog = false}>
				<X size={14} />
			</button>
		</div>

		<!-- Job filter + search + sort row -->
		<div class="flex items-center gap-2 px-3 py-1.5 border-b border-border bg-surface/50 shrink-0">
			<!-- Per-job dropdown -->
			<DropdownMenu.Root>
				<DropdownMenu.Trigger class="skeu-btn flex items-center gap-1 text-[10px] shrink-0 max-w-[180px]">
					<span class="truncate">
						{selectedJobId
							? (app.jobs.find((j: any) => j.id === selectedJobId) as any)?.name ?? 'Unknown'
							: 'All Jobs'}
					</span>
					<ChevronDown size={8} class="shrink-0" />
				</DropdownMenu.Trigger>
				<DropdownMenu.Portal>
					<DropdownMenu.Content class="menu-content" sideOffset={4} align="start">
						<DropdownMenu.Item
							class="menu-item {selectedJobId === '' ? 'text-primary font-medium' : ''}"
							onSelect={() => { selectedJobId = ''; }}
						>All Jobs ({app.hits.length})</DropdownMenu.Item>
						{#if app.jobs.length > 0}
							<DropdownMenu.Separator class="menu-sep" />
							{#each app.jobs as job}
								<DropdownMenu.Item
									class="menu-item {selectedJobId === (job as any).id ? 'text-primary font-medium' : ''}"
									onSelect={() => {
										selectedJobId = (job as any).id;
										send('get_job_hits', { id: (job as any).id });
									}}
								>
									{(job as any).name}
									<span class="ml-auto text-[9px] text-muted-foreground pl-2">
										{app.jobHitsDb[(job as any).id]?.length ?? 0} hits · {(job as any).state}
									</span>
								</DropdownMenu.Item>
							{/each}
						{:else}
							<DropdownMenu.Item class="menu-item opacity-50 cursor-default">No jobs</DropdownMenu.Item>
						{/if}
					</DropdownMenu.Content>
				</DropdownMenu.Portal>
			</DropdownMenu.Root>

			{#if selectedJobId}
				<button class="skeu-btn text-[9px] flex items-center gap-0.5 shrink-0" onclick={refreshJobHits} title="Refresh from backend">
					<RefreshCw size={9} />
				</button>
			{/if}

			<!-- Search -->
			<div class="relative flex-1">
				<Search size={10} class="absolute left-2 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none" />
				<input
					type="text"
					bind:value={search}
					placeholder="Search data, captures, proxy..."
					class="skeu-input w-full text-[10px] pl-6"
				/>
			</div>

			<!-- Sort dropdown -->
			<DropdownMenu.Root>
				<DropdownMenu.Trigger class="skeu-btn flex items-center gap-1 text-[10px] shrink-0">
					<ArrowUpDown size={10} />{SORT_LABELS[sortKey]}<ChevronDown size={8} />
				</DropdownMenu.Trigger>
				<DropdownMenu.Portal>
					<DropdownMenu.Content class="menu-content" sideOffset={4} align="end">
						{#each Object.entries(SORT_LABELS) as [key, label]}
							<DropdownMenu.Item
								class="menu-item {sortKey === key ? 'text-primary font-medium' : ''}"
								onSelect={() => sortKey = key as SortKey}
							>{label}</DropdownMenu.Item>
						{/each}
					</DropdownMenu.Content>
				</DropdownMenu.Portal>
			</DropdownMenu.Root>

			<span class="text-[9px] text-muted-foreground shrink-0">{displayed.length} shown</span>
		</div>

		<!-- DataGrid -->
		<div class="flex-1 overflow-auto">
			{#if displayed.length === 0}
				<div class="flex flex-col items-center justify-center h-32 text-muted-foreground text-xs gap-2">
					<CheckCircle size={20} class="opacity-30" />
					{#if totalForJob === 0}
						{#if selectedJobId}
							No hits for this job yet.
						{:else}
							No hits yet — start a job or the runner to populate results.
						{/if}
					{:else}
						No hits match the current filter.
					{/if}
				</div>
			{:else}
				<table class="w-full text-[10px] border-collapse">
					<thead class="sticky top-0 bg-surface border-b border-border z-10">
						<tr class="text-[9px] uppercase tracking-wider text-muted-foreground">
							<th class="text-left px-3 py-1.5 font-medium w-8">#</th>
							<th class="text-left px-3 py-1.5 font-medium">Data</th>
							<th class="text-left px-3 py-1.5 font-medium">Captures</th>
							<th class="text-left px-3 py-1.5 font-medium w-32">Proxy</th>
							<th class="text-left px-3 py-1.5 font-medium w-20">Time</th>
							<th class="text-center px-3 py-1.5 font-medium w-16">Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each displayed as hit, i}
							<tr class="border-b border-border/30 hover:bg-accent/10 transition-colors group">
								<td class="px-3 py-1.5 text-muted-foreground/50 font-mono">{i + 1}</td>
								<td class="px-3 py-1.5 font-mono text-foreground max-w-[220px]">
									<div class="truncate" title={hit.data_line}>{hit.data_line}</div>
								</td>
								<td class="px-3 py-1.5">
									{#if Object.keys(hit.captures).length > 0}
										<div class="flex flex-wrap gap-0.5">
											{#each Object.entries(hit.captures) as [k, v]}
												<span class="text-[8px] bg-primary/10 text-primary px-1 py-0.5 rounded font-mono whitespace-nowrap">
													{k}: {v.length > 40 ? v.slice(0, 40) + '…' : v}
												</span>
											{/each}
										</div>
									{:else}
										<span class="text-muted-foreground/40">—</span>
									{/if}
								</td>
								<td class="px-3 py-1.5 text-muted-foreground font-mono text-[9px] max-w-[128px]">
									<div class="truncate" title={hit.proxy ?? ''}>{hit.proxy ?? '—'}</div>
								</td>
								<td class="px-3 py-1.5 text-muted-foreground font-mono text-[9px]">
									{new Date(hit.received_at).toLocaleTimeString()}
								</td>
								<td class="px-3 py-1.5 text-center">
									<div class="flex items-center justify-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
										<button
											class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-foreground"
											title="Copy row"
											onclick={() => copyRow(hit)}
										><Copy size={10} /></button>
										<button
											class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-red"
											title="Delete"
											onclick={() => deleteRow(hit)}
										><Trash2 size={10} /></button>
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</div>

		<!-- Footer -->
		<div class="px-3 py-1.5 border-t border-border bg-surface/50 shrink-0 flex items-center justify-between">
			<span class="text-[9px] text-muted-foreground">
				{displayed.length} of {totalForJob} hits
				{#if search} · filtered{/if}
				{#if selectedJobId} · {(app.jobs.find((j: any) => j.id === selectedJobId) as any)?.name ?? 'job'}{/if}
			</span>
			<div class="flex gap-1">
				<button class="skeu-btn text-[9px]" onclick={exportTxt}>Export TXT</button>
				<button class="skeu-btn text-[9px]" onclick={exportCsv}>Export CSV</button>
			</div>
		</div>
	</Dialog.Content>
</Dialog.Root>
