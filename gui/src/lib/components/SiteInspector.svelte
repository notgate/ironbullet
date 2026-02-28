<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send, onResponse } from '$lib/ipc';
	import Copy from '@lucide/svelte/icons/copy';
	import Globe from '@lucide/svelte/icons/globe';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Check from '@lucide/svelte/icons/check';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import Square from '@lucide/svelte/icons/square';

	// ── State ──────────────────────────────────────────────────────────────────
	let url      = $state('https://');
	let method   = $state('GET');
	let proxy    = $state('');
	let browser  = $state('chrome');
	let bodyText = $state('');
	let extraHeaders = $state<[string, string][]>([]);
	let headerMode = $state<'kv' | 'raw'>('kv');
	let rawHeaderText = $state('');

	let loading  = $state(false);
	let errorMsg = $state('');
	let result   = $state<InspectResult | null>(null);
	let viewTab  = $state<'resp-headers' | 'req-headers' | 'body' | 'cookies'>('resp-headers');
	let copied   = $state<string | null>(null);

	let showApplyPanel  = $state(false);
	let applySource     = $state<'request' | 'response'>('response');
	let applySelection  = $state<Set<string>>(new Set());

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

	const METHODS  = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
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
	let unsub: (() => void) | null = null;

	function capture() {
		if (!url.trim() || url === 'https://') return;
		loading  = true;
		errorMsg = '';

		// Sync raw headers back to KV before sending
		const hdrs: [string, string][] = headerMode === 'raw' ? rawToKv(rawHeaderText) : extraHeaders;

		unsub?.();
		unsub = onResponse('site_inspect_result', (data: unknown) => {
			loading = false;
			unsub?.(); unsub = null;
			const d = data as InspectResult;
			if (d.error && !d.status) { errorMsg = d.error; return; }
			result = d;
			const keys = new Set(Object.keys(d.headers));
			applySelection = keys;
			viewTab = 'resp-headers';
		});

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
		unsub?.(); unsub = null;
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

	function statusColor(c: number): string {
		if (c >= 200 && c < 300) return 'text-green';
		if (c >= 300 && c < 400) return 'text-blue';
		if (c >= 400 && c < 500) return 'text-orange';
		return 'text-red';
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

	<!-- ══ Address bar ═══════════════════════════════════════════════════════ -->
	<div class="flex items-center gap-1.5 px-2 py-1.5 panel-raised shrink-0">
		<!-- Method -->
		<select bind:value={method} class="skeu-select text-[11px] font-medium w-[80px] shrink-0">
			{#each METHODS as m}<option value={m}>{m}</option>{/each}
		</select>

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
		<select bind:value={browser} class="skeu-select text-[11px] w-[72px] shrink-0">
			{#each BROWSERS as b}<option value={b.value}>{b.label}</option>{/each}
		</select>

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
</div>
