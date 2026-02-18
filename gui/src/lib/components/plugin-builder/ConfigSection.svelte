<script lang="ts">
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import { CATEGORIES, FIELD_TYPES, type PluginBlock } from './types';

	interface Props {
		pluginName: string;
		pluginVersion: string;
		pluginAuthor: string;
		pluginDescription: string;
		blocks: PluginBlock[];
		extraDeps: Array<{ name: string; version: string; features: string }>;
		projectDir: string;
		buildRelease: boolean;
		onAddBlock: () => void;
		onRemoveBlock: (idx: number) => void;
		onAddField: (blockIdx: number) => void;
		onRemoveField: (blockIdx: number, fieldIdx: number) => void;
		onAddDep: () => void;
		onRemoveDep: (idx: number) => void;
	}

	let {
		pluginName = $bindable(),
		pluginVersion = $bindable(),
		pluginAuthor = $bindable(),
		pluginDescription = $bindable(),
		blocks = $bindable(),
		extraDeps = $bindable(),
		projectDir = $bindable(),
		buildRelease = $bindable(),
		onAddBlock,
		onRemoveBlock,
		onAddField,
		onRemoveField,
		onAddDep,
		onRemoveDep,
	}: Props = $props();
</script>

<div class="p-3 space-y-3">
	<!-- Plugin metadata -->
	<details open class="group">
		<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
			<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
			Plugin Metadata
		</summary>
		<div class="mt-2 space-y-2">
			<label class="block">
				<span class="text-[10px] text-muted-foreground">Plugin Name</span>
				<input class="skeu-input w-full text-[11px] mt-0.5 truncate" bind:value={pluginName} placeholder="MyPlugin" />
			</label>
			<label class="block">
				<span class="text-[10px] text-muted-foreground">Version</span>
				<input class="skeu-input w-full text-[11px] mt-0.5 truncate" bind:value={pluginVersion} placeholder="0.1.0" />
			</label>
			<label class="block">
				<span class="text-[10px] text-muted-foreground">Author</span>
				<input class="skeu-input w-full text-[11px] mt-0.5 truncate" bind:value={pluginAuthor} placeholder="Your name" />
			</label>
			<label class="block">
				<span class="text-[10px] text-muted-foreground">Description</span>
				<input class="skeu-input w-full text-[11px] mt-0.5 truncate" bind:value={pluginDescription} placeholder="What does this plugin do?" />
			</label>
		</div>
	</details>

	<!-- Blocks -->
	<details open class="group">
		<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
			<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
			Blocks ({blocks.length})
		</summary>
		<div class="mt-2 space-y-3">
			{#each blocks as block, bi}
				<div class="border border-border rounded p-2 space-y-1.5 bg-accent/5 overflow-hidden">
					<div class="flex items-center gap-1.5">
						<div class="w-2.5 h-2.5 rounded shrink-0" style="background: {block.color}"></div>
						<span class="text-[11px] font-medium text-foreground flex-1 truncate">{block.name}</span>
						{#if blocks.length > 1}
							<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground shrink-0" onclick={() => onRemoveBlock(bi)} title="Remove block">
								<Trash2 size={10} />
							</button>
						{/if}
					</div>
					<div class="grid grid-cols-2 gap-1.5">
						<label class="block min-w-0">
							<span class="text-[9px] text-muted-foreground">Name</span>
							<input class="skeu-input w-full text-[10px] mt-0.5 truncate" bind:value={block.name} placeholder="BlockName" />
						</label>
						<label class="block min-w-0">
							<span class="text-[9px] text-muted-foreground">Label</span>
							<input class="skeu-input w-full text-[10px] mt-0.5 truncate" bind:value={block.label} placeholder="Display Name" />
						</label>
					</div>
					<div class="grid grid-cols-2 gap-1.5">
						<div class="min-w-0">
							<span class="text-[9px] text-muted-foreground block mb-0.5">Category</span>
							<SkeuSelect
								value={block.category}
								onValueChange={(v) => { block.category = v; }}
								options={CATEGORIES.map(c => ({ value: c, label: c }))}
								class="w-full text-[10px]"
							/>
						</div>
						<label class="block min-w-0">
							<span class="text-[9px] text-muted-foreground">Color</span>
							<div class="flex items-center gap-1 mt-0.5">
								<input type="color" class="w-5 h-5 rounded border border-border cursor-pointer shrink-0" bind:value={block.color} />
								<input class="skeu-input flex-1 min-w-0 text-[10px]" bind:value={block.color} />
							</div>
						</label>
					</div>

					<!-- Settings fields -->
					<div class="mt-1">
						<div class="flex items-center gap-1 mb-1">
							<span class="text-[9px] text-muted-foreground uppercase tracking-wider">Settings Fields</span>
							<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground ml-auto shrink-0" onclick={() => onAddField(bi)} title="Add field">
								<Plus size={10} />
							</button>
						</div>
						{#each block.settingsFields as field, fi}
							<div class="flex items-center gap-1 mb-1">
								<input class="skeu-input flex-1 min-w-0 text-[10px]" bind:value={field.name} placeholder="field_name" />
								<div class="w-14 shrink-0">
									<SkeuSelect
										value={field.type}
										onValueChange={(v) => { field.type = v; }}
										options={FIELD_TYPES}
										class="w-full text-[10px]"
									/>
								</div>
								<input class="skeu-input w-16 min-w-0 text-[10px]" bind:value={field.default} placeholder="default" />
								<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground shrink-0" onclick={() => onRemoveField(bi, fi)}>
									<Trash2 size={9} />
								</button>
							</div>
						{/each}
					</div>
				</div>
			{/each}
			<button class="skeu-btn w-full text-[10px] text-muted-foreground flex items-center justify-center gap-1 py-1.5" onclick={onAddBlock}>
				<Plus size={10} />Add Block
			</button>
		</div>
	</details>

	<!-- Extra dependencies -->
	<details class="group">
		<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
			<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
			Extra Dependencies ({extraDeps.length})
		</summary>
		<div class="mt-2 space-y-1.5">
			{#each extraDeps as dep, di}
				<div class="flex items-center gap-1">
					<input class="skeu-input flex-1 min-w-0 text-[10px]" bind:value={dep.name} placeholder="crate_name" />
					<input class="skeu-input w-14 min-w-0 text-[10px]" bind:value={dep.version} placeholder="ver" />
					<input class="skeu-input w-20 min-w-0 text-[10px]" bind:value={dep.features} placeholder="features" />
					<button class="p-0.5 rounded hover:bg-accent/30 text-muted-foreground shrink-0" onclick={() => onRemoveDep(di)}>
						<Trash2 size={9} />
					</button>
				</div>
			{/each}
			<button class="skeu-btn w-full text-[10px] text-muted-foreground flex items-center justify-center gap-1 py-1" onclick={onAddDep}>
				<Plus size={10} />Add Dependency
			</button>
		</div>
	</details>

	<!-- Build settings -->
	<details class="group">
		<summary class="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold cursor-pointer flex items-center gap-1.5 select-none py-1">
			<ChevronRight size={12} class="transition-transform group-open:rotate-90" />
			Build Settings
		</summary>
		<div class="mt-2 space-y-2">
			<label class="block min-w-0">
				<span class="text-[10px] text-muted-foreground">Project Directory (optional)</span>
				<input class="skeu-input w-full text-[10px] mt-0.5 truncate" bind:value={projectDir} placeholder="temp dir if empty" />
			</label>
			<label class="flex items-center gap-2 text-[10px] text-foreground/80">
				<input type="checkbox" bind:checked={buildRelease} class="accent-green" />
				Release mode (--release)
			</label>
		</div>
	</details>
</div>
