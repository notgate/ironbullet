<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import { fmt, formatDuration } from '$lib/utils';
	import { onDestroy } from 'svelte';
	import type { Job } from '$lib/types';
	import SkeuSelect from './SkeuSelect.svelte';
	import Plus from '@lucide/svelte/icons/plus';
	import Play from '@lucide/svelte/icons/play';
	import Pause from '@lucide/svelte/icons/pause';
	import Square from '@lucide/svelte/icons/square';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import HelpCircle from '@lucide/svelte/icons/help-circle';
	import Database from '@lucide/svelte/icons/database';
	import HelpModal from './HelpModal.svelte';

	let showNewJob = $state(false);
	let showHelp = $state(false);

	// Poll stats for all running jobs every second
	let statsInterval: ReturnType<typeof setInterval> | null = null;
	$effect(() => {
		const hasRunning = app.jobs.some((j: any) => j.state === 'Running');
		if (hasRunning) {
			if (!statsInterval) {
				console.log('[JobMonitor] starting stats poll interval (1s)');
				statsInterval = setInterval(() => {
					for (const job of app.jobs) {
						if ((job as any).state === 'Running') {
							send('get_job_stats', { id: (job as any).id });
						}
					}
				}, 1000);
			}
		} else {
			if (statsInterval) {
				console.log('[JobMonitor] no running jobs — clearing stats poll interval');
				clearInterval(statsInterval);
				statsInterval = null;
			}
		}
	});

	onDestroy(() => {
		if (statsInterval) clearInterval(statsInterval);
	});

	function selectJob(id: string) {
		const prev = app.activeJobId;
		app.activeJobId = id;
		console.log('[JobMonitor] selectJob:', id, '(was:', prev ?? 'none', ')');
	}

	function viewJobHits(id: string) {
		app.activeJobId = id;
		send('get_job_hits', { id });
		app.bottomTab = 'data';
		console.log('[JobMonitor] viewJobHits: switching to data tab for job', id);
	}

	const helpSections = [
		{
			heading: 'What are Jobs?',
			content: `Jobs are pipeline execution instances that process data at scale. Each job:

- Runs your configured pipeline against a data source
- Uses multiple threads for parallel execution
- Tracks real-time statistics (CPM, hits, progress, errors)
- Operates independently with its own lifecycle and thread pool

Architecture: Pipeline + Data Source + Thread Pool = Automated Job Processing`
		},
		{
			heading: 'Creating a Job',
			content: `Prerequisite: Debug your pipeline first (F5 or Debug panel)

Job creation workflow:

1. Click "New Job" button

2. Configure job parameters:
   Name
     Descriptive identifier (e.g., "Gmail Check", "Site Login Test")

   Threads
     1-1000 parallel executions
     Recommendation: Start low (10-50), increase based on performance

3. Select data source:

   File
     Full path to wordlist or CSV file
     Example: C:\\data\\combo.txt
     Format: One entry per line (user:pass)

   Inline
     Paste data directly into text area
     Format: One entry per line
     Example:
       user1:pass1
       user2:pass2
       user3:pass3

4. Set start condition:

   Immediate
     Begins processing immediately after creation

   Delayed
     Waits specified seconds before auto-starting

5. Click "Create Job"

Job appears in table below with "Queued" state`
		},
		{
			heading: 'Job Controls',
			content: `Action buttons (rightmost column):

Play (Green)
  Available: Queued or Waiting states
  Action: Start processing data
  Effect: Changes state to Running

Pause (Orange)
  Available: Running state
  Action: Temporarily halt processing
  Effect: Changes state to Paused
  Note: Preserves position, can resume later

Resume (Green)
  Available: Paused state
  Action: Continue from paused position
  Effect: Changes state to Running
  Note: Maintains all statistics and progress

Stop (Red)
  Available: Running or Paused states
  Action: Permanently terminate job
  Effect: Changes state to Stopped
  Warning: Cannot resume after stopping

Delete (Gray)
  Available: Any state except Running
  Action: Remove job from list
  Warning: Permanently deletes all job data`
		},
		{
			heading: 'Job Lifecycle States',
			content: `State transitions:

Queued → Running → Completed
   ↓         ↓
Waiting   Paused → Stopped

State definitions:

Queued (Gray indicator)
  Initial state after creation
  No processing has started
  Action required: Click Play to begin

Waiting (Yellow indicator)
  Delayed start countdown active
  Will auto-transition to Running when timer expires
  Can be started manually via Play button

Running (Green indicator)
  Actively processing data entries
  Statistics updating in real-time
  Thread pool executing pipeline iterations

Paused (Orange indicator)
  Temporarily suspended
  Progress and statistics preserved
  Can be resumed or stopped

Completed (Blue indicator)
  All data processed successfully
  Final statistics available
  No further actions possible

Stopped (Red indicator)
  Manually terminated before completion
  Partial results available
  Cannot be resumed`
		},
		{
			heading: 'Statistics and Metrics',
			content: `Real-time metrics (updated every second):

Progress Bar
  Visual representation: 0% to 100%
  Calculation: (Processed / Total) * 100

Processed / Total
  Example: 1,250 / 10,000
  Interpretation: 1,250 entries completed out of 10,000 total

CPM (Checks Per Minute)
  Execution throughput metric
  Formula: (Processed / Elapsed Minutes)
  Factors affecting CPM:
    - Thread count
    - Server response time
    - Network latency
    - Block delay settings

Hits
  Count of successful results
  Based on KeyCheck block SUCCESS conditions
  Examples: Valid credentials, successful logins, matches found

Time Elapsed
  Format: MM:SS or HH:MM:SS
  Total job runtime including paused duration
  Does not reset on pause/resume

Performance optimization:

Thread tuning
  Start: 10-50 threads to baseline server behavior
  Monitor: CPM and error rate
  Adjust: Increase if CPM stable, decrease if errors spike

Rate limiting
  Symptom: CPM decreases as threads increase
  Cause: Server-side rate limiting or connection pooling
  Solution: Reduce threads, add delays between requests

Error handling
  High ban rate: Reduce threads, increase delays
  High fail rate: Verify pipeline logic in debug mode
  High retry rate: Check proxy quality and network stability`
		},
	];
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
		{#if app.activeJobId}
			{@const activeJob = app.jobs.find((j: any) => j.id === app.activeJobId)}
			<div class="flex items-center gap-1 text-[10px] bg-primary/10 border border-primary/30 rounded px-1.5 py-0.5">
				<Database size={9} class="text-primary" />
				<span class="text-primary font-medium truncate max-w-[100px]">{activeJob ? (activeJob as any).name : app.activeJobId}</span>
				<button
					class="text-primary/60 hover:text-primary ml-0.5"
					onclick={() => { console.log('[JobMonitor] cleared activeJobId'); app.activeJobId = null; }}
					title="Deselect job"
				>×</button>
			</div>
		{/if}
		<button
			class="p-1 rounded hover:bg-accent/20 text-muted-foreground hover:text-foreground transition-colors"
			onclick={() => { showHelp = true; }}
			title="Help & Documentation"
		>
			<HelpCircle size={14} />
		</button>
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
						{@const isActive = app.activeJobId === (job as any).id}
						<tr
							class="border-b border-border/50 hover:bg-accent/20 cursor-pointer transition-colors {isActive ? 'bg-primary/8 border-l-2 border-l-primary' : ''}"
							onclick={() => selectJob((job as any).id)}
						>
							<td class="px-2 py-1 font-medium">
								<div class="flex items-center gap-1">
									{#if isActive}
										<span class="w-1 h-1 rounded-full bg-primary shrink-0"></span>
									{/if}
									{job.name}
								</div>
							</td>
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
							<td class="px-2 py-1 text-center" onclick={(e) => e.stopPropagation()}>
								<div class="flex items-center justify-center gap-0.5">
									{#if job.state === 'Queued' || job.state === 'Waiting'}
										<button class="p-0.5 rounded hover:bg-secondary text-green" title="Start" onclick={() => startJob((job as any).id)}><Play size={11} /></button>
									{:else if job.state === 'Running'}
										<button class="p-0.5 rounded hover:bg-secondary text-orange" title="Pause" onclick={() => pauseJob((job as any).id)}><Pause size={11} /></button>
										<button class="p-0.5 rounded hover:bg-secondary text-red" title="Stop" onclick={() => stopJob((job as any).id)}><Square size={11} /></button>
									{:else if job.state === 'Paused'}
										<button class="p-0.5 rounded hover:bg-secondary text-green" title="Resume" onclick={() => resumeJob((job as any).id)}><Play size={11} /></button>
										<button class="p-0.5 rounded hover:bg-secondary text-red" title="Stop" onclick={() => stopJob((job as any).id)}><Square size={11} /></button>
									{/if}
									<button
										class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-primary transition-colors"
										title="View hits in Data panel"
										onclick={() => viewJobHits((job as any).id)}
									><Database size={11} /></button>
									{#if job.state !== 'Running'}
										<button class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-red transition-colors" title="Remove" onclick={() => removeJob((job as any).id)}><Trash2 size={11} /></button>
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

<HelpModal bind:open={showHelp} title="Jobs & Runner Guide" sections={helpSections} />
