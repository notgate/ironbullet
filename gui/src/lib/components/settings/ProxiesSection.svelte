<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import { toast } from '$lib/toast.svelte';
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import type { ProxyGroup, ProxySource } from '$lib/types';

	let { searchQuery, shouldShowSetting }: {
		searchQuery: string;
		shouldShowSetting: (section: string, label: string) => boolean;
	} = $props();

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
</script>

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
