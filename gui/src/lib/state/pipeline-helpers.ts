import { app } from './app.svelte';
import type { Block } from '$lib/types';

// ── Collapse / Expand ──

function getAllContainerIds(blocks: Block[]): string[] {
	const ids: string[] = [];
	for (const b of blocks) {
		const s = b.settings as any;
		if (s.type === 'IfElse' || s.type === 'Loop' || s.type === 'Group') {
			ids.push(b.id);
		}
		if (s.type === 'IfElse') {
			ids.push(...getAllContainerIds(s.true_blocks || []));
			ids.push(...getAllContainerIds(s.false_blocks || []));
		}
		if (s.type === 'Loop' || s.type === 'Group') {
			ids.push(...getAllContainerIds(s.blocks || []));
		}
	}
	return ids;
}

export function collapseAllBlocks() {
	const ids = getAllContainerIds(app.pipeline.blocks);
	app.collapsedBlockIds = new Set(ids);
}

export function expandAllBlocks() {
	app.collapsedBlockIds = new Set();
}

export function toggleBlockCollapse(blockId: string) {
	const next = new Set(app.collapsedBlockIds);
	if (next.has(blockId)) next.delete(blockId);
	else next.add(blockId);
	app.collapsedBlockIds = next;
}

export function isBlockCollapsed(blockId: string): boolean {
	return app.collapsedBlockIds.has(blockId);
}

// ── Block Search ──

export function blockMatchesSearch(block: Block, query: string): boolean {
	if (!query) return true;
	const q = query.toLowerCase();
	if (block.label.toLowerCase().includes(q)) return true;
	if (block.block_type.toLowerCase().includes(q)) return true;
	const s = block.settings as any;
	// Search common settings fields
	if (s.url?.toLowerCase().includes(q)) return true;
	if (s.output_var?.toLowerCase().includes(q)) return true;
	if (s.input_var?.toLowerCase().includes(q)) return true;
	if (s.pattern?.toLowerCase().includes(q)) return true;
	if (s.selector?.toLowerCase().includes(q)) return true;
	if (s.json_path?.toLowerCase().includes(q)) return true;
	if (s.message?.toLowerCase().includes(q)) return true;
	if (s.name?.toLowerCase().includes(q)) return true;
	if (s.value?.toLowerCase().includes(q)) return true;
	if (s.code?.toLowerCase().includes(q)) return true;
	if (s.host?.toLowerCase().includes(q)) return true;
	if (s.left?.toLowerCase().includes(q)) return true;
	if (s.right?.toLowerCase().includes(q)) return true;
	return false;
}

// ── Change Indicators (Snapshot) ──

export function takePipelineSnapshot() {
	const snapshot: Record<string, string> = {};
	function snap(blocks: Block[]) {
		for (const b of blocks) {
			snapshot[b.id] = JSON.stringify(b);
			const s = b.settings as any;
			if (s.true_blocks) snap(s.true_blocks);
			if (s.false_blocks) snap(s.false_blocks);
			if (s.blocks) snap(s.blocks);
		}
	}
	snap(app.pipeline.blocks);
	app.savedBlocksSnapshot = snapshot;
}

export function isBlockModified(block: Block): boolean {
	const saved = app.savedBlocksSnapshot[block.id];
	if (!saved) return true; // new block = modified
	return saved !== JSON.stringify(block);
}

// ── Templates ──

export function saveBlockTemplate(name: string, blocks: Block[]) {
	const clean = JSON.parse(JSON.stringify(blocks)).map((b: Block) => ({
		...b,
		id: '', // IDs get regenerated on paste
	}));
	const templates = [...app.blockTemplates, { name, blocks: clean }];
	app.blockTemplates = templates;
	localStorage.setItem('ironbullet_block_templates', JSON.stringify(templates));
}

export function deleteBlockTemplate(index: number) {
	const templates = app.blockTemplates.filter((_, i) => i !== index);
	app.blockTemplates = templates;
	localStorage.setItem('ironbullet_block_templates', JSON.stringify(templates));
}

// ── Variable Tracking ──

export function getAvailableVariables(upToBlockId?: string): string[] {
	const vars = new Set<string>();

	// Data settings slices
	for (const slice of app.pipeline.data_settings.slices) {
		vars.add(`input.${slice}`);
	}
	vars.add('data.SOURCE');
	vars.add('data.STATUS');
	vars.add('data.PROXY');
	vars.add('data.BOTNUM');

	// Walk blocks and collect output variables
	function walk(blocks: Block[]): boolean {
		for (const b of blocks) {
			if (upToBlockId && b.id === upToBlockId) return true; // stop here
			const s = b.settings as any;
			if (s.output_var) vars.add(s.output_var);
			if (s.type === 'SetVariable' && s.name) vars.add(s.name);
			if (s.type === 'HttpRequest') {
				vars.add('SOURCE');
				vars.add('RESPONSECODE');
				vars.add('COOKIES');
				vars.add('ADDRESS');
			}
			if (s.type === 'RandomUserAgent') vars.add('USERAGENT');
			if (s.type === 'RandomData') {
				if (s.output_var) vars.add(s.output_var);
			}
			if (s.true_blocks && walk(s.true_blocks)) return true;
			if (s.false_blocks && walk(s.false_blocks)) return true;
			if (s.blocks && walk(s.blocks)) return true;
		}
		return false;
	}
	walk(app.pipeline.blocks);

	return [...vars].sort();
}

// ── Preview Mode (Variable Resolution) ──

/** Replace <VAR> tokens in a string with placeholder hints */
export function resolvePreviewVars(text: string, blockId: string): string {
	if (!text) return text;
	const vars = getAvailableVariables(blockId);
	const varSet = new Set(vars);
	return text.replace(/<([^>]+)>/g, (match, name: string) => {
		if (varSet.has(name)) {
			// Show as resolved placeholder
			return `[${name}]`;
		}
		// Check common data slices
		if (name.startsWith('input.')) return `[${name}]`;
		if (name.startsWith('data.')) return `[${name}]`;
		return match; // leave unresolved
	});
}
