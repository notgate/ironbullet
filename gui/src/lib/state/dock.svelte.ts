/**
 * Dock layout state for IronBullet panels.
 * Panels can be placed in the bottom bar, right sidebar, or floated as windows.
 * Layout is persisted to localStorage.
 */

export type PanelId = 'debugger' | 'code' | 'data' | 'jobs' | 'network' | 'variables' | 'inspector';
/** Dock zones:
 * - bottom / right / left: docked tab areas in the main window
 * - float:   CSS overlay (legacy, kept for non-Windows)
 * - native:  panel is open in a real native OS window (managed by Rust)
 */
export type DockZone = 'bottom' | 'right' | 'float' | 'left' | 'native';

export interface FloatState {
	x: number;
	y: number;
	width: number;
	height: number;
	minimized: boolean;
}

export interface PanelConfig {
	id: PanelId;
	zone: DockZone;
	order: number;
	float?: FloatState;
}

export const PANEL_LABELS: Record<PanelId, string> = {
	inspector: 'Inspect',
	debugger: 'Debugger',
	code: 'Code View',
	data: 'Data / Proxy',
	jobs: 'Jobs',
	network: 'Network',
	variables: 'Variables',
};

const STORAGE_KEY = 'ironbullet_dock_layout_v2';

const DEFAULT_PANELS: PanelConfig[] = [
	{ id: 'debugger', zone: 'bottom', order: 0 },
	{ id: 'code',     zone: 'bottom', order: 1 },
	{ id: 'data',     zone: 'bottom', order: 2 },
	{ id: 'jobs',     zone: 'bottom', order: 3 },
	{ id: 'network',  zone: 'bottom', order: 4 },
	{ id: 'variables', zone: 'bottom', order: 5 },
	{ id: 'inspector', zone: 'bottom', order: 6 },
];

function loadFromStorage(): PanelConfig[] {
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (raw) {
			const parsed = JSON.parse(raw) as PanelConfig[];
			// Reset transient zones to 'bottom' on startup:
			// 'native' = OS windows don't persist; 'float' = CSS overlays don't persist
			const normalized = parsed.map(p =>
				(p.zone === 'native' || p.zone === 'float')
					? { ...p, zone: 'bottom' as DockZone }
					: p
			);
			// Fill any missing panels with their defaults
			const ids = normalized.map(p => p.id);
			const defaults = DEFAULT_PANELS.filter(d => !ids.includes(d.id));
			return [...normalized, ...defaults];
		}
	} catch {}
	return [...DEFAULT_PANELS];
}

function saveToStorage(panels: PanelConfig[]) {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(panels));
	} catch {}
}

function createDockState() {
	let panels = $state<PanelConfig[]>(loadFromStorage());
	let dragging = $state<PanelId | null>(null);
	let dragOver = $state<DockZone | null>(null);

	function panelsIn(zone: DockZone) {
		return panels.filter(p => p.zone === zone).sort((a, b) => a.order - b.order);
	}

	function movePanel(id: PanelId, zone: DockZone) {
		panels = panels.map(p => {
			if (p.id !== id) return p;
			const targetOrder = panelsIn(zone).length;
			return { ...p, zone, order: targetOrder, float: zone === 'float'
				? (p.float ?? { x: 200, y: 100, width: 520, height: 380, minimized: false })
				: p.float };
		});
		// Re-normalize orders within zones
		(['bottom', 'right', 'float', 'left', 'native'] as DockZone[]).forEach(z => {
			panelsIn(z).forEach((p, i) => {
				const found = panels.find(x => x.id === p.id);
				if (found) found.order = i;
			});
		});
		saveToStorage(panels);
	}

	function reorderPanel(id: PanelId, targetId: PanelId) {
		const src = panels.find(p => p.id === id);
		const tgt = panels.find(p => p.id === targetId);
		if (!src || !tgt || src.zone !== tgt.zone) return;
		const zone = src.zone;
		const zPanels = panelsIn(zone);
		const srcIdx = zPanels.findIndex(p => p.id === id);
		const tgtIdx = zPanels.findIndex(p => p.id === targetId);
		zPanels.splice(srcIdx, 1);
		zPanels.splice(tgtIdx, 0, src);
		zPanels.forEach((p, i) => {
			const found = panels.find(x => x.id === p.id);
			if (found) found.order = i;
		});
		panels = [...panels];
		saveToStorage(panels);
	}

	function setFloatPosition(id: PanelId, x: number, y: number) {
		panels = panels.map(p => p.id !== id ? p : { ...p, float: { ...(p.float ?? { x, y, width: 520, height: 380, minimized: false }), x, y } });
		saveToStorage(panels);
	}

	function setFloatSize(id: PanelId, width: number, height: number) {
		panels = panels.map(p => p.id !== id ? p : { ...p, float: { ...(p.float ?? { x: 200, y: 100, width, height, minimized: false }), width, height } });
		saveToStorage(panels);
	}

	function toggleMinimize(id: PanelId) {
		panels = panels.map(p => p.id !== id ? p : { ...p, float: { ...(p.float ?? { x: 200, y: 100, width: 520, height: 380, minimized: false }), minimized: !(p.float?.minimized ?? false) } });
		saveToStorage(panels);
	}

	function resetLayout() {
		panels = [...DEFAULT_PANELS];
		saveToStorage(panels);
	}

	return {
		get panels() { return panels; },
		get dragging() { return dragging; },
		set dragging(v) { dragging = v; },
		get dragOver() { return dragOver; },
		set dragOver(v) { dragOver = v; },
		panelsIn,
		movePanel,
		reorderPanel,
		setFloatPosition,
		setFloatSize,
		toggleMinimize,
		resetLayout,
	};
}

export const dock = createDockState();
