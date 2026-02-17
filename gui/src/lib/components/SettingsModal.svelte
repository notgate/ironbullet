<script lang="ts">
	import { app, zoomIn, zoomOut, zoomReset } from '$lib/state.svelte';
	import { send, saveSettings } from '$lib/ipc';
	import { toast } from '$lib/toast.svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import SkeuSelect from './SkeuSelect.svelte';
	import Minus from '@lucide/svelte/icons/minus';
	import Plus from '@lucide/svelte/icons/plus';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
	import Search from '@lucide/svelte/icons/search';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import type { ProxyGroup, ProxySource } from '$lib/types';

	let newGroupName = $state('');
	let isCheckingProxies = $state(false);

	const PROXY_MODE_OPTIONS = [
		{ value: 'None', label: 'None' },
		{ value: 'Rotate', label: 'Rotate' },
		{ value: 'Sticky', label: 'Sticky' },
		{ value: 'CpmLimited', label: 'CPM Limited' },
	];

	function addProxyGroup() {
		if (!newGroupName.trim()) return;
		const group: ProxyGroup = { name: newGroupName.trim(), mode: 'Rotate', sources: [], cpm_per_proxy: 0 };
		app.pipeline.proxy_settings.proxy_groups = [...app.pipeline.proxy_settings.proxy_groups, group];
		newGroupName = '';
	}

	function removeProxyGroup(idx: number) {
		const groups = [...app.pipeline.proxy_settings.proxy_groups];
		const removed = groups.splice(idx, 1)[0];
		app.pipeline.proxy_settings.proxy_groups = groups;
		if (app.pipeline.proxy_settings.active_group === removed.name) {
			app.pipeline.proxy_settings.active_group = '';
		}
	}

	function addGroupSource(gi: number) {
		const groups = [...app.pipeline.proxy_settings.proxy_groups];
		groups[gi] = { ...groups[gi], sources: [...groups[gi].sources, { source_type: 'File', value: '', refresh_interval_secs: 0 }] };
		app.pipeline.proxy_settings.proxy_groups = groups;
	}

	function removeGroupSource(gi: number, si: number) {
		const groups = [...app.pipeline.proxy_settings.proxy_groups];
		groups[gi] = { ...groups[gi], sources: groups[gi].sources.filter((_: ProxySource, i: number) => i !== si) };
		app.pipeline.proxy_settings.proxy_groups = groups;
	}

	function checkProxies() {
		isCheckingProxies = true;
		send('check_proxies');
		toast('Checking proxies...', 'info');
		setTimeout(() => { isCheckingProxies = false; }, 15000);
	}

	let open = $derived(app.showSettings);

	function onOpenChange(v: boolean) {
		if (!v) saveSettings();
		app.showSettings = v;
	}

	const FONTS = [
		{ value: 'Segoe UI', label: 'Segoe UI' },
		{ value: 'Inter', label: 'Inter' },
		{ value: 'Cascadia Code', label: 'Cascadia Code' },
		{ value: 'Consolas', label: 'Consolas' },
		{ value: 'system-ui', label: 'System Default' },
	];

	type Section = 'display' | 'layout' | 'paths' | 'proxies' | 'runner' | 'output' | 'about';
	let activeSection = $state<Section>('display');
	let searchQuery = $state('');

	const SECTIONS: { id: Section; label: string }[] = [
		{ id: 'display', label: 'Display' },
		{ id: 'layout', label: 'Layout' },
		{ id: 'paths', label: 'Paths' },
		{ id: 'proxies', label: 'Proxies' },
		{ id: 'runner', label: 'Runner' },
		{ id: 'output', label: 'Output' },
		{ id: 'plugins', label: 'Plugins' },
		{ id: 'about', label: 'About' },
	];

	// Settings definitions for search
	const ALL_SETTINGS: { section: Section; label: string; keywords: string }[] = [
		{ section: 'display', label: 'UI Scale', keywords: 'zoom scale size' },
		{ section: 'display', label: 'Font Family', keywords: 'font typeface' },
		{ section: 'display', label: 'Font Size', keywords: 'font text size' },
		{ section: 'layout', label: 'Block Palette', keywords: 'palette sidebar blocks visible' },
		{ section: 'layout', label: 'Palette Width', keywords: 'palette width sidebar size' },
		{ section: 'layout', label: 'Bottom Panel', keywords: 'bottom panel height size' },
		{ section: 'paths', label: 'Collections Folder', keywords: 'collections configs folder path' },
		{ section: 'paths', label: 'Wordlist Directory', keywords: 'wordlist combos default path folder' },
		{ section: 'paths', label: 'Proxy Directory', keywords: 'proxy proxies default path folder' },
		{ section: 'proxies', label: 'Proxy Groups', keywords: 'proxy groups named sets' },
		{ section: 'proxies', label: 'Check Proxies', keywords: 'proxy check alive dead test' },
		{ section: 'proxies', label: 'Proxy Mode', keywords: 'proxy mode rotate sticky cpm' },
		{ section: 'proxies', label: 'Ban Duration', keywords: 'proxy ban duration seconds timeout' },
		{ section: 'runner', label: 'Thread Count', keywords: 'threads concurrency parallel bots' },
		{ section: 'runner', label: 'Skip Lines', keywords: 'skip offset data wordlist' },
		{ section: 'runner', label: 'Take Lines', keywords: 'take limit max data' },
		{ section: 'runner', label: 'Max Retries', keywords: 'retry retries max attempts' },
		{ section: 'runner', label: 'Custom Status', keywords: 'custom status name label bot' },
		{ section: 'runner', label: 'Continue Statuses', keywords: 'continue retry ban requeue' },
		{ section: 'runner', label: 'Gradual Start', keywords: 'gradual start threads ramp slow' },
		{ section: 'runner', label: 'Auto Thread Count', keywords: 'automatic thread cpm optimize' },
		{ section: 'runner', label: 'Lower on Retry', keywords: 'lower reduce threads retry' },
		{ section: 'runner', label: 'Pause on Ratelimit', keywords: 'pause ratelimit 429 throttle' },
		{ section: 'runner', label: 'Proxyless Only', keywords: 'proxyless no proxy direct' },
		{ section: 'output', label: 'Save to File', keywords: 'file text output save hits' },
		{ section: 'output', label: 'Save to Database', keywords: 'database sqlite db save' },
		{ section: 'output', label: 'Include Response', keywords: 'debug response body include output' },
		{ section: 'output', label: 'Output Format', keywords: 'format template output' },
		{ section: 'output', label: 'Output Directory', keywords: 'directory folder path output' },
	];

	function matchesSearch(label: string, keywords: string): boolean {
		if (!searchQuery) return true;
		const q = searchQuery.toLowerCase();
		return label.toLowerCase().includes(q) || keywords.toLowerCase().includes(q);
	}

	function sectionHasMatch(sectionId: Section): boolean {
		if (!searchQuery) return sectionId === activeSection;
		return ALL_SETTINGS.some(s => s.section === sectionId && matchesSearch(s.label, s.keywords));
	}

	function shouldShowSetting(section: Section, label: string): boolean {
		const setting = ALL_SETTINGS.find(s => s.section === section && s.label === label);
		if (!setting) return !searchQuery;
		if (!searchQuery) return true;
		return matchesSearch(setting.label, setting.keywords);
	}
</script>

<Dialog.Root {open} onOpenChange={onOpenChange}>
	<Dialog.Content class="bg-surface border-border max-w-[560px] p-0 gap-0 overflow-hidden flex flex-col" showCloseButton={false}>
		<!-- Header -->
		<div class="flex items-center justify-between px-4 py-2.5 border-b border-border-dark bg-surface panel-raised">
			<span class="text-[13px] font-semibold text-foreground tracking-tight">Settings</span>
			<button
				class="text-muted-foreground hover:text-foreground text-lg leading-none px-1"
				onclick={() => { saveSettings(); app.showSettings = false; }}
			>&times;</button>
		</div>

		<!-- Body: sidebar + content -->
		<div class="flex" style="height: 380px;">
			<!-- Sidebar -->
			<div class="w-[130px] shrink-0 border-r border-border-dark bg-background flex flex-col">
				<!-- Search -->
				<div class="p-2 border-b border-border-dark">
					<div class="relative">
						<Search size={11} class="absolute left-1.5 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none" />
						<input
							type="text"
							bind:value={searchQuery}
							placeholder="Search..."
							class="w-full skeu-input text-[10px]"
							style="padding-left: 22px;"
						/>
					</div>
				</div>

				<!-- Section list -->
				<div class="flex-1 overflow-y-auto py-1">
					{#each SECTIONS as section}
						<button
							class="w-full text-left px-3 py-1.5 text-[11px] transition-colors {activeSection === section.id && !searchQuery ? 'bg-secondary text-foreground font-medium' : 'text-muted-foreground hover:text-foreground hover:bg-secondary/50'}"
							onclick={() => { activeSection = section.id; searchQuery = ''; }}
						>{section.label}</button>
					{/each}
				</div>
			</div>

			<!-- Content -->
			<div class="flex-1 overflow-y-auto p-4">
				<!-- Display Section -->
				{#if sectionHasMatch('display')}
					{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2">Display</div>{/if}

					{#if shouldShowSetting('display', 'UI Scale')}
						<div class="flex items-center justify-between py-1.5">
							<span class="text-[11px] text-muted-foreground">UI Scale</span>
							<div class="flex items-center gap-1.5">
								<button class="skeu-btn text-[10px] w-6 h-6 flex items-center justify-center p-0" onclick={() => zoomOut()} title="Zoom Out">
									<Minus size={10} />
								</button>
								<span class="text-[11px] text-foreground font-mono w-10 text-center">{Math.round(app.zoom * 100)}%</span>
								<button class="skeu-btn text-[10px] w-6 h-6 flex items-center justify-center p-0" onclick={() => zoomIn()} title="Zoom In">
									<Plus size={10} />
								</button>
								<button class="skeu-btn text-[10px] w-6 h-6 flex items-center justify-center p-0 ml-1" onclick={() => zoomReset()} title="Reset">
									<RotateCcw size={10} />
								</button>
							</div>
						</div>
					{/if}

					{#if shouldShowSetting('display', 'Font Family')}
						<div class="flex items-center justify-between py-1.5">
							<span class="text-[11px] text-muted-foreground">Font Family</span>
							<SkeuSelect
								value={app.fontFamily}
								onValueChange={(v) => { app.fontFamily = v; }}
								options={FONTS}
								class="text-[11px] w-[160px]"
							/>
						</div>
					{/if}

					{#if shouldShowSetting('display', 'Font Size')}
						<div class="flex items-center justify-between py-1.5">
							<span class="text-[11px] text-muted-foreground">Font Size</span>
							<div class="flex items-center gap-1.5">
								<input
									type="range"
									min="10"
									max="18"
									step="1"
									bind:value={app.fontSize}
									class="w-24 accent-[var(--primary)]"
								/>
								<span class="text-[11px] text-foreground font-mono w-8 text-right">{app.fontSize}px</span>
							</div>
						</div>
					{/if}

					{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
				{/if}

				<!-- Layout Section -->
				{#if sectionHasMatch('layout')}
					{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Layout</div>{/if}

					{#if shouldShowSetting('layout', 'Block Palette')}
						<div class="flex items-center justify-between py-1.5">
							<span class="text-[11px] text-muted-foreground">Block Palette</span>
							<!-- svelte-ignore a11y_label_has_associated_control -->
							<label class="flex items-center gap-2 cursor-pointer">
								<input
									type="checkbox"
									checked={app.showBlockPalette}
									onchange={() => { app.showBlockPalette = !app.showBlockPalette; }}
									class="skeu-checkbox"
								/>
								<span class="text-[11px] text-foreground">{app.showBlockPalette ? 'Visible' : 'Hidden'}</span>
							</label>
						</div>
					{/if}

					{#if shouldShowSetting('layout', 'Palette Width')}
						<div class="flex items-center justify-between py-1.5">
							<span class="text-[11px] text-muted-foreground">Palette Width</span>
							<div class="flex items-center gap-1.5">
								<input
									type="range"
									min="120"
									max="400"
									step="10"
									bind:value={app.leftPanelWidth}
									class="w-24 accent-[var(--primary)]"
								/>
								<span class="text-[11px] text-foreground font-mono w-10 text-right">{app.leftPanelWidth}px</span>
							</div>
						</div>
					{/if}

					{#if shouldShowSetting('layout', 'Bottom Panel')}
						<div class="flex items-center justify-between py-1.5">
							<span class="text-[11px] text-muted-foreground">Bottom Panel</span>
							<div class="flex items-center gap-1.5">
								<input
									type="range"
									min="100"
									max="500"
									step="10"
									bind:value={app.bottomPanelHeight}
									class="w-24 accent-[var(--primary)]"
								/>
								<span class="text-[11px] text-foreground font-mono w-10 text-right">{app.bottomPanelHeight}px</span>
							</div>
						</div>
					{/if}

					{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
				{/if}

				<!-- Paths Section -->
				{#if sectionHasMatch('paths')}
					{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Paths</div>{/if}

					{#if shouldShowSetting('paths', 'Collections Folder')}
						<div class="py-1.5">
							<div class="flex items-center justify-between">
								<div>
									<span class="text-[11px] text-muted-foreground">Collections folder</span>
									<p class="text-[9px] text-muted-foreground/60">Folder containing .rfx configs for quick access</p>
								</div>
								<button class="skeu-btn text-[10px]" onclick={() => send('browse_folder', { field: 'collections' })}>Browse</button>
							</div>
							<input
								type="text"
								bind:value={app.collectionsPath}
								class="w-full skeu-input text-[10px] font-mono mt-1"
								placeholder="Not set"
							/>
						</div>
					{/if}

					{#if shouldShowSetting('paths', 'Wordlist Directory')}
						<div class="py-1.5">
							<div class="flex items-center justify-between">
								<div>
									<span class="text-[11px] text-muted-foreground">Default wordlist directory</span>
									<p class="text-[9px] text-muted-foreground/60">Start browsing wordlists from this folder</p>
								</div>
								<button class="skeu-btn text-[10px]" onclick={() => send('browse_folder', { field: 'wordlist_dir' })}>Browse</button>
							</div>
							<input
								type="text"
								bind:value={app.defaultWordlistPath}
								class="w-full skeu-input text-[10px] font-mono mt-1"
								placeholder="Not set"
							/>
						</div>
					{/if}

					{#if shouldShowSetting('paths', 'Proxy Directory')}
						<div class="py-1.5">
							<div class="flex items-center justify-between">
								<div>
									<span class="text-[11px] text-muted-foreground">Default proxy directory</span>
									<p class="text-[9px] text-muted-foreground/60">Start browsing proxy files from this folder</p>
								</div>
								<button class="skeu-btn text-[10px]" onclick={() => send('browse_folder', { field: 'proxy_dir' })}>Browse</button>
							</div>
							<input
								type="text"
								bind:value={app.defaultProxyPath}
								class="w-full skeu-input text-[10px] font-mono mt-1"
								placeholder="Not set"
							/>
						</div>
					{/if}

					{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
				{/if}

				<!-- Proxies Section -->
				{#if sectionHasMatch('proxies')}
					{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Proxies</div>{/if}

					{#if shouldShowSetting('proxies', 'Proxy Mode')}
						<div class="flex items-center justify-between py-1.5">
							<div>
								<span class="text-[11px] text-muted-foreground">Default proxy mode</span>
								<p class="text-[9px] text-muted-foreground/60">How proxies are assigned to requests</p>
							</div>
							<SkeuSelect
								value={app.pipeline.proxy_settings.proxy_mode}
								onValueChange={(v) => { app.pipeline.proxy_settings.proxy_mode = v; }}
								options={PROXY_MODE_OPTIONS}
								class="text-[11px] w-[120px]"
							/>
						</div>
					{/if}

					{#if shouldShowSetting('proxies', 'Ban Duration')}
						<div class="flex items-center justify-between py-1.5">
							<div>
								<span class="text-[11px] text-muted-foreground">Ban duration</span>
								<p class="text-[9px] text-muted-foreground/60">Seconds to ban a proxy after failures</p>
							</div>
							<div class="flex items-center gap-1">
								<input
									type="number"
									min="0"
									bind:value={app.pipeline.proxy_settings.ban_duration_secs}
									class="w-16 skeu-input text-[11px] text-center"
								/>
								<span class="text-[10px] text-muted-foreground">sec</span>
							</div>
						</div>
					{/if}

					{#if shouldShowSetting('proxies', 'Check Proxies')}
						<div class="flex items-center justify-between py-1.5">
							<div>
								<span class="text-[11px] text-muted-foreground">Check proxies</span>
								<p class="text-[9px] text-muted-foreground/60">Test all loaded proxies for connectivity</p>
							</div>
							<button
								class="skeu-btn text-[10px] {isCheckingProxies ? 'opacity-50' : ''}"
								onclick={checkProxies}
								disabled={isCheckingProxies}
							>{isCheckingProxies ? 'Checking...' : 'Check Proxies'}</button>
						</div>
					{/if}

					{#if shouldShowSetting('proxies', 'Proxy Groups')}
						{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
						<div class="py-1.5">
							<span class="text-[11px] text-muted-foreground font-medium">Proxy Groups</span>
							<p class="text-[9px] text-muted-foreground/60 mb-2">Named proxy sets with independent modes (like OB2)</p>

							<!-- Active group selector -->
							{#if app.pipeline.proxy_settings.proxy_groups.length > 0}
								<div class="flex items-center justify-between mb-2">
									<span class="text-[10px] text-muted-foreground">Active group</span>
									<SkeuSelect
										value={app.pipeline.proxy_settings.active_group || '(default)'}
										onValueChange={(v) => { app.pipeline.proxy_settings.active_group = v === '(default)' ? '' : v; }}
										options={[{ value: '(default)', label: 'Default' }, ...app.pipeline.proxy_settings.proxy_groups.map(g => ({ value: g.name, label: g.name }))]}
										class="text-[10px] w-[140px]"
									/>
								</div>
							{/if}

							<!-- Group cards -->
							{#each app.pipeline.proxy_settings.proxy_groups as group, gi}
								<div class="bg-background rounded border border-border p-2 mb-1.5">
									<div class="flex items-center justify-between mb-1">
										<span class="text-[11px] text-foreground font-medium">{group.name}</span>
										<div class="flex items-center gap-1">
											<SkeuSelect
												value={group.mode}
												onValueChange={(v) => { const g = [...app.pipeline.proxy_settings.proxy_groups]; g[gi] = { ...g[gi], mode: v }; app.pipeline.proxy_settings.proxy_groups = g; }}
												options={PROXY_MODE_OPTIONS}
												class="text-[9px]"
											/>
											<button class="p-0.5 text-muted-foreground hover:text-red" onclick={() => removeProxyGroup(gi)} title="Remove group">
												<Trash2 size={10} />
											</button>
										</div>
									</div>
									<!-- Sources -->
									<div class="space-y-1 ml-1">
										{#each group.sources as src, si}
											<div class="flex gap-1 items-center">
												<input type="text" bind:value={src.value} placeholder="path or URL" class="flex-1 skeu-input text-[9px] font-mono" />
												<button class="p-0.5 text-muted-foreground hover:text-red shrink-0" onclick={() => removeGroupSource(gi, si)}>
													<Trash2 size={9} />
												</button>
											</div>
										{/each}
										<button class="text-[9px] text-primary hover:underline" onclick={() => addGroupSource(gi)}>+ Add source</button>
									</div>
								</div>
							{/each}

							<!-- Add new group -->
							<div class="flex gap-1 mt-1">
								<input
									type="text"
									bind:value={newGroupName}
									placeholder="New group name"
									class="flex-1 skeu-input text-[10px]"
									onkeydown={(e) => { if (e.key === 'Enter') addProxyGroup(); }}
								/>
								<button class="skeu-btn text-[10px]" onclick={addProxyGroup}>Add</button>
							</div>
						</div>
					{/if}

					{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
				{/if}

				<!-- Runner Section -->
				{#if sectionHasMatch('runner')}
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
				{/if}

				<!-- Output Section -->
				{#if sectionHasMatch('output')}
					{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Output</div>{/if}

					{#if shouldShowSetting('output', 'Save to File')}
						<div class="flex items-center justify-between py-1.5">
							<div>
								<span class="text-[11px] text-muted-foreground">Save to text files</span>
								<p class="text-[9px] text-muted-foreground/60">Write hits to .txt files per status</p>
							</div>
							<!-- svelte-ignore a11y_label_has_associated_control -->
							<label class="flex items-center gap-2 cursor-pointer">
								<input
									type="checkbox"
									checked={app.pipeline.output_settings.save_to_file}
									onchange={() => { app.pipeline.output_settings.save_to_file = !app.pipeline.output_settings.save_to_file; }}
									class="skeu-checkbox"
								/>
							</label>
						</div>
					{/if}

					{#if shouldShowSetting('output', 'Save to Database')}
						<div class="flex items-center justify-between py-1.5">
							<div>
								<span class="text-[11px] text-muted-foreground">Save to database</span>
								<p class="text-[9px] text-muted-foreground/60">Store results in SQLite database</p>
							</div>
							<!-- svelte-ignore a11y_label_has_associated_control -->
							<label class="flex items-center gap-2 cursor-pointer">
								<input
									type="checkbox"
									checked={app.pipeline.output_settings.save_to_database}
									onchange={() => { app.pipeline.output_settings.save_to_database = !app.pipeline.output_settings.save_to_database; }}
									class="skeu-checkbox"
								/>
							</label>
						</div>
						{#if app.pipeline.output_settings.save_to_database}
							<div class="flex items-center justify-between py-1 pl-4">
								<span class="text-[10px] text-muted-foreground/80">Database path</span>
								<input
									type="text"
									bind:value={app.pipeline.output_settings.database_path}
									class="w-36 skeu-input text-[10px]"
									placeholder="results.db"
								/>
							</div>
						{/if}
					{/if}

					{#if shouldShowSetting('output', 'Include Response')}
						<div class="flex items-center justify-between py-1.5">
							<div>
								<span class="text-[11px] text-muted-foreground">Include response (-d)</span>
								<p class="text-[9px] text-muted-foreground/60">Append full response body in output</p>
							</div>
							<!-- svelte-ignore a11y_label_has_associated_control -->
							<label class="flex items-center gap-2 cursor-pointer">
								<input
									type="checkbox"
									checked={app.pipeline.output_settings.include_response}
									onchange={() => { app.pipeline.output_settings.include_response = !app.pipeline.output_settings.include_response; }}
									class="skeu-checkbox"
								/>
							</label>
						</div>
					{/if}

					{#if shouldShowSetting('output', 'Output Directory')}
						<div class="flex items-center justify-between py-1.5">
							<div>
								<span class="text-[11px] text-muted-foreground">Output directory</span>
								<p class="text-[9px] text-muted-foreground/60">Folder for result text files</p>
							</div>
							<input
								type="text"
								bind:value={app.pipeline.output_settings.output_directory}
								class="w-28 skeu-input text-[10px]"
								placeholder="results"
							/>
						</div>
					{/if}

					{#if shouldShowSetting('output', 'Output Format')}
						<div class="py-1.5">
							<span class="text-[11px] text-muted-foreground">Output format</span>
							<p class="text-[9px] text-muted-foreground/60 mb-1">Template: {'{data}'}, {'{captures}'}, {'{status}'}, {'{response}'}</p>
							<input
								type="text"
								bind:value={app.pipeline.output_settings.output_format}
								class="w-full skeu-input text-[10px]"
								placeholder="{'{data}'} | {'{captures}'}"
							/>
						</div>

						<!-- Output Format Type -->
						<div class="flex items-center justify-between">
							<span class="text-[11px]">File Format</span>
							<SkeuSelect
								value={app.pipeline.output_settings.output_format_type}
								onValueChange={(v) => { app.pipeline.output_settings.output_format_type = v as any; }}
								options={[{value:'Txt',label:'TXT'},{value:'Csv',label:'CSV'},{value:'Json',label:'JSON'}]}
								class="text-[10px] w-24"
							/>
						</div>

						<!-- Capture Filters -->
						<div>
							<div class="flex items-center justify-between mb-1">
								<span class="text-[11px]">Capture Filters</span>
								<button class="skeu-btn text-[9px] px-2 py-0.5" onclick={() => {
									app.pipeline.output_settings.capture_filters = [
										...app.pipeline.output_settings.capture_filters,
										{ variable_name: '*', filter_type: 'NotEmpty', value: '', negate: false }
									];
								}}>+ Add</button>
							</div>
							{#each app.pipeline.output_settings.capture_filters as filter, i}
								<div class="flex gap-1 items-center mb-1">
									<input
										type="text"
										bind:value={filter.variable_name}
										class="skeu-input text-[9px] w-20"
										placeholder="* or var name"
									/>
									<SkeuSelect
										value={filter.filter_type}
										onValueChange={(v) => { filter.filter_type = v as any; }}
										options={[
											{value:'Contains',label:'Contains'},{value:'Equals',label:'Equals'},
											{value:'StartsWith',label:'Starts With'},{value:'EndsWith',label:'Ends With'},
											{value:'MatchesRegex',label:'Regex'},{value:'MinLength',label:'Min Length'},
											{value:'MaxLength',label:'Max Length'},{value:'NotEmpty',label:'Not Empty'},
										]}
										class="text-[9px] w-24"
									/>
									<input
										type="text"
										bind:value={filter.value}
										class="skeu-input text-[9px] w-16"
										placeholder="value"
									/>
									<label class="flex items-center gap-1 text-[9px]">
										<input type="checkbox" bind:checked={filter.negate} class="skeu-checkbox" />
										<span>Negate</span>
									</label>
									<button class="text-[9px] text-red-400 hover:text-red-300 px-1" onclick={() => {
										app.pipeline.output_settings.capture_filters = app.pipeline.output_settings.capture_filters.filter((_, idx) => idx !== i);
									}}>x</button>
								</div>
							{/each}
						</div>
					{/if}

					{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
				{/if}

				<!-- Plugins Section -->
				{#if sectionHasMatch('plugins')}
					{#if searchQuery}<div class="text-[10px] uppercase tracking-widest text-muted-foreground/70 font-semibold mb-2 mt-2">Plugins</div>{/if}

					{#if shouldShowSetting('plugins', 'Plugins Directory')}
						<div class="py-1.5">
							<div class="flex items-center justify-between">
								<div>
									<span class="text-[11px] text-muted-foreground">Plugins directory</span>
									<p class="text-[9px] text-muted-foreground/60">Folder containing .dll plugin files</p>
								</div>
								<div class="flex gap-1">
									<button class="skeu-btn text-[10px]" onclick={() => send('browse_folder', { field: 'plugins' })}>Browse</button>
									<button class="skeu-btn text-[10px]" onclick={() => send('reload_plugins')}>Reload</button>
								</div>
							</div>
							<input
								type="text"
								value={(app.config as any)?.plugins_path || ''}
								class="w-full skeu-input text-[10px] font-mono mt-1"
								placeholder="Not set"
								oninput={(e) => { (app.config as any).plugins_path = (e.target as HTMLInputElement).value; }}
							/>
						</div>
					{/if}

					{#if !searchQuery}<div class="my-2 groove-h h-px"></div>{/if}
				{/if}

				<!-- About Section -->
				{#if sectionHasMatch('about')}
					{#if !searchQuery}
						<div class="space-y-2 py-2">
							<div class="text-[13px] font-semibold text-foreground">reqflow</div>
							<div class="text-[11px] text-muted-foreground">Version 0.1.0</div>
							<div class="text-[10px] text-muted-foreground mt-2">
								Pipeline-based HTTP automation toolkit with visual block editor.
							</div>
						</div>
					{/if}
				{/if}
			</div>
		</div>
	</Dialog.Content>
</Dialog.Root>
