import type { Pipeline, Block, RunnerStats, BlockResult, NetworkEntry, ConfigTab, Job, PluginBlockMeta, PluginMeta } from '$lib/types';

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
	rightPanelWidth: number;
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
	pendingAppClose: boolean;

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

	// Changelog
	showChangelog: boolean;

	// Fingerprint
	showFingerprint: boolean;

	// Security
	securityIssues: Array<{ severity: string; title: string; description: string; code_snippet: string }>;

	// Update state
	updateAvailable: boolean;
	updateChecking: boolean;
	updateLatestVersion: string;
	updateCurrentVersion: string;
	updateReleaseNotes: string;
	updateDownloadUrl: string;
	updatePublishedAt: string;
	updateInstalling: boolean;
	updateProgress: number;
	updateComplete: boolean;
	showUpdateDialog: boolean;

	// Pipeline UX features
	collapsedBlockIds: Set<string>;
	pipelineSearchQuery: string;
	pipelineSearchFocused: boolean;
	savedBlocksSnapshot: Record<string, string>;
	showMinimap: boolean;
	blockTemplates: Array<{ name: string; blocks: Block[] }>;
	previewMode: boolean;

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
	let rightPanelWidth = $state(360);
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
	let pendingAppClose = $state(false);
	let jobs = $state<Job[]>([]);
	let activeJobId = $state<string | null>(null);
	let pluginBlocks = $state<PluginBlockMeta[]>([]);
	let pluginMetas = $state<PluginMeta[]>([]);
	let showBlockDocs = $state(false);
	let blockDocsInitialType = $state<string | null>(null);
	let showPluginBuilder = $state(false);
	let showChangelog = $state(false);
	let showFingerprint = $state(false);
	let securityIssues = $state<Array<{ severity: string; title: string; description: string; code_snippet: string }>>([]);

	// Update state
	let updateAvailable = $state(false);
	let updateChecking = $state(false);
	let updateLatestVersion = $state('');
	let updateCurrentVersion = $state('0.1.0');
	let updateReleaseNotes = $state('');
	let updateDownloadUrl = $state('');
	let updatePublishedAt = $state('');
	let updateInstalling = $state(false);
	let updateProgress = $state(0);
	let updateComplete = $state(false);
	let showUpdateDialog = $state(false);

	// Pipeline UX features
	let collapsedBlockIds = $state<Set<string>>(new Set());
	let pipelineSearchQuery = $state('');
	let pipelineSearchFocused = $state(false);
	let savedBlocksSnapshot = $state<Record<string, string>>({});
	let showMinimap = $state(true);
	let blockTemplates = $state<Array<{ name: string; blocks: Block[] }>>(loadTemplates());
	let previewMode = $state(false);

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
		get rightPanelWidth() { return rightPanelWidth; },
		set rightPanelWidth(v) { rightPanelWidth = v; },
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
		get pendingAppClose() { return pendingAppClose; },
		set pendingAppClose(v) { pendingAppClose = v; },
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
		get showChangelog() { return showChangelog; },
		set showChangelog(v) { showChangelog = v; },
		get showFingerprint() { return showFingerprint; },
		set showFingerprint(v) { showFingerprint = v; },
		get securityIssues() { return securityIssues; },
		set securityIssues(v) { securityIssues = v; },
		get updateAvailable() { return updateAvailable; },
		set updateAvailable(v) { updateAvailable = v; },
		get updateChecking() { return updateChecking; },
		set updateChecking(v) { updateChecking = v; },
		get updateLatestVersion() { return updateLatestVersion; },
		set updateLatestVersion(v) { updateLatestVersion = v; },
		get updateCurrentVersion() { return updateCurrentVersion; },
		set updateCurrentVersion(v) { updateCurrentVersion = v; },
		get updateReleaseNotes() { return updateReleaseNotes; },
		set updateReleaseNotes(v) { updateReleaseNotes = v; },
		get updateDownloadUrl() { return updateDownloadUrl; },
		set updateDownloadUrl(v) { updateDownloadUrl = v; },
		get updatePublishedAt() { return updatePublishedAt; },
		set updatePublishedAt(v) { updatePublishedAt = v; },
		get updateInstalling() { return updateInstalling; },
		set updateInstalling(v) { updateInstalling = v; },
		get updateProgress() { return updateProgress; },
		set updateProgress(v) { updateProgress = v; },
		get updateComplete() { return updateComplete; },
		set updateComplete(v) { updateComplete = v; },
		get showUpdateDialog() { return showUpdateDialog; },
		set showUpdateDialog(v) { showUpdateDialog = v; },
		get collapsedBlockIds() { return collapsedBlockIds; },
		set collapsedBlockIds(v) { collapsedBlockIds = v; },
		get pipelineSearchQuery() { return pipelineSearchQuery; },
		set pipelineSearchQuery(v) { pipelineSearchQuery = v; },
		get pipelineSearchFocused() { return pipelineSearchFocused; },
		set pipelineSearchFocused(v) { pipelineSearchFocused = v; },
		get savedBlocksSnapshot() { return savedBlocksSnapshot; },
		set savedBlocksSnapshot(v) { savedBlocksSnapshot = v; },
		get showMinimap() { return showMinimap; },
		set showMinimap(v) { showMinimap = v; },
		get blockTemplates() { return blockTemplates; },
		set blockTemplates(v) { blockTemplates = v; },
		get previewMode() { return previewMode; },
		set previewMode(v) { previewMode = v; },
		_compileOutputCallback: null as ((data: { line: string; done: boolean; success: boolean; dll_path?: string }) => void) | null,
	};
}

function loadTemplates(): Array<{ name: string; blocks: Block[] }> {
	try {
		const stored = localStorage.getItem('ironbullet_block_templates');
		return stored ? JSON.parse(stored) : [];
	} catch { return []; }
}


export const app = createAppState();
