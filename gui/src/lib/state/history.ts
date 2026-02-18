import type { Block } from '$lib/types';
import { app } from './app.svelte';

const MAX_HISTORY = 50;

interface HistoryEntry {
	blocks: Block[];
	selectedBlockIds: string[];
}

let _undoStack: HistoryEntry[] = [];
let _redoStack: HistoryEntry[] = [];

function cloneBlocks(blocks: Block[]): Block[] {
	return JSON.parse(JSON.stringify(blocks));
}

/** Push current state onto undo stack before a mutation. Call this BEFORE changing blocks. */
export function pushUndo() {
	_undoStack.push({
		blocks: cloneBlocks(app.pipeline.blocks),
		selectedBlockIds: [...app.selectedBlockIds],
	});
	if (_undoStack.length > MAX_HISTORY) {
		_undoStack.shift();
	}
	_redoStack = [];
	app.canUndo = _undoStack.length > 0;
	app.canRedo = false;
}

export function undo() {
	if (_undoStack.length === 0) return;
	const entry = _undoStack.pop()!;
	_redoStack.push({
		blocks: cloneBlocks(app.pipeline.blocks),
		selectedBlockIds: [...app.selectedBlockIds],
	});
	app.pipeline.blocks = entry.blocks;
	app.selectedBlockIds = entry.selectedBlockIds;
	app.canUndo = _undoStack.length > 0;
	app.canRedo = _redoStack.length > 0;
}

export function redo() {
	if (_redoStack.length === 0) return;
	const entry = _redoStack.pop()!;
	_undoStack.push({
		blocks: cloneBlocks(app.pipeline.blocks),
		selectedBlockIds: [...app.selectedBlockIds],
	});
	app.pipeline.blocks = entry.blocks;
	app.selectedBlockIds = entry.selectedBlockIds;
	app.canUndo = _undoStack.length > 0;
	app.canRedo = _redoStack.length > 0;
}

/** Reset undo/redo stacks (called on tab switch) */
export function resetHistory() {
	_undoStack = [];
	_redoStack = [];
	app.canUndo = false;
	app.canRedo = false;
}

export function getSelectedBlock(): Block | null {
	if (app.selectedBlockIds.length === 0) return null;
	return app.pipeline.blocks.find(b => b.id === app.selectedBlockIds[0]) || null;
}

function findBlockRecursive(blocks: Block[], id: string): Block | null {
	for (const b of blocks) {
		if (b.id === id) return b;
		if (b.settings.type === 'IfElse') {
			const found = findBlockRecursive(b.settings.true_blocks, id) || findBlockRecursive(b.settings.false_blocks, id);
			if (found) return found;
		}
		if (b.settings.type === 'Loop') {
			const found = findBlockRecursive(b.settings.blocks, id);
			if (found) return found;
		}
		if (b.settings.type === 'Group') {
			const found = findBlockRecursive(b.settings.blocks, id);
			if (found) return found;
		}
	}
	return null;
}

export function getEditingBlock(): Block | null {
	if (!app.editingBlockId) return null;
	return findBlockRecursive(app.pipeline.blocks, app.editingBlockId);
}
