<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import SkeuSelect from './SkeuSelect.svelte';
	import type { ProxySource } from '$lib/types';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Database from '@lucide/svelte/icons/database';
	import FileText from '@lucide/svelte/icons/file-text';
	import Shield from '@lucide/svelte/icons/shield';
	import Globe from '@lucide/svelte/icons/globe';
	import Link from '@lucide/svelte/icons/link';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import ListIcon from '@lucide/svelte/icons/list';
	import CheckCircle from '@lucide/svelte/icons/check-circle';
	import Copy from '@lucide/svelte/icons/copy';
	import Filter from '@lucide/svelte/icons/filter';

	// Placeholder for hits data (will be populated by backend)
	interface HitRecord {
		data: string;
		type: string;
		config: string;
		date: string;
		wordlist: string;
		proxy: string;
		captured: Record<string, string>;
	}

	let hitsData = $state<HitRecord[]>([]);
	let autoRefreshHits = $state(true);
	let hitsFilter = $state('');

	function browseWordlist() {
		send('browse_file', { field: 'wordlist' });
	}

	function browseProxies() {
		send('browse_file', { field: 'proxies' });
	}

	// Slice management
	function updateSlices(raw: string) {
		app.pipeline.data_settings.slices = raw.split(',').map(s => s.trim()).filter(s => s.length > 0);
	}

	// Wordlist type presets (OB2-style)
	const WORDLIST_PRESETS = [
		{ value: 'Combolist', label: 'Combolist (USER:PASS)', sep: ':', slices: ['USER', 'PASS'] },
		{ value: 'CombolistEmail', label: 'Combolist (EMAIL:PASS)', sep: ':', slices: ['EMAIL', 'PASS'] },
		{ value: 'CombolistLogin', label: 'Combolist (LOGIN:PASS)', sep: ':', slices: ['LOGIN', 'PASS'] },
		{ value: 'Emails', label: 'Email list', sep: ':', slices: ['EMAIL'] },
		{ value: 'Tokens', label: 'Token list', sep: ':', slices: ['TOKEN'] },
		{ value: 'Custom', label: 'Custom', sep: ':', slices: [] },
	];

	function applyPreset(preset: typeof WORDLIST_PRESETS[0]) {
		app.pipeline.data_settings.wordlist_type = preset.value;
		app.pipeline.data_settings.separator = preset.sep;
		if (preset.slices.length > 0) {
			app.pipeline.data_settings.slices = [...preset.slices];
		}
	}

	// Proxy ban settings
	function updateBanDuration(val: string) {
		const num = parseInt(val);
		if (!isNaN(num) && num >= 0) {
			app.pipeline.proxy_settings.ban_duration_secs = num;
		}
	}

	function updateMaxRetries(val: string) {
		const num = parseInt(val);
		if (!isNaN(num) && num >= 0) {
			app.pipeline.proxy_settings.max_retries_before_ban = num;
		}
	}

	function updateConcurrentPerProxy(val: string) {
		const num = parseInt(val);
		if (!isNaN(num) && num >= 0) {
			app.pipeline.runner_settings.concurrent_per_proxy = num;
		}
	}

	// Proxy source management
	function addProxySource(type: 'File' | 'Url' | 'Inline') {
		app.pipeline.proxy_settings.proxy_sources = [
			...app.pipeline.proxy_settings.proxy_sources,
			{ source_type: type, value: '', refresh_interval_secs: 0 },
		];
	}

	function removeProxySource(index: number) {
		app.pipeline.proxy_settings.proxy_sources = app.pipeline.proxy_settings.proxy_sources.filter((_, i) => i !== index);
	}

	// Proxy group management
	let newGroupName = $state('');

	function addProxyGroup() {
		const name = newGroupName.trim();
		if (!name) return;
		if (app.pipeline.proxy_settings.proxy_groups.some(g => g.name === name)) return;
		app.pipeline.proxy_settings.proxy_groups = [
			...app.pipeline.proxy_settings.proxy_groups,
			{ name, mode: 'Rotate', sources: [], cpm_per_proxy: 0 },
		];
		newGroupName = '';
	}

	function removeProxyGroup(index: number) {
		const removed = app.pipeline.proxy_settings.proxy_groups[index];
		app.pipeline.proxy_settings.proxy_groups = app.pipeline.proxy_settings.proxy_groups.filter((_, i) => i !== index);
		if (app.pipeline.proxy_settings.active_group === removed?.name) {
			app.pipeline.proxy_settings.active_group = '';
		}
	}

	function addGroupSource(groupIdx: number, type: 'File' | 'Url' | 'Inline') {
		const groups = [...app.pipeline.proxy_settings.proxy_groups];
		groups[groupIdx] = { ...groups[groupIdx], sources: [...groups[groupIdx].sources, { source_type: type, value: '', refresh_interval_secs: 0 }] };
		app.pipeline.proxy_settings.proxy_groups = groups;
	}

	function removeGroupSource(groupIdx: number, sourceIdx: number) {
		const groups = [...app.pipeline.proxy_settings.proxy_groups];
		groups[groupIdx] = { ...groups[groupIdx], sources: groups[groupIdx].sources.filter((_, i) => i !== sourceIdx) };
		app.pipeline.proxy_settings.proxy_groups = groups;
	}
</script>

<div class="flex flex-col h-full bg-surface p-2 space-y-2.5 overflow-y-auto">
	<!-- Data Settings -->
	<div>
		<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 flex items-center gap-1.5">
			<Database size={11} />
			Data Settings
		</div>
		<div class="space-y-1.5">
			<div>
				<label class="text-[10px] text-muted-foreground">Wordlist Type</label>
				<SkeuSelect value={app.pipeline.data_settings.wordlist_type}
					onValueChange={(v) => {
						const preset = WORDLIST_PRESETS.find(p => p.value === v);
						if (preset) applyPreset(preset);
					}}
					options={WORDLIST_PRESETS.map(p => ({ value: p.value, label: p.label }))}
					class="w-full text-[11px] mt-0.5"
				/>
			</div>
			<div class="flex gap-2">
				<div class="w-16">
					<label class="text-[10px] text-muted-foreground">Separator</label>
					<input type="text" value={app.pipeline.data_settings.separator}
						class="w-full skeu-input text-[11px] text-center font-mono mt-0.5"
						oninput={(e) => { app.pipeline.data_settings.separator = (e.target as HTMLInputElement).value; }} />
				</div>
				<div class="flex-1">
					<label class="text-[10px] text-muted-foreground">Variable slices</label>
					<input type="text" value={app.pipeline.data_settings.slices.join(', ')}
						class="w-full skeu-input text-[11px] font-mono mt-0.5"
						placeholder="USER, PASS"
						oninput={(e) => updateSlices((e.target as HTMLInputElement).value)} />
					<p class="text-[9px] text-muted-foreground mt-0.5">Comma-separated. Available as <code class="text-foreground/70">input.USER</code>, <code class="text-foreground/70">input.PASS</code></p>
				</div>
			</div>
		</div>
	</div>

	<!-- Wordlist -->
	<div>
		<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 flex items-center gap-1.5">
			<FileText size={11} />
			Wordlist
		</div>
		<div class="flex gap-1">
			<input type="text" bind:value={app.wordlistPath} placeholder="Select wordlist file..."
				class="flex-1 skeu-input text-[11px] font-mono" />
			<button class="skeu-btn text-[11px]" onclick={browseWordlist}>Browse</button>
		</div>
		{#if app.pipeline.runner_settings.skip > 0 || app.pipeline.runner_settings.take > 0}
			<div class="flex gap-2 mt-1">
				{#if app.pipeline.runner_settings.skip > 0}
					<span class="text-[9px] text-muted-foreground bg-background px-1.5 py-0.5 rounded border border-border">Skip: {app.pipeline.runner_settings.skip}</span>
				{/if}
				{#if app.pipeline.runner_settings.take > 0}
					<span class="text-[9px] text-muted-foreground bg-background px-1.5 py-0.5 rounded border border-border">Take: {app.pipeline.runner_settings.take}</span>
				{/if}
			</div>
		{/if}
	</div>

	<!-- Proxy Settings -->
	<div>
		<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 flex items-center gap-1.5">
			<Shield size={11} />
			Proxy Settings
		</div>
		<div class="space-y-1.5">
			<SkeuSelect value={app.pipeline.proxy_settings.proxy_mode}
				onValueChange={(v) => { app.pipeline.proxy_settings.proxy_mode = v as any; }}
				options={[
					{value:'None',label:'No Proxies'},
					{value:'Rotate',label:'Random proxy per request'},
					{value:'Sticky',label:'1 proxy per account check'},
					{value:'CpmLimited',label:'CPM limit per proxy'},
				]}
				class="w-full text-[11px]"
			/>

			<!-- Proxy Sources -->
			{#if app.pipeline.proxy_settings.proxy_mode !== 'None'}
				<div class="space-y-1">
					{#each app.pipeline.proxy_settings.proxy_sources as source, i}
						<div class="bg-background rounded p-1.5 border border-border space-y-1">
							<div class="flex items-center gap-1">
								{#if source.source_type === 'Url'}
									<Link size={10} class="text-muted-foreground shrink-0" />
								{:else if source.source_type === 'File'}
									<FileText size={10} class="text-muted-foreground shrink-0" />
								{:else}
									<ListIcon size={10} class="text-muted-foreground shrink-0" />
								{/if}
								<span class="text-[9px] text-muted-foreground/70 w-6 shrink-0">{source.source_type === 'Url' ? 'URL' : source.source_type}</span>
								<input type="text"
									bind:value={source.value}
									placeholder={source.source_type === 'Url' ? 'https://domain.com/proxylist.txt' : source.source_type === 'File' ? '/path/to/proxies.txt' : 'ip:port per line...'}
									class="flex-1 skeu-input text-[10px] font-mono"
								/>
								{#if source.source_type === 'File'}
									<button class="skeu-btn text-[9px] px-1.5 py-0.5" onclick={browseProxies}>...</button>
								{/if}
								<button class="p-0.5 text-muted-foreground hover:text-red" onclick={() => removeProxySource(i)} title="Remove">
									<Trash2 size={10} />
								</button>
							</div>
							{#if source.source_type === 'Url'}
								<div class="flex items-center gap-1.5 pl-4">
									<RefreshCw size={9} class="text-muted-foreground/60" />
									<span class="text-[9px] text-muted-foreground/60">Refresh every</span>
									<input type="number" min="0"
										bind:value={source.refresh_interval_secs}
										class="w-14 skeu-input text-[9px] text-center"
										placeholder="0"
									/>
									<span class="text-[9px] text-muted-foreground/60">sec (0 = once)</span>
								</div>
							{/if}
						</div>
					{/each}
				</div>

				<div class="flex gap-1">
					<button class="skeu-btn text-[9px] flex items-center gap-1 flex-1" onclick={() => addProxySource('File')}>
						<FileText size={9} /> File
					</button>
					<button class="skeu-btn text-[9px] flex items-center gap-1 flex-1" onclick={() => addProxySource('Url')}>
						<Link size={9} /> URL
					</button>
					<button class="skeu-btn text-[9px] flex items-center gap-1 flex-1" onclick={() => addProxySource('Inline')}>
						<ListIcon size={9} /> Inline
					</button>
				</div>

				<!-- Legacy file picker for quick add -->
				<div class="flex gap-1">
					<input type="text" bind:value={app.proxyPath} placeholder="Quick add proxy file..."
						class="flex-1 skeu-input text-[10px] font-mono" />
					<button class="skeu-btn text-[10px]" onclick={browseProxies}>Browse</button>
				</div>

				{#if app.pipeline.proxy_settings.proxy_mode === 'CpmLimited'}
					<div class="flex items-center justify-between bg-background rounded p-2 border border-border">
						<div>
							<span class="text-[10px] text-muted-foreground">CPM per proxy</span>
							<p class="text-[9px] text-muted-foreground/60">Max checks per minute per proxy</p>
						</div>
						<input type="number" min="1"
							bind:value={app.pipeline.proxy_settings.cpm_per_proxy}
							class="w-16 skeu-input text-[10px] text-center"
							placeholder="60"
						/>
					</div>
				{/if}

				<div class="bg-background rounded p-2 border border-border space-y-1.5 mt-1">
					<div class="flex items-center justify-between">
						<div>
							<span class="text-[10px] text-muted-foreground">Ban duration</span>
							<p class="text-[9px] text-muted-foreground/60">Seconds to ban a proxy after failures</p>
						</div>
						<div class="flex items-center gap-1">
							<input type="number" value={app.pipeline.proxy_settings.ban_duration_secs}
								class="w-16 skeu-input text-[10px] text-center"
								oninput={(e) => updateBanDuration((e.target as HTMLInputElement).value)} />
							<span class="text-[9px] text-muted-foreground">s</span>
						</div>
					</div>
					<div class="flex items-center justify-between">
						<div>
							<span class="text-[10px] text-muted-foreground">Max retries before ban</span>
							<p class="text-[9px] text-muted-foreground/60">Failures before proxy is banned</p>
						</div>
						<input type="number" value={app.pipeline.proxy_settings.max_retries_before_ban}
							class="w-16 skeu-input text-[10px] text-center"
							oninput={(e) => updateMaxRetries((e.target as HTMLInputElement).value)} />
					</div>
					<div class="flex items-center justify-between">
						<div>
							<span class="text-[10px] text-muted-foreground">Concurrent per proxy</span>
							<p class="text-[9px] text-muted-foreground/60">0 = unlimited</p>
						</div>
						<input type="number" value={app.pipeline.runner_settings.concurrent_per_proxy}
							class="w-16 skeu-input text-[10px] text-center"
							oninput={(e) => updateConcurrentPerProxy((e.target as HTMLInputElement).value)} />
					</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- Proxy Groups -->
	{#if app.pipeline.proxy_settings.proxy_mode !== 'None'}
		<div>
			<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 flex items-center gap-1.5">
				<Shield size={11} />
				Proxy Groups
			</div>
			<div class="space-y-1.5">
				<!-- Active group selector -->
				{#if app.pipeline.proxy_settings.proxy_groups.length > 0}
					<SkeuSelect
						value={app.pipeline.proxy_settings.active_group || ''}
						onValueChange={(v) => { app.pipeline.proxy_settings.active_group = v; }}
						options={[
							{ value: '', label: 'Default (above sources)' },
							...app.pipeline.proxy_settings.proxy_groups.map(g => ({ value: g.name, label: g.name })),
						]}
						class="w-full text-[11px]"
					/>
				{/if}

				<!-- Group cards -->
				{#each app.pipeline.proxy_settings.proxy_groups as group, gi}
					<div class="bg-background rounded p-1.5 border border-border space-y-1">
						<div class="flex items-center gap-1">
							<span class="text-[10px] font-medium text-foreground flex-1 truncate">{group.name}</span>
							<span class="text-[9px] text-muted-foreground/60">{group.sources.length} src</span>
							<button class="p-0.5 text-muted-foreground hover:text-red" onclick={() => removeProxyGroup(gi)} title="Remove group">
								<Trash2 size={10} />
							</button>
						</div>
						<SkeuSelect
							value={group.mode}
							onValueChange={(v) => {
								const groups = [...app.pipeline.proxy_settings.proxy_groups];
								groups[gi] = { ...groups[gi], mode: v as any };
								app.pipeline.proxy_settings.proxy_groups = groups;
							}}
							options={[{value:'Rotate',label:'Rotate'},{value:'Sticky',label:'Sticky'},{value:'CpmLimited',label:'CPM Limited'}]}
							class="w-full text-[9px]"
						/>
						{#each group.sources as src, si}
							<div class="flex items-center gap-1">
								<span class="text-[9px] text-muted-foreground/60 w-5 shrink-0">{src.source_type === 'Url' ? 'URL' : src.source_type === 'File' ? 'File' : 'Inl'}</span>
								<input type="text" bind:value={src.value} placeholder="proxy source..." class="flex-1 skeu-input text-[9px] font-mono" />
								<button class="p-0.5 text-muted-foreground hover:text-red" onclick={() => removeGroupSource(gi, si)}><Trash2 size={9} /></button>
							</div>
						{/each}
						<div class="flex gap-0.5">
							<button class="skeu-btn text-[8px] flex-1" onclick={() => addGroupSource(gi, 'File')}>+ File</button>
							<button class="skeu-btn text-[8px] flex-1" onclick={() => addGroupSource(gi, 'Url')}>+ URL</button>
							<button class="skeu-btn text-[8px] flex-1" onclick={() => addGroupSource(gi, 'Inline')}>+ Inline</button>
						</div>
					</div>
				{/each}

				<!-- Add new group -->
				<div class="flex gap-1">
					<input type="text" bind:value={newGroupName} placeholder="New group name..."
						class="flex-1 skeu-input text-[10px]"
						onkeydown={(e) => { if (e.key === 'Enter') addProxyGroup(); }} />
					<button class="skeu-btn text-[9px] flex items-center gap-0.5" onclick={addProxyGroup}>
						<Plus size={9} /> Add
					</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Browser Settings -->
	<div>
		<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 flex items-center gap-1.5">
			<Globe size={11} />
			Browser Settings
		</div>
		<div class="space-y-1.5">
			<div>
				<label class="text-[10px] text-muted-foreground">Browser Profile</label>
				<SkeuSelect value={app.pipeline.browser_settings.browser}
					onValueChange={(v) => { app.pipeline.browser_settings.browser = v; }}
					options={[{value:'chrome',label:'Chrome'},{value:'firefox',label:'Firefox'},{value:'safari',label:'Safari'},{value:'edge',label:'Edge'},{value:'custom',label:'Custom'}]}
					class="w-full text-[11px] mt-0.5"
				/>
			</div>
			<div>
				<label class="text-[10px] text-muted-foreground">User Agent</label>
				<input type="text"
					value={app.pipeline.browser_settings.user_agent || ''}
					placeholder="Auto"
					class="w-full skeu-input text-[11px] font-mono mt-0.5"
					oninput={(e) => { app.pipeline.browser_settings.user_agent = (e.target as HTMLInputElement).value || null; }}
				/>
			</div>
			<div>
				<label class="text-[10px] text-muted-foreground">JA3 Fingerprint</label>
				<input type="text"
					value={app.pipeline.browser_settings.ja3 || ''}
					placeholder="Auto (derived from browser profile)"
					class="w-full skeu-input text-[11px] font-mono mt-0.5"
					oninput={(e) => { app.pipeline.browser_settings.ja3 = (e.target as HTMLInputElement).value || null; }}
				/>
				<p class="text-[9px] text-muted-foreground mt-0.5">TLS client hello fingerprint string</p>
			</div>
			<div>
				<label class="text-[10px] text-muted-foreground">HTTP/2 Fingerprint</label>
				<input type="text"
					value={app.pipeline.browser_settings.http2_fingerprint || ''}
					placeholder="Auto (derived from browser profile)"
					class="w-full skeu-input text-[11px] font-mono mt-0.5"
					oninput={(e) => { app.pipeline.browser_settings.http2_fingerprint = (e.target as HTMLInputElement).value || null; }}
				/>
				<p class="text-[9px] text-muted-foreground mt-0.5">HTTP/2 SETTINGS frame + priority fingerprint</p>
			</div>
		</div>
	</div>

	<!-- Hits Database -->
	<div>
		<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 flex items-center gap-1.5">
			<CheckCircle size={11} />
			Hits Database
			<div class="flex-1"></div>
			<button
				class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors"
				onclick={() => { autoRefreshHits = !autoRefreshHits; }}
				title={autoRefreshHits ? 'Auto-refresh ON' : 'Auto-refresh OFF'}
			>
				<RefreshCw size={10} class={autoRefreshHits ? 'text-green' : ''} />
			</button>
		</div>
		<div class="space-y-1.5">
			<!-- Filter -->
			<div class="flex gap-1">
				<div class="relative flex-1">
					<Filter size={10} class="absolute left-2 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none" />
					<input
						type="text"
						bind:value={hitsFilter}
						placeholder="Filter hits by data, type, or config..."
						class="w-full skeu-input text-[10px] pl-6"
					/>
				</div>
				<button class="skeu-btn text-[10px] text-muted-foreground" onclick={() => { hitsFilter = ''; }}>
					Clear
				</button>
			</div>

			<!-- Hits Table -->
			<div class="bg-background rounded border border-border overflow-hidden">
				{#if hitsData.length === 0}
					<div class="p-4 text-center text-[10px] text-muted-foreground">
						No hits yet. Run a job to see results here.
					</div>
				{:else}
					<div class="max-h-[300px] overflow-y-auto">
						<table class="w-full text-[10px]">
							<thead class="sticky top-0 bg-surface border-b border-border">
								<tr class="text-[9px] uppercase tracking-wider text-muted-foreground">
									<th class="text-left px-2 py-1 font-medium">Data</th>
									<th class="text-left px-2 py-1 font-medium w-16">Type</th>
									<th class="text-left px-2 py-1 font-medium w-24">Config</th>
									<th class="text-left px-2 py-1 font-medium w-20">Date</th>
									<th class="text-left px-2 py-1 font-medium w-24">Wordlist</th>
									<th class="text-left px-2 py-1 font-medium w-24">Proxy</th>
									<th class="text-left px-2 py-1 font-medium">Captured</th>
									<th class="text-center px-2 py-1 font-medium w-16">Actions</th>
								</tr>
							</thead>
							<tbody>
								{#each hitsData as hit}
									<tr class="border-b border-border/30 hover:bg-accent/10 transition-colors">
										<td class="px-2 py-1.5 font-mono text-foreground/90 truncate max-w-[150px]" title={hit.data}>
											{hit.data}
										</td>
										<td class="px-2 py-1.5">
											<span class="text-green text-[9px] font-medium uppercase tracking-wide">
												{hit.type}
											</span>
										</td>
										<td class="px-2 py-1.5 text-muted-foreground truncate" title={hit.config}>
											{hit.config}
										</td>
										<td class="px-2 py-1.5 text-muted-foreground font-mono text-[9px]">
											{new Date(hit.date).toLocaleTimeString()}
										</td>
										<td class="px-2 py-1.5 text-muted-foreground truncate" title={hit.wordlist}>
											{hit.wordlist}
										</td>
										<td class="px-2 py-1.5 text-muted-foreground font-mono truncate" title={hit.proxy}>
											{hit.proxy}
										</td>
										<td class="px-2 py-1.5">
											{#if Object.keys(hit.captured).length > 0}
												<div class="flex flex-wrap gap-0.5">
													{#each Object.entries(hit.captured).slice(0, 2) as [key, value]}
														<span class="text-[8px] bg-primary/10 text-primary px-1 py-0.5 rounded">
															{key}: {value.length > 20 ? value.slice(0, 20) + '...' : value}
														</span>
													{/each}
													{#if Object.keys(hit.captured).length > 2}
														<span class="text-[8px] text-muted-foreground">
															+{Object.keys(hit.captured).length - 2} more
														</span>
													{/if}
												</div>
											{:else}
												<span class="text-muted-foreground/50 text-[9px]">None</span>
											{/if}
										</td>
										<td class="px-2 py-1.5 text-center">
											<div class="flex items-center justify-center gap-0.5">
												<button
													class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-foreground transition-colors"
													onclick={() => {
														navigator.clipboard.writeText(hit.data);
													}}
													title="Copy data"
												>
													<Copy size={10} />
												</button>
												<button
													class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-red transition-colors"
													onclick={() => {
														hitsData = hitsData.filter(h => h !== hit);
													}}
													title="Delete"
												>
													<Trash2 size={10} />
												</button>
											</div>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
					<div class="px-2 py-1.5 bg-surface border-t border-border flex items-center justify-between">
						<span class="text-[9px] text-muted-foreground">
							{hitsData.length} hit{hitsData.length !== 1 ? 's' : ''}
						</span>
						<div class="flex gap-1">
							<button class="skeu-btn text-[9px] text-muted-foreground" onclick={() => { hitsData = []; }}>
								Clear All
							</button>
							<button class="skeu-btn text-[9px]" onclick={() => {
								const unique = new Map();
								for (const hit of hitsData) {
									unique.set(hit.data, hit);
								}
								hitsData = Array.from(unique.values());
							}}>
								Remove Duplicates
							</button>
						</div>
					</div>
				{/if}
			</div>

			<p class="text-[9px] text-muted-foreground">
				Hits are successful results captured during job execution. Auto-refreshes when enabled.
			</p>
		</div>
	</div>
</div>
