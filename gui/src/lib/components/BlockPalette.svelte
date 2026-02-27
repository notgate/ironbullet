<script lang="ts">
	import { BLOCK_CATALOG, type BlockType, type BlockMeta } from '$lib/types';
	import { send } from '$lib/ipc';
	import { app, pushUndo, deleteBlockTemplate } from '$lib/state.svelte';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Bookmark from '@lucide/svelte/icons/bookmark';
	import Globe from '@lucide/svelte/icons/globe';
	import Scissors from '@lucide/svelte/icons/scissors';
	import Braces from '@lucide/svelte/icons/braces';
	import Code from '@lucide/svelte/icons/code';
	import FileCode from '@lucide/svelte/icons/file-code';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import Type from '@lucide/svelte/icons/type';
	import List from '@lucide/svelte/icons/list';
	import Lock from '@lucide/svelte/icons/lock';
	import ArrowRightLeft from '@lucide/svelte/icons/arrow-right-left';
	import GitBranch from '@lucide/svelte/icons/git-branch';
	import Repeat from '@lucide/svelte/icons/repeat';
	import Clock from '@lucide/svelte/icons/clock';
	import Terminal from '@lucide/svelte/icons/terminal';
	import FileText from '@lucide/svelte/icons/file-text';
	import Variable from '@lucide/svelte/icons/variable';
	import Cookie from '@lucide/svelte/icons/cookie';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Monitor from '@lucide/svelte/icons/monitor';
	import MousePointerClick from '@lucide/svelte/icons/mouse-pointer-click';
	import Keyboard from '@lucide/svelte/icons/keyboard';
	import Hourglass from '@lucide/svelte/icons/hourglass';
	import ScanText from '@lucide/svelte/icons/scan-text';
	import Camera from '@lucide/svelte/icons/camera';
	import Dices from '@lucide/svelte/icons/dices';
	import Plug from '@lucide/svelte/icons/plug';
	import Cpu from '@lucide/svelte/icons/cpu';
	import User from '@lucide/svelte/icons/user';
	import Shield from '@lucide/svelte/icons/shield';
	import ScanEye from '@lucide/svelte/icons/scan-eye';
	import Key from '@lucide/svelte/icons/key';
	import Cloud from '@lucide/svelte/icons/cloud';
	import CheckCircle from '@lucide/svelte/icons/check-circle';
	import ListTree from '@lucide/svelte/icons/list-tree';
	import Cable from '@lucide/svelte/icons/cable';
	import Radio from '@lucide/svelte/icons/radio';
	import HardDriveDownload from '@lucide/svelte/icons/hard-drive-download';
	import Mail from '@lucide/svelte/icons/mail';
	import Send from '@lucide/svelte/icons/send';
	import Inbox from '@lucide/svelte/icons/inbox';
	import Calendar from '@lucide/svelte/icons/calendar';
	import FileType from '@lucide/svelte/icons/file-type';
	import Database from '@lucide/svelte/icons/database';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Calculator from '@lucide/svelte/icons/calculator';
	import Hash from '@lucide/svelte/icons/hash';
	import Fingerprint from '@lucide/svelte/icons/fingerprint';
	import Phone from '@lucide/svelte/icons/phone';
	import Workflow from '@lucide/svelte/icons/workflow';
	import type { Component } from 'svelte';

	const ICON_MAP: Record<string, Component<any>> = {
		'globe': Globe,
		'scissors': Scissors,
		'regex': Code,
		'braces': Braces,
		'code': Code,
		'file-code': FileCode,
		'shield-check': ShieldCheck,
		'type': Type,
		'list': List,
		'lock': Lock,
		'arrow-right-left': ArrowRightLeft,
		'git-branch': GitBranch,
		'repeat': Repeat,
		'clock': Clock,
		'terminal': Terminal,
		'file-text': FileText,
		'variable': Variable,
		'cookie': Cookie,
		'monitor': Monitor,
		'mouse-pointer-click': MousePointerClick,
		'keyboard': Keyboard,
		'hourglass': Hourglass,
		'scan-text': ScanText,
		'camera': Camera,
		'dices': Dices,
		'plug': Plug,
		'cpu': Cpu,
		'user': User,
		'shield': Shield,
		'scan-eye': ScanEye,
		'key': Key,
		'cloud': Cloud,
		'check-circle': CheckCircle,
		'list-tree': ListTree,
		'cable': Cable,
		'radio': Radio,
		'hard-drive-download': HardDriveDownload,
		'mail': Mail,
		'send': Send,
		'inbox': Inbox,
		'calendar': Calendar,
		'file-type': FileType,
		'database': Database,
		'book-open': BookOpen,
		'calculator': Calculator,
		'hash': Hash,
		'fingerprint': Fingerprint,
		'phone': Phone,
		'workflow': Workflow,
	};

	// Category colors for the expander header accent
	const CATEGORY_COLORS: Record<string, string> = {
		'Requests': '#0078d4',
		'Parsing': '#4ec9b0',
		'Checks': '#d7ba7d',
		'Functions': '#c586c0',
		'Control': '#dcdcaa',
		'Browser': '#e06c75',
		'Utilities': '#858585',
		'Bypass': '#e5c07b',
		'Sensors': '#2dd4bf',
	};

	let searchFilter = $state('');
	let expandedCategories = $state<Set<string>>(new Set(['Requests', 'Parsing', 'Checks', 'Functions', 'Control', 'Browser', 'Utilities', 'Bypass', 'Sensors']));

	// Auto-expand any category that appears after initial load (e.g. plugin categories)
	$effect(() => {
		const allCats = [...categories().keys()];
		const newCats = allCats.filter(c => !expandedCategories.has(c));
		if (newCats.length > 0) {
			expandedCategories = new Set([...expandedCategories, ...newCats]);
		}
	});

	const categories = $derived(() => {
		const cats = new Map<string, BlockMeta[]>();
		const allBlocks: BlockMeta[] = [
			...BLOCK_CATALOG,
			...app.pluginBlocks.map(pb => ({
				type: 'Plugin' as BlockType,
				label: pb.label,
				category: pb.category || 'Plugins',
				color: pb.color || '#9b59b6',
				icon: pb.icon || 'plug',
				_pluginBlockType: pb.block_type_name,
				_defaultSettingsJson: pb.default_settings_json,
			}))
		];
		const filtered = allBlocks.filter(b =>
			b.label.toLowerCase().includes(searchFilter.toLowerCase()) ||
			b.category.toLowerCase().includes(searchFilter.toLowerCase())
		);
		for (const block of filtered) {
			if (!cats.has(block.category)) cats.set(block.category, []);
			cats.get(block.category)!.push(block);
		}
		return cats;
	});

	function toggleCategory(cat: string) {
		const next = new Set(expandedCategories);
		if (next.has(cat)) next.delete(cat);
		else next.add(cat);
		expandedCategories = next;
	}

	// When searching, expand all matching categories
	$effect(() => {
		if (searchFilter) {
			expandedCategories = new Set([...categories().keys()]);
		}
	});

	function addBlock(block: BlockMeta) {
		const pb = block as any;
		if (pb._pluginBlockType) {
			send('add_block', {
				block_type: 'Plugin',
				plugin_block_type: pb._pluginBlockType,
				settings_json: pb._defaultSettingsJson || '{}',
				label: block.label,
			});
		} else {
			send('add_block', { block_type: block.type });
		}
	}

	function onDragStart(e: DragEvent, block: BlockMeta) {
		const pb = block as any;
		if (pb._pluginBlockType) {
			e.dataTransfer?.setData('text/plain', JSON.stringify({
				type: 'Plugin',
				plugin_block_type: pb._pluginBlockType,
				settings_json: pb._defaultSettingsJson || '{}',
				label: block.label,
			}));
		} else {
			e.dataTransfer?.setData('text/plain', block.type);
		}
	}
</script>

<div class="flex flex-col h-full bg-surface">
	<!-- Search -->
	<div class="p-1.5">
		<input
			type="text"
			placeholder="Search blocks..."
			class="w-full skeu-input text-[11px] font-mono"
			bind:value={searchFilter}
		/>
	</div>

	<!-- Block list -->
	<div class="flex-1 overflow-y-auto px-1 pb-2">
		{#each [...categories().entries()] as [category, blocks]}
			<div class="mb-0.5">
				<!-- Category expander header -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="flex items-center gap-1.5 px-1.5 py-1 cursor-pointer select-none rounded hover:bg-accent/20 transition-colors"
					onclick={() => toggleCategory(category)}
				>
					<div
						class="transition-transform duration-150 text-muted-foreground"
						style="transform: rotate({expandedCategories.has(category) ? '90deg' : '0deg'})"
					>
						<ChevronRight size={11} />
					</div>
					<div class="w-1.5 h-1.5 rounded-full shrink-0" style="background: {CATEGORY_COLORS[category] || '#858585'}"></div>
					<span class="text-[10px] uppercase tracking-wider text-muted-foreground font-semibold flex-1">{category}</span>
					<span class="text-[9px] text-muted-foreground/50 font-mono">{blocks.length}</span>
				</div>

				<!-- Collapsible block list -->
				{#if expandedCategories.has(category)}
					<div class="ml-2 border-l border-border/30 pl-0.5">
						{#each blocks as block}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="flex items-center gap-2 px-1.5 py-0.5 mx-0.5 rounded cursor-pointer hover:bg-accent/30 text-xs text-foreground hover-lift"
								draggable="true"
								ondragstart={(e) => onDragStart(e, block)}
								ondblclick={() => addBlock(block)}
								title="Drag to canvas or double-click to add"
							>
								{#if ICON_MAP[block.icon]}
									{@const IconComponent = ICON_MAP[block.icon]}
									<IconComponent size={12} style="color: {block.color}" />
								{:else}
									<div class="w-3 h-3 rounded-sm shrink-0" style="background: {block.color}"></div>
								{/if}
								<span>{block.label}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/each}

		<!-- Templates section -->
		{#if app.blockTemplates.length > 0}
			<div class="mt-2 pt-1 border-t border-border/30">
				<div class="flex items-center gap-1.5 px-1.5 py-1">
					<Bookmark size={11} class="text-muted-foreground" />
					<span class="text-[10px] uppercase tracking-wider text-muted-foreground font-semibold flex-1">Templates</span>
					<span class="text-[9px] text-muted-foreground/50 font-mono">{app.blockTemplates.length}</span>
				</div>
				<div class="ml-2 border-l border-border/30 pl-0.5">
					{#each app.blockTemplates as template, ti}
						<div class="group/tpl flex items-center gap-2 px-1.5 py-0.5 mx-0.5 rounded hover:bg-accent/30 text-xs text-foreground">
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="flex-1 flex items-center gap-2 cursor-pointer"
								ondblclick={() => { pushUndo(); send('paste_blocks', { blocks: JSON.parse(JSON.stringify(template.blocks)) }); }}
								title="Double-click to insert {template.blocks.length} block{template.blocks.length !== 1 ? 's' : ''}"
							>
								<Bookmark size={10} class="text-primary/60 shrink-0" />
								<span class="truncate">{template.name}</span>
								<span class="text-[9px] text-muted-foreground/50">{template.blocks.length}b</span>
							</div>
							<button
								class="p-0.5 rounded opacity-0 group-hover/tpl:opacity-100 hover:bg-destructive/20 text-muted-foreground hover:text-red transition-opacity"
								onclick={() => deleteBlockTemplate(ti)}
								title="Delete template"
							>
								<Trash2 size={10} />
							</button>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</div>
