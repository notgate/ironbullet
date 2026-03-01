import type { Pipeline, ConfigTab } from '$lib/types';
import { app } from './app.svelte';
import { send } from '$lib/ipc';
import { resetHistory } from './history';

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
		runner_settings: { threads: 100, skip: 0, take: 0, continue_statuses: ['Retry'], custom_status_name: 'CUSTOM', max_retries: 3, concurrent_per_proxy: 0, start_threads_gradually: false, gradual_delay_ms: 100, automatic_thread_count: false, lower_threads_on_retry: false, retry_thread_reduction_pct: 25, pause_on_ratelimit: false, only_proxyless: false },
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
	resetHistory();
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
	resetHistory();
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
		resetHistory();
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
	resetHistory();
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

/** Request to close the entire application — prompts for each unsaved tab */
export function requestAppClose() {
	// Update dirty state for the active tab first
	saveCurrentTabState();
	// Find all unsaved tabs
	const unsaved = app.configTabs.filter(t => t.isDirty);
	if (unsaved.length === 0) {
		send('close');
		return;
	}
	// Start the sequential unsaved-tab close flow
	app.pendingAppClose = true;
	app.pendingCloseTabId = unsaved[0].id;
	app.showUnsavedDialog = true;
}

/** Continue the app-close flow: find the next unsaved tab or close the app */
export function continueAppClose() {
	const unsaved = app.configTabs.filter(t => t.isDirty);
	if (unsaved.length === 0) {
		app.pendingAppClose = false;
		app.pendingCloseTabId = null;
		app.showUnsavedDialog = false;
		send('close');
	} else {
		app.pendingCloseTabId = unsaved[0].id;
		app.showUnsavedDialog = true;
	}
}

/** Cancel the app-close flow */
export function cancelAppClose() {
	app.pendingAppClose = false;
	app.pendingCloseTabId = null;
	app.showUnsavedDialog = false;
}
