<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send, onResponse } from '$lib/ipc';
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';
	import Copy from '@lucide/svelte/icons/copy';
	import Globe from '@lucide/svelte/icons/globe';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Check from '@lucide/svelte/icons/check';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import Square from '@lucide/svelte/icons/square';
	import Play from '@lucide/svelte/icons/play';
	import MonitorPlay from '@lucide/svelte/icons/monitor-play';
	import MonitorOff from '@lucide/svelte/icons/monitor-off';
	import List from '@lucide/svelte/icons/list';
	import Chrome from '@lucide/svelte/icons/chrome';
	import ExternalLink from '@lucide/svelte/icons/external-link';
	import Terminal from '@lucide/svelte/icons/terminal';

	// ── Mode ───────────────────────────────────────────────────────────────────
	let mode = $state<'manual' | 'browser' | 'proxy'>('manual');

	// ── Proxy Capture State ────────────────────────────────────────────────────
	let proxyPort        = $state(8877);
	let proxyActive      = $state(false);
	let proxyError       = $state('');

	// ── Browser Capture State ──────────────────────────────────────────────────
	interface CapturedRequest {
		id: string; url: string; method: string; resource_type: string;
		headers: Record<string, string>; post_data: string | null;
		// populated by response_meta / response_body events
		resp_status?: number; resp_status_text?: string; resp_mime?: string;
		resp_headers?: Record<string, string>; resp_body?: string;
	}

	// filter / search — capturedRequests persisted in localStorage so popping
	// the panel out to an external window (which remounts the component) doesn't
	// wipe the captured session. browserOpen/browserLoading are never persisted
	// since we can't know if Chrome is still alive after a remount.
	let browserUrl       = $state((() => { try { return localStorage.getItem('ib_inspector_url') || 'https://'; } catch { return 'https://'; } })());
	let browserOpen      = $state(false);
	let browserLoading   = $state(false);
	let browserError     = $state('');
	let chromeNotInstalled = $derived(
		browserError.toLowerCase().includes('chrome not installed') ||
		browserError.toLowerCase().includes('chrome') && browserError.toLowerCase().includes('install')
	);
	let fixChromeChecking = $state(false);

	function openChromeDownload() {
		send('open_url', { url: 'https://www.google.com/chrome/' });
	}

	function getInstallCommand(): string {
		const ua = navigator.userAgent.toLowerCase();
		if (ua.includes('win')) {
			return 'winget install Google.Chrome';
		} else if (ua.includes('mac')) {
			return 'brew install --cask google-chrome';
		} else {
			return 'sudo apt install google-chrome-stable\n# or: sudo dnf install google-chrome-stable';
		}
	}

	let installCmdCopied = $state(false);
	function copyInstallCmd() {
		navigator.clipboard.writeText(getInstallCommand());
		installCmdCopied = true;
		setTimeout(() => installCmdCopied = false, 2000);
	}

	function recheckChrome() {
		fixChromeChecking = true;
		send('check_chrome', {});
	}

	onResponse('check_chrome', (data: any) => {
		fixChromeChecking = false;
		if (data?.found) {
			browserError = '';
		} else {
			browserError = 'Chrome still not detected. Restart IronBullet after installing.';
		}
	});
	let capturedRequests = $state<CapturedRequest[]>((() => { try { const r = localStorage.getItem('ib_inspector_captures'); return r ? JSON.parse(r) as CapturedRequest[] : []; } catch { return []; } })());
	let selectedReqId    = $state<string | null>((() => { try { return localStorage.getItem('ib_inspector_sel') || null; } catch { return null; } })());
	let searchQuery      = $state('');
	let typeFilter       = $state('all');

	$effect(() => { try { localStorage.setItem('ib_inspector_url', browserUrl); } catch {} });

	// Helpers: write to localStorage immediately (not via $effect) so external
	// panel windows can read the latest data as soon as it changes.
	function persistCaptures(reqs: CapturedRequest[]) {
		try {
			const slim = reqs.slice(-300).map(r => ({ ...r, resp_body: r.resp_body?.slice(0, 4096) }));
			localStorage.setItem('ib_inspector_captures', JSON.stringify(slim));
		} catch {}
	}
	function persistResult(r: InspectResult | null) {
		try {
			if (r) localStorage.setItem('ib_inspector_result', JSON.stringify({ ...r, body: r.body?.slice(0, 65536) }));
			else localStorage.removeItem('ib_inspector_result');
		} catch {}
	}

	// detail UI
	let detailTab        = $state<'headers' | 'payload' | 'response' | 'params'>('headers');
	let prettyMode       = $state(true);
	let applySelReq      = $state<Set<string>>(new Set());
	let applySelResp     = $state<Set<string>>(new Set());
	let applyFrom        = $state<'request' | 'response'>('request');
	let applyPanelOpen   = $state(false);

	// splitter
	let splitWidth       = $state(272); // left list px

	const TYPE_MAP: Record<string, string[]> = {
		doc:  ['Document'],
		xhr:  ['Xhr', 'Fetch'],
		js:   ['Script'],
		css:  ['Stylesheet'],
		img:  ['Image', 'Media'],
		font: ['Font'],
		ws:   ['WebSocket'],
	};
	function typeGroup(t: string): string {
		for (const [g, types] of Object.entries(TYPE_MAP)) {
			if (types.includes(t)) return g;
		}
		return 'other';
	}

	const filteredRequests = $derived(capturedRequests.filter(r => {
		if (typeFilter !== 'all') {
			const g = typeGroup(r.resource_type);
			if (typeFilter === 'other') { if (g !== 'other') return false; }
			else if (g !== typeFilter) return false;
		}
		if (searchQuery) {
			const q = searchQuery.toLowerCase();
			if (!r.url.toLowerCase().includes(q) && !r.method.toLowerCase().includes(q)) return false;
		}
		return true;
	}));

	const selectedReq = $derived(
		selectedReqId ? (capturedRequests.find(r => r.id === selectedReqId) ?? null) : null
	);

	// 25-second safety net — prevents infinite loading if Chrome hangs silently.
	let loadTimerId: number | null = null;

	// Register browser capture listener on mount so external panel windows
	// (which receive eval_js broadcasts but can't call openBrowser) also get
	// live capture events without needing to call openBrowser() themselves.
	$effect(() => {
		onResponse('inspector_browser_event', (data: unknown) => {
			const ev = data as {
				type: string; url?: string; message?: string; id?: string;
				method?: string; resource_type?: string;
				headers?: Record<string, string>; post_data?: string | null;
				status?: number; status_text?: string; mime_type?: string;
				body?: string;
			};
			// Clear safety timeout on any terminal event.
			if (ev.type === 'error' || ev.type === 'opened' || ev.type === 'closed') {
				if (loadTimerId !== null) { clearTimeout(loadTimerId); loadTimerId = null; }
			}
			if (ev.type === 'error')  { browserError = ev.message ?? 'Unknown error'; browserOpen = false; browserLoading = false; return; }
			if (ev.type === 'opened') { browserOpen = true; browserLoading = false; return; }
			if (ev.type === 'closed') { browserOpen = false; browserLoading = false; return; }

			if (ev.type === 'request') {
				if (capturedRequests.some(r => r.id === ev.id)) return;
				const next = [...capturedRequests, {
					id: ev.id!, url: ev.url!, method: ev.method!,
					resource_type: ev.resource_type ?? 'Other',
					headers: ev.headers ?? {}, post_data: ev.post_data ?? null,
				}];
				capturedRequests = next;
				if (!selectedReqId) { selectedReqId = ev.id!; applySelReq = new Set(Object.keys(ev.headers ?? {})); }
				persistCaptures(next);
			} else if (ev.type === 'response_meta') {
				const next = capturedRequests.map(r => r.id !== ev.id ? r : {
					...r, resp_status: ev.status, resp_status_text: ev.status_text,
					resp_mime: ev.mime_type, resp_headers: ev.headers,
				});
				capturedRequests = next;
				persistCaptures(next);
			} else if (ev.type === 'response_body') {
				const next = capturedRequests.map(r => r.id !== ev.id ? r : { ...r, resp_body: ev.body });
				capturedRequests = next;
				persistCaptures(next);
			}
		});
	});

	// Proxy capture event listener
	$effect(() => {
		onResponse('inspector_proxy_event', (data: unknown) => {
			const ev = data as any;
			if (ev.type === 'error')  { proxyError = ev.message ?? 'Unknown error'; proxyActive = false; return; }
			if (ev.type === 'ready')  { proxyActive = true; proxyError = ''; return; }
			if (ev.type === 'request') {
				if (capturedRequests.some(r => r.id === ev.id)) return;
				const next = [...capturedRequests, {
					id: ev.id, url: ev.url, method: ev.method,
					resource_type: ev.resource_type ?? 'fetch',
					headers: ev.headers ?? {}, post_data: ev.post_data ?? null,
				}];
				capturedRequests = next;
				if (!selectedReqId) { selectedReqId = ev.id; applySelReq = new Set(Object.keys(ev.headers ?? {})); }
				persistCaptures(next);
			} else if (ev.type === 'response') {
				const next = capturedRequests.map(r => r.id !== ev.id ? r : {
					...r,
					resp_status: ev.resp_status, resp_status_text: ev.resp_status_text,
					resp_mime: ev.resp_mime, resp_headers: ev.resp_headers, resp_body: ev.resp_body,
				});
				capturedRequests = next;
				persistCaptures(next);
			}
		});
	});

	function startProxy() {
		proxyError = '';
		capturedRequests = [];
		try { localStorage.removeItem('ib_inspector_captures'); localStorage.removeItem('ib_inspector_sel'); } catch {}
		selectedReqId = null; applyPanelOpen = false;
		send('inspect_proxy_start', { port: proxyPort });
	}

	function stopProxy() {
		send('inspect_proxy_stop', {});
		proxyActive = false;
	}

	// Register manual-inspect result listener on mount for the same reason —
	// the external panel window receives the broadcast and updates its display.
	$effect(() => {
		onResponse('site_inspect_result', (data: unknown) => {
			loading = false;
			const d = data as InspectResult;
			if (d.error && !d.status) { errorMsg = d.error; return; }
			result = d;
			persistResult(d);
			applySelection = new Set(Object.keys(d.headers));
			viewTab = 'resp-headers';
		});
	});

	function openBrowser() {
		if (!browserUrl.trim() || browserUrl === 'https://') { browserError = 'Enter a URL first'; return; }
		// Prevent double-fire — if already loading or open, ignore extra clicks
		if (browserLoading || browserOpen) return;
		browserError = '';
		capturedRequests = [];
		try { localStorage.removeItem('ib_inspector_captures'); localStorage.removeItem('ib_inspector_sel'); } catch {}
		selectedReqId = null; applyPanelOpen = false;
		browserLoading = true;
		if (loadTimerId !== null) clearTimeout(loadTimerId);
		loadTimerId = window.setTimeout(() => {
			loadTimerId = null;
			if (browserLoading && !browserOpen) {
				browserError = 'Chrome did not respond within 25 seconds. Try again.';
				browserLoading = false;
			}
		}, 25000);
		send('inspect_browser_open', { url: browserUrl.trim() });
	}

	function closeBrowser() {
		send('inspect_browser_close', {});
		if (loadTimerId !== null) { clearTimeout(loadTimerId); loadTimerId = null; }
		browserOpen = false; browserLoading = false;
	}

	function selectReq(id: string) {
		if (selectedReqId === id) return;
		selectedReqId = id; applyPanelOpen = false; detailTab = 'headers';
		const r = capturedRequests.find(x => x.id === id);
		if (r) { applySelReq = new Set(Object.keys(r.headers)); applySelResp = new Set(Object.keys(r.resp_headers ?? {})); }
	}

	function browserApplyToBlock() {
		if (!selectedReq) return;
		const src = applyFrom === 'request' ? selectedReq.headers : (selectedReq.resp_headers ?? {});
		const sel = applyFrom === 'request' ? applySelReq : applySelResp;
		const selected = Object.entries(src).filter(([k]) => sel.has(k));

		const targetBlock = app.pipeline.blocks.find(
			b => b.id === app.selectedBlockId && b.settings.type === 'HttpRequest'
		) ?? app.pipeline.blocks.find(b => b.settings.type === 'HttpRequest');

		if (!targetBlock || targetBlock.settings.type !== 'HttpRequest') {
			browserError = 'Select an HTTP Request block first'; return;
		}
		const existing: [string, string][] = [...(targetBlock.settings.headers ?? [])];
		for (const [key, value] of selected) {
			const idx = existing.findIndex(([k]) => k.toLowerCase() === key.toLowerCase());
			if (idx >= 0) existing[idx] = [existing[idx][0], value];
			else existing.push([key, value]);
		}
		const needsUrl = !targetBlock.settings.url?.trim();
		app.pipeline.blocks = app.pipeline.blocks.map(b => b.id !== targetBlock.id ? b : {
			...b, settings: {
				...b.settings, headers: existing,
				...(needsUrl ? { url: selectedReq!.url } : {}),
				...(selectedReq!.post_data && applyFrom === 'request' ? { body: selectedReq!.post_data } : {}),
			}
		});
		applyPanelOpen = false; browserError = '';
	}

	// splitter drag
	let splitterDragging = $state(false);
	function startSplitDrag(e: MouseEvent) {
		e.preventDefault();
		const startX = e.clientX, startW = splitWidth;
		splitterDragging = true;
		function onMove(ev: MouseEvent) { splitWidth = Math.max(180, Math.min(480, startW + ev.clientX - startX)); }
		function onUp() { splitterDragging = false; window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); }
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	// pretty print
	function formatBody(body: string | null | undefined, mime?: string): string {
		if (!body) return '';
		if (!prettyMode) return body;
		if (mime?.includes('json') || /^\s*[{\[]/.test(body)) {
			try { return JSON.stringify(JSON.parse(body), null, 2); } catch {}
		}
		if (mime?.includes('x-www-form-urlencoded') || (body.includes('=') && body.includes('&') && !body.includes('\n'))) {
			try {
				const p = new URLSearchParams(body);
				return Array.from(p.entries()).map(([k, v]) => `${decodeURIComponent(k)}: ${decodeURIComponent(v)}`).join('\n');
			} catch {}
		}
		return body;
	}

	function parseParams(url: string): [string, string][] {
		try { return Array.from(new URL(url).searchParams.entries()); } catch { return []; }
	}

	function methodColor(m: string): string {
		if (m === 'GET')    return 'text-green';
		if (m === 'POST')   return 'text-orange';
		if (m === 'PUT' || m === 'PATCH') return 'text-blue';
		if (m === 'DELETE') return 'text-red';
		return 'text-muted-foreground';
	}

	function statusColor(s?: number): string {
		if (!s) return 'text-muted-foreground/40';
		if (s < 300) return 'text-green'; if (s < 400) return 'text-blue';
		if (s < 500) return 'text-orange'; return 'text-red';
	}

	function typeBadgeClass(t: string): string {
		const g = typeGroup(t);
		if (g === 'doc') return 'bg-primary/20 text-primary';
		if (g === 'xhr') return 'bg-orange/20 text-orange';
		if (g === 'js')  return 'bg-yellow-500/20 text-yellow-400';
		if (g === 'css') return 'bg-blue/20 text-blue';
		if (g === 'img') return 'bg-purple/20 text-purple';
		if (g === 'font') return 'bg-pink-500/20 text-pink-400';
		return 'bg-muted/30 text-muted-foreground';
	}

	function shortUrl(u: string): string {
		try { const p = new URL(u); return (p.pathname + p.search) || '/'; } catch { return u; }
	}

	// custom checkbox (matches app theme)
	function toggleCheck(set: Set<string>, key: string): Set<string> {
		const n = new Set(set);
		if (n.has(key)) n.delete(key); else n.add(key);
		return n;
	}

	// ── Manual State ────────────────────────────────────────────────────────────
	let url      = $state((() => { try { return localStorage.getItem('ib_inspector_manual_url') || 'https://'; } catch { return 'https://'; } })());
	let method   = $state((() => { try { return localStorage.getItem('ib_inspector_method') || 'GET'; } catch { return 'GET'; } })());
	let proxy    = $state('');
	let browser  = $state('chrome');
	let bodyText = $state('');
	let extraHeaders = $state<[string, string][]>([]);
	let headerMode = $state<'kv' | 'raw'>('kv');
	let rawHeaderText = $state('');

	let loading  = $state(false);
	let errorMsg = $state('');
	let result   = $state<InspectResult | null>((() => { try { const r = localStorage.getItem('ib_inspector_result'); return r ? JSON.parse(r) as InspectResult : null; } catch { return null; } })());
	let viewTab  = $state<'resp-headers' | 'req-headers' | 'body' | 'cookies'>('resp-headers');
	let copied   = $state<string | null>(null);

	let showApplyPanel  = $state(false);
	let applySource     = $state<'request' | 'response'>('response');
	let applySelection  = $state<Set<string>>(new Set((() => { try { const r = localStorage.getItem('ib_inspector_result'); return r ? Object.keys((JSON.parse(r) as InspectResult).headers ?? {}) : []; } catch { return []; } })()));

	// Persist URL and method on each keystroke so the external window and
	// remounted panels restore the last-used values.
	$effect(() => { try { localStorage.setItem('ib_inspector_manual_url', url); } catch {} });
	$effect(() => { try { localStorage.setItem('ib_inspector_method', method); } catch {} });

	interface InspectResult {
		status: number;
		final_url: string;
		timing_ms: number;
		headers: Record<string, string>;
		request_headers: Record<string, string>;
		cookies: Record<string, string>;
		body: string;
		error?: string;
		via: string;
		browser?: string;
	}

	const METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'].map(m => ({ value: m, label: m }));
	const BROWSERS = [
		{ value: 'chrome',  label: 'Chrome'  },
		{ value: 'firefox', label: 'Firefox' },
		{ value: 'safari',  label: 'Safari'  },
		{ value: 'edge',    label: 'Edge'    },
	];

	const bodyAllowed = $derived(!['GET', 'HEAD', 'OPTIONS'].includes(method));

	// Convert KV headers ↔ raw text
	function kvToRaw(kv: [string, string][]): string {
		return kv.filter(([k]) => k.trim()).map(([k, v]) => `${k}: ${v}`).join('\n');
	}
	function rawToKv(raw: string): [string, string][] {
		return raw.split('\n').map(l => l.trim()).filter(Boolean).map(l => {
			const idx = l.indexOf(':');
			return idx === -1 ? [l, ''] as [string, string] : [l.slice(0, idx).trim(), l.slice(idx + 1).trim()] as [string, string];
		});
	}
	function switchHeaderMode(m: 'kv' | 'raw') {
		if (m === 'raw') rawHeaderText = kvToRaw(extraHeaders);
		else extraHeaders = rawToKv(rawHeaderText);
		headerMode = m;
	}

	// ── Capture ────────────────────────────────────────────────────────────────
	function capture() {
		if (!url.trim() || url === 'https://') return;
		loading  = true;
		errorMsg = '';
		// Result is handled by the mount-time $effect onResponse('site_inspect_result') above.
		const hdrs: [string, string][] = headerMode === 'raw' ? rawToKv(rawHeaderText) : extraHeaders;
		send('site_inspect', {
			url:     url.trim(),
			method,
			proxy:   proxy.trim() || null,
			browser,
			body:    bodyAllowed && bodyText.trim() ? bodyText : null,
			headers: hdrs.filter(([k]) => k.trim()).map(([k, v]) => [k, v]),
		});
	}

	function stop() {
		loading = false;
	}

	// ── Apply to Block ─────────────────────────────────────────────────────────
	function applyToBlock() {
		if (!result) return;
		const src = applySource === 'request' ? result.request_headers : result.headers;
		const selected = Object.entries(src).filter(([k]) => applySelection.has(k));
		if (!selected.length) return;

		const targetBlock = app.pipeline.blocks.find(
			b => b.id === app.selectedBlockId && b.settings.type === 'HttpRequest'
		) ?? app.pipeline.blocks.find(b => b.settings.type === 'HttpRequest');

		if (!targetBlock || targetBlock.settings.type !== 'HttpRequest') {
			errorMsg = 'Select an HTTP Request block first'; return;
		}

		const existing: [string, string][] = [...(targetBlock.settings.headers ?? [])];
		for (const [key, value] of selected) {
			const idx = existing.findIndex(([k]) => k.toLowerCase() === key.toLowerCase());
			if (idx >= 0) existing[idx] = [existing[idx][0], value];
			else existing.push([key, value]);
		}
		app.pipeline.blocks = app.pipeline.blocks.map(b =>
			b.id !== targetBlock.id ? b : { ...b, settings: { ...b.settings, headers: existing } }
		);
		showApplyPanel = false;
		errorMsg = '';
	}

	// ── Copy ──────────────────────────────────────────────────────────────────
	async function copyText(key: string, text: string) {
		try { await navigator.clipboard.writeText(text); copied = key; setTimeout(() => { copied = null; }, 1500); } catch {}
	}


	function prettyBody(b: string): string {
		try { return JSON.stringify(JSON.parse(b), null, 2); } catch { return b; }
	}

	function tabCount(t: typeof viewTab): string {
		if (!result) return '';
		if (t === 'resp-headers') return ` (${Object.keys(result.headers).length})`;
		if (t === 'req-headers')  return ` (${Object.keys(result.request_headers).length})`;
		if (t === 'cookies')      return ` (${Object.keys(result.cookies).length})`;
		return '';
	}
</script>

<div class="flex flex-col h-full bg-surface text-foreground text-[11px] select-none">

	<!-- ══ Mode Toggle ════════════════════════════════════════════════════════ -->
	<div class="flex items-center gap-0 px-2 py-1 border-b border-border shrink-0 bg-background/60">
		<button
			class="flex items-center gap-1 px-2.5 py-0.5 rounded text-[10px] transition-colors {mode === 'manual' ? 'bg-primary/20 text-primary font-medium' : 'text-muted-foreground hover:text-foreground hover:bg-accent/20'}"
			onclick={() => { mode = 'manual'; }}
		><Globe size={10} />Manual</button>
		<button
			class="flex items-center gap-1 px-2.5 py-0.5 rounded text-[10px] transition-colors {mode === 'browser' ? 'bg-primary/20 text-primary font-medium' : 'text-muted-foreground hover:text-foreground hover:bg-accent/20'}"
			onclick={() => { mode = 'browser'; }}
		><MonitorPlay size={10} />Browser Capture</button>
		<button
			class="flex items-center gap-1 px-2.5 py-0.5 rounded text-[10px] transition-colors {mode === 'proxy' ? 'bg-primary/20 text-primary font-medium' : 'text-muted-foreground hover:text-foreground hover:bg-accent/20'}"
			onclick={() => { mode = 'proxy'; }}
		><List size={10} />Proxy Capture</button>
		{#if mode === 'browser' && browserOpen}
			<span class="ml-2 flex items-center gap-1 text-[9px] text-green animate-pulse"><span class="w-1.5 h-1.5 rounded-full bg-green inline-block"></span>Capturing</span>
		{:else if mode === 'browser' && browserLoading}
			<span class="ml-2 flex items-center gap-1 text-[9px] text-muted-foreground"><Loader2 size={9} class="animate-spin" />Launching Chrome…</span>
		{:else if mode === 'proxy' && proxyActive}
			<span class="ml-2 flex items-center gap-1 text-[9px] text-green animate-pulse"><span class="w-1.5 h-1.5 rounded-full bg-green inline-block"></span>Proxy Active</span>
		{/if}
	</div>

	{#if mode === 'browser'}
	<!-- ══ BROWSER CAPTURE MODE ═════════════════════════════════════════════ -->

	<!-- Address + controls bar -->
	<div class="flex items-center gap-1.5 px-2 py-1.5 panel-raised shrink-0">
		<input type="text" bind:value={browserUrl} placeholder="https://target.com/login"
			class="skeu-input text-[11px] font-mono flex-1 min-w-0"
			onkeydown={(e) => { if (e.key === 'Enter' && !browserOpen) openBrowser(); }}
		/>
		{#if browserOpen}
			<button class="skeu-btn flex items-center gap-1 text-[11px] text-red shrink-0" onclick={closeBrowser}>
				<MonitorOff size={11} />Close
			</button>
		{:else if browserLoading}
			<button class="skeu-btn flex items-center gap-1 text-[11px] text-muted-foreground shrink-0" disabled>
				<Loader2 size={11} class="animate-spin" />Launching…
			</button>
		{:else}
			<button class="skeu-btn flex items-center gap-1 text-[11px] shrink-0" onclick={openBrowser}>
				<MonitorPlay size={11} />Open Browser
			</button>
		{/if}
		<button class="skeu-btn text-[10px] text-muted-foreground shrink-0"
			onclick={() => { capturedRequests = []; selectedReqId = null; }}>Clear</button>
		<span class="text-[9px] text-muted-foreground/50 shrink-0 tabular-nums">{filteredRequests.length}/{capturedRequests.length}</span>
	</div>

	<!-- Filter bar: search + type buttons -->
	<div class="flex items-center gap-1 px-2 py-1 border-b border-border shrink-0 bg-background/30 flex-wrap">
		<input type="text" bind:value={searchQuery} placeholder="Filter by URL or method…"
			class="skeu-input text-[10px] font-mono flex-1 min-w-[120px]"
		/>
		{#each [['all','All'],['doc','Doc'],['xhr','XHR'],['js','JS'],['css','CSS'],['img','Img'],['font','Font'],['ws','WS'],['other','Other']] as [g, lbl]}
			<button
				class="px-1.5 py-0.5 rounded text-[9px] font-medium transition-colors {typeFilter === g ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-accent/30'}"
				onclick={() => typeFilter = g as string}
			>{lbl}</button>
		{/each}
	</div>

	{#if browserError}
		{#if chromeNotInstalled}
			<!-- ── Fix Chrome panel ───────────────────────────────────── -->
			<div class="shrink-0 border-b border-border bg-background/60 p-3 flex flex-col gap-2.5">
				<!-- Header row -->
				<div class="flex items-center gap-2">
					<div class="w-6 h-6 rounded-md bg-red/10 border border-red/20 flex items-center justify-center shrink-0">
						<Chrome size={13} class="text-red/70" />
					</div>
					<div class="flex-1 min-w-0">
						<p class="text-[11px] font-medium text-foreground leading-tight">Chrome not found</p>
						<p class="text-[9px] text-muted-foreground mt-0.5">Browser Capture requires Google Chrome or Chromium.</p>
					</div>
				</div>

				<!-- Install command -->
				<div class="bg-muted/30 border border-border rounded-md px-2.5 py-1.5 flex items-center gap-2 group">
					<Terminal size={10} class="text-muted-foreground shrink-0" />
					<code class="text-[9px] text-foreground/80 font-mono flex-1 overflow-hidden text-ellipsis whitespace-nowrap">
						{getInstallCommand().split('\n')[0]}
					</code>
					<button
						onclick={copyInstallCmd}
						class="shrink-0 text-[8px] font-medium px-1.5 py-0.5 rounded border transition-colors
							{installCmdCopied
								? 'bg-green/10 border-green/30 text-green'
								: 'bg-muted/40 border-border text-muted-foreground hover:text-foreground hover:border-border/80'}"
					>
						{installCmdCopied ? '✓ Copied' : 'Copy'}
					</button>
				</div>

				<!-- Action buttons -->
				<div class="flex gap-1.5">
					<button
						onclick={openChromeDownload}
						class="skeu-btn flex-1 flex items-center justify-center gap-1.5 text-[10px] py-1.5"
					>
						<ExternalLink size={10} />
						Download Chrome
					</button>
					<button
						onclick={recheckChrome}
						disabled={fixChromeChecking}
						class="skeu-btn flex items-center justify-center gap-1.5 text-[10px] py-1.5 px-3 disabled:opacity-50"
					>
						{#if fixChromeChecking}
							<Loader2 size={10} class="animate-spin" />
						{:else}
							<Check size={10} />
						{/if}
						{fixChromeChecking ? 'Checking…' : 'Recheck'}
					</button>
				</div>
			</div>
		{:else}
			<!-- Generic error banner -->
			<div class="px-2 py-0.5 bg-red/10 border-b border-red/20 text-red text-[10px] shrink-0">{browserError}</div>
		{/if}
	{/if}

	{#if !browserOpen && capturedRequests.length === 0}
		<div class="flex flex-col items-center justify-center flex-1 gap-3 text-muted-foreground panel-inset">
			<MonitorPlay size={32} class="text-muted-foreground/20" />
			<div class="text-[11px] text-center leading-relaxed max-w-[280px]">
				Enter the login page URL and click <strong>Open Browser</strong>.<br/>
				A Chrome window opens — log in normally.<br/>
				Every HTTP request is captured here in real time.
			</div>
			<div class="text-[9px] text-muted-foreground/40 text-center max-w-[260px]">
				Select any request to inspect headers, body, and response.<br/>
				Use <strong>Apply to Block</strong> to fill your HTTP block instantly.
			</div>
		</div>
	{:else}
	<!-- ── Main split: resizable list | detail ─────────────────────── -->
	<div class="flex flex-1 min-h-0 overflow-hidden" class:select-none={splitterDragging}>

		<!-- LEFT: request list -->
		<div class="shrink-0 flex flex-col border-r border-border bg-background/40 overflow-hidden" style="width:{splitWidth}px">
			<!-- Column header -->
			<div class="grid text-[8px] uppercase tracking-wider text-muted-foreground/60 font-semibold border-b border-border/50 px-1 py-0.5 shrink-0"
				style="grid-template-columns: 36px 28px 1fr 32px">
				<span>Meth</span><span>Type</span><span>Path</span><span class="text-right">St</span>
			</div>
			<div class="flex-1 overflow-y-auto">
				{#if filteredRequests.length === 0}
					<div class="p-3 text-[9px] text-muted-foreground/40 italic text-center">
						{browserOpen ? 'Waiting for requests…' : capturedRequests.length ? 'No matches' : 'No requests captured'}
					</div>
				{:else}
					{#each filteredRequests as req}
						{@const isSelected = selectedReqId === req.id}
						<button
							class="w-full text-left px-1 py-[3px] border-b border-border/20 hover:bg-accent/20 transition-colors grid items-center gap-1 {isSelected ? 'bg-primary/10 border-l-2 border-l-primary' : ''}"
							style="grid-template-columns: 36px 28px 1fr 32px"
							onclick={() => selectReq(req.id)}
						>
							<span class="font-mono font-bold text-[9px] {methodColor(req.method)} truncate">{req.method}</span>
							<span class="inline-flex items-center justify-center text-[7px] leading-none h-[13px] px-1 rounded font-medium shrink-0 {typeBadgeClass(req.resource_type)}">{req.resource_type.slice(0,4)}</span>
							<span class="font-mono text-[9px] text-foreground/75 truncate leading-tight">{shortUrl(req.url)}</span>
							<span class="text-right font-mono text-[9px] tabular-nums {statusColor(req.resp_status)}">{req.resp_status ?? '—'}</span>
						</button>
					{/each}
				{/if}
			</div>
		</div>

		<!-- Drag splitter -->
		<div
			class="w-[4px] shrink-0 cursor-col-resize hover:bg-primary/40 bg-border/50 transition-colors flex items-center justify-center"
			onmousedown={startSplitDrag}
		></div>

		<!-- RIGHT: detail pane -->
		<div class="flex-1 flex flex-col min-w-0 overflow-hidden">
			{#if selectedReq}
				<!-- Request summary bar -->
				<div class="flex items-center gap-2 px-2 py-0.5 border-b border-border bg-background/60 shrink-0 min-w-0">
					<span class="font-mono font-bold text-[10px] {methodColor(selectedReq.method)} shrink-0">{selectedReq.method}</span>
					{#if selectedReq.resp_status}
						<span class="font-mono font-bold text-[10px] {statusColor(selectedReq.resp_status)} shrink-0">{selectedReq.resp_status}</span>
					{/if}
					<span class="font-mono text-primary truncate flex-1 min-w-0 text-[9px]" title={selectedReq.url}>{selectedReq.url}</span>
					<span class="text-[8px] text-muted-foreground/50 shrink-0">{selectedReq.resp_mime ?? ''}</span>
					<button
						class="skeu-btn flex items-center gap-1 text-[10px] text-primary shrink-0"
						onclick={() => { applyPanelOpen = !applyPanelOpen; }}
					><ArrowRight size={10} />Apply to Block</button>
				</div>

				<!-- Apply to Block panel -->
				{#if applyPanelOpen}
					<div class="px-2 py-1.5 border-b border-border bg-background shrink-0">
						<div class="flex items-center gap-1.5 mb-1">
							<span class="text-[10px] font-medium flex-1">Apply to HTTP Block</span>
							<!-- Source toggle -->
							<div class="flex rounded border border-border overflow-hidden">
								{#each [['request','Request Hdrs'],['response','Response Hdrs']] as [v, l]}
									<button
										class="px-1.5 py-0.5 text-[9px] transition-colors {applyFrom === v ? 'bg-primary text-primary-foreground' : 'bg-background text-muted-foreground hover:bg-accent/20'}"
										onclick={() => { applyFrom = v as typeof applyFrom; }}
									>{l}</button>
								{/each}
							</div>
							<button class="text-[9px] text-primary hover:underline"
								onclick={() => {
									const src = applyFrom === 'request' ? selectedReq!.headers : (selectedReq!.resp_headers ?? {});
									if (applyFrom === 'request') applySelReq = new Set(Object.keys(src));
									else applySelResp = new Set(Object.keys(src));
								}}>All</button>
							<button class="text-[9px] text-muted-foreground hover:underline"
								onclick={() => { if (applyFrom === 'request') applySelReq = new Set(); else applySelResp = new Set(); }}>None</button>
						</div>
						<div class="max-h-[100px] overflow-y-auto space-y-px mb-1.5 select-text">
							{#each Object.entries(applyFrom === 'request' ? selectedReq.headers : (selectedReq.resp_headers ?? {})) as [key, value]}
								{@const applySel = applyFrom === 'request' ? applySelReq : applySelResp}
								<button class="w-full flex items-center gap-1.5 hover:bg-accent/20 rounded px-0.5"
									onclick={() => {
										if (applyFrom === 'request') applySelReq = toggleCheck(applySelReq, key);
										else applySelResp = toggleCheck(applySelResp, key);
									}}>
									<!-- Custom themed checkbox -->
									<span class="w-3 h-3 rounded border border-border flex items-center justify-center shrink-0 transition-colors {applySel.has(key) ? 'bg-primary border-primary' : 'bg-background'}">
										{#if applySel.has(key)}<Check size={8} class="text-primary-foreground" />{/if}
									</span>
									<span class="text-orange font-mono text-[9px] shrink-0 w-[130px] truncate text-left">{key}:</span>
									<span class="text-foreground font-mono text-[9px] truncate flex-1 min-w-0 text-left">{value}</span>
								</button>
							{/each}
						</div>
						<button class="skeu-btn text-[10px] text-primary" onclick={browserApplyToBlock}>
							Apply {(applyFrom === 'request' ? applySelReq : applySelResp).size} headers{selectedReq.post_data && applyFrom === 'request' ? ' + body' : ''} to block
						</button>
					</div>
				{/if}

				<!-- Detail tabs -->
				<div class="flex items-center border-b border-border shrink-0">
					{#each [
						['headers', 'Headers'],
						['payload', 'Payload'],
						['response','Response'],
						['params',  'Params'],
					] as [tid, tlbl]}
						<button
							class="px-2.5 py-0.5 text-[10px] {detailTab === tid ? 'text-foreground font-medium border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'}"
							onclick={() => detailTab = tid as typeof detailTab}
						>{tlbl}</button>
					{/each}
					<div class="flex-1"></div>
					<!-- Pretty toggle -->
					{#if detailTab === 'payload' || detailTab === 'response'}
						<div class="flex rounded border border-border overflow-hidden mr-1.5">
							{#each [['pretty','Pretty'],['raw','Raw']] as [m,l]}
								<button
									class="px-1.5 py-0.5 text-[9px] transition-colors {prettyMode === (m==='pretty') ? 'bg-primary text-primary-foreground' : 'bg-background text-muted-foreground hover:bg-accent/20'}"
									onclick={() => prettyMode = (m === 'pretty')}
								>{l}</button>
							{/each}
						</div>
					{/if}
				</div>

				<!-- Tab content -->
				<div class="flex-1 overflow-auto panel-inset min-h-0 select-text">
					{#if detailTab === 'headers'}
						<div class="p-2 space-y-3">
							<!-- Request headers -->
							<div>
								<div class="text-[9px] uppercase tracking-wider text-muted-foreground/70 font-semibold mb-1 flex items-center gap-1">
									Request Headers
									<span class="text-[8px] normal-case text-muted-foreground/40">({Object.keys(selectedReq.headers).length})</span>
								</div>
								{#each Object.entries(selectedReq.headers) as [key, value]}
									<div class="flex items-baseline gap-1 font-mono text-[10px] group py-px">
										<span class="text-orange shrink-0 w-[190px] truncate">{key}:</span>
										<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
										<button class="shrink-0 opacity-0 group-hover:opacity-100 ml-1" onclick={() => copyText(key, `${key}: ${value}`)}>
											{#if copied === key}<Check size={8} class="text-green" />{:else}<Copy size={8} class="text-muted-foreground" />{/if}
										</button>
									</div>
								{/each}
							</div>
							<!-- Response headers -->
							{#if selectedReq.resp_headers}
								<div class="border-t border-border/50 pt-2">
									<div class="text-[9px] uppercase tracking-wider text-muted-foreground/70 font-semibold mb-1 flex items-center gap-1">
										Response Headers
										<span class="text-[8px] normal-case text-muted-foreground/40 font-normal {statusColor(selectedReq.resp_status)}">{selectedReq.resp_status} {selectedReq.resp_status_text}</span>
									</div>
									{#each Object.entries(selectedReq.resp_headers) as [key, value]}
										<div class="flex items-baseline gap-1 font-mono text-[10px] group py-px">
											<span class="text-blue shrink-0 w-[190px] truncate">{key}:</span>
											<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
										</div>
									{/each}
								</div>
							{/if}
						</div>

					{:else if detailTab === 'payload'}
						<div class="p-2">
							{#if selectedReq.post_data}
								<pre class="font-mono text-[10px] text-foreground whitespace-pre-wrap break-all">{formatBody(selectedReq.post_data, 'application/x-www-form-urlencoded')}</pre>
							{:else}
								<div class="text-muted-foreground/40 text-[10px] italic">No request body</div>
							{/if}
						</div>

					{:else if detailTab === 'response'}
						<div class="p-2">
							{#if selectedReq.resp_body !== undefined}
								<pre class="font-mono text-[10px] text-foreground whitespace-pre-wrap break-all">{formatBody(selectedReq.resp_body, selectedReq.resp_mime)}</pre>
							{:else if selectedReq.resp_status}
								<div class="text-muted-foreground/40 text-[10px] italic">
									{selectedReq.resp_status >= 300 ? 'Redirect — no body' : 'Loading response body…'}
								</div>
							{:else}
								<div class="text-muted-foreground/40 text-[10px] italic">Waiting for response…</div>
							{/if}
						</div>

					{:else if detailTab === 'params'}
						{#if parseParams(selectedReq.url).length}
							<div class="p-2">
								<div class="text-[9px] uppercase tracking-wider text-muted-foreground/70 font-semibold mb-1">Query Parameters</div>
								{#each parseParams(selectedReq.url) as [k, v]}
									<div class="flex items-baseline gap-1 font-mono text-[10px] py-px">
										<span class="text-primary shrink-0 w-[160px] truncate">{decodeURIComponent(k)}:</span>
										<span class="text-foreground break-all flex-1 min-w-0">{decodeURIComponent(v)}</span>
									</div>
								{/each}
							</div>
						{:else}
							<div class="p-2 text-muted-foreground/40 text-[10px] italic">No query parameters</div>
						{/if}
					{/if}
				</div>
			{:else}
				<div class="flex items-center justify-center flex-1 text-muted-foreground/40 text-[11px] panel-inset">
					Select a request from the list
				</div>
			{/if}
		</div>
	</div>
	{/if}

	{:else if mode === 'proxy'}
	<!-- ══ PROXY CAPTURE MODE ═══════════════════════════════════════════════ -->

	<!-- Controls bar -->
	<div class="flex items-center gap-2 px-2 py-1.5 panel-raised shrink-0">
		<span class="text-[10px] text-muted-foreground shrink-0">Port</span>
		<input type="number" min="1024" max="65535" bind:value={proxyPort}
			class="skeu-input text-[11px] font-mono w-20 shrink-0"
			disabled={proxyActive}
		/>
		{#if proxyActive}
			<button class="skeu-btn flex items-center gap-1 text-[11px] text-red shrink-0" onclick={stopProxy}>
				<Square size={11} />Stop Proxy
			</button>
		{:else}
			<button class="skeu-btn flex items-center gap-1 text-[11px] text-green shrink-0" onclick={startProxy}>
				<Play size={11} />Start Proxy
			</button>
		{/if}
		<button class="skeu-btn text-[10px] text-muted-foreground ml-auto shrink-0" onclick={() => { capturedRequests = []; selectedReqId = null; try { localStorage.removeItem('ib_inspector_captures'); } catch {} }}>
			<Trash2 size={10} />Clear
		</button>
	</div>

	<!-- Setup instructions -->
	{#if !proxyActive && capturedRequests.length === 0}
	<div class="flex flex-col items-center justify-center flex-1 gap-3 p-6 text-center">
		<List size={28} class="text-muted-foreground/30" />
		<div class="space-y-1">
			<p class="text-[11px] font-medium text-foreground">Proxy Capture</p>
			<p class="text-[10px] text-muted-foreground max-w-xs">No Chrome required. Start the proxy, then configure your browser or system to use <code class="text-foreground/80">127.0.0.1:{proxyPort}</code> as an HTTP proxy.</p>
		</div>
		<ol class="text-left text-[10px] text-muted-foreground space-y-1 max-w-xs">
			<li>1. Click <span class="text-green font-medium">Start Proxy</span></li>
			<li>2. In Chrome/Firefox: Settings → Proxy → Manual → <code class="text-foreground/80">127.0.0.1:{proxyPort}</code></li>
			<li>3. Browse to your target — requests appear here in real time</li>
			<li>4. HTTPS tunnels are relayed but not decrypted (no MitM)</li>
		</ol>
		{#if proxyError}
			<p class="text-[10px] text-red bg-red/10 rounded px-3 py-1.5 border border-red/20 max-w-xs">{proxyError}</p>
		{/if}
	</div>
	{:else}
	<!-- Reuse the same captured requests panel from browser mode -->
	<div class="flex flex-1 min-h-0">
		<!-- Left: request list -->
		<div class="flex flex-col border-r border-border shrink-0 overflow-hidden" style="width: {splitWidth}px">
			<div class="flex items-center gap-1 px-2 py-1 border-b border-border shrink-0 bg-background/40">
				<input type="text" bind:value={searchQuery} placeholder="Filter…" class="skeu-input text-[10px] flex-1 min-w-0" />
			</div>
			<div class="flex-1 overflow-y-auto">
				{#each filteredRequests as req (req.id)}
					<button
						class="w-full text-left flex items-center gap-1.5 px-2 py-0.5 hover:bg-accent/10 transition-colors border-b border-border/30 {selectedReqId === req.id ? 'bg-primary/10' : ''}"
						onclick={() => selectReq(req.id)}
					>
						<span class="font-mono text-[9px] shrink-0 {req.method === 'CONNECT' ? 'text-muted-foreground' : req.method === 'POST' ? 'text-orange' : 'text-blue-400'}">{req.method}</span>
						<span class="text-[9px] text-muted-foreground truncate font-mono">{req.url.replace(/^https?:\/\//, '')}</span>
						{#if req.resp_status}
							<span class="ml-auto text-[9px] font-mono shrink-0 {req.resp_status >= 400 ? 'text-red' : req.resp_status >= 300 ? 'text-orange' : 'text-green'}">{req.resp_status}</span>
						{/if}
					</button>
				{/each}
				{#if filteredRequests.length === 0}
					<p class="text-[10px] text-muted-foreground p-4 text-center">{proxyActive ? 'Waiting for traffic…' : 'No requests captured.'}</p>
				{/if}
			</div>
		</div>
		<!-- Right: detail — reuse selectedReq display identical to browser mode -->
		<div class="flex-1 min-w-0 flex flex-col overflow-hidden">
			{#if selectedReq}
				<div class="flex items-center gap-1 px-2 py-1 border-b border-border bg-background/40 shrink-0 flex-wrap">
					{#each ['headers','payload','response'] as t}
						<button
							class="px-2 py-0.5 rounded text-[10px] transition-colors {detailTab === t ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:text-foreground'}"
							onclick={() => detailTab = t as any}
						>{t.charAt(0).toUpperCase() + t.slice(1)}</button>
					{/each}
				</div>
				<div class="flex-1 overflow-y-auto p-2 font-mono text-[10px]">
					{#if detailTab === 'headers'}
						<p class="text-muted-foreground mb-1">Request Headers</p>
						{#each Object.entries(selectedReq.headers) as [k, v]}
							<div class="flex gap-2 py-0.5 border-b border-border/20"><span class="text-primary/80 shrink-0">{k}</span><span class="text-foreground/70 truncate">{v}</span></div>
						{/each}
					{:else if detailTab === 'payload'}
						<pre class="whitespace-pre-wrap break-all text-foreground/80">{selectedReq.post_data ?? '(no body)'}</pre>
					{:else if detailTab === 'response'}
						{#if selectedReq.resp_headers}
							<p class="text-muted-foreground mb-1">Response Headers</p>
							{#each Object.entries(selectedReq.resp_headers) as [k, v]}
								<div class="flex gap-2 py-0.5 border-b border-border/20"><span class="text-primary/80 shrink-0">{k}</span><span class="text-foreground/70 truncate">{v}</span></div>
							{/each}
						{/if}
						{#if selectedReq.resp_body}
							<p class="text-muted-foreground mt-2 mb-1">Body</p>
							<pre class="whitespace-pre-wrap break-all text-foreground/80">{selectedReq.resp_body}</pre>
						{/if}
					{/if}
				</div>
			{:else}
				<div class="flex-1 flex items-center justify-center text-muted-foreground text-[10px]">Select a request</div>
			{/if}
		</div>
	</div>
	{/if}

	{:else}
	<!-- ══ MANUAL MODE ═══════════════════════════════════════════════════════ -->

	<!-- ══ Address bar ═══════════════════════════════════════════════════════ -->
	<div class="flex items-center gap-1.5 px-2 py-1.5 panel-raised shrink-0">
		<!-- Method -->
		<SkeuSelect
			value={method}
			onValueChange={(v) => { method = v; }}
			options={METHODS}
			class="text-[11px] font-medium w-[80px] shrink-0"
		/>

		<!-- URL -->
		<input
			type="text" bind:value={url}
			placeholder="https://target.com/login"
			class="skeu-input text-[11px] font-mono flex-1 min-w-0"
			onkeydown={(e) => { if (e.key === 'Enter') capture(); }}
		/>

		<!-- Go / Stop -->
		{#if loading}
			<button class="skeu-btn flex items-center gap-1 text-[11px] text-red shrink-0" onclick={stop}>
				<Square size={10} fill="currentColor" />Stop
			</button>
		{:else}
			<button class="skeu-btn flex items-center gap-1 text-[11px] shrink-0" onclick={capture}>
				<Globe size={11} />Go
			</button>
		{/if}

		<!-- Browser -->
		<SkeuSelect
			value={browser}
			onValueChange={(v) => { browser = v; }}
			options={BROWSERS}
			class="text-[11px] w-[72px] shrink-0"
		/>

		<!-- Proxy -->
		<input
			type="text" bind:value={proxy}
			placeholder="Proxy (optional)"
			class="skeu-input text-[10px] font-mono w-[150px] shrink-0"
		/>

		{#if result}
			<button
				class="skeu-btn flex items-center gap-1 text-[11px] text-primary shrink-0"
				onclick={() => { showApplyPanel = !showApplyPanel; }}
			><ArrowRight size={11} />Apply to Block</button>
		{/if}
	</div>

	<!-- ══ Error bar ══════════════════════════════════════════════════════════ -->
	{#if errorMsg}
		<div class="px-2 py-0.5 bg-red/10 border-b border-red/20 text-red text-[10px] shrink-0">{errorMsg}</div>
	{/if}

	<!-- ══ Main split: Request Builder | Response Viewer ════════════════════ -->
	<div class="flex flex-1 min-h-0 overflow-hidden">

		<!-- ── LEFT: Request Builder ─────────────────────────────────────────── -->
		<div class="w-[300px] shrink-0 flex flex-col border-r border-border bg-background/40">
			<div class="px-2 py-1 text-[9px] uppercase tracking-widest text-muted-foreground font-semibold border-b border-border/50">Request</div>

			<div class="flex-1 overflow-y-auto p-2 space-y-2">

				<!-- ── Headers ───────────────────────────────────────────── -->
				<div>
					<div class="flex items-center gap-1 mb-1">
						<span class="text-[9px] uppercase tracking-wider text-muted-foreground flex-1">Headers</span>
						<!-- KV / Raw toggle -->
						<div class="flex rounded border border-border overflow-hidden">
							{#each [['kv','KV'],['raw','Raw']] as [m, l]}
								<button
									class="px-1.5 py-0 text-[9px] transition-colors {headerMode === m ? 'bg-primary text-primary-foreground' : 'bg-background text-muted-foreground hover:bg-accent/20'}"
									onclick={() => switchHeaderMode(m as 'kv' | 'raw')}
								>{l}</button>
							{/each}
						</div>
						{#if headerMode === 'kv'}
							<button class="text-[9px] text-primary hover:underline" onclick={() => extraHeaders = [...extraHeaders, ['', '']]}>
								<Plus size={9} class="inline" /> Add
							</button>
						{/if}
					</div>

					{#if headerMode === 'kv'}
						<div class="space-y-0.5">
							{#each extraHeaders as [k, v], i}
								<div class="flex gap-1 items-center">
									<input
										class="skeu-input text-[10px] font-mono flex-1 min-w-0" placeholder="Header-Name"
										bind:value={extraHeaders[i][0]}
									/>
									<span class="text-muted-foreground/40 shrink-0">:</span>
									<input
										class="skeu-input text-[10px] font-mono flex-1 min-w-0" placeholder="value"
										bind:value={extraHeaders[i][1]}
									/>
									<button class="p-0.5 text-muted-foreground hover:text-red shrink-0"
										onclick={() => extraHeaders = extraHeaders.filter((_, j) => j !== i)}>
										<Trash2 size={9} />
									</button>
								</div>
							{/each}
							{#if extraHeaders.length === 0}
								<div class="text-[9px] text-muted-foreground/40 italic">No custom headers — click Add</div>
							{/if}
						</div>
					{:else}
						<textarea
							bind:value={rawHeaderText}
							placeholder="Header-Name: value&#10;Another-Header: value"
							class="skeu-input text-[10px] font-mono w-full resize-y"
							rows={4}
						></textarea>
					{/if}
				</div>

				<!-- ── Request Body ───────────────────────────────────────── -->
				<div>
					<div class="flex items-center gap-1 mb-1">
						<span class="text-[9px] uppercase tracking-wider {bodyAllowed ? 'text-muted-foreground' : 'text-muted-foreground/30'} flex-1">
							Request Body
						</span>
						{#if !bodyAllowed}
							<span class="text-[9px] text-muted-foreground/30 italic">not used for {method}</span>
						{/if}
					</div>
					<textarea
						bind:value={bodyText}
						placeholder={bodyAllowed
							? 'Paste POST body here — JSON, form data, XML…\ne.g. username=admin&password=test\nor {"username":"admin","password":"test"}'
							: `Body is ignored for ${method} requests`}
						disabled={!bodyAllowed}
						class="skeu-input text-[10px] font-mono w-full resize-y {!bodyAllowed ? 'opacity-40 cursor-not-allowed' : ''}"
						rows={6}
					></textarea>
					{#if bodyAllowed && bodyText.trim()}
						<p class="text-[8px] text-muted-foreground/50 mt-0.5 leading-tight">
							Tip: add <code class="font-mono">Content-Type: application/json</code> header above if sending JSON
						</p>
					{/if}
				</div>
			</div>
		</div>

		<!-- ── RIGHT: Response Viewer ─────────────────────────────────────────── -->
		<div class="flex-1 flex flex-col min-w-0">

			{#if result}
				<!-- Status bar -->
				<div class="flex items-center gap-2 px-2 py-0.5 border-b border-border bg-background/60 shrink-0">
					<span class="font-bold tabular-nums {statusColor(result.status)}">{result.status}</span>
					<span class="font-mono text-primary truncate flex-1 min-w-0 text-[10px]">{result.final_url}</span>
					<span class="text-muted-foreground tabular-nums shrink-0 text-[10px]">{result.timing_ms}ms</span>
					<span class="text-[9px] shrink-0 {result.via === 'reqwest' ? 'text-orange/60' : 'text-green/60'}"
						title={result.via === 'reqwest' ? 'AzureTLS sidecar not running — native HTTP used' : 'AzureTLS ' + (result.browser ?? '')}>
						{result.via === 'reqwest' ? 'native' : result.browser ?? 'azuretls'}
					</span>
				</div>

				<!-- Apply panel -->
				{#if showApplyPanel}
					<div class="px-2 py-2 border-b border-border bg-background shrink-0">
						<div class="flex items-center gap-2 mb-1.5">
							<span class="text-[10px] font-medium text-foreground">Apply to HTTP Request Block</span>
							<div class="flex rounded border border-border overflow-hidden">
								{#each [['response','Response Hdrs'],['request','Request Hdrs']] as [val, lbl]}
									<button
										class="px-2 py-0.5 text-[9px] transition-colors {applySource === val ? 'bg-primary text-primary-foreground' : 'bg-background text-muted-foreground hover:bg-accent/20'}"
										onclick={() => {
											applySource = val as 'request' | 'response';
											applySelection = new Set(Object.keys(val === 'request' ? result!.request_headers : result!.headers));
										}}
									>{lbl}</button>
								{/each}
							</div>
							<div class="flex-1"></div>
							<button class="text-[9px] text-primary hover:underline" onclick={() => applySelection = new Set(Object.keys(applySource === 'request' ? result!.request_headers : result!.headers))}>All</button>
							<button class="text-[9px] text-muted-foreground hover:underline" onclick={() => applySelection = new Set()}>None</button>
						</div>
						<div class="max-h-[100px] overflow-y-auto space-y-0.5 mb-2 select-text">
							{#each Object.entries(applySource === 'request' ? result.request_headers : result.headers) as [key, value]}
								<label class="flex items-center gap-1.5 cursor-pointer">
									<input type="checkbox"
										checked={applySelection.has(key)}
										onchange={(e) => {
											const n = new Set(applySelection);
											if ((e.target as HTMLInputElement).checked) n.add(key); else n.delete(key);
											applySelection = n;
										}}
										class="w-3 h-3 shrink-0"
									/>
									<span class="text-primary font-mono shrink-0 w-[140px] truncate text-[10px]">{key}:</span>
									<span class="text-foreground font-mono truncate flex-1 min-w-0 text-[10px]">{value}</span>
								</label>
							{/each}
						</div>
						<button class="skeu-btn text-[11px] text-primary" onclick={applyToBlock}>
							Apply {applySelection.size} header{applySelection.size !== 1 ? 's' : ''} to block
						</button>
					</div>
				{/if}

				<!-- Response tabs -->
				<div class="flex items-center border-b border-border shrink-0">
					{#each [
						['resp-headers', 'Response Headers'],
						['req-headers',  'Request Headers'],
						['body',         'Body'],
						['cookies',      'Cookies'],
					] as [id, lbl]}
						<button
							class="px-2.5 py-0.5 text-[11px] {viewTab === id ? 'text-foreground font-medium border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'}"
							onclick={() => viewTab = id as typeof viewTab}
						>{lbl}<span class="text-[9px] text-muted-foreground/50">{tabCount(id as typeof viewTab)}</span></button>
					{/each}
					<div class="flex-1"></div>
					<!-- Copy all -->
					<button class="px-2 py-0.5 text-muted-foreground hover:text-foreground"
						onclick={async () => {
							let t = '';
							if (viewTab === 'resp-headers') t = Object.entries(result!.headers).map(([k,v])=>`${k}: ${v}`).join('\n');
							else if (viewTab === 'req-headers') t = Object.entries(result!.request_headers).map(([k,v])=>`${k}: ${v}`).join('\n');
							else if (viewTab === 'body') t = result!.body;
							else t = Object.entries(result!.cookies).map(([k,v])=>`${k}=${v}`).join('\n');
							await copyText('__all__', t);
						}} title="Copy all">
						{#if copied === '__all__'}<Check size={11} class="text-green" />{:else}<Copy size={11} />{/if}
					</button>
				</div>

				<!-- Response content -->
				<div class="flex-1 overflow-auto panel-inset min-h-0">
					{#if viewTab === 'resp-headers' || viewTab === 'req-headers'}
						{@const entries = Object.entries(viewTab === 'resp-headers' ? result.headers : result.request_headers)}
						<div class="p-2 space-y-0.5 select-text">
							{#if entries.length === 0}
								{#if viewTab === 'req-headers'}
									<div class="text-[10px] text-muted-foreground/50 italic">
										Request headers not captured. Start a debug run to initialise the AzureTLS sidecar, then capture again.
									</div>
								{:else}
									<div class="text-[10px] text-muted-foreground/50">No headers</div>
								{/if}
							{:else}
								{#each entries as [key, value]}
									<div class="flex items-baseline gap-1 font-mono text-[10px] group">
										<span class="{viewTab === 'resp-headers' ? 'text-primary' : 'text-orange'} shrink-0 w-[200px] truncate">{key}:</span>
										<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
										<button class="shrink-0 p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground"
											onclick={() => copyText(key, `${key}: ${value}`)} title="Copy">
											{#if copied === key}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if}
										</button>
									</div>
								{/each}
							{/if}
						</div>

					{:else if viewTab === 'body'}
						<div class="p-2 select-text">
							<pre class="font-mono text-[10px] text-foreground whitespace-pre-wrap break-words">{prettyBody(result.body)}</pre>
						</div>

					{:else if viewTab === 'cookies'}
						<div class="p-2 space-y-0.5 select-text">
							{#if Object.keys(result.cookies).length === 0}
								<div class="text-muted-foreground/50 text-[10px]">No cookies set</div>
							{:else}
								{#each Object.entries(result.cookies) as [key, value]}
									<div class="flex items-baseline gap-1 font-mono text-[10px] group">
										<span class="text-purple shrink-0 w-[160px] truncate">{key}=</span>
										<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
										<button class="shrink-0 p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground"
											onclick={() => copyText('ck_'+key, `${key}=${value}`)} title="Copy">
											{#if copied === 'ck_'+key}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if}
										</button>
									</div>
								{/each}
							{/if}
						</div>
					{/if}
				</div>

			{:else if loading}
				<div class="flex items-center justify-center flex-1 gap-2 text-muted-foreground panel-inset">
					<Loader2 size={14} class="animate-spin" /><span>Sending request...</span>
				</div>

			{:else}
				<!-- Empty state -->
				<div class="flex flex-col items-center justify-center flex-1 gap-2 text-muted-foreground panel-inset">
					<Globe size={28} class="text-muted-foreground/25" />
					<div class="text-[11px] text-center leading-relaxed">
						Enter a URL and click <strong>Go</strong> to inspect the site.<br/>
						Response headers, request headers, body and cookies all appear here.
					</div>
					<div class="text-[9px] text-muted-foreground/40 text-center max-w-[280px] leading-relaxed">
						Uses AzureTLS with the selected browser profile for real TLS fingerprinting.<br/>
						Use <strong>Apply to Block</strong> to fill headers directly into your HTTP Request block.
					</div>
				</div>
			{/if}
		</div>
	</div>

	{/if}
	<!-- ══ END MODE IF ══════════════════════════════════════════════════════ -->
</div>
