import { app } from './app.svelte';

export function toggleBlockSelection(blockId: string, ctrl: boolean, shift: boolean) {
	if (shift && app.selectedBlockIds.length > 0) {
		// Range select: from last selected to clicked
		const lastId = app.selectedBlockIds[app.selectedBlockIds.length - 1];
		const lastIdx = app.pipeline.blocks.findIndex(b => b.id === lastId);
		const clickIdx = app.pipeline.blocks.findIndex(b => b.id === blockId);
		if (lastIdx === -1 || clickIdx === -1) {
			// Fallback to single select if indices not found
			app.selectedBlockIds = [blockId];
		} else {
			const [from, to] = lastIdx < clickIdx ? [lastIdx, clickIdx] : [clickIdx, lastIdx];
			app.selectedBlockIds = app.pipeline.blocks.slice(from, to + 1).map(b => b.id);
		}
	} else if (ctrl) {
		// Toggle individual - use Set for O(1) operations
		const set = new Set(app.selectedBlockIds);
		if (set.has(blockId)) {
			set.delete(blockId);
		} else {
			set.add(blockId);
		}
		app.selectedBlockIds = Array.from(set);
	} else {
		// Single select
		app.selectedBlockIds = [blockId];
	}
}

export function isBlockSelected(blockId: string): boolean {
	// Direct check - reactive to app.selectedBlockIds changes
	return app.selectedBlockIds.includes(blockId);
}

export function selectAllBlocks() {
	app.selectedBlockIds = app.pipeline.blocks.map(b => b.id);
}
