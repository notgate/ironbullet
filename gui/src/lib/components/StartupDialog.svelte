<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import * as Dialog from '$lib/components/ui/dialog';
	import Workflow from '@lucide/svelte/icons/workflow';
	import FolderOpen from '@lucide/svelte/icons/folder-open';
	import Plus from '@lucide/svelte/icons/plus';
	import Clock from '@lucide/svelte/icons/clock';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import FileText from '@lucide/svelte/icons/file-text';
	import Upload from '@lucide/svelte/icons/upload';

	let open = $derived(app.showStartup);
	let mode = $state<'home' | 'new'>('home');

	// New config form
	let configName = $state('New Config');
	let configDescription = $state('');

	function onOpenChange(v: boolean) {
		if (!v) app.showStartup = false;
	}

	function createNew() {
		app.pipeline.name = configName.trim() || 'New Config';
		app.showStartup = false;
		mode = 'home';
	}

	function openRecent(path: string) {
		send('load_pipeline', { path });
		app.showStartup = false;
	}

	function openFilePicker() {
		send('load_pipeline');
		app.showStartup = false;
	}

	function importConfig() {
		send('import_config');
		app.showStartup = false;
	}

	function formatDate(iso: string): string {
		try {
			const d = new Date(iso);
			const now = new Date();
			const diffMs = now.getTime() - d.getTime();
			const diffMins = Math.floor(diffMs / 60000);
			if (diffMins < 1) return 'just now';
			if (diffMins < 60) return `${diffMins}m ago`;
			const diffHours = Math.floor(diffMins / 60);
			if (diffHours < 24) return `${diffHours}h ago`;
			const diffDays = Math.floor(diffHours / 24);
			if (diffDays < 7) return `${diffDays}d ago`;
			return d.toLocaleDateString();
		} catch { return ''; }
	}
</script>

<Dialog.Root {open} onOpenChange={onOpenChange}>
	<Dialog.Content class="bg-surface border-border max-w-[480px] p-0 gap-0 overflow-hidden flex flex-col" showCloseButton={false} interactOutsideBehavior="ignore" onEscapeKeydown={(e) => e.preventDefault()}>
		<!-- Header -->
		<div class="flex items-center gap-2 px-4 py-3 border-b border-border-dark panel-raised">
			<Workflow size={16} class="text-primary" />
			<span class="text-sm font-semibold text-foreground tracking-tight">Ironbullet</span>
			<span class="text-[10px] text-muted-foreground/60 font-mono ml-1">v0.1.0</span>
		</div>

		{#if mode === 'home'}
			<div class="p-4 space-y-3" style="min-height: 280px;">
				<!-- Quick actions -->
				<div class="flex gap-2">
					<button
						class="flex-1 flex items-center gap-2 p-3 rounded border border-border hover:border-border-dark hover:bg-secondary/30 transition-colors cursor-pointer"
						onclick={() => { mode = 'new'; }}
					>
						<Plus size={15} class="text-muted-foreground shrink-0" />
						<div class="text-left">
							<div class="text-[12px] font-medium text-foreground">New Config</div>
							<div class="text-[10px] text-muted-foreground">Create a blank pipeline</div>
						</div>
					</button>
					<button
						class="flex-1 flex items-center gap-2 p-3 rounded border border-border hover:border-border-dark hover:bg-secondary/30 transition-colors cursor-pointer"
						onclick={openFilePicker}
					>
						<FolderOpen size={15} class="text-muted-foreground shrink-0" />
						<div class="text-left">
							<div class="text-[12px] font-medium text-foreground">Open Config</div>
							<div class="text-[10px] text-muted-foreground">Load an .rfx file</div>
						</div>
					</button>
				</div>

				<!-- Import button -->
				<button
					class="w-full flex items-center gap-2 px-3 py-2 rounded border border-border hover:border-primary/50 hover:bg-accent/20 transition-colors cursor-pointer"
					onclick={importConfig}
				>
					<Upload size={13} class="text-muted-foreground" />
					<span class="text-[11px] text-muted-foreground">Import .SVB / .OPK config...</span>
				</button>

				<!-- Recent configs -->
				{#if app.recentConfigs.length > 0}
					<div>
						<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5 flex items-center gap-1">
							<Clock size={10} />
							Recent Configs
						</div>
						<div class="space-y-0.5 max-h-[180px] overflow-y-auto">
							{#each app.recentConfigs as entry}
								<button
									class="w-full flex items-center gap-2 px-2 py-1.5 rounded hover:bg-accent/30 transition-colors cursor-pointer group text-left"
									onclick={() => openRecent(entry.path)}
								>
									<FileText size={13} class="text-muted-foreground shrink-0" />
									<div class="flex-1 min-w-0">
										<div class="text-[11px] text-foreground font-medium truncate">{entry.name}</div>
										<div class="text-[9px] text-muted-foreground truncate font-mono">{entry.path}</div>
									</div>
									<span class="text-[9px] text-muted-foreground/60 shrink-0">{formatDate(entry.last_opened)}</span>
								</button>
							{/each}
						</div>
					</div>
				{:else}
					<div class="text-center py-6">
						<div class="text-[11px] text-muted-foreground">No recent configs</div>
						<div class="text-[10px] text-muted-foreground/60 mt-1">Create a new config or open an existing one to get started.</div>
					</div>
				{/if}
			</div>
		{:else}
			<!-- New Config form -->
			<div class="p-4 space-y-3" style="min-height: 280px;">
				<button class="text-[10px] text-muted-foreground hover:text-foreground" onclick={() => { mode = 'home'; }}>
					&larr; Back
				</button>

				<div class="text-[13px] font-medium text-foreground">New Config</div>

				<div>
					<label class="text-[10px] uppercase tracking-wider text-muted-foreground">Name *</label>
					<input
						type="text"
						bind:value={configName}
						class="w-full skeu-input text-[12px] mt-0.5"
						placeholder="My Config"
					/>
				</div>

				<div>
					<label class="text-[10px] uppercase tracking-wider text-muted-foreground">Description</label>
					<textarea
						bind:value={configDescription}
						class="w-full skeu-input text-[11px] mt-0.5 min-h-[60px] resize-y"
						placeholder="Optional description..."
					></textarea>
				</div>

				<div class="flex justify-end gap-2 pt-2">
					<button class="skeu-btn text-[11px]" onclick={() => { mode = 'home'; }}>Cancel</button>
					<button
						class="skeu-btn text-[11px] bg-primary/20 text-primary hover:bg-primary/30"
						onclick={createNew}
					>Create Config</button>
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
