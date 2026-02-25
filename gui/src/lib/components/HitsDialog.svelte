<script lang="ts">
	import { app } from '$lib/state.svelte';
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

	type HitRecord = { data_line: string; captures: Record<string, string>; proxy: string | null; received_at: string };
	type SortKey = 'time_asc' | 'time_desc' | 'data_asc' | 'data_desc' | 'captures_desc';
	type FilterTab = 'ALL' | 'HIT' | 'BAN' | 'FAIL';

	let open = $derived(app.showHitsDialog);
	function onOpenChange(v: boolean) { if (!v) app.showHitsDialog = false; }

	let search = $state('');
	let sortKey = $state<SortKey>('time_desc');
	let activeTab = $state<FilterTab>('ALL');

	const SORT_LABELS: Record<SortKey, string> = {
		time_desc: 'Newest first',
		time_asc: 'Oldest first',
		data_asc: 'Data A→Z',
		data_desc: 'Data Z→A',
		captures_desc: 'Most captures',
	};

	const TAB_LABELS: FilterTab[] = ['ALL', 'HIT', 'BAN', 'FAIL'];

	// ALL hits are HITs (runner only emits hits for successes; BAN/FAIL are counters only)
	// Future: extend when runner emits individual fail/ban records
	function tabFilter(h: HitRecord): boolean {
		if (activeTab === 'ALL' || activeTab === 'HIT') return true;
		return false; // BAN/FAIL not yet tracked per-record
	}

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
		(app.hits as HitRecord[])
			.filter(h => tabFilter(h) && matchesSearch(h))
			.sort((a, b) => sortFn(a, b))
	);

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
		console.log('[HitsDialog] exported', displayed.length, 'hits as', name);
	}

	function copyAll() {
		const text = displayed.map(h => h.data_line).join('\n');
		navigator.clipboard.writeText(text);
		console.log('[HitsDialog] copied', displayed.length, 'data lines to clipboard');
	}

	function copyRow(h: HitRecord) {
		const caps = Object.entries(h.captures).map(([k, v]) => `${k}=${v}`).join(' | ');
		navigator.clipboard.writeText(caps ? `${h.data_line} | ${caps}` : h.data_line);
	}

	function deleteRow(h: HitRecord) {
		const before = app.hits.length;
		app.hits = app.hits.filter(r => r !== h);
		console.log('[HitsDialog] deleteRow, remaining:', app.hits.length, '(was', before, ')');
	}

	function clearAll() {
		console.log('[HitsDialog] clearAll:', app.hits.length, 'hits');
		app.hits = [];
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content class="max-w-[900px] w-[90vw] p-0 gap-0 overflow-hidden flex flex-col" style="max-height: 80vh;" showCloseButton={false}>
		<!-- Header -->
		<div class="flex items-center gap-3 px-4 py-2.5 border-b border-border bg-surface shrink-0">
			<CheckCircle size={14} class="text-primary" />
			<Dialog.Title class="text-sm font-semibold text-foreground">Hits Database</Dialog.Title>
			<span class="text-[10px] text-muted-foreground bg-primary/10 text-primary px-1.5 py-0.5 rounded">
				{app.hits.length} total
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

			<button class="skeu-btn text-[10px] text-red flex items-center gap-1" onclick={clearAll} title="Clear all hits">
				<Trash2 size={10} />Clear All
			</button>
			<button class="p-1 rounded hover:bg-accent/20 text-muted-foreground" onclick={() => app.showHitsDialog = false}>
				<X size={14} />
			</button>
		</div>

		<!-- Filter tabs + search + sort row -->
		<div class="flex items-center gap-2 px-3 py-1.5 border-b border-border bg-surface/50 shrink-0">
			<!-- Status tabs -->
			<div class="flex rounded border border-border overflow-hidden shrink-0">
				{#each TAB_LABELS as tab}
					<button
						class="px-3 py-1 text-[10px] font-medium transition-colors border-r border-border last:border-r-0
							{activeTab === tab ? 'bg-primary text-white' : 'bg-surface text-muted-foreground hover:bg-accent/30'}
							{(tab === 'BAN' || tab === 'FAIL') ? 'opacity-50 cursor-default' : ''}"
						onclick={() => { if (tab !== 'BAN' && tab !== 'FAIL') activeTab = tab; }}
						title={tab === 'BAN' || tab === 'FAIL' ? 'Not yet tracked per-record (shown in runner stats counters)' : undefined}
					>{tab}</button>
				{/each}
			</div>

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
					{#if app.hits.length === 0}
						No hits yet — start the runner or a job to populate results.
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
				{displayed.length} of {app.hits.length} hits
				{#if search} · filtered{/if}
				{#if activeTab !== 'ALL'} · {activeTab}{/if}
			</span>
			<div class="flex gap-1">
				<button class="skeu-btn text-[9px]" onclick={exportTxt}>Export TXT</button>
				<button class="skeu-btn text-[9px]" onclick={exportCsv}>Export CSV</button>
			</div>
		</div>
	</Dialog.Content>
</Dialog.Root>
