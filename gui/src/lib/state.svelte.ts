import type { Pipeline, RunnerStats, BlockResult, Block, NetworkEntry, ConfigTab, Job, PluginBlockMeta, PluginMeta } from './types';

const MAX_HISTORY = 50;

interface HistoryEntry {
	blocks: Block[];
	selectedBlockIds: string[];
}

interface AppState {
	// Pipeline
	pipeline: Pipeline;
	selectedBlockIds: string[];

	// UI state
	activeTab: string;
	bottomTab: string;
	showBlockPalette: boolean;
	bottomPanelHeight: number;
	leftPanelWidth: number;
	zoom: number;
	editingBlockId: string | null;

	// Config
	config: Record<string, unknown>;

	// Runner
	runnerStats: RunnerStats | null;
	isRunning: boolean;
	isPaused: boolean;
	threadCount: number;
	hits: Array<{ data_line: string; captures: Record<string, string>; proxy: string | null }>;

	// Debug
	debugResult: BlockResult | null;
	debugResults: BlockResult[];
	debugLog: string[];
	networkLog: NetworkEntry[];

	// Response viewer
	showResponseViewer: boolean;

	// Code view
	generatedCode: string;

	// File paths
	wordlistPath: string;
	proxyPath: string;

	// Status
	statusText: string;

	// UI modals
	showSettings: boolean;
	showStartup: boolean;
	renamingBlockId: string | null;

	// Recent configs
	recentConfigs: Array<{ path: string; name: string; description: string; last_opened: string }>;

	// Context menu
	contextMenu: { x: number; y: number; blockId: string; blockIndex: number } | null;

	// Appearance
	fontFamily: string;
	fontSize: number;

	// Paths
	collectionsPath: string;
	defaultWordlistPath: string;
	defaultProxyPath: string;
	collectionConfigs: Array<{ path: string; name: string }>;

	// Undo/Redo
	canUndo: boolean;
	canRedo: boolean;

	// Compat shim — single selected block ID (first of multi-select)
	selectedBlockId: string | null;

	// Config tabs
	configTabs: ConfigTab[];
	activeTabId: string;
	showUnsavedDialog: boolean;
	pendingCloseTabId: string | null;

	// Jobs
	jobs: Job[];
	activeJobId: string | null;

	// Plugins
	pluginBlocks: PluginBlockMeta[];
	pluginMetas: PluginMeta[];

	// Block docs
	showBlockDocs: boolean;
	blockDocsInitialType: string | null;

	// Plugin Builder
	showPluginBuilder: boolean;

	// Security
	securityIssues: Array<{ severity: string; title: string; description: string; code_snippet: string }>;

	// Internal callback for compile output (not persisted)
	_compileOutputCallback: ((data: { line: string; done: boolean; success: boolean; dll_path?: string }) => void) | null;
}

function createAppState(): AppState {
	let pipeline = $state<Pipeline>({
		id: crypto.randomUUID(),
		name: 'New Config 1',
		author: '',
		created: new Date().toISOString(),
		modified: new Date().toISOString(),
		blocks: [],
		startup_blocks: [],
		data_settings: { wordlist_type: 'Credentials', separator: ':', slices: ['USER', 'PASS'] },
		proxy_settings: { proxy_mode: 'None', proxy_sources: [], ban_duration_secs: 300, max_retries_before_ban: 3, cpm_per_proxy: 0, proxy_groups: [], active_group: '' },
		browser_settings: { browser: 'chrome', ja3: null, http2_fingerprint: null, user_agent: null },
		runner_settings: { threads: 100, skip: 0, take: 0, continue_statuses: ['Retry'], custom_status_name: 'CUSTOM', max_retries: 3, concurrent_per_proxy: 0, start_threads_gradually: true, gradual_delay_ms: 100, automatic_thread_count: false, lower_threads_on_retry: false, retry_thread_reduction_pct: 25, pause_on_ratelimit: false, only_proxyless: false },
		output_settings: { save_to_file: true, save_to_database: false, include_response: false, output_directory: 'results', output_format: '{data} | {captures}', database_path: '', output_format_type: 'Txt', capture_filters: [] },
	});

	let selectedBlockIds = $state<string[]>([]);
	let activeTab = $state('editor');
	let bottomTab = $state('debugger');
	let showBlockPalette = $state(true);
	let bottomPanelHeight = $state(250);
	let leftPanelWidth = $state(200);
	let zoom = $state(0.8);
	let editingBlockId = $state<string | null>(null);
	let config = $state<Record<string, unknown>>({});
	let runnerStats = $state<RunnerStats | null>(null);
	let isRunning = $state(false);
	let isPaused = $state(false);
	let threadCount = $state(100);
	let hits = $state<Array<{ data_line: string; captures: Record<string, string>; proxy: string | null }>>([]);
	let debugResult = $state<BlockResult | null>(null);
	let debugResults = $state<BlockResult[]>([]);
	let debugLog = $state<string[]>([]);
	let networkLog = $state<NetworkEntry[]>([]);
	let showResponseViewer = $state(false);
	let generatedCode = $state('');
	let wordlistPath = $state('');
	let proxyPath = $state('');
	let statusText = $state('Ready');
	let showSettings = $state(false);
	let showStartup = $state(true);
	let renamingBlockId = $state<string | null>(null);
	let recentConfigs = $state<Array<{ path: string; name: string; description: string; last_opened: string }>>([]);
	let contextMenu = $state<{ x: number; y: number; blockId: string; blockIndex: number } | null>(null);
	let fontFamily = $state('Cascadia Code');
	let fontSize = $state(12);
	let collectionsPath = $state('');
	let defaultWordlistPath = $state('');
	let defaultProxyPath = $state('');
	let collectionConfigs = $state<Array<{ path: string; name: string }>>([]);

	// Undo/Redo history
	let canUndo = $state(false);
	let canRedo = $state(false);

	// Config tabs
	const initialTabId = crypto.randomUUID();
	let configTabs = $state<ConfigTab[]>([{
		id: initialTabId,
		name: 'New Config 1',
		filePath: null,
		pipeline: pipeline,
		isDirty: false,
		savedSnapshot: '',
	}]);
	let activeTabId = $state(initialTabId);
	let showUnsavedDialog = $state(false);
	let pendingCloseTabId = $state<string | null>(null);
	let jobs = $state<Job[]>([]);
	let activeJobId = $state<string | null>(null);
	let pluginBlocks = $state<PluginBlockMeta[]>([]);
	let pluginMetas = $state<PluginMeta[]>([]);
	let showBlockDocs = $state(false);
	let blockDocsInitialType = $state<string | null>(null);
	let showPluginBuilder = $state(false);
	let securityIssues = $state<Array<{ severity: string; title: string; description: string; code_snippet: string }>>([]);

	return {
		get pipeline() { return pipeline; },
		set pipeline(v) { pipeline = v; },
		get selectedBlockIds() { return selectedBlockIds; },
		set selectedBlockIds(v) { selectedBlockIds = v; },
		// Compat shim: selectedBlockId maps to first element of selectedBlockIds
		get selectedBlockId() { return selectedBlockIds.length > 0 ? selectedBlockIds[0] : null; },
		set selectedBlockId(v) { selectedBlockIds = v ? [v] : []; },
		get activeTab() { return activeTab; },
		set activeTab(v) { activeTab = v; },
		get bottomTab() { return bottomTab; },
		set bottomTab(v) { bottomTab = v; },
		get showBlockPalette() { return showBlockPalette; },
		set showBlockPalette(v) { showBlockPalette = v; },
		get bottomPanelHeight() { return bottomPanelHeight; },
		set bottomPanelHeight(v) { bottomPanelHeight = v; },
		get leftPanelWidth() { return leftPanelWidth; },
		set leftPanelWidth(v) { leftPanelWidth = v; },
		get zoom() { return zoom; },
		set zoom(v) { zoom = v; },
		get editingBlockId() { return editingBlockId; },
		set editingBlockId(v) { editingBlockId = v; },
		get config() { return config; },
		set config(v) { config = v; },
		get runnerStats() { return runnerStats; },
		set runnerStats(v) { runnerStats = v; },
		get isRunning() { return isRunning; },
		set isRunning(v) { isRunning = v; },
		get isPaused() { return isPaused; },
		set isPaused(v) { isPaused = v; },
		get threadCount() { return threadCount; },
		set threadCount(v) { threadCount = v; },
		get hits() { return hits; },
		set hits(v) { hits = v; },
		get debugResult() { return debugResult; },
		set debugResult(v) { debugResult = v; },
		get debugResults() { return debugResults; },
		set debugResults(v) { debugResults = v; },
		get debugLog() { return debugLog; },
		set debugLog(v) { debugLog = v; },
		get networkLog() { return networkLog; },
		set networkLog(v) { networkLog = v; },
		get showResponseViewer() { return showResponseViewer; },
		set showResponseViewer(v) { showResponseViewer = v; },
		get generatedCode() { return generatedCode; },
		set generatedCode(v) { generatedCode = v; },
		get wordlistPath() { return wordlistPath; },
		set wordlistPath(v) { wordlistPath = v; },
		get proxyPath() { return proxyPath; },
		set proxyPath(v) { proxyPath = v; },
		get statusText() { return statusText; },
		set statusText(v) { statusText = v; },
		get showSettings() { return showSettings; },
		set showSettings(v) { showSettings = v; },
		get showStartup() { return showStartup; },
		set showStartup(v) { showStartup = v; },
		get renamingBlockId() { return renamingBlockId; },
		set renamingBlockId(v) { renamingBlockId = v; },
		get recentConfigs() { return recentConfigs; },
		set recentConfigs(v) { recentConfigs = v; },
		get contextMenu() { return contextMenu; },
		set contextMenu(v) { contextMenu = v; },
		get fontFamily() { return fontFamily; },
		set fontFamily(v) { fontFamily = v; },
		get fontSize() { return fontSize; },
		set fontSize(v) { fontSize = v; },
		get canUndo() { return canUndo; },
		set canUndo(v: boolean) { canUndo = v; },
		get canRedo() { return canRedo; },
		set canRedo(v: boolean) { canRedo = v; },
		get collectionsPath() { return collectionsPath; },
		set collectionsPath(v) { collectionsPath = v; },
		get defaultWordlistPath() { return defaultWordlistPath; },
		set defaultWordlistPath(v) { defaultWordlistPath = v; },
		get defaultProxyPath() { return defaultProxyPath; },
		set defaultProxyPath(v) { defaultProxyPath = v; },
		get collectionConfigs() { return collectionConfigs; },
		set collectionConfigs(v) { collectionConfigs = v; },
		get configTabs() { return configTabs; },
		set configTabs(v) { configTabs = v; },
		get activeTabId() { return activeTabId; },
		set activeTabId(v) { activeTabId = v; },
		get showUnsavedDialog() { return showUnsavedDialog; },
		set showUnsavedDialog(v) { showUnsavedDialog = v; },
		get pendingCloseTabId() { return pendingCloseTabId; },
		set pendingCloseTabId(v) { pendingCloseTabId = v; },
		get jobs() { return jobs; },
		set jobs(v) { jobs = v; },
		get activeJobId() { return activeJobId; },
		set activeJobId(v) { activeJobId = v; },
		get pluginBlocks() { return pluginBlocks; },
		set pluginBlocks(v) { pluginBlocks = v; },
		get pluginMetas() { return pluginMetas; },
		set pluginMetas(v) { pluginMetas = v; },
		get showBlockDocs() { return showBlockDocs; },
		set showBlockDocs(v) { showBlockDocs = v; },
		get blockDocsInitialType() { return blockDocsInitialType; },
		set blockDocsInitialType(v) { blockDocsInitialType = v; },
		get showPluginBuilder() { return showPluginBuilder; },
		set showPluginBuilder(v) { showPluginBuilder = v; },
		get securityIssues() { return securityIssues; },
		set securityIssues(v) { securityIssues = v; },
		_compileOutputCallback: null as ((data: { line: string; done: boolean; success: boolean; dll_path?: string }) => void) | null,
	};
}

export const app = createAppState();

// --- Multi-select helpers ---

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

// --- Undo/Redo system ---

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

export function zoomIn() {
	app.zoom = Math.min(2.0, Math.round((app.zoom + 0.1) * 10) / 10);
}

export function zoomOut() {
	app.zoom = Math.max(0.5, Math.round((app.zoom - 0.1) * 10) / 10);
}

export function zoomReset() {
	app.zoom = 1.0;
}

// --- Config Tab management ---

function defaultPipeline(): Pipeline {
	return {
		id: crypto.randomUUID(),
		name: 'New Config',
		author: '',
		created: new Date().toISOString(),
		modified: new Date().toISOString(),
		blocks: [],
		startup_blocks: [],
		data_settings: { wordlist_type: 'Credentials', separator: ':', slices: ['USER', 'PASS'] },
		proxy_settings: { proxy_mode: 'None', proxy_sources: [], ban_duration_secs: 300, max_retries_before_ban: 3, cpm_per_proxy: 0, proxy_groups: [], active_group: '' },
		browser_settings: { browser: 'chrome', ja3: null, http2_fingerprint: null, user_agent: null },
		runner_settings: { threads: 100, skip: 0, take: 0, continue_statuses: ['Retry'], custom_status_name: 'CUSTOM', max_retries: 3, concurrent_per_proxy: 0, start_threads_gradually: true, gradual_delay_ms: 100, automatic_thread_count: false, lower_threads_on_retry: false, retry_thread_reduction_pct: 25, pause_on_ratelimit: false, only_proxyless: false },
		output_settings: { save_to_file: true, save_to_database: false, include_response: false, output_directory: 'results', output_format: '{data} | {captures}', database_path: '', output_format_type: 'Txt', capture_filters: [] },
	};
}

/** Sync the active pipeline to the Rust backend (avoids circular import with ipc.ts) */
function syncPipelineToBackend() {
	if (typeof window !== 'undefined' && (window as any).ipc) {
		const data = JSON.parse(JSON.stringify(app.pipeline));
		(window as any).ipc.postMessage(JSON.stringify({ cmd: 'update_pipeline', data }));
	}
}

/** Generate a unique "New Config N" name based on existing tabs */
function nextNewConfigName(): string {
	const existing = app.configTabs.map(t => t.name);
	let n = 1;
	while (existing.includes(`New Config ${n}`)) n++;
	return `New Config ${n}`;
}

/** Save current pipeline state into the active tab before switching */
function saveCurrentTabState() {
	const tab = app.configTabs.find(t => t.id === app.activeTabId);
	if (tab) {
		tab.pipeline = JSON.parse(JSON.stringify(app.pipeline));
		if (!tab.filePath) tab.name = app.pipeline.name;
		tab.isDirty = JSON.stringify(app.pipeline) !== tab.savedSnapshot;
	}
}

/** Switch to a different tab */
export function switchTab(tabId: string) {
	if (tabId === app.activeTabId) return;
	saveCurrentTabState();
	const tab = app.configTabs.find(t => t.id === tabId);
	if (!tab) return;
	app.activeTabId = tabId;
	app.pipeline = JSON.parse(JSON.stringify(tab.pipeline));
	app.selectedBlockIds = [];
	app.editingBlockId = null;
	// Reset undo/redo for new tab context
	_undoStack = [];
	_redoStack = [];
	app.canUndo = false;
	app.canRedo = false;
	syncPipelineToBackend();
}

/** Create a new empty config tab */
export function createNewTab() {
	saveCurrentTabState();
	const id = crypto.randomUUID();
	const name = nextNewConfigName();
	const pip = defaultPipeline();
	pip.name = name;
	const tab: ConfigTab = {
		id,
		name,
		filePath: null,
		pipeline: pip,
		isDirty: false,
		savedSnapshot: JSON.stringify(pip),
	};
	app.configTabs = [...app.configTabs, tab];
	app.activeTabId = id;
	app.pipeline = JSON.parse(JSON.stringify(pip));
	app.selectedBlockIds = [];
	app.editingBlockId = null;
	_undoStack = [];
	_redoStack = [];
	app.canUndo = false;
	app.canRedo = false;
	syncPipelineToBackend();
}

/** Request to close a tab — checks for unsaved changes */
export function requestCloseTab(tabId: string) {
	if (app.configTabs.length <= 1) return; // can't close last tab
	const tab = app.configTabs.find(t => t.id === tabId);
	if (!tab) return;
	// Check dirty state (compare current pipeline if it's the active tab)
	if (tabId === app.activeTabId) {
		tab.pipeline = JSON.parse(JSON.stringify(app.pipeline));
		tab.isDirty = JSON.stringify(app.pipeline) !== tab.savedSnapshot;
	}
	if (tab.isDirty) {
		app.pendingCloseTabId = tabId;
		app.showUnsavedDialog = true;
	} else {
		closeTab(tabId);
	}
}

/** Force close a tab (after user confirmed) */
export function closeTab(tabId: string) {
	const idx = app.configTabs.findIndex(t => t.id === tabId);
	if (idx < 0 || app.configTabs.length <= 1) return;
	app.configTabs = app.configTabs.filter(t => t.id !== tabId);
	if (app.activeTabId === tabId) {
		// Switch to adjacent tab
		const newIdx = Math.min(idx, app.configTabs.length - 1);
		const newTab = app.configTabs[newIdx];
		app.activeTabId = newTab.id;
		app.pipeline = JSON.parse(JSON.stringify(newTab.pipeline));
		app.selectedBlockIds = [];
		app.editingBlockId = null;
		_undoStack = [];
		_redoStack = [];
		app.canUndo = false;
		app.canRedo = false;
		syncPipelineToBackend();
	}
	app.showUnsavedDialog = false;
	app.pendingCloseTabId = null;
}

/** Load a pipeline into the active tab (called after IPC pipeline_loaded) */
export function loadPipelineIntoTab(pipeline: Pipeline, filePath?: string) {
	const tab = app.configTabs.find(t => t.id === app.activeTabId);
	if (tab) {
		tab.pipeline = JSON.parse(JSON.stringify(pipeline));
		tab.name = filePath ? filePath.split(/[/\\]/).pop() || pipeline.name : pipeline.name;
		tab.filePath = filePath || null;
		tab.savedSnapshot = JSON.stringify(pipeline);
		tab.isDirty = false;
	}
}

/** Open a pipeline in a new tab */
export function openInNewTab(pipeline: Pipeline, filePath?: string) {
	saveCurrentTabState();
	const id = crypto.randomUUID();
	const snapshot = JSON.stringify(pipeline);
	const tab: ConfigTab = {
		id,
		name: filePath ? filePath.split(/[/\\]/).pop() || pipeline.name : pipeline.name,
		filePath: filePath || null,
		pipeline: JSON.parse(snapshot),
		isDirty: false,
		savedSnapshot: snapshot,
	};
	app.configTabs = [...app.configTabs, tab];
	app.activeTabId = id;
	app.pipeline = JSON.parse(snapshot);
	app.selectedBlockIds = [];
	app.editingBlockId = null;
	_undoStack = [];
	_redoStack = [];
	app.canUndo = false;
	app.canRedo = false;
}

/** Mark the active tab as saved */
export function markTabSaved(filePath?: string) {
	const tab = app.configTabs.find(t => t.id === app.activeTabId);
	if (tab) {
		tab.savedSnapshot = JSON.stringify(app.pipeline);
		tab.isDirty = false;
		if (filePath) {
			tab.filePath = filePath;
			tab.name = filePath.split(/[/\\]/).pop() || app.pipeline.name;
		}
	}
}

/** Reorder tabs by moving a tab from one index to another */
export function reorderTabs(fromIdx: number, toIdx: number) {
	if (fromIdx === toIdx) return;
	const tabs = [...app.configTabs];
	const [moved] = tabs.splice(fromIdx, 1);
	tabs.splice(toIdx, 0, moved);
	app.configTabs = tabs;
}

/** Update dirty state for active tab */
export function updateTabDirtyState() {
	const tab = app.configTabs.find(t => t.id === app.activeTabId);
	if (tab) {
		tab.isDirty = JSON.stringify(app.pipeline) !== tab.savedSnapshot;
	}
}
