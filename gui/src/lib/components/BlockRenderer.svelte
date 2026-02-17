<script lang="ts">
	import { getBlockCssClass, getBlockColor, type Block } from '$lib/types';
	import { app, pushUndo, toggleBlockSelection, isBlockSelected } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import Eye from '@lucide/svelte/icons/eye';
	import EyeOff from '@lucide/svelte/icons/eye-off';
	import X from '@lucide/svelte/icons/x';

	let { block, index }: { block: Block; index: number } = $props();

	let isSelected = $derived(isBlockSelected(block.id));
	let isRenaming = $derived(app.renamingBlockId === block.id);
	let cssClass = $derived(getBlockCssClass(block.block_type));
	let color = $derived(getBlockColor(block.block_type));

	let renameValue = $state('');

	function handleClick(e: MouseEvent) {
		const isMultiSelect = e.ctrlKey || e.metaKey || e.shiftKey;
		toggleBlockSelection(block.id, e.ctrlKey || e.metaKey, e.shiftKey);
		if (!isMultiSelect) {
			app.editingBlockId = block.id;
		}
	}

	function onDragStart(e: DragEvent) {
		e.dataTransfer?.setData('application/x-block-index', String(index));
		e.dataTransfer?.setData('application/x-block-id', block.id);
		e.dataTransfer!.effectAllowed = 'move';
	}

	function onContextMenu(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();
		// If the right-clicked block isn't already selected, single-select it
		if (!isBlockSelected(block.id)) {
			toggleBlockSelection(block.id, false, false);
		}
		app.contextMenu = { x: e.clientX, y: e.clientY, blockId: block.id, blockIndex: index };
	}

	function toggleDisabled(e: MouseEvent) {
		e.stopPropagation();
		pushUndo();
		block.disabled = !block.disabled;
		send('update_block', block);
	}

	function removeBlock(e: MouseEvent) {
		e.stopPropagation();
		pushUndo();
		send('remove_block', { block_id: block.id });
		app.selectedBlockIds = app.selectedBlockIds.filter(id => id !== block.id);
		if (app.editingBlockId === block.id) app.editingBlockId = null;
	}

	function commitRename() {
		if (renameValue.trim() && renameValue !== block.label) {
			pushUndo();
			const updated = { ...block, label: renameValue.trim() };
			send('update_block', updated);
		}
		app.renamingBlockId = null;
	}

	function cancelRename() {
		app.renamingBlockId = null;
	}

	function onRenameKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') { e.preventDefault(); commitRename(); }
		else if (e.key === 'Escape') { e.preventDefault(); cancelRename(); }
	}

	function getBlockSummary(): string {
		const s = block.settings;
		switch (s.type) {
			case 'HttpRequest': return `${s.method} ${s.url || '(no URL)'}`;
			case 'ParseLR': return `"${s.left}" ... "${s.right}" → ${s.output_var}`;
			case 'ParseRegex': return `/${s.pattern}/ → ${s.output_var}`;
			case 'ParseJSON': return `${s.json_path} → ${s.output_var}`;
			case 'ParseCSS': return `${s.selector}${s.attribute ? `[${s.attribute}]` : ''} → ${s.output_var}`;
			case 'ParseXPath': return `${s.xpath} → ${s.output_var}`;
			case 'ParseCookie': return `cookie "${s.cookie_name}" → ${s.output_var}`;
			case 'KeyCheck': return s.keychains.map((k: { result: string }) => k.result).join(', ');
			case 'StringFunction': return `${s.function_type}(${s.input_var}) → ${s.output_var}`;
			case 'ListFunction': return `${s.function_type}(${s.input_var}) → ${s.output_var}`;
			case 'CryptoFunction': return `${s.function_type}(${s.input_var}) → ${s.output_var}`;
			case 'ConversionFunction': return `${s.from_type} → ${s.to_type}: ${s.input_var}`;
			case 'IfElse': return `${s.condition.source} ${s.condition.comparison} ${s.condition.value}`;
			case 'Loop': return s.loop_type === 'ForEach' ? `each ${s.list_var} as ${s.item_var}` : `repeat ${s.count}x`;
			case 'Script': return s.code.split('\n')[0]?.substring(0, 40) || '(empty)';
			case 'SetVariable': return `${s.name} = ${s.value}`;
			case 'Delay': return `${s.min_ms}${s.min_ms !== s.max_ms ? `-${s.max_ms}` : ''}ms`;
			case 'Log': return s.message;
			case 'ClearCookies': return 'clear session cookies';
			case 'Webhook': return `${s.method} ${s.url || '(no URL)'}`;
			case 'WebSocket': return `${s.action} ${s.url || '(no URL)'}`;
			case 'TcpRequest': return `${s.host}:${s.port}${s.use_tls ? ' (TLS)' : ''}`;
			case 'UdpRequest': return `${s.host}:${s.port}`;
			case 'FtpRequest': return `${s.host}:${s.port} ${s.command}`;
			case 'SshRequest': return `${s.host}:${s.port} ${s.command?.substring(0, 30) || ''}`;
			case 'ImapRequest': return `${s.host}:${s.port} ${s.command}`;
			case 'SmtpRequest': return `${s.host}:${s.port} ${s.command}`;
			case 'PopRequest': return `${s.host}:${s.port} ${s.command}`;
			case 'CaptchaSolver': return `${s.solver_service} ${s.captcha_type} → ${s.output_var}`;
			case 'CloudflareBypass': return `${s.url || '(no URL)'} via ${s.flaresolverr_url}`;
			case 'LaravelCsrf': return `${s.url || '(no URL)'} → ${s.output_var}`;
			case 'DateFunction': return `${s.function_type}${s.input_var ? `(${s.input_var})` : ''} → ${s.output_var}`;
			case 'CaseSwitch': return `switch ${s.input_var} (${s.cases?.length || 0} cases) → ${s.output_var}`;
			case 'CookieContainer': return `${s.source_type === 'file' ? s.source : 'text'} → ${s.output_var}`;
			case 'BrowserOpen': return `${s.browser_type}${s.headless ? ' (headless)' : ''}`;
			case 'NavigateTo': return s.url || '(no URL)';
			case 'ClickElement': return s.selector || '(no selector)';
			case 'TypeText': return `${s.selector} ← "${s.text?.substring(0, 20) || ''}"`;
			case 'WaitForElement': return `${s.selector} [${s.state}]`;
			case 'GetElementText': return `${s.selector} → ${s.output_var}`;
			case 'Screenshot': return s.full_page ? 'full page' : (s.selector || 'viewport');
			case 'ExecuteJs': return s.code?.split('\n')[0]?.substring(0, 40) || '(empty)';
			case 'Group': {
				const n = (s as any).blocks?.length || 0;
				return `${n} block${n !== 1 ? 's' : ''}${(s as any).collapsed ? ' (collapsed)' : ''}`;
			}
			default: return '';
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="group {cssClass} bg-card rounded mb-0.5 transition-all cursor-pointer {block.disabled ? 'opacity-40' : ''}"
	style="box-shadow: var(--btn-raised); {isSelected ? 'outline: 1.5px solid var(--primary); outline-offset: -1px;' : ''}"
	data-block-id={block.id}
	onclick={handleClick}
	oncontextmenu={onContextMenu}
	draggable="true"
	ondragstart={onDragStart}
>
	<div class="flex items-center gap-2 px-2 py-1.5">
		<span class="text-[10px] text-muted-foreground w-4 text-right shrink-0">{index + 1}</span>
		<div class="w-2.5 h-2.5 rounded-sm shrink-0" style="background: {color}"></div>

		{#if isRenaming}
			<!-- svelte-ignore a11y_autofocus -->
			<input
				type="text"
				bind:value={renameValue}
				class="flex-1 skeu-input text-xs font-medium py-0"
				onblur={commitRename}
				onkeydown={onRenameKeydown}
				onclick={(e) => e.stopPropagation()}
				autofocus
			/>
		{:else}
			<span class="text-xs font-medium text-foreground flex-1 truncate">{block.label}</span>
		{/if}

		<span class="text-[10px] text-muted-foreground truncate max-w-[200px] hidden lg:block">
			{getBlockSummary()}
		</span>

		<div class="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
			<button
				class="p-0.5 rounded hover:bg-secondary text-muted-foreground hover:text-foreground"
				onclick={toggleDisabled}
				title={block.disabled ? 'Enable' : 'Disable'}
			>
				{#if block.disabled}<EyeOff size={12} />{:else}<Eye size={12} />{/if}
			</button>
			<button
				class="p-0.5 rounded hover:bg-destructive/20 text-muted-foreground hover:text-red"
				onclick={removeBlock}
				title="Remove"
			>
				<X size={12} />
			</button>
		</div>
	</div>
</div>
