import { app, undo, redo, pushUndo, zoomIn, zoomOut, zoomReset, selectAllBlocks, createNewTab } from '$lib/state.svelte';
import { send } from '$lib/ipc';
import type { Block } from '$lib/types';

/**
 * Check if focus is in a text-editable element (input, textarea, contenteditable, Monaco).
 */
export function isEditableFocused(): boolean {
	const el = document.activeElement;
	if (!el) return false;
	const tag = el.tagName;
	if (tag === 'INPUT' || tag === 'TEXTAREA') return true;
	if ((el as HTMLElement).isContentEditable) return true;
	// Monaco editor uses a textarea inside .monaco-editor
	if (el.closest?.('.monaco-editor')) return true;
	return false;
}

/**
 * Create the global keydown handler.
 *
 * `getClipboard` / `setClipboard` give the handler access to the block
 * clipboard that lives in the page component (local state, not global).
 */
export function createKeydownHandler(
	getClipboard: () => Block[],
	setClipboard: (blocks: Block[]) => void,
): (e: KeyboardEvent) => void {
	return function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && app.contextMenu) { app.contextMenu = null; return; }

		const editable = isEditableFocused();

		// Global shortcuts that always work (even in text inputs)
		if (e.ctrlKey && e.key === 's') { e.preventDefault(); send('save_pipeline', {}); return; }
		if (e.ctrlKey && e.key === 'o') { e.preventDefault(); send('load_pipeline'); return; }
		if (e.key === 'F1') { e.preventDefault(); app.showBlockDocs = true; app.blockDocsInitialType = null; return; }
		if (e.ctrlKey && e.key === 'f') { e.preventDefault(); app.pipelineSearchFocused = true; return; }
		if (e.key === 'F5') { e.preventDefault(); send('debug_pipeline'); return; }
		if (e.ctrlKey && (e.key === '=' || e.key === '+')) { e.preventDefault(); zoomIn(); return; }
		if (e.ctrlKey && e.key === '-') { e.preventDefault(); zoomOut(); return; }
		if (e.ctrlKey && e.key === '0') { e.preventDefault(); zoomReset(); return; }
		if (e.ctrlKey && e.key === 't') { e.preventDefault(); createNewTab(); return; }

		// Block-editing shortcuts — only when NOT focused on a text input
		if (editable) return;

		if (e.ctrlKey && e.key === 'z') { e.preventDefault(); undo(); }
		else if (e.ctrlKey && e.key === 'y') { e.preventDefault(); redo(); }
		else if (e.key === 'Delete' && app.selectedBlockIds.length > 0) {
			pushUndo();
			send('remove_blocks', { ids: [...app.selectedBlockIds] });
			if (app.editingBlockId && app.selectedBlockIds.includes(app.editingBlockId)) app.editingBlockId = null;
			app.selectedBlockIds = [];
		}
		else if (e.ctrlKey && e.key === 'a') {
			e.preventDefault();
			selectAllBlocks();
		}
		else if (e.ctrlKey && e.key === 'c' && app.selectedBlockIds.length > 0) {
			e.preventDefault();
			const selected = app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id));
			setClipboard(JSON.parse(JSON.stringify(selected)));
		}
		else if (e.ctrlKey && e.key === 'x' && app.selectedBlockIds.length > 0) {
			// Cut: copy to clipboard then remove from pipeline
			e.preventDefault();
			const selected = app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id));
			setClipboard(JSON.parse(JSON.stringify(selected)));
			pushUndo();
			send('remove_blocks', { ids: [...app.selectedBlockIds] });
			if (app.editingBlockId && app.selectedBlockIds.includes(app.editingBlockId)) app.editingBlockId = null;
			app.selectedBlockIds = [];
		}
		else if (e.ctrlKey && e.key === 'v' && getClipboard().length > 0) {
			e.preventDefault();
			pushUndo();
			send('paste_blocks', { blocks: JSON.parse(JSON.stringify(getClipboard())) });
		}
		else if (e.ctrlKey && e.key === 'd' && app.selectedBlockIds.length > 0) {
			e.preventDefault();
			const selected = app.pipeline.blocks.filter(b => app.selectedBlockIds.includes(b.id));
			pushUndo();
			send('paste_blocks', { blocks: JSON.parse(JSON.stringify(selected)) });
		}
		else if (e.key === 'F2' && app.selectedBlockIds.length === 1) {
			e.preventDefault();
			app.renamingBlockId = app.selectedBlockIds[0];
		}
	};
}
