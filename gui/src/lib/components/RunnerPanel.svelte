<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import { fmt, formatDuration } from '$lib/utils';
	import { onDestroy } from 'svelte';
	import Play from '@lucide/svelte/icons/play';
	import Pause from '@lucide/svelte/icons/pause';
	import Square from '@lucide/svelte/icons/square';

	// Poll stats while running
	let statsInterval: ReturnType<typeof setInterval> | null = null;
	$effect(() => {
		if (app.isRunning) {
			if (!statsInterval) {
				statsInterval = setInterval(() => send('get_runner_stats'), 500);
			}
		} else {
			if (statsInterval) {
				clearInterval(statsInterval);
				statsInterval = null;
			}
		}
	});
	onDestroy(() => {
		if (statsInterval) clearInterval(statsInterval);
	});

	function start() {
		send('start_runner', { threads: app.threadCount, wordlist_path: app.wordlistPath, proxy_path: app.proxyPath });
		app.isRunning = true;
	}

	function pause() {
		send('pause_runner');
		app.isPaused = true;
	}

	function resume() {
		send('resume_runner');
		app.isPaused = false;
	}

	function stop() {
		send('stop_runner');
		app.isRunning = false;
		app.isPaused = false;
	}

	let stats = $derived(app.runnerStats);
	let progressPct = $derived(stats && stats.total > 0 ? (stats.processed / stats.total * 100) : 0);
</script>

<div class="flex flex-col h-full bg-surface">
	<!-- Controls -->
	<div class="flex items-center gap-2 px-2 py-1.5 panel-raised">
		<div class="flex items-center gap-1">
			{#if !app.isRunning}
				<button
					class="skeu-btn flex items-center gap-1 text-green text-xs"
					onclick={start}
				><Play size={11} />Start</button>
			{:else if app.isPaused}
				<button
					class="skeu-btn flex items-center gap-1 text-primary text-xs"
					onclick={resume}
				><Play size={11} />Resume</button>
			{:else}
				<button
					class="skeu-btn flex items-center gap-1 text-orange text-xs"
					onclick={pause}
				><Pause size={11} />Pause</button>
			{/if}
			{#if app.isRunning}
				<button
					class="skeu-btn flex items-center gap-1 text-red text-xs"
					onclick={stop}
				><Square size={11} />Stop</button>
			{/if}
		</div>

		<div class="flex items-center gap-1">
			<span class="text-[10px] text-muted-foreground">Threads:</span>
			<input
				type="number"
				min="1"
				max="1000"
				bind:value={app.threadCount}
				disabled={app.isRunning}
				class="w-16 skeu-input text-xs text-center disabled:opacity-50"
			/>
		</div>

		{#if stats}
			<div class="flex-1 flex items-center gap-4 text-[11px]">
				<span class="text-muted-foreground">CPM: <span class="text-foreground font-medium">{Math.round(stats.cpm)}</span></span>
				<span class="text-green">Hits: {fmt(stats.hits)}</span>
				<span class="text-muted-foreground">Fails: {fmt(stats.fails)}</span>
				<span class="text-red">Bans: {fmt(stats.bans)}</span>
				<span class="text-orange">Retries: {fmt(stats.retries)}</span>
				<span class="text-muted-foreground">{formatDuration(stats.elapsed_secs)}</span>
			</div>
		{/if}
	</div>

	<!-- Progress bar -->
	{#if stats}
		<div class="h-1 bg-background">
			<div class="h-full bg-primary transition-all" style="width: {progressPct}%"></div>
		</div>
	{/if}

	<!-- Hits table -->
	<div class="flex-1 overflow-auto panel-inset">
		{#if app.hits.length === 0}
			<div class="flex items-center justify-center h-full text-muted-foreground text-xs">
				{#if app.isRunning}
					Running... waiting for hits
				{:else}
					No hits yet. Start a run to see results.
				{/if}
			</div>
		{:else}
			<table class="w-full text-xs">
				<thead class="sticky top-0 bg-surface">
					<tr class="border-b border-border text-muted-foreground text-left">
						<th class="px-2 py-1 font-medium">#</th>
						<th class="px-2 py-1 font-medium">Data</th>
						<th class="px-2 py-1 font-medium">Captures</th>
						<th class="px-2 py-1 font-medium">Proxy</th>
					</tr>
				</thead>
				<tbody>
					{#each app.hits as hit, i}
						<tr class="border-b border-border/50 hover:bg-accent/20">
							<td class="px-2 py-1 text-muted-foreground">{i + 1}</td>
							<td class="px-2 py-1 font-mono">{hit.data_line}</td>
							<td class="px-2 py-1 font-mono text-green">
								{Object.entries(hit.captures).map(([k, v]) => `${k}=${v}`).join(' | ')}
							</td>
							<td class="px-2 py-1 text-muted-foreground">{hit.proxy || '-'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>
</div>
