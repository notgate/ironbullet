<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import { getBlockColor } from '$lib/types';
	import type { BlockResult } from '$lib/types';
	import Play from '@lucide/svelte/icons/play';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import CircleX from '@lucide/svelte/icons/circle-x';
	import ExternalLink from '@lucide/svelte/icons/external-link';

	let testDataLine = $state('user@example.com:pass123');
	let testProxy = $state('');

	function runDebug() {
		send('debug_pipeline', {
			test_data_line: testDataLine,
			test_proxy: testProxy || null,
		});
	}

	let results = $derived(app.debugResults);
	let hasResults = $derived(results.length > 0);

	function openResponseViewer(index: number) {
		const r = results[index];
		if (r?.response) {
			app.showResponseViewer = true;
		}
	}

	function truncate(s: string, max: number): string {
		return s.length > max ? s.slice(0, max) + '...' : s;
	}
</script>

<div class="flex flex-col h-full bg-surface">
	<!-- Debug toolbar -->
	<div class="flex items-center gap-2 px-2 py-1 panel-raised flex-wrap">
		<button
			class="skeu-btn flex items-center gap-1 text-green text-[11px] shrink-0"
			onclick={runDebug}
		><Play size={11} />Debug Run</button>

		<div class="flex items-center gap-1 flex-1 min-w-0">
			<label class="text-[9px] uppercase tracking-wider text-muted-foreground shrink-0">Data:</label>
			<input
				type="text"
				bind:value={testDataLine}
				placeholder="user:pass"
				class="skeu-input text-[10px] font-mono flex-1 min-w-[120px] py-0"
			/>
		</div>

		<div class="flex items-center gap-1 min-w-0">
			<label class="text-[9px] uppercase tracking-wider text-muted-foreground shrink-0">Proxy:</label>
			<input
				type="text"
				bind:value={testProxy}
				placeholder="http://ip:port (optional)"
				class="skeu-input text-[10px] font-mono w-[180px] py-0"
			/>
		</div>
	</div>

	{#if hasResults}
		<!-- Block timeline table -->
		<div class="flex-1 overflow-auto panel-inset">
			<table class="w-full text-[11px]">
				<thead class="sticky top-0 bg-surface z-10">
					<tr class="border-b border-border text-[9px] uppercase tracking-wider text-muted-foreground">
						<th class="text-left px-2 py-1 w-7">#</th>
						<th class="text-left px-2 py-1 w-[140px]">Block</th>
						<th class="text-left px-2 py-1">Result</th>
						<th class="text-right px-2 py-1 w-16">Time</th>
						<th class="text-center px-2 py-1 w-8"></th>
					</tr>
				</thead>
				<tbody>
					{#each results as r, i}
						<tr
							class="border-b border-border/50 hover:bg-secondary/30 cursor-pointer transition-colors"
							class:opacity-60={!r.success}
							onclick={() => openResponseViewer(i)}
						>
							<td class="px-2 py-0.5 text-muted-foreground font-mono">{i + 1}</td>
							<td class="px-2 py-0.5">
								<div class="flex items-center gap-1.5">
									<span
										class="w-2 h-2 rounded-full shrink-0"
										style="background-color: {getBlockColor(r.block_type)}"
									></span>
									<span class="text-foreground truncate">{r.block_label}</span>
								</div>
							</td>
							<td class="px-2 py-0.5 font-mono text-muted-foreground truncate max-w-0">
								{truncate(r.log_message || '', 80)}
							</td>
							<td class="px-2 py-0.5 text-right font-mono text-muted-foreground tabular-nums">
								{r.timing_ms}ms
							</td>
							<td class="px-2 py-0.5 text-center">
								<div class="flex items-center justify-center gap-1">
									{#if r.success}
										<CircleCheck size={12} class="text-green" />
									{:else}
										<CircleX size={12} class="text-red" />
									{/if}
									{#if r.response}
										<ExternalLink size={10} class="text-primary/50" />
									{/if}
								</div>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<div class="flex items-center justify-center flex-1 text-muted-foreground text-xs panel-inset">
			Enter test credentials and click "Debug Run" to execute the pipeline once
		</div>
	{/if}
</div>
