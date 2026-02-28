<script lang="ts">
	import { app, getEditingBlock } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import type { BlockResult, Block } from '$lib/types';
	import X from '@lucide/svelte/icons/x';
	import Copy from '@lucide/svelte/icons/copy';
	import ExternalLink from '@lucide/svelte/icons/external-link';
	import Maximize2 from '@lucide/svelte/icons/maximize-2';
	import Check from '@lucide/svelte/icons/check';
	import Search from '@lucide/svelte/icons/search';
	import Shield from '@lucide/svelte/icons/shield';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import Minus from '@lucide/svelte/icons/minus';
	import Plus from '@lucide/svelte/icons/plus';

	// When true, renders full-screen inside a native OS window (launched via float_panel_native)
	let { nativeMode = false }: { nativeMode?: boolean } = $props();

	let viewTab = $state<'request' | 'body' | 'headers' | 'cookies'>('body');
	let prettyFormat = $state(true);
	// Load persisted font size from localStorage (fallback to 11)
	const FONTSIZE_KEY = 'ib_response_viewer_font_size';
	const _initFontSize = (() => {
		try {
			const stored = parseInt(localStorage.getItem(FONTSIZE_KEY) ?? '', 10);
			return Number.isFinite(stored) ? Math.max(9, Math.min(20, stored)) : 11;
		} catch { return 11; }
	})();
	let fontSize = $state(_initFontSize);
	let selectedResultIndex = $state(0);
	let copied = $state(false);

	// Search state
	let showSearch = $state(false);
	let searchQuery = $state('');
	let searchMatchIndex = $state(0);
	let searchInputEl = $state<HTMLInputElement | null>(null);

	// Draggable state
	let posX = $state(100);
	let posY = $state(80);
	let width = $state(620);
	let height = $state(440);
	let isDragging = $state(false);
	let isResizing = $state(false);
	let dragOffsetX = 0;
	let dragOffsetY = 0;

	// Persist font size whenever it changes
	const RESULTS_KEY = 'ib_last_debug_results';
	$effect(() => {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(FONTSIZE_KEY, String(fontSize));
		}
	});

	// Main window: persist debug results to sessionStorage so native window can load them
	$effect(() => {
		if (!nativeMode) {
			const r = app.debugResults;
			if (r.length > 0) {
				try { sessionStorage.setItem(RESULTS_KEY, JSON.stringify(r)); } catch {}
			}
		}
	});

	// Native window: seed from sessionStorage on first render if results are empty
	$effect(() => {
		if (nativeMode && app.debugResults.length === 0) {
			try {
				const stored = sessionStorage.getItem(RESULTS_KEY);
				if (stored) {
					const parsed = JSON.parse(stored);
					if (Array.isArray(parsed) && parsed.length > 0) {
						app.debugResults = parsed;
						app.showResponseViewer = true;
					}
				}
			} catch {}
		}
	});

	let results = $derived(app.debugResults);
	let hasResults = $derived(results.length > 0);

	// Find the HTTP request result (the one with response data)
	let httpResults = $derived(results.filter(r => r.response));
	let currentResult = $derived(httpResults[selectedResultIndex] || null);

	// Get current editing block for live parser feedback
	let editingBlock = $derived(getEditingBlock());

	// Helper: find all match start positions in text
	function findAll(text: string, query: string): number[] {
		const positions: number[] = [];
		const textLower = text.toLowerCase();
		const qLower = query.toLowerCase();
		let idx = 0;
		while (idx < textLower.length) {
			const found = textLower.indexOf(qLower, idx);
			if (found === -1) break;
			positions.push(found);
			idx = found + 1;
			if (positions.length > 5000) break;
		}
		return positions;
	}

	function getHeadersText(): string {
		if (!currentResult?.response) return '';
		return Object.entries(currentResult.response.headers).map(([k, v]) => `${k}: ${v}`).join('\n');
	}

	function getCookiesText(): string {
		if (!currentResult?.response) return '';
		return Object.entries(currentResult.response.cookies).map(([k, v]) => `${k}=${v}`).join('\n');
	}

	// Cross-tab search matches
	let allMatches = $derived.by(() => {
		if (!searchQuery || !currentResult?.response) return [];
		const q = searchQuery.toLowerCase();
		const flat: { tab: 'body' | 'headers' | 'cookies'; idx: number }[] = [];
		for (const pos of findAll(formatBody(currentResult.response.body || ''), q)) flat.push({ tab: 'body', idx: pos });
		for (const pos of findAll(getHeadersText(), q)) flat.push({ tab: 'headers', idx: pos });
		for (const pos of findAll(getCookiesText(), q)) flat.push({ tab: 'cookies', idx: pos });
		return flat;
	});

	let searchMatchCount = $derived(allMatches.length);

	// Get which local match within a tab is the current global match
	function getCurrentLocalIndex(tab: 'body' | 'headers' | 'cookies'): number {
		if (!allMatches.length || searchMatchIndex >= allMatches.length) return -1;
		if (allMatches[searchMatchIndex].tab !== tab) return -1;
		let localIdx = 0;
		for (let i = 0; i < searchMatchIndex; i++) {
			if (allMatches[i].tab === tab) localIdx++;
		}
		return localIdx;
	}

	let tabJustSwitched = false;

	// Reset match index when query changes
	$effect(() => {
		searchQuery;
		searchMatchIndex = 0;
	});

	function toggleSearch() {
		showSearch = !showSearch;
		if (showSearch) {
			setTimeout(() => searchInputEl?.focus(), 50);
		} else {
			searchQuery = '';
		}
	}

	function nextSearchMatch() {
		if (searchMatchCount > 0) {
			searchMatchIndex = (searchMatchIndex + 1) % searchMatchCount;
			const match = allMatches[searchMatchIndex];
			if (match && match.tab !== viewTab) {
				viewTab = match.tab;
				tabJustSwitched = true;
			}
		}
	}

	function prevSearchMatch() {
		if (searchMatchCount > 0) {
			searchMatchIndex = (searchMatchIndex - 1 + searchMatchCount) % searchMatchCount;
			const match = allMatches[searchMatchIndex];
			if (match && match.tab !== viewTab) {
				viewTab = match.tab;
				tabJustSwitched = true;
			}
		}
	}

	function onSearchKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			if (e.shiftKey) prevSearchMatch();
			else nextSearchMatch();
		} else if (e.key === 'Escape') {
			toggleSearch();
		}
	}

	// Compute live parser matches against the response body
	let parserMatches = $derived.by(() => {
		if (!currentResult?.response?.body || !editingBlock) return null;
		const body = currentResult.response.body;
		const s = editingBlock.settings;

		if (s.type === 'ParseLR') {
			return findLRMatches(body, s.left, s.right);
		}
		if (s.type === 'ParseRegex') {
			return findRegexMatches(body, s.pattern);
		}
		if (s.type === 'ParseJSON') {
			return findJSONMatches(body, s.json_path);
		}
		if (s.type === 'ParseCSS') {
			return findCSSMatches(body, s.selector, s.attribute);
		}
		if (s.type === 'ParseXPath') {
			return findXPathMatches(body, s.xpath);
		}
		if (s.type === 'ParseCookie' && currentResult?.response?.cookies) {
			return findCookieMatches(currentResult.response.cookies, s.cookie_name);
		}
		return null;
	});

	// --- Parser match helpers ---
	interface MatchResult {
		type: string;
		matches: string[];
		positions: { start: number; end: number }[];
	}

	function findLRMatches(body: string, left: string, right: string): MatchResult | null {
		if (!left && !right) return null;
		const matches: string[] = [];
		const positions: { start: number; end: number }[] = [];
		let searchFrom = 0;
		while (searchFrom < body.length) {
			const lIdx = left ? body.indexOf(left, searchFrom) : searchFrom;
			if (lIdx === -1) break;
			const start = lIdx + left.length;
			const rIdx = right ? body.indexOf(right, start) : body.length;
			if (rIdx === -1) break;
			matches.push(body.slice(start, rIdx));
			positions.push({ start, end: rIdx });
			searchFrom = rIdx + right.length;
			if (matches.length >= 50) break;
		}
		return matches.length > 0 ? { type: 'ParseLR', matches, positions } : null;
	}

	function findRegexMatches(body: string, pattern: string): MatchResult | null {
		if (!pattern) return null;
		try {
			const re = new RegExp(pattern, 'g');
			const matches: string[] = [];
			const positions: { start: number; end: number }[] = [];
			let m;
			while ((m = re.exec(body)) !== null) {
				matches.push(m[0]);
				positions.push({ start: m.index, end: m.index + m[0].length });
				if (matches.length >= 50) break;
			}
			return matches.length > 0 ? { type: 'ParseRegex', matches, positions } : null;
		} catch {
			return null;
		}
	}

	function findJSONMatches(body: string, jsonPath: string): MatchResult | null {
		if (!jsonPath) return null;
		try {
			const parsed = JSON.parse(body);
			const parts = jsonPath.split('.');
			let current: unknown = parsed;
			for (const part of parts) {
				if (current == null || typeof current !== 'object') return null;
				current = (current as Record<string, unknown>)[part];
			}
			if (current !== undefined) {
				const val = typeof current === 'string' ? current : JSON.stringify(current);
				return { type: 'ParseJSON', matches: [val], positions: [] };
			}
		} catch {}
		return null;
	}

	function findCSSMatches(body: string, selector: string, attribute: string): MatchResult | null {
		if (!selector) return null;
		try {
			const parser = new DOMParser();
			const doc = parser.parseFromString(body, 'text/html');
			const elements = doc.querySelectorAll(selector);
			const matches: string[] = [];
			elements.forEach(el => {
				const val = attribute ? el.getAttribute(attribute) : el.textContent;
				if (val) matches.push(val.trim());
			});
			return matches.length > 0 ? { type: 'ParseCSS', matches, positions: [] } : null;
		} catch {
			return null;
		}
	}

	function findXPathMatches(body: string, xpath: string): MatchResult | null {
		if (!xpath) return null;
		try {
			const parser = new DOMParser();
			const doc = parser.parseFromString(body, 'text/html');
			const result = doc.evaluate(xpath, doc, null, XPathResult.ORDERED_NODE_SNAPSHOT_TYPE, null);
			const matches: string[] = [];
			for (let i = 0; i < result.snapshotLength && i < 50; i++) {
				const node = result.snapshotItem(i);
				if (node) matches.push(node.textContent || '');
			}
			return matches.length > 0 ? { type: 'ParseXPath', matches, positions: [] } : null;
		} catch {
			return null;
		}
	}

	function findCookieMatches(cookies: Record<string, string>, cookieName: string): MatchResult | null {
		if (!cookieName) return null;
		const value = cookies[cookieName];
		if (value !== undefined) {
			return { type: 'ParseCookie', matches: [value], positions: [] };
		}
		return null;
	}

	// --- Highlight body with parser matches + search highlights ---
	function highlightBody(body: string, pm: MatchResult | null): string {
		if (!pm || pm.positions.length === 0) {
			return applySearchHighlights(escapeHtml(body), getCurrentLocalIndex('body'));
		}
		// Sort positions and highlight
		const sorted = [...pm.positions].sort((a, b) => a.start - b.start);
		let result = '';
		let lastEnd = 0;
		for (const pos of sorted) {
			if (pos.start < lastEnd) continue;
			result += escapeHtml(body.slice(lastEnd, pos.start));
			result += `<mark class="bg-primary/30 text-foreground rounded-sm px-px">${escapeHtml(body.slice(pos.start, pos.end))}</mark>`;
			lastEnd = pos.end;
		}
		result += escapeHtml(body.slice(lastEnd));
		return applySearchHighlights(result, getCurrentLocalIndex('body'));
	}

	function applySearchHighlights(html: string, currentMatchInTab: number): string {
		if (!searchQuery) return html;
		const q = escapeHtml(searchQuery);
		const qLower = q.toLowerCase();
		let result = '';
		let idx = 0;
		let matchNum = 0;
		const htmlLower = html.toLowerCase();
		while (idx < html.length) {
			// Skip existing HTML tags
			if (html[idx] === '<') {
				const closeIdx = html.indexOf('>', idx);
				if (closeIdx !== -1) {
					result += html.slice(idx, closeIdx + 1);
					idx = closeIdx + 1;
					continue;
				}
			}
			const found = htmlLower.indexOf(qLower, idx);
			if (found === -1) {
				result += html.slice(idx);
				break;
			}
			// Check if the match crosses an HTML tag
			const tagStart = html.indexOf('<', idx);
			if (tagStart !== -1 && tagStart < found) {
				// Flush up to tag, then skip the tag
				result += html.slice(idx, tagStart);
				const tagEnd = html.indexOf('>', tagStart);
				if (tagEnd !== -1) {
					result += html.slice(tagStart, tagEnd + 1);
					idx = tagEnd + 1;
					continue;
				}
			}
			result += html.slice(idx, found);
			const isCurrent = matchNum === currentMatchInTab;
			const cls = isCurrent ? 'bg-orange text-background rounded-sm px-px' : 'bg-yellow/30 text-foreground rounded-sm px-px';
			const idAttr = isCurrent ? ' id="search-current"' : '';
			result += `<mark class="${cls}"${idAttr}>${html.slice(found, found + q.length)}</mark>`;
			idx = found + q.length;
			matchNum++;
		}
		return result;
	}

	// Scroll to current search match
	$effect(() => {
		const _count = searchMatchCount;
		const _idx = searchMatchIndex;
		if (_count > 0) {
			const delay = tabJustSwitched ? 50 : 10;
			setTimeout(() => {
				const el = document.getElementById('search-current');
				el?.scrollIntoView({ block: 'center', behavior: 'smooth' });
				tabJustSwitched = false;
			}, delay);
		}
	});

	function escapeHtml(s: string): string {
		return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
	}

	function formatBody(body: string): string {
		if (!prettyFormat) return body;
		try {
			return JSON.stringify(JSON.parse(body), null, 2);
		} catch {
			return body;
		}
	}

	function getHighlightedHeadersHtml(): string {
		if (!currentResult?.response) return '';
		const localCurrent = getCurrentLocalIndex('headers');
		const lines: string[] = [];
		for (const [key, value] of Object.entries(currentResult.response.headers)) {
			lines.push(`<span class="text-primary">${escapeHtml(key)}:</span> ${escapeHtml(value)}`);
		}
		return applySearchHighlights(lines.join('\n'), localCurrent);
	}

	function getHighlightedCookiesHtml(): string {
		if (!currentResult?.response) return '';
		const localCurrent = getCurrentLocalIndex('cookies');
		const lines: string[] = [];
		for (const [key, value] of Object.entries(currentResult.response.cookies)) {
			lines.push(`<span class="text-purple">${escapeHtml(key)}=</span>${escapeHtml(value)}`);
		}
		return applySearchHighlights(lines.join('\n'), localCurrent);
	}

	function close() {
		app.showResponseViewer = false;
		showSearch = false;
		searchQuery = '';
	}

	async function copyBody() {
		if (!currentResult?.response?.body) return;
		try {
			await navigator.clipboard.writeText(currentResult.response.body);
			copied = true;
			setTimeout(() => { copied = false; }, 1500);
		} catch {}
	}

	async function copyCurrentTab() {
		let text = '';
		if (viewTab === 'body') text = currentResult?.response?.body ?? '';
		else if (viewTab === 'headers') text = getHeadersText();
		else if (viewTab === 'cookies') text = getCookiesText();
		else if (viewTab === 'request') {
			const req = currentResult?.request;
			if (req) {
				const hdrs = Object.entries(req.headers ?? {}).map(([k,v]) => `${k}: ${v}`).join('\n');
				text = `${req.method} ${req.url}\n${hdrs}${req.body ? '\n\n' + req.body : ''}`;
			}
		}
		if (text) {
			try {
				await navigator.clipboard.writeText(text);
				copied = true;
				setTimeout(() => { copied = false; }, 1500);
			} catch {}
		}
	}

	// --- Drag handlers ---
	function onTitleMouseDown(e: MouseEvent) {
		if ((e.target as HTMLElement).closest('button')) return;
		isDragging = true;
		dragOffsetX = e.clientX - posX;
		dragOffsetY = e.clientY - posY;
		window.addEventListener('mousemove', onDragMove);
		window.addEventListener('mouseup', onDragUp);
	}

	function onDragMove(e: MouseEvent) {
		if (isDragging) {
			posX = Math.max(0, e.clientX - dragOffsetX);
			posY = Math.max(0, e.clientY - dragOffsetY);
		}
	}

	function onDragUp() {
		isDragging = false;
		window.removeEventListener('mousemove', onDragMove);
		window.removeEventListener('mouseup', onDragUp);
	}

	// --- Resize handlers ---
	function onResizeMouseDown(e: MouseEvent) {
		e.preventDefault();
		e.stopPropagation();
		isResizing = true;
		const startX = e.clientX;
		const startY = e.clientY;
		const startW = width;
		const startH = height;

		function onMove(ev: MouseEvent) {
			width = Math.max(400, startW + ev.clientX - startX);
			height = Math.max(250, startH + ev.clientY - startY);
		}
		function onUp() {
			isResizing = false;
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		}
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	// Block result selector label
	function resultLabel(r: BlockResult): string {
		if (r.response) {
			return `${r.block_label} [${r.response.status_code}]`;
		}
		return r.block_label;
	}

	// Keyboard shortcuts: Ctrl+F to search, Ctrl+C to copy current tab (when nothing selected)
	function onViewerKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
			e.preventDefault();
			e.stopPropagation();
			toggleSearch();
		} else if ((e.ctrlKey || e.metaKey) && e.key === 'c' && !window.getSelection()?.toString()) {
			e.preventDefault();
			copyCurrentTab();
		}
	}
</script>

{#if (nativeMode || app.showResponseViewer) && hasResults}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="{nativeMode
			? 'flex flex-col h-screen w-screen bg-surface'
			: 'fixed z-50 flex flex-col bg-surface border border-border shadow-2xl'}"
		style="{nativeMode
			? ''
			: `left: ${posX}px; top: ${posY}px; width: ${width}px; height: ${height}px; box-shadow: 0 8px 32px rgba(0,0,0,0.6), 0 2px 8px rgba(0,0,0,0.3);`}"
		onkeydown={onViewerKeydown}
	>
		<!-- Title bar -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="flex items-center gap-2 px-2 py-1.5 bg-surface panel-raised shrink-0 {nativeMode ? 'cursor-default' : 'cursor-move'} select-none"
			onmousedown={nativeMode ? undefined : onTitleMouseDown}
		>
			<ExternalLink size={12} class="text-primary shrink-0" />
			<span class="text-xs font-medium text-foreground flex-1 truncate">Response Viewer</span>

			{#if httpResults.length > 1}
				<select
					class="skeu-select text-[10px] max-w-[180px]"
					value={selectedResultIndex}
					onchange={(e) => { selectedResultIndex = parseInt((e.target as HTMLSelectElement).value); }}
				>
					{#each httpResults as r, i}
						<option value={i}>{resultLabel(r)}</option>
					{/each}
				</select>
			{/if}

			<!-- Font size controls -->
			<div class="flex items-center gap-0.5 shrink-0">
				<button
					class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary"
					onmousedown={(e) => e.stopPropagation()}
					onclick={(e) => { e.stopPropagation(); fontSize = Math.max(9, fontSize - 1); }}
					title="Decrease font size"
				><Minus size={10} /></button>
				<span class="text-[9px] text-muted-foreground w-5 text-center">{fontSize}</span>
				<button
					class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary"
					onmousedown={(e) => e.stopPropagation()}
					onclick={(e) => { e.stopPropagation(); fontSize = Math.min(20, fontSize + 1); }}
					title="Increase font size"
				><Plus size={10} /></button>
			</div>
			{#if !nativeMode}
			<button
				class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary"
				onclick={() => send('float_panel_native', { id: 'response-viewer' })}
				title="Launch as external window"
			>
				<Maximize2 size={12} />
			</button>
			{/if}
			<button
				class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary"
				onclick={() => { app.showFingerprint = true; }}
				title="Site Fingerprint"
			>
				<Shield size={12} />
			</button>
			<button
				class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary"
				onclick={toggleSearch}
				title="Search (Ctrl+F)"
			>
				<Search size={12} />
			</button>
			<button
				class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary"
				onclick={copyCurrentTab}
				title="Copy current tab (Ctrl+C)"
			>
				{#if copied}<Check size={12} class="text-green" />{:else}<Copy size={12} />{/if}
			</button>
			{#if !nativeMode}
			<button
				class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary"
				onclick={close}
				title="Close"
			>
				<X size={12} />
			</button>
			{/if}
		</div>

		<!-- Search bar -->
		{#if showSearch}
			<div class="flex items-center gap-1.5 px-2 py-1 border-b border-border bg-background shrink-0">
				<Search size={11} class="text-muted-foreground shrink-0" />
				<!-- svelte-ignore a11y_autofocus -->
				<input
					type="text"
					bind:this={searchInputEl}
					bind:value={searchQuery}
					placeholder="Search response..."
					class="flex-1 skeu-input text-[11px] font-mono py-0"
					onkeydown={onSearchKeydown}
					autofocus
				/>
				{#if searchQuery}
					<span class="text-[10px] text-muted-foreground shrink-0 min-w-[60px] text-center">
						{searchMatchCount > 0 ? `${searchMatchIndex + 1}/${searchMatchCount}` : 'No matches'}
					</span>
					<button class="p-0.5 rounded text-muted-foreground hover:text-foreground" onclick={prevSearchMatch} title="Previous (Shift+Enter)">
						<ChevronUp size={12} />
					</button>
					<button class="p-0.5 rounded text-muted-foreground hover:text-foreground" onclick={nextSearchMatch} title="Next (Enter)">
						<ChevronDown size={12} />
					</button>
				{/if}
				<button class="p-0.5 rounded text-muted-foreground hover:text-foreground" onclick={toggleSearch} title="Close search">
					<X size={11} />
				</button>
			</div>
		{/if}

		{#if currentResult}
			<!-- Status bar -->
			<div class="flex items-center gap-3 px-2 py-1 border-b border-border bg-background shrink-0 text-xs">
				{#if currentResult.request}
					<span class="font-medium text-foreground">{currentResult.request.method}</span>
					<span class="text-primary font-mono truncate flex-1 min-w-0">{currentResult.request.url}</span>
				{/if}
				{#if currentResult.response}
					<span class="font-medium shrink-0 {currentResult.response.status_code < 400 ? 'text-green' : 'text-red'}">
						{currentResult.response.status_code}
					</span>
					<span class="text-[10px] text-muted-foreground shrink-0">{currentResult.response.timing_ms}ms</span>
				{/if}
			</div>

			<!-- View tabs -->
			<div class="flex border-b border-border shrink-0 items-center">
				{#if currentResult?.request}
				<button
					class="px-2 py-0.5 text-[11px] {viewTab === 'request' ? 'text-foreground font-medium border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'}"
					onclick={() => viewTab = 'request'}
				>Request</button>
				{/if}
				<button
					class="px-2 py-0.5 text-[11px] {viewTab === 'body' ? 'text-foreground font-medium border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'}"
					onclick={() => viewTab = 'body'}
				>Response</button>
				<button
					class="px-2 py-0.5 text-[11px] {viewTab === 'headers' ? 'text-foreground font-medium border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'}"
					onclick={() => viewTab = 'headers'}
				>Headers</button>
				<button
					class="px-2 py-0.5 text-[11px] {viewTab === 'cookies' ? 'text-foreground font-medium border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'}"
					onclick={() => viewTab = 'cookies'}
				>Cookies</button>

				<div class="flex-1"></div>

				<!-- Pretty format checkbox (body/request only) -->
				{#if viewTab === 'body' || viewTab === 'request'}
				<label class="flex items-center gap-1 text-[10px] text-muted-foreground px-2 cursor-pointer select-none">
					<input type="checkbox" bind:checked={prettyFormat} class="w-3 h-3" />
					Pretty
				</label>
				{/if}

				<!-- Parser match indicator -->
				{#if parserMatches}
					<span class="text-[10px] text-green px-2 py-0.5 flex items-center gap-1">
						{parserMatches.matches.length} match{parserMatches.matches.length !== 1 ? 'es' : ''}
						<span class="text-muted-foreground">({parserMatches.type.replace('Parse', '')})</span>
					</span>
				{/if}
			</div>

			<!-- Content -->
			<div class="flex-1 overflow-auto panel-inset min-h-0">
				{#if viewTab === 'request' && currentResult?.request}
					<!-- Request details -->
					<div class="p-2 space-y-2 select-text" style="font-size: {fontSize}px">
						<!-- Request line -->
						<div class="flex items-baseline gap-2 font-mono">
							<span class="text-orange font-bold shrink-0">{currentResult.request.method}</span>
							<span class="text-primary break-all">{currentResult.request.url}</span>
						</div>
						<!-- Request headers -->
						{#if currentResult.request.headers && Object.keys(currentResult.request.headers).length > 0}
							<div>
								<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">Request Headers</div>
								{#each Object.entries(currentResult.request.headers ?? {}) as [key, value]}
									<div class="flex font-mono gap-1 group">
										<span class="text-orange shrink-0">{key}:</span>
										<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
										<button
											class="p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0"
											onclick={async () => { try { await navigator.clipboard.writeText(String(value)); } catch {} }}
											title="Copy value"
										><Copy size={9} /></button>
									</div>
								{/each}
							</div>
						{/if}
						<!-- Request body -->
						{#if currentResult.request.body}
							<div>
								<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">Request Body</div>
								<pre class="font-mono whitespace-pre-wrap break-words text-foreground">{prettyFormat ? (() => { try { return JSON.stringify(JSON.parse(currentResult.request.body), null, 2); } catch { return currentResult.request.body; } })() : currentResult.request.body}</pre>
							</div>
						{:else}
							<div class="text-muted-foreground/50 font-mono">— no request body —</div>
						{/if}
					</div>
				{:else if viewTab === 'body' && currentResult.response}
					<!-- Parser match results panel -->
					{#if parserMatches && parserMatches.matches.length > 0}
						<div class="border-b border-border bg-background px-2 py-1.5">
							<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">Matched Values</div>
							<div class="space-y-0.5 max-h-[80px] overflow-y-auto">
								{#each parserMatches.matches as match, mi}
									<div class="flex items-center gap-1 group">
										<span class="text-[10px] text-muted-foreground w-4 text-right shrink-0">{mi + 1}</span>
										<code class="text-[10px] text-green font-mono truncate flex-1 min-w-0 select-text">{match}</code>
										<button
											class="p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground"
											onclick={async () => { try { await navigator.clipboard.writeText(match); } catch {} }}
											title="Copy value"
										>
											<Copy size={9} />
										</button>
									</div>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Response body with highlights -->
					<div class="p-2 select-text" style="font-size: {fontSize}px">
						{#if parserMatches && parserMatches.positions.length > 0}
							<pre class="text-foreground font-mono whitespace-pre-wrap break-words">{@html highlightBody(currentResult.response.body, parserMatches)}</pre>
						{:else if searchQuery}
							<pre class="text-foreground font-mono whitespace-pre-wrap break-words">{@html applySearchHighlights(escapeHtml(formatBody(currentResult.response.body)), getCurrentLocalIndex('body'))}</pre>
						{:else}
							<pre class="text-foreground font-mono whitespace-pre-wrap break-words">{formatBody(currentResult.response.body)}</pre>
						{/if}
					</div>
				{:else if viewTab === 'headers' && currentResult.response}
					<div class="p-2 space-y-0.5 select-text" style="font-size: {fontSize}px">
						{#if searchQuery}
							<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">Response Headers</div>
							<pre class="text-xs font-mono whitespace-pre-wrap break-words">{@html getHighlightedHeadersHtml()}</pre>
							{#if currentResult.request}
								<div class="mt-2 text-[10px] uppercase tracking-wider text-muted-foreground mb-1">Request Headers</div>
								{#each currentResult.request.headers as [key, value]}
									<div class="flex text-xs font-mono gap-1">
										<span class="text-orange shrink-0">{key}:</span>
										<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
									</div>
								{/each}
							{/if}
						{:else}
							<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">Response Headers</div>
							{#each Object.entries(currentResult.response.headers) as [key, value]}
								<div class="flex text-xs font-mono gap-1 group">
									<span class="text-primary shrink-0">{key}:</span>
									<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
									<button
										class="p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0"
										onclick={async () => { try { await navigator.clipboard.writeText(value); } catch {} }}
										title="Copy value"
									>
										<Copy size={9} />
									</button>
								</div>
							{/each}
							{#if currentResult.request}
								<div class="mt-2 text-[10px] uppercase tracking-wider text-muted-foreground mb-1">Request Headers</div>
								{#each currentResult.request.headers as [key, value]}
									<div class="flex text-xs font-mono gap-1">
										<span class="text-orange shrink-0">{key}:</span>
										<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
									</div>
								{/each}
							{/if}
						{/if}
					</div>
				{:else if viewTab === 'cookies' && currentResult.response}
					<div class="p-2 space-y-0.5 select-text" style="font-size: {fontSize}px">
						{#if Object.keys(currentResult.response.cookies).length > 0}
							{#if searchQuery}
								<pre class="text-xs font-mono whitespace-pre-wrap break-words">{@html getHighlightedCookiesHtml()}</pre>
							{:else}
								{#each Object.entries(currentResult.response.cookies) as [key, value]}
									<div class="flex text-xs font-mono gap-1 group">
										<span class="text-purple shrink-0">{key}=</span>
										<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
										<button
											class="p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0"
											onclick={async () => { try { await navigator.clipboard.writeText(key); } catch {} }}
											title="Copy name"
										>
											<Copy size={9} />
										</button>
										<button
											class="p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground shrink-0"
											onclick={async () => { try { await navigator.clipboard.writeText(value); } catch {} }}
											title="Copy value"
										>
											<Copy size={9} />
										</button>
									</div>
								{/each}
							{/if}
						{:else}
							<div class="text-xs text-muted-foreground">No cookies in response</div>
						{/if}
					</div>
				{/if}
			</div>
		{:else}
			<div class="flex items-center justify-center flex-1 text-muted-foreground text-xs panel-inset">
				No HTTP response data available
			</div>
		{/if}

		<!-- Resize handle (hidden in native window mode) -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		{#if !nativeMode}
		<div
			class="absolute bottom-0 right-0 w-4 h-4 cursor-se-resize"
			onmousedown={onResizeMouseDown}
		>
			<svg class="w-3 h-3 text-muted-foreground/40 absolute bottom-0.5 right-0.5" viewBox="0 0 12 12">
				<path d="M11 1v10H1" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
				<path d="M11 5v6H5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
			</svg>
		</div>
		{/if}
	</div>
{/if}
