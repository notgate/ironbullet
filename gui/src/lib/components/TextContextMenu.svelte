<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { send, sendAsync } from '$lib/ipc';
	import Copy from '@lucide/svelte/icons/copy';
	import ClipboardPaste from '@lucide/svelte/icons/clipboard-paste';
	import TextSelect from '@lucide/svelte/icons/text-select';

	let menu = $state<{ x: number; y: number; text: string; target: HTMLElement } | null>(null);
	let menuEl: HTMLDivElement | undefined = $state();

	function getSelectedOrElementText(target: HTMLElement): string {
		const sel = window.getSelection();
		if (sel && sel.toString().trim()) return sel.toString();
		return target.innerText || target.textContent || '';
	}

	function handleContextMenu(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target) return;

		// Don't override block context menu, job context menu, or buttons
		if (target.closest('[data-block-id], .block-renderer, .job-row, button, .skeu-btn')) return;

		// Only on: inputs, textareas, contenteditable, code blocks
		const isEditable = target.closest('textarea, input[type="text"], input:not([type]), [contenteditable="true"]');
		const isCodeBlock = target.closest('pre, code');
		const hasSelection = window.getSelection()?.toString().trim();

		if (isEditable || isCodeBlock || hasSelection) {
			e.preventDefault();
			e.stopPropagation();
			menu = { x: e.clientX, y: e.clientY, text: getSelectedOrElementText(target), target };
		}
	}

	function handleClickOutside(e: MouseEvent) {
		if (!menu) return;
		// If clicking inside the menu itself, don't close
		if (menuEl && menuEl.contains(e.target as Node)) return;
		menu = null;
	}

	function doCopy() {
		if (!menu?.text) { menu = null; return; }
		send('clipboard_copy', { text: menu.text });
		menu = null;
	}

	async function doPaste() {
		const target = menu?.target;
		menu = null;
		const resp = await sendAsync('clipboard_paste', 'clipboard_paste');
		if (!resp?.text || !target) return;
		const text = resp.text as string;
		if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
			target.focus();
			const start = target.selectionStart ?? target.value.length;
			const end = target.selectionEnd ?? target.value.length;
			target.value = target.value.slice(0, start) + text + target.value.slice(end);
			target.selectionStart = target.selectionEnd = start + text.length;
			target.dispatchEvent(new Event('input', { bubbles: true }));
		}
	}

	function doSelectAll() {
		const target = menu?.target;
		menu = null;
		if (!target) return;

		if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
			target.focus();
			target.select();
		} else {
			const container = target.closest('.panel-inset, .overflow-auto, pre, code') || target;
			const sel = window.getSelection();
			if (sel) {
				const range = document.createRange();
				range.selectNodeContents(container);
				sel.removeAllRanges();
				sel.addRange(range);
			}
		}
	}

	onMount(() => {
		document.addEventListener('contextmenu', handleContextMenu, true);
		document.addEventListener('mousedown', handleClickOutside, true);
	});

	onDestroy(() => {
		document.removeEventListener('contextmenu', handleContextMenu, true);
		document.removeEventListener('mousedown', handleClickOutside, true);
	});
</script>

{#if menu}
	<div
		bind:this={menuEl}
		class="fixed bg-popover border border-border rounded shadow-lg py-1 text-xs min-w-[130px] z-[201]"
		style="left:{menu.x}px;top:{menu.y}px"
	>
		<button class="w-full px-3 py-1.5 text-left hover:bg-accent/20 flex items-center gap-2" onclick={doCopy}>
			<Copy size={12} class="text-muted-foreground" /> Copy
		</button>
		<button class="w-full px-3 py-1.5 text-left hover:bg-accent/20 flex items-center gap-2" onclick={doPaste}>
			<ClipboardPaste size={12} class="text-muted-foreground" /> Paste
		</button>
		<div class="border-t border-border/50 my-0.5"></div>
		<button class="w-full px-3 py-1.5 text-left hover:bg-accent/20 flex items-center gap-2" onclick={doSelectAll}>
			<TextSelect size={12} class="text-muted-foreground" /> Select All
		</button>
	</div>
{/if}
