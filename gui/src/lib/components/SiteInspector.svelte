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

	// ── State ──────────────────────────────────────────────────────────────────
	let url     = $state('https://');
	let method  = $state('GET');
	let proxy   = $state('');
	let browser = $state('chrome');
	let bodyText = $state('');
	let extraHeaders = $state<[string, string][]>([]);

	let loading   = $state(false);
	let errorMsg  = $state('');
	let result    = $state<InspectResult | null>(null);
	let viewTab   = $state<'request' | 'response' | 'body' | 'cookies'>('response');
	let copied    = $state<string | null>(null); // key of copied item

	// Which headers to apply: Set of "key: value" strings that are checked
	let applySelection = $state<Set<string>>(new Set());
	let showApplyPanel = $state(false);
	let applySource   = $state<'request' | 'response'>('response');

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

	const METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'];
	const BROWSERS = [
		{ value: 'chrome',  label: 'Chrome' },
		{ value: 'firefox', label: 'Firefox' },
		{ value: 'safari',  label: 'Safari' },
		{ value: 'edge',    label: 'Edge' },
	];

	// ── IPC ────────────────────────────────────────────────────────────────────
	let unsub: (() => void) | null = null;

	function capture() {
		if (!url.trim() || url === 'https://') return;
		loading  = true;
		errorMsg = '';
		result   = null;

		unsub?.();
		unsub = onResponse('site_inspect_result', (data: unknown) => {
			loading = false;
			unsub?.();
			unsub = null;
			const d = data as InspectResult;
			if (d.error && !d.status) {
				errorMsg = d.error;
				return;
			}
			result = d;
			// Pre-select all response headers for "Apply"
			const keys = new Set<string>();
			for (const k of Object.keys(d.headers)) keys.add(k);
			applySelection = keys;
		});

		send('site_inspect', {
			url:     url.trim(),
			method,
			proxy:   proxy.trim() || null,
			browser,
			body:    bodyText || null,
			headers: extraHeaders.filter(([k]) => k.trim()).map(([k, v]) => [k, v]),
		});
	}

	// ── Apply to Block ─────────────────────────────────────────────────────────
	function applyToBlock() {
		if (!result) return;
		const src = applySource === 'request' ? result.request_headers : result.headers;
		const selected = Object.entries(src).filter(([k]) => applySelection.has(k));
		if (!selected.length) return;

		// Find currently editing/selected HTTP block
		const targetBlock = app.pipeline.blocks.find(
			b => b.id === app.selectedBlockId && b.settings.type === 'HttpRequest'
		) ?? app.pipeline.blocks.find(b => b.settings.type === 'HttpRequest');

		if (!targetBlock || targetBlock.settings.type !== 'HttpRequest') {
			errorMsg = 'Select an HTTP Request block first';
			return;
		}

		// Merge: keep existing headers, add/overwrite with selected
		const existing: [string, string][] = [...(targetBlock.settings.headers ?? [])];
		for (const [key, value] of selected) {
			const idx = existing.findIndex(([k]) => k.toLowerCase() === key.toLowerCase());
			if (idx >= 0) existing[idx] = [existing[idx][0], value];
			else existing.push([key, value]);
		}

		// Update block in pipeline
		app.pipeline.blocks = app.pipeline.blocks.map(b => {
			if (b.id !== targetBlock.id) return b;
			return { ...b, settings: { ...b.settings, headers: existing } };
		});

		showApplyPanel = false;
		errorMsg = '';
	}

	// ── Copy helper ────────────────────────────────────────────────────────────
	async function copyText(key: string, text: string) {
		try {
			await navigator.clipboard.writeText(text);
			copied = key;
			setTimeout(() => { copied = null; }, 1500);
		} catch {}
	}

	function statusColor(code: number): string {
		if (code >= 200 && code < 300) return 'text-green';
		if (code >= 300 && code < 400) return 'text-blue';
		if (code >= 400 && code < 500) return 'text-orange';
		return 'text-red';
	}

	function formatBody(body: string): string {
		try { return JSON.stringify(JSON.parse(body), null, 2); } catch { return body; }
	}
</script>

<div class="flex flex-col h-full bg-surface text-foreground text-[11px]">

	<!-- ── Toolbar ─────────────────────────────────────────────────────────── -->
	<div class="flex items-center gap-1.5 px-2 py-1.5 panel-raised flex-wrap shrink-0">
		<!-- Method -->
		<select
			bind:value={method}
			class="skeu-select text-[11px] w-[76px] shrink-0"
		>
			{#each METHODS as m}<option value={m}>{m}</option>{/each}
		</select>

		<!-- URL -->
		<input
			type="text"
			bind:value={url}
			placeholder="https://example.com/login"
			class="skeu-input text-[11px] font-mono flex-1 min-w-[200px]"
			onkeydown={(e) => { if (e.key === 'Enter') capture(); }}
		/>

		<!-- Browser profile -->
		<select bind:value={browser} class="skeu-select text-[11px] w-[76px] shrink-0">
			{#each BROWSERS as b}<option value={b.value}>{b.label}</option>{/each}
		</select>

		<!-- Proxy -->
		<input
			type="text"
			bind:value={proxy}
			placeholder="Proxy (optional)"
			class="skeu-input text-[11px] font-mono w-[160px] shrink-0"
		/>

		<button
			class="skeu-btn flex items-center gap-1 text-[11px] shrink-0 {loading ? 'opacity-50' : ''}"
			onclick={capture}
			disabled={loading}
		>
			{#if loading}
				<Loader2 size={11} class="animate-spin" />Capturing...
			{:else}
				<Globe size={11} />Capture
			{/if}
		</button>

		{#if result}
			<button
				class="skeu-btn flex items-center gap-1 text-[11px] text-primary shrink-0"
				onclick={() => { showApplyPanel = !showApplyPanel; }}
			>
				<ArrowRight size={11} />Apply to Block
			</button>
		{/if}
	</div>

	<!-- ── Extra request headers (collapsible) ─────────────────────────────── -->
	{#if extraHeaders.length > 0 || method === 'POST' || method === 'PUT' || method === 'PATCH'}
		<div class="px-2 py-1 border-b border-border bg-background/50 shrink-0">
			<div class="flex items-center gap-1 mb-1">
				<span class="text-[9px] uppercase tracking-wider text-muted-foreground">Request Headers</span>
				<button class="text-[9px] text-primary hover:underline ml-auto" onclick={() => extraHeaders = [...extraHeaders, ['', '']]}>+ Add</button>
			</div>
			{#each extraHeaders as [k, v], i}
				<div class="flex gap-1 mb-0.5 items-center">
					<input class="skeu-input text-[10px] font-mono flex-1" bind:value={extraHeaders[i][0]} placeholder="Header-Name" />
					<span class="text-muted-foreground/40">:</span>
					<input class="skeu-input text-[10px] font-mono flex-1" bind:value={extraHeaders[i][1]} placeholder="value" />
					<button class="p-0.5 text-muted-foreground hover:text-red shrink-0" onclick={() => extraHeaders = extraHeaders.filter((_, j) => j !== i)}><Trash2 size={9} /></button>
				</div>
			{/each}
			{#if method !== 'GET' && method !== 'HEAD'}
				<textarea
					bind:value={bodyText}
					placeholder="Request body (optional)"
					class="skeu-input text-[10px] font-mono w-full mt-1 resize-y"
					rows={2}
				></textarea>
			{/if}
		</div>
	{:else}
		<!-- minimal add-headers link -->
		<div class="px-2 py-0.5 border-b border-border/30 shrink-0">
			<button class="text-[9px] text-muted-foreground/50 hover:text-primary hover:underline" onclick={() => extraHeaders = [...extraHeaders, ['', '']]}>+ Custom headers</button>
		</div>
	{/if}

	<!-- ── Error ────────────────────────────────────────────────────────────── -->
	{#if errorMsg}
		<div class="px-2 py-1 bg-red/10 border-b border-red/30 text-red shrink-0 text-[10px]">{errorMsg}</div>
	{/if}

	<!-- ── Apply panel ──────────────────────────────────────────────────────── -->
	{#if showApplyPanel && result}
		<div class="px-2 py-2 border-b border-border bg-background shrink-0">
			<div class="flex items-center gap-2 mb-1.5">
				<span class="text-[10px] font-medium">Apply to HTTP Request Block</span>
				<!-- source selector -->
				<div class="flex rounded border border-border overflow-hidden">
					{#each [['response', 'Response Headers'], ['request', 'Request Headers']] as [val, label]}
						<button
							class="px-2 py-0.5 text-[9px] font-medium transition-colors {applySource === val ? 'bg-primary text-primary-foreground' : 'bg-background text-muted-foreground hover:bg-accent/20'}"
							onclick={() => { applySource = val as 'request' | 'response'; applySelection = new Set(Object.keys(val === 'request' ? result!.request_headers : result!.headers)); }}
						>{label}</button>
					{/each}
				</div>
				<div class="flex-1"></div>
				<button class="text-[9px] text-primary hover:underline" onclick={() => applySelection = new Set(Object.keys(applySource === 'request' ? result!.request_headers : result!.headers))}>All</button>
				<button class="text-[9px] text-muted-foreground hover:underline" onclick={() => applySelection = new Set()}>None</button>
			</div>
			<div class="max-h-[110px] overflow-y-auto space-y-0.5 mb-2">
				{#each Object.entries(applySource === 'request' ? result.request_headers : result.headers) as [key, value]}
					<label class="flex items-center gap-1.5 cursor-pointer group select-text">
						<input type="checkbox"
							checked={applySelection.has(key)}
							onchange={(e) => {
								const next = new Set(applySelection);
								if ((e.target as HTMLInputElement).checked) next.add(key); else next.delete(key);
								applySelection = next;
							}}
							class="w-3 h-3 shrink-0"
						/>
						<span class="text-primary font-mono shrink-0 min-w-[140px] truncate">{key}:</span>
						<span class="text-foreground font-mono truncate flex-1 min-w-0">{value}</span>
					</label>
				{/each}
			</div>
			<button
				class="skeu-btn text-[11px] text-primary"
				onclick={applyToBlock}
			>Apply {applySelection.size} header{applySelection.size !== 1 ? 's' : ''} to block</button>
		</div>
	{/if}

	{#if result}
		<!-- ── Status bar ────────────────────────────────────────────────────── -->
		<div class="flex items-center gap-3 px-2 py-1 border-b border-border bg-background shrink-0 text-[11px]">
			<span class="font-bold {statusColor(result.status)}">{result.status}</span>
			<span class="font-mono text-primary truncate flex-1 min-w-0">{result.final_url}</span>
			<span class="text-muted-foreground tabular-nums shrink-0">{result.timing_ms}ms</span>
			{#if result.via === 'reqwest'}
				<span class="text-[9px] text-orange/70 shrink-0" title="AzureTLS sidecar not running — using native HTTP">native fallback</span>
			{:else}
				<span class="text-[9px] text-green/70 shrink-0">{result.browser ?? result.via}</span>
			{/if}
		</div>

		<!-- ── Tabs ──────────────────────────────────────────────────────────── -->
		<div class="flex border-b border-border shrink-0">
			{#each [['response','Response Headers'], ['request','Request Headers'], ['body','Body'], ['cookies','Cookies']] as [id, label]}
				<button
					class="px-2.5 py-0.5 text-[11px] {viewTab === id ? 'text-foreground font-medium border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'}"
					onclick={() => viewTab = id as typeof viewTab}
				>{label}
				{#if id === 'response'}
					<span class="ml-1 text-[9px] text-muted-foreground/60">({Object.keys(result.headers).length})</span>
				{:else if id === 'request'}
					<span class="ml-1 text-[9px] text-muted-foreground/60">({Object.keys(result.request_headers).length})</span>
				{:else if id === 'cookies'}
					<span class="ml-1 text-[9px] text-muted-foreground/60">({Object.keys(result.cookies).length})</span>
				{/if}
				</button>
			{/each}
			<div class="flex-1"></div>
			<!-- Copy all button for current tab -->
			<button
				class="px-2 py-0.5 text-muted-foreground hover:text-foreground"
				onclick={async () => {
					let text = '';
					if (viewTab === 'response') text = Object.entries(result!.headers).map(([k,v]) => `${k}: ${v}`).join('\n');
					else if (viewTab === 'request') text = Object.entries(result!.request_headers).map(([k,v]) => `${k}: ${v}`).join('\n');
					else if (viewTab === 'body') text = result!.body;
					else text = Object.entries(result!.cookies).map(([k,v]) => `${k}=${v}`).join('\n');
					await copyText('__all__', text);
				}}
				title="Copy all"
			>{#if copied === '__all__'}<Check size={11} class="text-green" />{:else}<Copy size={11} />{/if}</button>
		</div>

		<!-- ── Content ────────────────────────────────────────────────────────── -->
		<div class="flex-1 overflow-auto panel-inset min-h-0">
			{#if viewTab === 'response'}
				<div class="p-2 space-y-0.5 select-text">
					{#if Object.keys(result.headers).length === 0}
						<div class="text-muted-foreground/50">No response headers captured</div>
					{:else}
						{#each Object.entries(result.headers) as [key, value]}
							<div class="flex items-baseline gap-1 font-mono text-[10px] group">
								<span class="text-primary shrink-0 w-[200px] truncate">{key}:</span>
								<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
								<button class="shrink-0 p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground"
									onclick={() => copyText(key, `${key}: ${value}`)} title="Copy">
									{#if copied === key}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if}
								</button>
							</div>
						{/each}
					{/if}
				</div>

			{:else if viewTab === 'request'}
				<div class="p-2 space-y-0.5 select-text">
					{#if Object.keys(result.request_headers).length === 0}
						<div class="text-[10px] text-muted-foreground/50 italic">
							Request headers not available (sidecar not running or browser profile headers not captured).
							Start a debug run first to initialise the sidecar.
						</div>
					{:else}
						{#each Object.entries(result.request_headers) as [key, value]}
							<div class="flex items-baseline gap-1 font-mono text-[10px] group">
								<span class="text-orange shrink-0 w-[200px] truncate">{key}:</span>
								<span class="text-foreground break-all flex-1 min-w-0">{value}</span>
								<button class="shrink-0 p-0.5 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-foreground"
									onclick={() => copyText('req_'+key, `${key}: ${value}`)} title="Copy">
									{#if copied === 'req_'+key}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if}
								</button>
							</div>
						{/each}
					{/if}
				</div>

			{:else if viewTab === 'body'}
				<div class="p-2 select-text">
					<pre class="font-mono text-[10px] text-foreground whitespace-pre-wrap break-words">{formatBody(result.body)}</pre>
				</div>

			{:else if viewTab === 'cookies'}
				<div class="p-2 space-y-0.5 select-text">
					{#if Object.keys(result.cookies).length === 0}
						<div class="text-muted-foreground/50">No cookies set</div>
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

	{:else if !loading}
		<!-- ── Empty state ────────────────────────────────────────────────────── -->
		<div class="flex flex-col items-center justify-center flex-1 text-muted-foreground gap-2 panel-inset">
			<Globe size={28} class="text-muted-foreground/30" />
			<div class="text-[11px] text-center">Enter a URL and click <strong>Capture</strong> to inspect<br/>request & response headers from the site.</div>
			<div class="text-[9px] text-muted-foreground/50 text-center max-w-[280px]">
				Uses AzureTLS with your browser profile for real TLS fingerprinting.<br/>
				Click <strong>Apply to Block</strong> to fill headers into your HTTP Request block.
			</div>
		</div>
	{:else}
		<div class="flex items-center justify-center flex-1 gap-2 text-muted-foreground panel-inset">
			<Loader2 size={14} class="animate-spin" />
			<span class="text-[11px]">Sending request...</span>
		</div>
	{/if}
</div>
