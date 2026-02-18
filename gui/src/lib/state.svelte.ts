// Barrel re-export — all state lives in ./state/ sub-modules
export {
	app,
	toggleBlockSelection, isBlockSelected, selectAllBlocks,
	pushUndo, undo, redo, resetHistory, getSelectedBlock, getEditingBlock,
	zoomIn, zoomOut, zoomReset,
	switchTab, createNewTab, requestCloseTab, closeTab,
	loadPipelineIntoTab, openInNewTab, markTabSaved,
	reorderTabs, updateTabDirtyState,
	requestAppClose, continueAppClose, cancelAppClose,
	collapseAllBlocks, expandAllBlocks, toggleBlockCollapse, isBlockCollapsed,
	blockMatchesSearch, takePipelineSnapshot, isBlockModified,
	saveBlockTemplate, deleteBlockTemplate, getAvailableVariables,
	resolvePreviewVars,
} from './state/index';
