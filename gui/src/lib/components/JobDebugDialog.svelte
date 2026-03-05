<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { send, onResponse } from '$lib/ipc';
	import { app } from '$lib/state.svelte';
	import X from '@lucide/svelte/icons/x';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Clock from '@lucide/svelte/icons/clock';
	import AlertCircle from '@lucide/svelte/icons/alert-circle';
	import CheckCircle2 from '@lucide/svelte/icons/check-circle-2';
	import XCircle from '@lucide/svelte/icons/x-circle';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import Ban from '@lucide/svelte/icons/ban';
	import Minus from '@lucide/svelte/icons/minus';


	type BlockResult = {
		block_id: string;
		block_label: string;
		block_type: string;
		success: boolean;
		timing_ms: number;
		variables_after: Record<string, string>;
		log_message: string;
		request?: { method: string; url: string; headers: [string, string][]; body: string };
		response?: { status_code: number; headers: Record<string, string>; body: string; final_url: string; cookies: Record<string, string>; timing_ms: number };
	};

	type ResultEntry = {
		data_line: string;
		status: string;
		proxy?: string;
		captures: Record<string, string>;
		error?: string;
		ts_ms: number;
		block_results: BlockResult[];
	};

	let results = $state<ResultEntry[]>([]);
	let selectedIndex = $state<number | null>(null);
	let filter = $state('ALL');
	let pollInterval: ReturnType<typeof setInterval> | null = null;
	let expandedBlocks = $state<Set<string>>(new Set());
	let blockTab = $state<Record<string, 'request' | 'response' | 'vars'>>({});
	let listEl = $state<HTMLElement | null>(null);
	let autoScroll = $state(true);

	const FILTERS = ['ALL', 'SUCCESS', 'FAIL', 'BAN', 'ERROR', 'RETRY'];

	const statusColor: Record<string, string> = {
		SUCCESS: 'text-green',
		FAIL:    'text-red-400',
		BAN:     'text-orange-400',
		ERROR:   'text-yellow-400',
		RETRY:   'text-blue-400',
		NONE:    'text-muted-foreground',
	};

	const statusBg: Record<string, string> = {
		SUCCESS: 'bg-green/15 text-green',
		FAIL:    'bg-red-400/15 text-red-400',
		BAN:     'bg-orange-400/15 text-orange-400',
		ERROR:   'bg-yellow-400/15 text-yellow-400',
		RETRY:   'bg-blue-400/15 text-blue-400',
		NONE:    'bg-muted/30 text-muted-foreground',
	};

	function statusIcon(s: string) {
		switch (s) {
			case 'SUCCESS': return CheckCircle2;
			case 'FAIL':    return XCircle;
			case 'BAN':     return Ban;
			case 'ERROR':   return AlertCircle;
			case 'RETRY':   return RefreshCw;
			default:        return Minus;
		}
	}

	function relativeTime(ts_ms: number): string {
		const diff = Math.floor((Date.now() - ts_ms) / 1000);
		if (diff < 2) return 'just now';
		if (diff < 60) return `${diff}s ago`;
		if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
		return `${Math.floor(diff / 3600)}h ago`;
	}

	function fetchDebugLog() {
		if (!app.debugJobId) return;
		send('get_job_debug_log', { id: app.debugJobId, filter });
	}

	function close() {
		app.showJobDebugDialog = false;
		app.debugJobId = null;
		if (pollInterval) { clearInterval(pollInterval); pollInterval = null; }
	}

	function toggleBlock(key: string) {
		const next = new Set(expandedBlocks);
		if (next.has(key)) next.delete(key); else next.add(key);
		expandedBlocks = next;
	}

	const unsubscribe = onResponse('job_debug_log', (data: any) => {
		if (data.id !== app.debugJobId) return;
		const incoming: ResultEntry[] = data.results ?? [];
		results = incoming;
		if (autoScroll && listEl) {
			setTimeout(() => { if (listEl) listEl.scrollTop = listEl.scrollHeight; }, 0);
		}
		// Keep selection valid
		if (selectedIndex !== null && selectedIndex >= results.length) selectedIndex = null;
	});

	onMount(() => {
		fetchDebugLog();
		pollInterval = setInterval(fetchDebugLog, 2000);
	});

	onDestroy(() => {
		unsubscribe?.();
		if (pollInterval) clearInterval(pollInterval);
	});

	$effect(() => {
		// Re-fetch when filter changes
		filter;
		fetchDebugLog();
	});

	let filteredResults = $derived(
		filter === 'ALL' ? results : results.filter(r => r.status === filter)
	);

	let selected = $derived(selectedIndex !== null ? filteredResults[selectedIndex] : null);

	function jobName(): string {
		const job = app.jobs.find((j: any) => j.id === app.debugJobId);
		return (job as any)?.name ?? 'Job';
	}
</script>

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-4"
	onclick={(e) => { if (e.target === e.currentTarget) close(); }}>

	<div class="bg-surface border border-border rounded-lg flex flex-col w-full max-w-6xl h-[85vh] shadow-2xl overflow-hidden">

		<!-- Header -->
		<div class="flex items-center gap-3 px-4 py-2.5 border-b border-border bg-surface/80 shrink-0">
			<span class="text-xs font-semibold text-foreground">Debug Log — {jobName()}</span>
			<!-- Filter pills -->
			<div class="flex items-center gap-1 ml-2">
				{#each FILTERS as f}
					<button
						class="px-2 py-0.5 rounded text-[10px] font-medium transition-colors
							{filter === f ? 'bg-primary text-primary-foreground' : 'bg-muted/30 text-muted-foreground hover:bg-muted/60'}"
						onclick={() => { filter = f; }}>
						{f}
					</button>
				{/each}
			</div>
			<div class="ml-auto flex items-center gap-2">
				<!-- Auto-scroll toggle -->
				<label class="flex items-center gap-1 text-[10px] text-muted-foreground cursor-pointer select-none">
					<input type="checkbox" bind:checked={autoScroll} class="w-3 h-3" />
					Auto-scroll
				</label>
				<span class="text-[10px] text-muted-foreground">{filteredResults.length} entries</span>
				<button class="p-1 rounded hover:bg-accent/30 text-muted-foreground" onclick={close} title="Close">
					<X size={14} />
				</button>
			</div>
		</div>

		<!-- Body -->
		<div class="flex flex-1 overflow-hidden">

			<!-- Left: entry list -->
			<div class="w-[34%] border-r border-border flex flex-col overflow-hidden">
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="flex-1 overflow-y-auto" bind:this={listEl}
					onscroll={(e) => {
						const el = e.currentTarget as HTMLElement;
						autoScroll = el.scrollTop + el.clientHeight >= el.scrollHeight - 20;
					}}>
					{#if filteredResults.length === 0}
						<div class="flex items-center justify-center h-full text-[10px] text-muted-foreground">
							No entries yet…
						</div>
					{:else}
						{#each filteredResults as entry, i}
							{@const Icon = statusIcon(entry.status)}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="px-3 py-2 border-b border-border/40 cursor-pointer transition-colors
									{selectedIndex === i ? 'bg-primary/10 border-l-2 border-l-primary' : 'hover:bg-accent/10'}"
								onclick={() => { selectedIndex = i; }}>
								<div class="flex items-center gap-1.5 min-w-0">
									<Icon size={10} class="{statusColor[entry.status] ?? 'text-muted-foreground'} shrink-0" />
									<span class="font-mono text-[10px] truncate flex-1 text-foreground" title={entry.data_line}>
										{entry.data_line}
									</span>
									<span class="text-[9px] text-muted-foreground shrink-0">{relativeTime(entry.ts_ms)}</span>
								</div>
								{#if entry.error}
									<div class="text-[9px] text-red-400 truncate mt-0.5 pl-4">{entry.error}</div>
								{/if}
								{#if entry.block_results?.length}
									<div class="text-[9px] text-muted-foreground/60 pl-4 mt-0.5">
										{entry.block_results.length} blocks · {entry.block_results.reduce((s, b) => s + b.timing_ms, 0)}ms
									</div>
								{/if}
							</div>
						{/each}
					{/if}
				</div>
			</div>

			<!-- Right: detail -->
			<div class="flex-1 overflow-y-auto">
				{#if selected}
					<div class="p-4 space-y-4">

						<!-- Summary card -->
						<div class="bg-muted/20 border border-border rounded p-3 space-y-2">
							<div class="flex items-center gap-2">
								<span class="px-2 py-0.5 rounded text-[10px] font-semibold {statusBg[selected.status] ?? 'bg-muted text-muted-foreground'}">
									{selected.status}
								</span>
								<span class="font-mono text-xs text-foreground">{selected.data_line}</span>
							</div>
							{#if selected.proxy}
								<div class="text-[10px] text-muted-foreground">Proxy: <span class="font-mono">{selected.proxy}</span></div>
							{/if}
							{#if selected.error}
								<div class="text-[10px] text-red-400 font-mono bg-red-400/5 border border-red-400/20 rounded p-2 whitespace-pre-wrap break-all">
									{selected.error}
								</div>
							{/if}
							{#if selected.captures && Object.keys(selected.captures).length > 0}
								<div class="space-y-0.5">
									<div class="text-[10px] text-muted-foreground font-medium mb-1">Captures</div>
									{#each Object.entries(selected.captures) as [k, v]}
										<div class="flex gap-2 text-[10px]">
											<span class="text-primary font-mono">{k}</span>
											<span class="text-muted-foreground">=</span>
											<span class="font-mono text-foreground break-all">{v}</span>
										</div>
									{/each}
								</div>
							{/if}
						</div>

						<!-- Block trace -->
						{#if selected.block_results?.length > 0}
							<div>
								<div class="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider mb-2">
									Block Trace ({selected.block_results.length} blocks)
								</div>
								<div class="space-y-1">
									{#each selected.block_results as block, bi}
										{@const blockKey = `${selected.ts_ms}-${bi}`}
										{@const isExpanded = expandedBlocks.has(blockKey)}
										{@const tab = blockTab[blockKey] ?? 'request'}
										<div class="border border-border rounded overflow-hidden">
											<!-- Block header -->
											<!-- svelte-ignore a11y_no_static_element_interactions -->
											<div
												class="flex items-center gap-2 px-3 py-2 cursor-pointer hover:bg-accent/10 transition-colors
													{block.success ? 'bg-green/5' : 'bg-red-400/5'}"
												onclick={() => toggleBlock(blockKey)}>
												{#if isExpanded}
													<ChevronDown size={10} class="text-muted-foreground shrink-0" />
												{:else}
													<ChevronRight size={10} class="text-muted-foreground shrink-0" />
												{/if}
												{#if block.success}
													<CheckCircle2 size={10} class="text-green shrink-0" />
												{:else}
													<XCircle size={10} class="text-red-400 shrink-0" />
												{/if}
												<span class="text-[10px] font-medium text-foreground flex-1 truncate">
													{block.block_label || block.block_type}
												</span>
												<span class="text-[9px] text-muted-foreground bg-muted/40 px-1.5 py-0.5 rounded shrink-0">
													{block.block_type}
												</span>
												<div class="flex items-center gap-1 text-[9px] text-muted-foreground shrink-0">
													<Clock size={9} />
													{block.timing_ms}ms
												</div>
											</div>

											{#if isExpanded}
												<div class="border-t border-border bg-surface p-3 space-y-3">

													<!-- Log message -->
													{#if block.log_message}
														<div class="text-[10px] font-mono text-muted-foreground bg-muted/20 rounded px-2 py-1.5 whitespace-pre-wrap break-all">
															{block.log_message}
														</div>
													{/if}

													<!-- Tabs: Request / Response / Variables -->
													{#if block.request || block.response || Object.keys(block.variables_after ?? {}).length > 0}
														<div class="flex gap-1 border-b border-border pb-1">
															{#if block.request}
																<button
																	class="text-[10px] px-2 py-0.5 rounded transition-colors
																		{tab === 'request' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent/20'}"
																	onclick={() => { blockTab = { ...blockTab, [blockKey]: 'request' }; }}>
																	Request
																</button>
															{/if}
															{#if block.response}
																<button
																	class="text-[10px] px-2 py-0.5 rounded transition-colors
																		{tab === 'response' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent/20'}"
																	onclick={() => { blockTab = { ...blockTab, [blockKey]: 'response' }; }}>
																	Response
																	<span class="ml-1 {block.response.status_code < 400 ? 'text-green' : 'text-red-400'}">
																		{block.response.status_code}
																	</span>
																</button>
															{/if}
															{#if Object.keys(block.variables_after ?? {}).length > 0}
																<button
																	class="text-[10px] px-2 py-0.5 rounded transition-colors
																		{tab === 'vars' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent/20'}"
																	onclick={() => { blockTab = { ...blockTab, [blockKey]: 'vars' }; }}>
																	Variables ({Object.keys(block.variables_after).length})
																</button>
															{/if}
														</div>

														<!-- Request tab -->
														{#if tab === 'request' && block.request}
															<div class="space-y-2">
																<div class="flex gap-2 items-center">
																	<span class="text-[10px] font-semibold text-primary">{block.request.method}</span>
																	<span class="font-mono text-[10px] text-foreground break-all">{block.request.url}</span>
																</div>
																{#if block.request.headers?.length > 0}
																	<div>
																		<div class="text-[9px] text-muted-foreground uppercase mb-1">Headers</div>
																		<div class="bg-muted/20 rounded p-2 space-y-0.5">
																			{#each block.request.headers as [k, v]}
																				<div class="flex gap-2 text-[10px] font-mono">
																					<span class="text-primary shrink-0">{k}:</span>
																					<span class="text-foreground break-all">{v}</span>
																				</div>
																			{/each}
																		</div>
																	</div>
																{/if}
																{#if block.request.body}
																	<div>
																		<div class="text-[9px] text-muted-foreground uppercase mb-1">Body</div>
																		<pre class="bg-muted/20 rounded p-2 text-[10px] font-mono text-foreground overflow-x-auto whitespace-pre-wrap break-all max-h-48 overflow-y-auto">{block.request.body}</pre>
																	</div>
																{/if}
															</div>
														{/if}

														<!-- Response tab -->
														{#if tab === 'response' && block.response}
															<div class="space-y-2">
																<div class="flex gap-2 items-center">
																	<span class="px-1.5 py-0.5 rounded text-[10px] font-semibold
																		{block.response.status_code < 300 ? 'bg-green/15 text-green' :
																		 block.response.status_code < 400 ? 'bg-blue-400/15 text-blue-400' :
																		 'bg-red-400/15 text-red-400'}">
																		{block.response.status_code}
																	</span>
																	{#if block.response.final_url}
																		<span class="font-mono text-[9px] text-muted-foreground truncate">{block.response.final_url}</span>
																	{/if}
																	<span class="text-[9px] text-muted-foreground ml-auto shrink-0">{block.response.timing_ms}ms</span>
																</div>
																{#if Object.keys(block.response.headers ?? {}).length > 0}
																	<div>
																		<div class="text-[9px] text-muted-foreground uppercase mb-1">Headers</div>
																		<div class="bg-muted/20 rounded p-2 space-y-0.5 max-h-32 overflow-y-auto">
																			{#each Object.entries(block.response.headers) as [k, v]}
																				<div class="flex gap-2 text-[10px] font-mono">
																					<span class="text-primary shrink-0">{k}:</span>
																					<span class="text-foreground break-all">{v}</span>
																				</div>
																			{/each}
																		</div>
																	</div>
																{/if}
																{#if Object.keys(block.response.cookies ?? {}).length > 0}
																	<div>
																		<div class="text-[9px] text-muted-foreground uppercase mb-1">Cookies</div>
																		<div class="bg-muted/20 rounded p-2 space-y-0.5">
																			{#each Object.entries(block.response.cookies) as [k, v]}
																				<div class="flex gap-2 text-[10px] font-mono">
																					<span class="text-orange-400 shrink-0">{k}:</span>
																					<span class="text-foreground break-all">{v}</span>
																				</div>
																			{/each}
																		</div>
																	</div>
																{/if}
																{#if block.response.body}
																	<div>
																		<div class="text-[9px] text-muted-foreground uppercase mb-1">Body</div>
																		<pre class="bg-muted/20 rounded p-2 text-[10px] font-mono text-foreground overflow-x-auto whitespace-pre-wrap break-all max-h-64 overflow-y-auto">{block.response.body}</pre>
																	</div>
																{/if}
															</div>
														{/if}

														<!-- Variables tab -->
														{#if tab === 'vars' && Object.keys(block.variables_after ?? {}).length > 0}
															<div class="bg-muted/20 rounded p-2 space-y-0.5 max-h-64 overflow-y-auto">
																{#each Object.entries(block.variables_after) as [k, v]}
																	<div class="flex gap-2 text-[10px] font-mono">
																		<span class="text-primary shrink-0 truncate max-w-[30%]">{k}</span>
																		<span class="text-muted-foreground">=</span>
																		<span class="text-foreground break-all">{v}</span>
																	</div>
																{/each}
															</div>
														{/if}
													{/if}
												</div>
											{/if}
										</div>
									{/each}
								</div>
							</div>
						{/if}
					</div>
				{:else}
					<div class="flex items-center justify-center h-full text-[10px] text-muted-foreground">
						Select an entry on the left to see details
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>
