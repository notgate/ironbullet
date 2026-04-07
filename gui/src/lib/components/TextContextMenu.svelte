<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	let menu = $state<{ x: number; y: number; text: string } | null>(null);

	function getSelectedOrElementText(target: HTMLElement): string {
		const sel = window.getSelection();
		if (sel && sel.toString().trim()) return sel.toString();
		return target.innerText || target.textContent || '';
	}

	function handleContextMenu(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target) return;

		// Only show on text-bearing elements, not on buttons/inputs with existing menus
		const isTextElement = target.closest('pre, code, .font-mono, textarea, [contenteditable]') ||
			target.matches('pre, code, .font-mono, span, td, textarea') ||
			target.closest('.panel-inset, .overflow-auto, .response-body, .var-inspector-list');

		// Don't override block context menu or job context menu
		const isBlockCtx = target.closest('[data-block-id], .block-renderer');
		const isJobCtx = target.closest('.job-row');

		if (isTextElement && !isBlockCtx && !isJobCtx) {
			e.preventDefault();
			e.stopPropagation();
			const text = getSelectedOrElementText(target);
			menu = { x: e.clientX, y: e.clientY, text };
		}
	}

	async function copy() {
		if (menu?.text) {
			try { await navigator.clipboard.writeText(menu.text); } catch {}
		}
		menu = null;
	}

	async function paste() {
		try {
			const text = await navigator.clipboard.readText();
			const el = document.activeElement as HTMLInputElement | HTMLTextAreaElement;
			if (el && ('value' in el)) {
				const start = el.selectionStart ?? el.value.length;
				const end = el.selectionEnd ?? el.value.length;
				el.value = el.value.slice(0, start) + text + el.value.slice(end);
				el.selectionStart = el.selectionEnd = start + text.length;
				el.dispatchEvent(new Event('input', { bubbles: true }));
			}
		} catch {}
		menu = null;
	}

	function selectAll() {
		const sel = window.getSelection();
		if (sel) {
			const range = document.createRange();
			range.selectNodeContents(document.body);
			sel.removeAllRanges();
			sel.addRange(range);
		}
		menu = null;
	}

	onMount(() => {
		document.addEventListener('contextmenu', handleContextMenu, true);
	});

	onDestroy(() => {
		document.removeEventListener('contextmenu', handleContextMenu, true);
	});
</script>

{#if menu}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-[9998]" onclick={() => menu = null} oncontextmenu={(e) => { e.preventDefault(); menu = null; }}>
		<div
			class="fixed bg-popover border border-border rounded shadow-lg py-1 text-xs min-w-[120px] z-[9999]"
			style="left:{menu.x}px;top:{menu.y}px"
		>
			<button class="w-full px-3 py-1 text-left hover:bg-accent/20 flex items-center gap-2" onclick={copy}>
				<span class="text-muted-foreground text-[10px] w-4">&#xe8c8;</span> Copy
			</button>
			<button class="w-full px-3 py-1 text-left hover:bg-accent/20 flex items-center gap-2" onclick={paste}>
				<span class="text-muted-foreground text-[10px] w-4">&#xe85d;</span> Paste
			</button>
			<div class="border-t border-border/50 my-0.5"></div>
			<button class="w-full px-3 py-1 text-left hover:bg-accent/20 flex items-center gap-2" onclick={selectAll}>
				Select All
			</button>
		</div>
	</div>
{/if}
