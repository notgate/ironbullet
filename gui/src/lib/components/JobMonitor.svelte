<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import { fmt, formatDuration } from '$lib/utils';
	import type { Job } from '$lib/types';
	import SkeuSelect from './SkeuSelect.svelte';
	import Plus from '@lucide/svelte/icons/plus';
	import Play from '@lucide/svelte/icons/play';
	import Pause from '@lucide/svelte/icons/pause';
	import Square from '@lucide/svelte/icons/square';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';

	let showNewJob = $state(false);
	let newJobName = $state('');
	let newJobThreads = $state(100);
	let newJobDataSource = $state('');
	let newJobDataType = $state<'File' | 'Inline'>('File');
	let newJobStartCondition = $state<'Immediate' | 'Delayed'>('Immediate');
	let newJobDelaySecs = $state(0);

	function createJob() {
		send('create_job', {
			name: newJobName || 'New Job',
			pipeline: app.pipeline,
			thread_count: newJobThreads,
			data_source: {
				source_type: newJobDataType,
				value: newJobDataSource,
			},
			start_condition: newJobStartCondition === 'Immediate'
				? 'Immediate'
				: { Delayed: { delay_secs: newJobDelaySecs } },
		});
		showNewJob = false;
		newJobName = '';
		newJobDataSource = '';
	}

	function refreshJobs() {
		send('list_jobs');
	}

	function startJob(id: string) {
		send('start_job', { id });
	}

	function pauseJob(id: string) {
		send('pause_job', { id });
	}

	function resumeJob(id: string) {
		send('resume_job', { id });
	}

	function stopJob(id: string) {
		send('stop_job', { id });
	}

	function removeJob(id: string) {
		send('remove_job', { id });
	}

	function stateColor(state: string): string {
		switch (state) {
			case 'Running': return 'text-green';
			case 'Paused': return 'text-orange';
			case 'Completed': return 'text-primary';
			case 'Stopped': return 'text-red';
			case 'Queued': return 'text-muted-foreground';
			case 'Waiting': return 'text-yellow';
			default: return 'text-muted-foreground';
		}
	}

	function jobProgress(job: Job): number {
		if (!job.stats || job.stats.total === 0) return 0;
		return job.stats.processed / job.stats.total * 100;
	}
</script>

<div class="flex flex-col h-full bg-surface">
	<!-- Header -->
	<div class="flex items-center gap-2 px-2 py-1.5 panel-raised">
		<button
			class="skeu-btn flex items-center gap-1 text-xs"
			onclick={() => { showNewJob = !showNewJob; }}
		><Plus size={11} />New Job</button>
		<button
			class="skeu-btn flex items-center gap-1 text-xs text-muted-foreground"
			onclick={refreshJobs}
		><RefreshCw size={11} />Refresh</button>
		<div class="flex-1"></div>
		<span class="text-[10px] text-muted-foreground">{app.jobs.length} job{app.jobs.length !== 1 ? 's' : ''}</span>
	</div>

	<!-- New job form -->
	{#if showNewJob}
		<div class="px-3 py-2 bg-background border-b border-border">
			<div class="grid grid-cols-2 gap-2 text-xs">
				<div>
					<label class="text-muted-foreground text-[10px]">Name</label>
					<input type="text" bind:value={newJobName} placeholder="Job name" class="skeu-input w-full text-xs" />
				</div>
				<div>
					<label class="text-muted-foreground text-[10px]">Threads</label>
					<input type="number" min="1" max="1000" bind:value={newJobThreads} class="skeu-input w-full text-xs" />
				</div>
				<div>
					<label class="text-muted-foreground text-[10px]">Data Source Type</label>
					<SkeuSelect
						value={newJobDataType}
						onValueChange={(v) => { newJobDataType = v as any; }}
						options={[{value:'File',label:'File'},{value:'Inline',label:'Inline'}]}
						class="text-xs w-full"
					/>
				</div>
				<div>
					<label class="text-muted-foreground text-[10px]">Start Condition</label>
					<SkeuSelect
						value={newJobStartCondition}
						onValueChange={(v) => { newJobStartCondition = v as any; }}
						options={[{value:'Immediate',label:'Immediate'},{value:'Delayed',label:'Delayed'}]}
						class="text-xs w-full"
					/>
				</div>
				<div class="col-span-2">
					<label class="text-muted-foreground text-[10px]">{newJobDataType === 'File' ? 'File Path' : 'Data (one per line)'}</label>
					{#if newJobDataType === 'Inline'}
						<textarea bind:value={newJobDataSource} rows={3} class="skeu-input w-full text-xs font-mono" placeholder="line1&#10;line2"></textarea>
					{:else}
						<input type="text" bind:value={newJobDataSource} placeholder="C:\path\to\wordlist.txt" class="skeu-input w-full text-xs font-mono" />
					{/if}
				</div>
				{#if newJobStartCondition === 'Delayed'}
					<div>
						<label class="text-muted-foreground text-[10px]">Delay (seconds)</label>
						<input type="number" min="0" bind:value={newJobDelaySecs} class="skeu-input w-full text-xs" />
					</div>
				{/if}
			</div>
			<div class="flex items-center gap-2 mt-2">
				<button class="skeu-btn text-xs text-green" onclick={createJob}>Create Job</button>
				<button class="skeu-btn text-xs text-muted-foreground" onclick={() => { showNewJob = false; }}>Cancel</button>
			</div>
		</div>
	{/if}

	<!-- Jobs table -->
	<div class="flex-1 overflow-auto panel-inset">
		{#if app.jobs.length === 0}
			<div class="flex items-center justify-center h-full text-muted-foreground text-xs">
				No jobs. Click "New Job" to create one using the current config.
			</div>
		{:else}
			<table class="w-full text-xs">
				<thead class="sticky top-0 bg-surface">
					<tr class="border-b border-border text-muted-foreground text-left">
						<th class="px-2 py-1 font-medium">Name</th>
						<th class="px-2 py-1 font-medium">State</th>
						<th class="px-2 py-1 font-medium w-32">Progress</th>
						<th class="px-2 py-1 font-medium text-right">CPM</th>
						<th class="px-2 py-1 font-medium text-right">Hits</th>
						<th class="px-2 py-1 font-medium text-right">Processed</th>
						<th class="px-2 py-1 font-medium text-right">Time</th>
						<th class="px-2 py-1 font-medium text-center">Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each app.jobs as job}
						{@const pct = jobProgress(job)}
						<tr class="border-b border-border/50 hover:bg-accent/20">
							<td class="px-2 py-1 font-medium">{job.name}</td>
							<td class="px-2 py-1 {stateColor(job.state)}">
								<span class="inline-block w-1.5 h-1.5 rounded-full mr-1 {job.state === 'Running' ? 'bg-green' : job.state === 'Paused' ? 'bg-orange' : job.state === 'Completed' ? 'bg-primary' : 'bg-muted-foreground'}"></span>
								{job.state}
							</td>
							<td class="px-2 py-1">
								<div class="flex items-center gap-1">
									<div class="flex-1 h-1.5 bg-background rounded-sm overflow-hidden">
										<div class="h-full bg-primary transition-all" style="width: {pct}%"></div>
									</div>
									<span class="text-muted-foreground w-8 text-right">{Math.round(pct)}%</span>
								</div>
							</td>
							<td class="px-2 py-1 text-right">{job.stats ? Math.round(job.stats.cpm) : 0}</td>
							<td class="px-2 py-1 text-right text-green">{job.stats ? fmt(job.stats.hits) : 0}</td>
							<td class="px-2 py-1 text-right text-muted-foreground">{job.stats ? `${fmt(job.stats.processed)}/${fmt(job.stats.total)}` : '0/0'}</td>
							<td class="px-2 py-1 text-right text-muted-foreground">{job.stats ? formatDuration(job.stats.elapsed_secs) : '0:00'}</td>
							<td class="px-2 py-1 text-center">
								<div class="flex items-center justify-center gap-0.5">
									{#if job.state === 'Queued' || job.state === 'Waiting'}
										<button class="p-0.5 rounded hover:bg-secondary text-green" title="Start" onclick={() => startJob(job.id)}><Play size={11} /></button>
									{:else if job.state === 'Running'}
										<button class="p-0.5 rounded hover:bg-secondary text-orange" title="Pause" onclick={() => pauseJob(job.id)}><Pause size={11} /></button>
										<button class="p-0.5 rounded hover:bg-secondary text-red" title="Stop" onclick={() => stopJob(job.id)}><Square size={11} /></button>
									{:else if job.state === 'Paused'}
										<button class="p-0.5 rounded hover:bg-secondary text-green" title="Resume" onclick={() => resumeJob(job.id)}><Play size={11} /></button>
										<button class="p-0.5 rounded hover:bg-secondary text-red" title="Stop" onclick={() => stopJob(job.id)}><Square size={11} /></button>
									{/if}
									{#if job.state !== 'Running'}
										<button class="p-0.5 rounded hover:bg-secondary text-muted-foreground" title="Remove" onclick={() => removeJob(job.id)}><Trash2 size={11} /></button>
									{/if}
								</div>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>
</div>
