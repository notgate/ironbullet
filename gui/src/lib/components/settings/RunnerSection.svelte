<script lang="ts">
	import { app } from '$lib/state.svelte';

	let { searchQuery, shouldShowSetting }: {
		searchQuery: string;
		shouldShowSetting: (section: string, label: string) => boolean;
	} = $props();
</script>

{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Runner</div>{/if}

{#if shouldShowSetting('runner', 'Thread Count')}
	<div class="flex items-center justify-between py-1.5">
		<span class="text-[11px] text-muted-foreground">Bots (threads)</span>
		<div class="flex items-center gap-1.5">
			<input
				type="number"
				min="1"
				max="10000"
				bind:value={app.pipeline.runner_settings.threads}
				class="w-16 skeu-input text-[11px] text-center"
			/>
		</div>
	</div>
{/if}

{#if shouldShowSetting('runner', 'Skip Lines')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Skip</span>
			<p class="text-[9px] text-muted-foreground/60">Skip first N data lines</p>
		</div>
		<input
			type="number"
			min="0"
			bind:value={app.pipeline.runner_settings.skip}
			class="w-16 skeu-input text-[11px] text-center"
		/>
	</div>
{/if}

{#if shouldShowSetting('runner', 'Take Lines')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Take</span>
			<p class="text-[9px] text-muted-foreground/60">Max lines to process (0 = all)</p>
		</div>
		<input
			type="number"
			min="0"
			bind:value={app.pipeline.runner_settings.take}
			class="w-16 skeu-input text-[11px] text-center"
		/>
	</div>
{/if}

{#if shouldShowSetting('runner', 'Max Retries')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Max retries</span>
			<p class="text-[9px] text-muted-foreground/60">Per data line before error</p>
		</div>
		<input
			type="number"
			min="0"
			max="100"
			bind:value={app.pipeline.runner_settings.max_retries}
			class="w-16 skeu-input text-[11px] text-center"
		/>
	</div>
{/if}

{#if shouldShowSetting('runner', 'Custom Status')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Custom status name</span>
			<p class="text-[9px] text-muted-foreground/60">Label for the Custom bot status</p>
		</div>
		<input
			type="text"
			bind:value={app.pipeline.runner_settings.custom_status_name}
			class="w-24 skeu-input text-[11px] text-center"
			placeholder="CUSTOM"
		/>
	</div>
{/if}

{#if shouldShowSetting('runner', 'Continue Statuses')}
	<div class="py-1.5">
		<span class="text-[11px] text-muted-foreground">Continue on status</span>
		<p class="text-[9px] text-muted-foreground/60 mb-1">Re-queue data lines with these results</p>
		<div class="flex flex-wrap gap-1.5">
			{#each ['Retry', 'Ban', 'Error'] as status}
				{@const active = app.pipeline.runner_settings.continue_statuses.includes(status as any)}
				<button
					class="text-[10px] px-2 py-0.5 rounded border transition-colors {active ? 'bg-primary/20 border-primary text-foreground' : 'border-border text-muted-foreground hover:border-primary/50'}"
					onclick={() => {
						if (active) {
							app.pipeline.runner_settings.continue_statuses = app.pipeline.runner_settings.continue_statuses.filter(s => s !== status);
						} else {
							app.pipeline.runner_settings.continue_statuses = [...app.pipeline.runner_settings.continue_statuses, status as any];
						}
					}}
				>{status}</button>
			{/each}
		</div>
	</div>
{/if}

{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}

{#if shouldShowSetting('runner', 'Gradual Start')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Gradual thread start</span>
			<p class="text-[9px] text-muted-foreground/60">Start threads one-by-one instead of all at once</p>
		</div>
		<!-- svelte-ignore a11y_label_has_associated_control -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={app.pipeline.runner_settings.start_threads_gradually}
				onchange={() => { app.pipeline.runner_settings.start_threads_gradually = !app.pipeline.runner_settings.start_threads_gradually; }}
				class="skeu-checkbox"
			/>
		</label>
	</div>
	{#if app.pipeline.runner_settings.start_threads_gradually}
		<div class="flex items-center justify-between py-1 pl-4">
			<span class="text-[10px] text-muted-foreground/80">Delay between threads</span>
			<div class="flex items-center gap-1">
				<input
					type="number"
					min="10"
					max="5000"
					bind:value={app.pipeline.runner_settings.gradual_delay_ms}
					class="w-16 skeu-input text-[11px] text-center"
				/>
				<span class="text-[10px] text-muted-foreground">ms</span>
			</div>
		</div>
	{/if}
{/if}

{#if shouldShowSetting('runner', 'Auto Thread Count')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Auto thread count</span>
			<p class="text-[9px] text-muted-foreground/60">Optimize thread count based on CPM</p>
		</div>
		<!-- svelte-ignore a11y_label_has_associated_control -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={app.pipeline.runner_settings.automatic_thread_count}
				onchange={() => { app.pipeline.runner_settings.automatic_thread_count = !app.pipeline.runner_settings.automatic_thread_count; }}
				class="skeu-checkbox"
			/>
		</label>
	</div>
{/if}

{#if shouldShowSetting('runner', 'Lower on Retry')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Lower threads on retry</span>
			<p class="text-[9px] text-muted-foreground/60">Reduce thread count when retry rate is high</p>
		</div>
		<!-- svelte-ignore a11y_label_has_associated_control -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={app.pipeline.runner_settings.lower_threads_on_retry}
				onchange={() => { app.pipeline.runner_settings.lower_threads_on_retry = !app.pipeline.runner_settings.lower_threads_on_retry; }}
				class="skeu-checkbox"
			/>
		</label>
	</div>
	{#if app.pipeline.runner_settings.lower_threads_on_retry}
		<div class="flex items-center justify-between py-1 pl-4">
			<span class="text-[10px] text-muted-foreground/80">Reduction percentage</span>
			<div class="flex items-center gap-1">
				<input
					type="number"
					min="5"
					max="90"
					bind:value={app.pipeline.runner_settings.retry_thread_reduction_pct}
					class="w-16 skeu-input text-[11px] text-center"
				/>
				<span class="text-[10px] text-muted-foreground">%</span>
			</div>
		</div>
	{/if}
{/if}

{#if shouldShowSetting('runner', 'Pause on Ratelimit')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Pause on ratelimit</span>
			<p class="text-[9px] text-muted-foreground/60">Pause execution when 429 / rate-limited</p>
		</div>
		<!-- svelte-ignore a11y_label_has_associated_control -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={app.pipeline.runner_settings.pause_on_ratelimit}
				onchange={() => { app.pipeline.runner_settings.pause_on_ratelimit = !app.pipeline.runner_settings.pause_on_ratelimit; }}
				class="skeu-checkbox"
			/>
		</label>
	</div>
{/if}

{#if shouldShowSetting('runner', 'Proxyless Only')}
	<div class="flex items-center justify-between py-1.5">
		<div>
			<span class="text-[11px] text-muted-foreground">Proxyless only</span>
			<p class="text-[9px] text-muted-foreground/60">Run without proxies even if configured</p>
		</div>
		<!-- svelte-ignore a11y_label_has_associated_control -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={app.pipeline.runner_settings.only_proxyless}
				onchange={() => { app.pipeline.runner_settings.only_proxyless = !app.pipeline.runner_settings.only_proxyless; }}
				class="skeu-checkbox"
			/>
		</label>
	</div>
{/if}
