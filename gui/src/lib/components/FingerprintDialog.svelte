<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { send, onResponse } from '$lib/ipc';
	import { fingerprint, groupMatches, type ResponseInfo, type FingerprintResult } from '$lib/fingerprint';
	import * as Dialog from '$lib/components/ui/dialog';
	import Search from '@lucide/svelte/icons/search';
	import Shield from '@lucide/svelte/icons/shield';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import ShieldX from '@lucide/svelte/icons/shield-x';
	import Globe from '@lucide/svelte/icons/globe';
	import Server from '@lucide/svelte/icons/server';
	import Code from '@lucide/svelte/icons/code';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import Copy from '@lucide/svelte/icons/copy';

	let open = $derived(app.showFingerprint);

	let url = $state('');
	let scanning = $state(false);
	let result = $state<FingerprintResult | null>(null);
	let errorMsg = $state('');
	let showRawEvidence = $state(false);
	let copied = $state(false);

	function close() {
		app.showFingerprint = false;
	}

	function analyzeDebugResults() {
		errorMsg = '';
		const responses: ResponseInfo[] = app.debugResults
			.filter(r => r.response)
			.map(r => ({
				status_code: r.response!.status_code,
				headers: r.response!.headers,
				cookies: r.response!.cookies,
				body: r.response!.body,
			}));

		if (responses.length === 0) {
			errorMsg = 'No HTTP responses in debug results. Run a debug first.';
			return;
		}

		result = fingerprint(responses);
	}

	function scanUrl() {
		if (!url.trim()) return;
		scanning = true;
		errorMsg = '';
		result = null;

		onResponse('probe_result', (data: unknown) => {
			scanning = false;
			const d = data as any;
			if (!d || d.error) {
				errorMsg = d?.error || 'Probe failed';
				return;
			}
			const resp: ResponseInfo = {
				status_code: d.status_code,
				headers: d.headers || {},
				cookies: d.cookies || {},
				body: d.body_snippet || '',
			};
			result = fingerprint([resp]);
		});

		send('probe_url', { url: url.trim() });
	}

	function onInputKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !scanning) scanUrl();
	}

	async function copyRaw() {
		if (!result) return;
		const text = [
			'=== Headers ===',
			...Object.entries(result.raw.headers).map(([k, v]) => `${k}: ${v}`),
			'',
			'=== Cookies ===',
			...Object.entries(result.raw.cookies).map(([k, v]) => `${k}=${v}`),
		].join('\n');
		try {
			await navigator.clipboard.writeText(text);
			copied = true;
			setTimeout(() => { copied = false; }, 1500);
		} catch {}
	}

	let groups = $derived(result ? groupMatches(result.matches) : []);
	let hasDebugData = $derived(app.debugResults.some(r => r.response));

	function stackCategoryColor(cat: string): string {
		if (cat === 'waf' || cat === 'bot_protection') return 'bg-red/15 text-red border-red/30';
		if (cat === 'cdn') return 'bg-blue/15 text-blue border-blue/30';
		if (cat === 'server') return 'bg-orange/15 text-orange border-orange/30';
		if (cat === 'framework') return 'bg-purple/15 text-purple border-purple/30';
		return 'bg-secondary text-foreground border-border';
	}

	function stackCategoryLabel(cat: string): string {
		if (cat === 'waf' || cat === 'bot_protection') return 'WAF';
		if (cat === 'cdn') return 'CDN';
		if (cat === 'server') return 'Server';
		if (cat === 'framework') return 'Framework';
		return cat;
	}

	function confidenceColor(c: string): string {
		if (c === 'high') return 'text-red';
		if (c === 'medium') return 'text-orange';
		return 'text-muted-foreground';
	}

	function confidenceLabel(c: string): string {
		return c.toUpperCase();
	}

	function securityHeaderLabel(name: string): string {
		const labels: Record<string, string> = {
			'strict-transport-security': 'HSTS',
			'content-security-policy': 'CSP',
			'x-frame-options': 'X-Frame-Options',
			'x-content-type-options': 'X-Content-Type-Options',
			'x-xss-protection': 'X-XSS-Protection',
			'referrer-policy': 'Referrer-Policy',
			'permissions-policy': 'Permissions-Policy',
			'access-control-allow-origin': 'CORS',
		};
		return labels[name] || name;
	}
</script>

<Dialog.Root {open} onOpenChange={(v) => { app.showFingerprint = v; }}>
	<Dialog.Content class="sm:max-w-[560px] p-0 gap-0 overflow-hidden max-h-[80vh] flex flex-col" showCloseButton={false}>
		<!-- Header -->
		<div class="flex items-center gap-2.5 px-4 py-3 border-b border-border-dark panel-raised shrink-0">
			<div class="p-1.5 rounded-md bg-primary/10 text-primary">
				<Search size={15} />
			</div>
			<div>
				<Dialog.Title class="text-sm font-medium text-foreground">Site Fingerprint</Dialog.Title>
				<Dialog.Description class="text-[10px] text-muted-foreground">Detect WAF, CDN, server, and framework technologies</Dialog.Description>
			</div>
		</div>

		<!-- URL input -->
		<div class="flex items-center gap-1.5 px-3 py-2 border-b border-border shrink-0">
			<Globe size={12} class="text-muted-foreground shrink-0" />
			<input
				type="text"
				bind:value={url}
				placeholder="https://example.com"
				class="flex-1 skeu-input text-[11px] font-mono py-0.5"
				disabled={scanning}
				onkeydown={onInputKeydown}
			/>
			<button
				class="skeu-btn text-[10px] {scanning ? 'text-muted-foreground' : 'text-primary'}"
				onclick={scanUrl}
				disabled={scanning || !url.trim()}
			>
				{#if scanning}
					<Loader2 size={10} class="animate-spin inline mr-1" />Scanning...
				{:else}
					Scan
				{/if}
			</button>
		</div>

		<!-- OR separator + Analyze Debug -->
		<div class="flex items-center gap-2 px-3 py-1.5 border-b border-border shrink-0">
			<div class="flex-1 h-px bg-border"></div>
			<span class="text-[9px] text-muted-foreground uppercase tracking-wider">or</span>
			<div class="flex-1 h-px bg-border"></div>
			<button
				class="skeu-btn text-[10px] {hasDebugData ? 'text-foreground' : 'text-muted-foreground'}"
				onclick={analyzeDebugResults}
				disabled={!hasDebugData}
				title={hasDebugData ? 'Analyze HTTP responses from last debug run' : 'Run a debug first to get HTTP responses'}
			>
				Analyze Debug Results
			</button>
		</div>

		<!-- Error message -->
		{#if errorMsg}
			<div class="px-3 py-1.5 text-[11px] text-red bg-red/10 border-b border-border shrink-0">
				{errorMsg}
			</div>
		{/if}

		<!-- Results -->
		<div class="flex-1 overflow-y-auto min-h-0">
			{#if result}
				<!-- Stack summary -->
				{#if result.stack.length > 0}
					<div class="px-3 pt-3 pb-2 border-b border-border">
						<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1.5">Detected Stack</div>
						<div class="flex flex-wrap gap-1.5">
							{#each result.stack as entry}
								<span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-medium border {stackCategoryColor(entry.category)}">
									{entry.name}
									<span class="text-[8px] opacity-60 uppercase">{stackCategoryLabel(entry.category)}</span>
								</span>
							{/each}
						</div>
					</div>
				{:else}
					<div class="px-3 pt-3 pb-2 border-b border-border">
						<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1">Detected Stack</div>
						<div class="text-[11px] text-muted-foreground">No technologies detected</div>
					</div>
				{/if}

				<!-- Grouped matches -->
				{#each groups as group}
					<div class="px-3 pt-2.5 pb-1.5">
						<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1 flex items-center gap-1">
							{#if group.label.startsWith('WAF')}
								<Shield size={10} />
							{:else if group.label.startsWith('Server')}
								<Server size={10} />
							{:else}
								<Code size={10} />
							{/if}
							{group.label}
						</div>
						<div class="space-y-1">
							{#each group.matches as m}
								<div class="border border-border rounded bg-background px-2.5 py-1.5">
									<div class="flex items-center gap-2">
										<span class="text-[11px] font-medium text-foreground flex-1">{m.rule.name}</span>
										<span class="text-[9px] font-mono font-medium {confidenceColor(m.rule.confidence)} uppercase">
											{confidenceLabel(m.rule.confidence)}
										</span>
									</div>
									<div class="mt-0.5 space-y-0.5">
										{#each m.evidence as ev}
											<div class="text-[10px] text-muted-foreground font-mono truncate" title={ev}>{ev}</div>
										{/each}
									</div>
									{#if m.rule.details}
										<div class="mt-1 text-[10px] text-orange">{m.rule.details}</div>
									{/if}
								</div>
							{/each}
						</div>
					</div>
				{/each}

				<!-- Security Headers -->
				{#if result.securityHeaders.length > 0}
					<div class="px-3 pt-2.5 pb-1.5">
						<div class="text-[10px] uppercase tracking-wider text-muted-foreground mb-1 flex items-center gap-1">
							<ShieldCheck size={10} /> Security Headers
						</div>
						<div class="flex flex-wrap gap-1">
							{#each result.securityHeaders as sh}
								<span class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[10px] font-mono {sh.present ? 'bg-green/15 text-green border border-green/30' : 'bg-red/10 text-red/70 border border-red/20'}" title={sh.present ? sh.value : 'Not set'}>
									{#if sh.present}
										<ShieldCheck size={9} />
									{:else}
										<ShieldX size={9} />
									{/if}
									{securityHeaderLabel(sh.name)}
								</span>
							{/each}
						</div>
					</div>
				{/if}

				<!-- No matches -->
				{#if result.matches.length === 0}
					<div class="px-3 pt-4 text-center">
						<div class="text-xs text-muted-foreground">No WAF, CDN, or framework signatures detected.</div>
						<div class="text-[10px] text-muted-foreground mt-1">The target may use custom or unrecognized protection.</div>
					</div>
				{/if}

				<!-- Raw evidence -->
				<div class="px-3 pt-2.5 pb-2">
					<div class="flex items-center gap-1">
						<button
							class="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground transition-colors"
							onclick={() => showRawEvidence = !showRawEvidence}
						>
							{#if showRawEvidence}<ChevronDown size={10} />{:else}<ChevronRight size={10} />{/if}
							Raw Evidence ({Object.keys(result.raw.headers).length} headers, {Object.keys(result.raw.cookies).length} cookies)
						</button>
						{#if showRawEvidence}
							<button
								class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-secondary"
								onclick={copyRaw}
								title="Copy all"
							>
								<Copy size={9} />
							</button>
						{/if}
					</div>
					{#if showRawEvidence}
						<div class="mt-1 border border-border rounded bg-background p-2 max-h-[200px] overflow-auto">
							{#if Object.keys(result.raw.headers).length > 0}
								<div class="text-[9px] uppercase tracking-wider text-muted-foreground mb-0.5">Headers</div>
								{#each Object.entries(result.raw.headers) as [k, v]}
									<div class="text-[10px] font-mono truncate">
										<span class="text-primary">{k}:</span> <span class="text-foreground">{v}</span>
									</div>
								{/each}
							{/if}
							{#if Object.keys(result.raw.cookies).length > 0}
								<div class="text-[9px] uppercase tracking-wider text-muted-foreground mt-2 mb-0.5">Cookies</div>
								{#each Object.entries(result.raw.cookies) as [k, v]}
									<div class="text-[10px] font-mono truncate">
										<span class="text-purple">{k}=</span><span class="text-foreground">{v}</span>
									</div>
								{/each}
							{/if}
						</div>
					{/if}
				</div>
			{:else if !scanning && !errorMsg}
				<div class="flex items-center justify-center py-12 text-xs text-muted-foreground">
					Enter a URL and click Scan, or analyze debug results
				</div>
			{/if}
		</div>

		<!-- Footer -->
		<div class="flex items-center justify-end px-4 py-2.5 bg-background/50 border-t border-border shrink-0">
			<button class="skeu-btn text-[10px] text-foreground" onclick={close}>Close</button>
		</div>
	</Dialog.Content>
</Dialog.Root>
