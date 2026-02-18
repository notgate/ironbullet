<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { saveSettings } from '$lib/ipc';
	import * as Dialog from '$lib/components/ui/dialog';
	import Search from '@lucide/svelte/icons/search';
	import DisplaySection from '$lib/components/settings/DisplaySection.svelte';
	import LayoutSection from '$lib/components/settings/LayoutSection.svelte';
	import PathsSection from '$lib/components/settings/PathsSection.svelte';
	import ProxiesSection from '$lib/components/settings/ProxiesSection.svelte';
	import RunnerSection from '$lib/components/settings/RunnerSection.svelte';
	import OutputSection from '$lib/components/settings/OutputSection.svelte';
	import PluginsSection from '$lib/components/settings/PluginsSection.svelte';
	import AboutSection from '$lib/components/settings/AboutSection.svelte';
	import UpdateSection from '$lib/components/settings/UpdateSection.svelte';

	type Section = 'display' | 'layout' | 'paths' | 'proxies' | 'runner' | 'output' | 'plugins' | 'updates' | 'about';
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
		{ id: 'updates', label: 'Updates' },
		{ id: 'about', label: 'About' },
	];

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

	function shouldShowSetting(section: string, label: string): boolean {
		const setting = ALL_SETTINGS.find(s => s.section === section && s.label === label);
		if (!setting) return !searchQuery;
		if (!searchQuery) return true;
		return matchesSearch(setting.label, setting.keywords);
	}

	let open = $derived(app.showSettings);

	function onOpenChange(v: boolean) {
		if (!v) saveSettings();
		app.showSettings = v;
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
				{#if sectionHasMatch('display')}<DisplaySection {searchQuery} {shouldShowSetting} />{/if}
				{#if sectionHasMatch('layout')}<LayoutSection {searchQuery} {shouldShowSetting} />{/if}
				{#if sectionHasMatch('paths')}<PathsSection {searchQuery} {shouldShowSetting} />{/if}
				{#if sectionHasMatch('proxies')}<ProxiesSection {searchQuery} {shouldShowSetting} />{/if}
				{#if sectionHasMatch('runner')}<RunnerSection {searchQuery} {shouldShowSetting} />{/if}
				{#if sectionHasMatch('output')}<OutputSection {searchQuery} {shouldShowSetting} />{/if}
				{#if sectionHasMatch('plugins')}<PluginsSection {searchQuery} {shouldShowSetting} />{/if}
				{#if sectionHasMatch('updates')}<UpdateSection {searchQuery} />{/if}
				{#if sectionHasMatch('about')}<AboutSection {searchQuery} />{/if}
			</div>
		</div>
	</Dialog.Content>
</Dialog.Root>
