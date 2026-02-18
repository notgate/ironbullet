import { app } from './app.svelte';

export function toggleBlockSelection(blockId: string, ctrl: boolean, shift: boolean) {
	if (shift && app.selectedBlockIds.length > 0) {
		// Range select: from last selected to clicked
		const lastId = app.selectedBlockIds[app.selectedBlockIds.length - 1];
		const lastIdx = app.pipeline.blocks.findIndex(b => b.id === lastId);
		const clickIdx = app.pipeline.blocks.findIndex(b => b.id === blockId);
		const [from, to] = lastIdx < clickIdx ? [lastIdx, clickIdx] : [clickIdx, lastIdx];
		app.selectedBlockIds = app.pipeline.blocks.slice(from, to + 1).map(b => b.id);
	} else if (ctrl) {
		// Toggle individual
		const idx = app.selectedBlockIds.indexOf(blockId);
		if (idx >= 0) {
			app.selectedBlockIds = app.selectedBlockIds.filter(id => id !== blockId);
		} else {
			app.selectedBlockIds = [...app.selectedBlockIds, blockId];
		}
	} else {
		// Single select
		app.selectedBlockIds = [blockId];
	}
}

export function isBlockSelected(blockId: string): boolean {
	return app.selectedBlockIds.includes(blockId);
}

export function selectAllBlocks() {
	app.selectedBlockIds = app.pipeline.blocks.map(b => b.id);
}
