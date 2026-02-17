<script lang="ts">
	import { app } from '$lib/state.svelte';
	import type { NetworkEntry } from '$lib/types';
	import Globe from '@lucide/svelte/icons/globe';
	import Cookie from '@lucide/svelte/icons/cookie';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Shield from '@lucide/svelte/icons/shield';
	import Copy from '@lucide/svelte/icons/copy';

	let expandedIndex = $state<number | null>(null);
	let entries = $derived(app.networkLog);
	let hasEntries = $derived(entries.length > 0);

	// Compute max timing for waterfall bar scaling
	let maxTiming = $derived(Math.max(1, ...entries.map(e => e.timing_ms)));

	// Known security-related cookie names for highlighting
	const SECURITY_COOKIES: Record<string, string> = {
		'_abck': 'Akamai',
		'bm_sz': 'Akamai',
		'ak_bmsc': 'Akamai',
		'bm_sv': 'Akamai',
		'cf_clearance': 'Cloudflare',
		'__cf_bm': 'Cloudflare',
		'cf_chl_2': 'Cloudflare',
		'_px': 'PerimeterX',
		'_pxvid': 'PerimeterX',
		'_pxhd': 'PerimeterX',
		'datadome': 'DataDome',
		'__ddg1_': 'DataDome',
		'incap_ses_': 'Incapsula',
		'visid_incap_': 'Incapsula',
		'reese84': 'Reese84',
	};

	function getSecurityBadge(name: string): string | null {
		for (const [prefix, label] of Object.entries(SECURITY_COOKIES)) {
			if (name.startsWith(prefix)) return label;
		}
		return null;
	}

	function statusColor(code: number): string {
		if (code >= 200 && code < 300) return 'text-green';
		if (code >= 300 && code < 400) return 'text-blue';
		if (code >= 400 && code < 500) return 'text-orange';
		return 'text-red';
	}

	function formatSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / 1048576).toFixed(1)} MB`;
	}

	function truncateUrl(url: string): string {
		try {
			const u = new URL(url);
			const path = u.pathname + u.search;
			return path.length > 60 ? path.slice(0, 57) + '...' : path;
		} catch {
			return url.length > 60 ? url.slice(0, 57) + '...' : url;
		}
	}

	function toggleExpand(index: number) {
		expandedIndex = expandedIndex === index ? null : index;
	}
</script>

<div class="flex flex-col h-full bg-surface">
	{#if hasEntries}
		<div class="flex-1 overflow-auto panel-inset">
			<table class="w-full text-[11px]">
				<thead class="sticky top-0 bg-surface z-10">
					<tr class="border-b border-border text-[9px] uppercase tracking-wider text-muted-foreground">
						<th class="text-left px-1.5 py-1 w-6">#</th>
						<th class="text-left px-1.5 py-1 w-14">Method</th>
						<th class="text-left px-1.5 py-1">URL</th>
						<th class="text-center px-1.5 py-1 w-12">Status</th>
						<th class="text-left px-1.5 py-1 w-[120px]">Waterfall</th>
						<th class="text-right px-1.5 py-1 w-14">Size</th>
						<th class="text-center px-1.5 py-1 w-8"></th>
					</tr>
				</thead>
				<tbody>
					{#each entries as entry, i}
						<tr
							class="border-b border-border/50 hover:bg-secondary/30 cursor-pointer transition-colors"
							onclick={() => toggleExpand(i)}
						>
							<td class="px-1.5 py-0.5 text-muted-foreground font-mono">{i + 1}</td>
							<td class="px-1.5 py-0.5">
								<span class="text-xs font-medium text-foreground">{entry.method}</span>
							</td>
							<td class="px-1.5 py-0.5 font-mono text-primary truncate max-w-0" title={entry.url}>
								{truncateUrl(entry.url)}
							</td>
							<td class="px-1.5 py-0.5 text-center">
								<span class="font-medium {statusColor(entry.status_code)}">{entry.status_code}</span>
							</td>
							<td class="px-1.5 py-0.5">
								<div class="flex items-center gap-1">
									<div class="flex-1 h-3 bg-background rounded-sm overflow-hidden">
										<div
											class="h-full rounded-sm {entry.status_code < 400 ? 'bg-primary/60' : 'bg-red/60'}"
											style="width: {Math.max(2, (entry.timing_ms / maxTiming) * 100)}%"
										></div>
									</div>
									<span class="text-[9px] text-muted-foreground tabular-nums w-[40px] text-right shrink-0">{entry.timing_ms}ms</span>
								</div>
							</td>
							<td class="px-1.5 py-0.5 text-right font-mono text-muted-foreground">
								{formatSize(entry.response_size)}
							</td>
							<td class="px-1.5 py-0.5 text-center">
								<div class="flex items-center gap-0.5">
									{#if entry.cookies_set.length > 0}
										<Cookie size={10} class="text-purple" />
									{/if}
									{#if expandedIndex === i}
										<ChevronDown size={10} class="text-muted-foreground" />
									{:else}
										<ChevronRight size={10} class="text-muted-foreground" />
									{/if}
								</div>
							</td>
						</tr>

						<!-- Expanded detail row -->
						{#if expandedIndex === i}
							<tr>
								<td colspan="7" class="px-3 py-2 bg-background border-b border-border">
									<div class="grid grid-cols-2 gap-4 text-[10px]">
										<!-- Request info -->
										<div>
											<div class="text-[9px] uppercase tracking-wider text-muted-foreground mb-1 flex items-center gap-1">
												<Globe size={10} /> Request
											</div>
											<div class="font-mono text-foreground break-all mb-1">{entry.method} {entry.url}</div>
											<div class="text-muted-foreground">Block: {entry.block_label}</div>

											{#if entry.cookies_sent.length > 0}
												<div class="mt-1.5 text-[9px] uppercase tracking-wider text-muted-foreground mb-0.5 flex items-center gap-1">
													<Cookie size={9} /> Cookies Sent
												</div>
												{#each entry.cookies_sent as [name, value]}
													<div class="flex items-center gap-1 font-mono group">
														{#if getSecurityBadge(name)}
															<span class="text-[8px] px-1 py-px rounded bg-orange/20 text-orange shrink-0">{getSecurityBadge(name)}</span>
														{/if}
														<span class="text-purple">{name}</span>
														<button class="p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0"
															onclick={async () => { try { await navigator.clipboard.writeText(name); } catch {} }}
															title="Copy name">
															<Copy size={8} />
														</button>=<span class="text-foreground truncate">{value}</span>
													</div>
												{/each}
											{/if}
										</div>

										<!-- Response cookies -->
										<div>
											{#if entry.cookies_set.length > 0}
												<div class="text-[9px] uppercase tracking-wider text-muted-foreground mb-1 flex items-center gap-1">
													<Shield size={10} /> Cookies Set by Response
												</div>
												{#each entry.cookies_set as [name, value]}
													<div class="flex items-center gap-1 font-mono group">
														{#if getSecurityBadge(name)}
															<span class="text-[8px] px-1 py-px rounded bg-orange/20 text-orange shrink-0">{getSecurityBadge(name)}</span>
														{/if}
														<span class="text-purple">{name}</span>
														<button class="p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0"
															onclick={async () => { try { await navigator.clipboard.writeText(name); } catch {} }}
															title="Copy name">
															<Copy size={8} />
														</button>=<span class="text-foreground truncate max-w-[200px]">{value.length > 50 ? value.slice(0, 47) + '...' : value}</span>
													</div>
												{/each}
											{:else}
												<div class="text-muted-foreground">No cookies set</div>
											{/if}
										</div>
									</div>
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<div class="flex items-center justify-center flex-1 text-muted-foreground text-xs panel-inset">
			Run Debug to capture network requests
		</div>
	{/if}
</div>
