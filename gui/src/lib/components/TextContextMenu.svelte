<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import Copy from '@lucide/svelte/icons/copy';
	import ClipboardPaste from '@lucide/svelte/icons/clipboard-paste';
	import TextSelect from '@lucide/svelte/icons/text-select';

	let menu = $state<{ x: number; y: number; text: string; target: HTMLElement } | null>(null);

	function getSelectedOrElementText(target: HTMLElement): string {
		const sel = window.getSelection();
		if (sel && sel.toString().trim()) return sel.toString();
		return target.innerText || target.textContent || '';
	}

	function handleContextMenu(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target) return;

		// Don't override block context menu or job context menu
		if (target.closest('[data-block-id], .block-renderer, .job-row')) return;

		// Show on text-bearing elements
		const isText = target.closest('pre, code, .font-mono, textarea, input, [contenteditable]') ||
			target.matches('pre, code, .font-mono, span, td, p, div') ||
			target.closest('.panel-inset, .overflow-auto');

		if (isText) {
			e.preventDefault();
			e.stopPropagation();
			menu = { x: e.clientX, y: e.clientY, text: getSelectedOrElementText(target), target };
		}
	}

	function doCopy() {
		if (!menu?.text) { menu = null; return; }
		// Create a temporary textarea to use execCommand (no permissions popup)
		const ta = document.createElement('textarea');
		ta.value = menu.text;
		ta.style.position = 'fixed';
		ta.style.left = '-9999px';
		document.body.appendChild(ta);
		ta.select();
		document.execCommand('copy');
		document.body.removeChild(ta);
		menu = null;
	}

	function doPaste() {
		// Focus the original target and execute paste command
		const target = menu?.target;
		menu = null;
		if (target && (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement)) {
			target.focus();
			document.execCommand('paste');
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
			// Select text within the closest scrollable container or the element itself
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

	function close() { menu = null; }

	onMount(() => {
		document.addEventListener('contextmenu', handleContextMenu, true);
	});

	onDestroy(() => {
		document.removeEventListener('contextmenu', handleContextMenu, true);
	});
</script>

{#if menu}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-[200]" onclick={close} oncontextmenu={(e) => { e.preventDefault(); close(); }}>
		<div
			class="fixed bg-popover border border-border rounded shadow-lg py-1 text-xs min-w-[130px] z-[201]"
			style="left:{menu.x}px;top:{menu.y}px"
			onclick={(e) => e.stopPropagation()}
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
	</div>
{/if}
