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
	import Folder from '@lucide/svelte/icons/folder';
	import FileText from '@lucide/svelte/icons/file-text';
	import Pencil from '@lucide/svelte/icons/pencil';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import * as Dialog from '$lib/components/ui/dialog';
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
			heading: 'Overview',
			content: `Jobs are pipeline execution instances that process data at scale.

• Each job runs your \`pipeline\` against a \`data source\` using a configurable thread pool
• Statistics update in real-time: CPM, hits, bans, fails, processed count, elapsed time
• Two job types: \`Config Job\` (pipeline runner) and \`Proxy Check\` (latency/alive tester)
• Jobs are independent — multiple can run simultaneously
• Click any row to select it and view its hits in the \`Data\` tab`
		},
		{
			heading: 'Job Types',
			content: `\`Config Job\` — runs your pipeline against a wordlist
• Standard mode: each data line is fed through the full block pipeline
• Captures, hits, bans, and fails are tracked per run
• Requires a pipeline to be loaded in the editor

\`Proxy Check\` — tests a list of proxies against a URL
• Each proxy in the list is tested with a HEAD request
• Alive proxies appear as Hits with \`status=alive\` and \`latency_ms\` captures
• Dead proxies are silently dropped (not tracked individually)
• No pipeline required — select the type before creating`
		},
		{
			heading: 'Creating a Job',
			content: `1. Click \`New Job\` to open the creation form
2. Use the segmented selector to choose \`Config Job\` or \`Proxy Check\`
3. Set a \`Name\` and \`Threads\` count

For \`Config Job\`:
• Select \`Data Source Type\`: \`File\`, \`Folder\`, or \`Inline\`
• \`File\` — single .txt/.csv wordlist, one entry per line
• \`Folder\` — all .txt, .csv, .lst, .dat files processed alphabetically
• \`Inline\` — paste entries directly, one per line
• Use the \`File\` and \`Folder\` browse buttons to pick paths

For \`Proxy Check\`:
• Browse or paste a path to a proxy list file
• Set the \`Ping URL\` (default: \`http://www.google.com\`)
• One proxy per line: \`host:port\` or \`http://host:port\`

4. Click \`Create Job\` — it appears with state \`Queued\``
		},
		{
			heading: 'Editing Jobs',
			content: `Click the \`✏ pencil\` icon on any stopped or paused job to edit it.

Editable fields:
• \`Name\` — rename the job
• \`Threads\` — change thread count (takes effect on next start)
• \`Wordlist / Data Source\` — change the input file path
• \`Proxy List\` and \`Ping URL\` (for Proxy Check jobs)

Note: Jobs must not be in \`Running\` state to edit. Stop or pause first.`
		},
		{
			heading: 'Job Controls',
			content: `Each row has action buttons on the right:

\`▶ Play\` — start a Queued or Waiting job
\`⏸ Pause\` — temporarily halt a Running job (resumes from same position)
\`▶ Resume\` — continue a Paused job
\`■ Stop\` — permanently stop a Running or Paused job
\`✏ Edit\` — open the edit dialog (not available while Running)
\`🗑 Delete\` — remove the job and all its data

Stats columns: \`CPM\` (checks per minute) • \`Hits\` • \`Processed/Total\` • \`Elapsed Time\``
		},
		{
			heading: 'Hits Database',
			content: `• All hits are stored in the \`Data\` tab and persisted in memory
• Click a job row or the \`📊 database icon\` to view hits for that job
• Export hits via the \`Data\` panel buttons: \`TXT\` or \`CSV\`
• For advanced filtering and sorting, click \`Advanced\` or use the \`Hits\` menu in the toolbar
• Proxy Check hits include captures: \`status=alive\` and \`latency_ms=NNN\``
		},
		{
			heading: 'Thread Tuning',
			content: `Start conservative and increase gradually:

• Begin with \`10–50 threads\` to measure baseline server behavior
• Watch \`CPM\` — if it scales linearly with threads, increase further
• If CPM plateaus or drops as threads increase → rate limiting detected
• High \`ban rate\`: reduce threads, check proxy quality
• High \`fail rate\`: verify pipeline logic in \`Debug\` panel (F5)
• High \`retry rate\`: check proxy quality and network stability`
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
	// New job form state
	let newJobType = $state<'Config' | 'ProxyCheck'>('Config');
	let newJobName = $state('');
	let newJobThreads = $state(100);
	let newJobDataSource = $state('');
	let newJobDataType = $state<'File' | 'Folder' | 'Inline'>('File');
	let newJobStartCondition = $state<'Immediate' | 'Delayed'>('Immediate');
	let newJobDelaySecs = $state(0);
	let proxyCheckUrl = $state('http://www.google.com');
	let proxyCheckList = $state('');

	// Edit job dialog state
	let editingJob = $state<any>(null);
	let showEditDialog = $state(false);
	let editName = $state('');
	let editThreads = $state(100);
	let editDataSource = $state('');
	let editProxyCheckUrl = $state('http://www.google.com');
	let editProxyCheckList = $state('');

	function openEditDialog(job: any) {
		editingJob = job;
		editName = job.name ?? '';
		editThreads = job.thread_count ?? 100;
		editDataSource = job.data_source?.value ?? '';
		editProxyCheckUrl = job.proxy_check_url ?? 'http://www.google.com';
		editProxyCheckList = job.proxy_check_list ?? '';
		showEditDialog = true;
		console.log('[JobMonitor] openEditDialog:', job.id, job.name);
	}

	function saveJobEdit() {
		if (!editingJob) return;
		send('update_job', {
			id: editingJob.id,
			name: editName,
			thread_count: editThreads,
			data_source: { source_type: editingJob.data_source?.source_type ?? 'File', value: editDataSource },
			proxy_check_url: editProxyCheckUrl,
			proxy_check_list: editProxyCheckList,
		});
		console.log('[JobMonitor] saveJobEdit:', editingJob.id);
		showEditDialog = false;
		editingJob = null;
	}

	// When a file/folder browse completes, auto-fill the new job form
	$effect(() => {
		const pick = app.pendingJobWordlist;
		if (pick && showNewJob) {
			if (newJobType === 'ProxyCheck') {
				proxyCheckList = pick.path;
			} else {
				newJobDataSource = pick.path;
				newJobDataType = pick.isFolder ? 'Folder' : 'File';
			}
			console.log('[JobMonitor] pendingJobWordlist applied:', pick.path, 'type:', newJobType);
			app.pendingJobWordlist = null;
		}
	});

	function createJob() {
		console.log('[JobMonitor] createJob: jobType=', newJobType, 'dataType=', newJobDataType, 'threads=', newJobThreads);
		send('create_job', {
			name: newJobName || (newJobType === 'ProxyCheck' ? 'Proxy Check' : 'New Job'),
			pipeline: JSON.parse(JSON.stringify(app.pipeline)),
			thread_count: newJobThreads,
			job_type: newJobType,
			proxy_check_url: newJobType === 'ProxyCheck' ? proxyCheckUrl : undefined,
			proxy_check_list: newJobType === 'ProxyCheck' ? proxyCheckList : undefined,
			data_source: newJobType === 'Config' ? {
				source_type: newJobDataType,
				value: newJobDataSource,
			} : { source_type: 'File', value: proxyCheckList },
			start_condition: newJobStartCondition === 'Immediate'
				? 'Immediate'
				: { Delayed: { delay_secs: newJobDelaySecs } },
		});
		showNewJob = false;
		newJobName = '';
		newJobDataSource = '';
		proxyCheckList = '';
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
		<div class="px-3 py-2.5 bg-background border-b border-border space-y-2">
			<!-- Job type dropdown -->
			<div class="flex items-center gap-2">
				<label class="text-muted-foreground text-[10px] shrink-0">Job Type</label>
				<SkeuSelect
					value={newJobType}
					onValueChange={(v) => { newJobType = v as any; }}
					options={[
						{ value: 'Config', label: 'Config Job — Run pipeline against wordlist' },
						{ value: 'ProxyCheck', label: 'Proxy Check — Test proxy list liveness' },
					]}
					class="text-xs flex-1"
				/>
			</div>

			<div class="grid grid-cols-2 gap-2 text-xs">
				<div>
					<label class="text-muted-foreground text-[10px]">Name</label>
					<input type="text" bind:value={newJobName} placeholder={newJobType === 'ProxyCheck' ? 'Proxy Check' : 'Job name'} class="skeu-input w-full text-xs" />
				</div>
				<div>
					<label class="text-muted-foreground text-[10px]">Threads</label>
					<input type="number" min="1" max="1000" bind:value={newJobThreads} class="skeu-input w-full text-xs" />
				</div>

				{#if newJobType === 'Config'}
					<div>
						<label class="text-muted-foreground text-[10px]">Data Source Type</label>
						<SkeuSelect
							value={newJobDataType}
							onValueChange={(v) => { newJobDataType = v as any; }}
							options={[{value:'File',label:'File'},{value:'Folder',label:'Folder (all .txt/.csv)'},{value:'Inline',label:'Inline'}]}
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
						<label class="text-muted-foreground text-[10px]">
							{newJobDataType === 'File' ? 'Wordlist File' : newJobDataType === 'Folder' ? 'Wordlist Folder' : 'Data (one per line)'}
						</label>
						{#if newJobDataType === 'Inline'}
							<textarea bind:value={newJobDataSource} rows={3} class="skeu-input w-full text-xs font-mono" placeholder="line1&#10;line2"></textarea>
						{:else}
							<div class="flex gap-1">
								<input
									type="text"
									bind:value={newJobDataSource}
									placeholder={newJobDataType === 'Folder' ? 'Path to folder...' : 'Path to wordlist file...'}
									class="skeu-input flex-1 text-xs font-mono"
								/>
								<button class="skeu-btn text-[10px] flex items-center gap-1 shrink-0" onclick={() => send('browse_file', { field: 'job_wordlist' })} title="Browse file"><FileText size={10} />File</button>
								<button class="skeu-btn text-[10px] flex items-center gap-1 shrink-0" onclick={() => send('browse_folder', { field: 'job_folder' })} title="Browse folder"><Folder size={10} />Folder</button>
							</div>
							{#if newJobDataType === 'Folder'}
								<p class="text-[9px] text-muted-foreground mt-0.5">All .txt, .csv, .lst, .dat files processed alphabetically.</p>
							{/if}
						{/if}
					</div>
					{#if newJobStartCondition === 'Delayed'}
						<div>
							<label class="text-muted-foreground text-[10px]">Delay (seconds)</label>
							<input type="number" min="0" bind:value={newJobDelaySecs} class="skeu-input w-full text-xs" />
						</div>
					{/if}
				{:else}
					<!-- Proxy Check fields -->
					<div class="col-span-2">
						<label class="text-muted-foreground text-[10px]">Proxy List File</label>
						<div class="flex gap-1">
							<input type="text" bind:value={proxyCheckList} placeholder="Path to proxy list file..." class="skeu-input flex-1 text-xs font-mono" />
							<button class="skeu-btn text-[10px] flex items-center gap-1 shrink-0" onclick={() => send('browse_file', { field: 'job_wordlist' })} title="Browse"><FileText size={10} />Browse</button>
						</div>
						<p class="text-[9px] text-muted-foreground mt-0.5">One proxy per line: <code class="font-mono">host:port</code> or <code class="font-mono">http://host:port</code></p>
					</div>
					<div class="col-span-2">
						<label class="text-muted-foreground text-[10px]">Ping URL</label>
						<input type="text" bind:value={proxyCheckUrl} placeholder="http://www.google.com" class="skeu-input w-full text-xs font-mono" />
						<p class="text-[9px] text-muted-foreground mt-0.5">Alive proxies will appear as Hits. Dead proxies are silently dropped.</p>
					</div>
				{/if}
			</div>
			<div class="flex items-center gap-2 mt-1">
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
						<th class="px-2 py-1 font-medium w-28">Progress</th>
						<th class="px-2 py-1 font-medium text-right">CPM</th>
						<th class="px-2 py-1 font-medium text-right text-green">Hits</th>
						<th class="px-2 py-1 font-medium text-right text-red-400">Fails</th>
						<th class="px-2 py-1 font-medium text-right text-orange-400">Bans</th>
						<th class="px-2 py-1 font-medium text-right text-muted-foreground">Errs</th>
						<th class="px-2 py-1 font-medium text-right">Done/Total</th>
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
							<td class="px-2 py-1 text-right font-mono text-[10px]">{job.stats ? Math.round(job.stats.cpm) : 0}</td>
							<td class="px-2 py-1 text-right font-mono text-[10px] text-green font-semibold">{job.stats ? fmt(job.stats.hits) : 0}</td>
							<td class="px-2 py-1 text-right font-mono text-[10px] {(job.stats?.fails ?? 0) > 0 ? 'text-red-400' : 'text-muted-foreground/40'}">{job.stats ? fmt(job.stats.fails) : 0}</td>
							<td class="px-2 py-1 text-right font-mono text-[10px] {(job.stats?.bans ?? 0) > 0 ? 'text-orange-400' : 'text-muted-foreground/40'}">{job.stats ? fmt(job.stats.bans) : 0}</td>
							<td class="px-2 py-1 text-right font-mono text-[10px] {(job.stats?.errors ?? 0) > 0 ? 'text-yellow-400' : 'text-muted-foreground/40'}">{job.stats ? fmt(job.stats.errors) : 0}</td>
							<td class="px-2 py-1 text-right font-mono text-[10px] text-muted-foreground">{job.stats ? `${fmt(job.stats.processed)}/${fmt(job.stats.total)}` : '0/0'}</td>
							<td class="px-2 py-1 text-right font-mono text-[10px] text-muted-foreground">{job.stats ? formatDuration(job.stats.elapsed_secs) : '0:00'}</td>
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
										<button
											class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-blue transition-colors"
											title="Edit job"
											onclick={() => openEditDialog(job)}
										><Pencil size={11} /></button>
										<button class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-red transition-colors" title="Remove" onclick={() => removeJob((job as any).id)}><Trash2 size={11} /></button>
									{/if}
								</div>
							</td>
						</tr>

						<!-- Live result log — visible only for selected job when it has recent results -->
						{#if isActive && job.stats?.recent_results?.length}
							{@const recentSlice = job.stats.recent_results.slice(-20).reverse()}
							<tr class="border-b border-border/30 bg-background/60">
								<td colspan="11" class="px-2 pt-1 pb-2">
									<div class="text-[9px] text-muted-foreground mb-0.5 flex items-center gap-1">
										<span class="w-1.5 h-1.5 rounded-full bg-green animate-pulse inline-block"></span>
										Live results (last {recentSlice.length})
									</div>
									<div class="font-mono text-[10px] max-h-36 overflow-y-auto space-y-px pr-1"
										style="scrollbar-width: thin;">
										{#each recentSlice as r (r.data_line + r.ts_ms)}
											{@const statusColor =
												r.status === 'SUCCESS' ? 'text-green bg-green/10 border-green/30' :
												r.status === 'FAIL'    ? 'text-red-400 bg-red-400/10 border-red-400/30' :
												r.status === 'BAN'     ? 'text-orange-400 bg-orange-400/10 border-orange-400/30' :
												r.status === 'RETRY'   ? 'text-yellow-400 bg-yellow-400/10 border-yellow-400/30' :
												r.status === 'NONE'    ? 'text-muted-foreground/60 bg-muted/5 border-border/40' :
												                         'text-muted-foreground bg-muted/10 border-border'}
											<div class="flex items-center gap-1.5 py-0.5 hover:bg-accent/10 rounded px-1">
												<span class="shrink-0 border rounded px-1 py-px text-[9px] font-semibold {statusColor}">{r.status}</span>
												<span class="truncate text-foreground/80 flex-1">{r.data_line}</span>
												{#if r.proxy}
													<span class="shrink-0 text-muted-foreground/60 text-[9px] truncate max-w-[100px]" title={r.proxy}>via {r.proxy.replace(/https?:\/\//, '')}</span>
												{/if}
												{#if r.error}
													<span class="shrink-0 text-red-400/70 text-[9px] truncate max-w-[140px]" title={r.error}>{r.error}</span>
												{/if}
												{#if r.status === 'SUCCESS' && Object.keys(r.captures ?? {}).length}
													<span class="shrink-0 text-green/70 text-[9px]">
														{Object.entries(r.captures).map(([k, v]) => `${k}=${v}`).join(' · ')}
													</span>
												{/if}
											</div>
										{/each}
									</div>
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		{/if}
	</div>
</div>

<!-- Edit Job Dialog -->
<Dialog.Root bind:open={showEditDialog}>
	<Dialog.Content class="max-w-sm p-0 gap-0" showCloseButton={false}>
		<div class="flex items-center gap-2 px-4 py-2.5 border-b border-border">
			<Pencil size={13} class="text-primary" />
			<Dialog.Title class="text-sm font-semibold">Edit Job</Dialog.Title>
			<div class="flex-1"></div>
			<button class="p-1 rounded hover:bg-accent/20 text-muted-foreground" onclick={() => showEditDialog = false}>✕</button>
		</div>
		<div class="p-4 space-y-3">
			{#if editingJob?.state === 'Running'}
				<p class="text-xs text-amber-400 bg-amber-400/10 rounded px-2 py-1.5 border border-amber-400/20">Stop the job before making changes.</p>
			{:else}
				<div>
					<label class="text-[10px] text-muted-foreground">Job Name</label>
					<input type="text" bind:value={editName} class="skeu-input w-full text-xs mt-0.5" />
				</div>
				<div>
					<label class="text-[10px] text-muted-foreground">Threads</label>
					<input type="number" min="1" max="1000" bind:value={editThreads} class="skeu-input w-full text-xs mt-0.5" />
				</div>
				{#if editingJob?.job_type === 'ProxyCheck'}
					<div>
						<label class="text-[10px] text-muted-foreground">Proxy List File</label>
						<div class="flex gap-1 mt-0.5">
							<input type="text" bind:value={editProxyCheckList} placeholder="Path to proxy list..." class="skeu-input flex-1 text-xs font-mono" />
							<button class="skeu-btn text-[10px]" onclick={() => send('browse_file', { field: 'job_wordlist' })}>Browse</button>
						</div>
					</div>
					<div>
						<label class="text-[10px] text-muted-foreground">Ping URL</label>
						<input type="text" bind:value={editProxyCheckUrl} class="skeu-input w-full text-xs font-mono mt-0.5" />
					</div>
				{:else}
					<div>
						<label class="text-[10px] text-muted-foreground">Wordlist / Data Source</label>
						<div class="flex gap-1 mt-0.5">
							<input type="text" bind:value={editDataSource} placeholder="Path to wordlist..." class="skeu-input flex-1 text-xs font-mono" />
							<button class="skeu-btn text-[10px]" onclick={() => send('browse_file', { field: 'job_wordlist' })}>Browse</button>
						</div>
					</div>
				{/if}
				<div class="flex gap-2 pt-1">
					<button class="skeu-btn text-xs text-green flex-1" onclick={saveJobEdit}>Save Changes</button>
					<button class="skeu-btn text-xs text-muted-foreground" onclick={() => showEditDialog = false}>Cancel</button>
				</div>
			{/if}
		</div>
	</Dialog.Content>
</Dialog.Root>

<HelpModal bind:open={showHelp} title="Jobs & Runner Guide" sections={helpSections} />
