<script lang="ts">
	import Copy from '@lucide/svelte/icons/copy';
	import Check from '@lucide/svelte/icons/check';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
	import ArrowLeftRight from '@lucide/svelte/icons/arrow-left-right';

	// ── tab state ───────────────────────────────────────────────────────────────
	type Tab = 'encode' | 'diff' | 'cookies' | 'strings' | 'regex';
	let activeTab = $state<Tab>('encode');

	// ── shared copy helper ───────────────────────────────────────────────────────
	let copiedKey = $state<string | null>(null);
	async function copyText(text: string, key: string) {
		try { await navigator.clipboard.writeText(text); } catch {}
		copiedKey = key;
		setTimeout(() => { copiedKey = null; }, 1500);
	}

	// ═══════════════════════════════════════════════════════════════════════════
	// ENCODE / DECODE
	// ═══════════════════════════════════════════════════════════════════════════
	let encInput = $state('');
	let encMode = $state<'base64' | 'url' | 'html' | 'hex' | 'jwt' | 'sha256'>('base64');

	const ENC_MODES = [
		{ value: 'base64', label: 'Base64' },
		{ value: 'url',    label: 'URL' },
		{ value: 'html',   label: 'HTML' },
		{ value: 'hex',    label: 'Hex' },
		{ value: 'jwt',    label: 'JWT' },
		{ value: 'sha256', label: 'Hash' },
	] as const;

	// encode
	let encEncoded = $derived.by(() => {
		const s = encInput;
		try {
			if (encMode === 'base64') return btoa(unescape(encodeURIComponent(s)));
			if (encMode === 'url')    return encodeURIComponent(s);
			if (encMode === 'html')   return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;').replace(/'/g,'&#39;');
			if (encMode === 'hex')    return Array.from(new TextEncoder().encode(s)).map(b => b.toString(16).padStart(2,'0')).join('');
			if (encMode === 'jwt')    return '— encode not supported for JWT; paste below to decode —';
			if (encMode === 'sha256') return '— hashing is decode-only (no reverse); use decoded field —';
		} catch { return '⚠ error'; }
		return '';
	});

	// decode
	let encDecoded = $derived.by(() => {
		const s = encInput;
		try {
			if (encMode === 'base64') return decodeURIComponent(escape(atob(s.trim())));
			if (encMode === 'url')    return decodeURIComponent(s);
			if (encMode === 'html')   return s.replace(/&amp;/g,'&').replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&quot;/g,'"').replace(/&#39;/g,"'");
			if (encMode === 'hex')    return new TextDecoder().decode(Uint8Array.from((s.replace(/\s/g,'')).match(/.{1,2}/g)?.map(b => parseInt(b, 16)) ?? []));
			if (encMode === 'jwt')    return decodeJwt(s.trim());
			if (encMode === 'sha256') return '— SHA-256 is one-way; enter plain text above to see its hash in the "Hash" output —';
		} catch { return '⚠ error (check input)'; }
		return '';
	});

	// sha256 via SubtleCrypto
	let sha256Result = $state('');
	$effect(() => {
		const s = encInput;
		if (encMode !== 'sha256' || !s) { sha256Result = ''; return; }
		crypto.subtle.digest('SHA-256', new TextEncoder().encode(s)).then(buf => {
			sha256Result = Array.from(new Uint8Array(buf)).map(b => b.toString(16).padStart(2,'0')).join('');
		});
	});

	function decodeJwt(token: string): string {
		const parts = token.split('.');
		if (parts.length < 2) return '⚠ not a valid JWT (need header.payload[.sig])';
		try {
			const decode = (p: string) => JSON.stringify(JSON.parse(decodeURIComponent(escape(atob(p.replace(/-/g,'+').replace(/_/g,'/'))))), null, 2);
			return `── Header ──\n${decode(parts[0])}\n\n── Payload ──\n${decode(parts[1])}\n\n── Signature ──\n${parts[2] ?? '(none)'}`;
		} catch { return '⚠ failed to parse JWT'; }
	}

	// ═══════════════════════════════════════════════════════════════════════════
	// TEXT DIFF
	// ═══════════════════════════════════════════════════════════════════════════
	let diffA = $state('');
	let diffB = $state('');

	interface DiffLine { type: 'same' | 'add' | 'del' | 'change'; a: string; b: string }

	let diffLines = $derived.by((): DiffLine[] => {
		const as_ = diffA.split('\n');
		const bs = diffB.split('\n');
		const len = Math.max(as_.length, bs.length);
		const out: DiffLine[] = [];
		for (let i = 0; i < len; i++) {
			const a = as_[i] ?? '';
			const b = bs[i] ?? '';
			if (a === b) out.push({ type: 'same', a, b });
			else if (!as_[i]) out.push({ type: 'add', a: '', b });
			else if (!bs[i]) out.push({ type: 'del', a, b: '' });
			else out.push({ type: 'change', a, b });
		}
		return out;
	});

	let diffStats = $derived.by(() => {
		let adds = 0, dels = 0, changes = 0;
		for (const l of diffLines) {
			if (l.type === 'add') adds++;
			else if (l.type === 'del') dels++;
			else if (l.type === 'change') changes++;
		}
		return { adds, dels, changes, same: diffLines.filter(l => l.type === 'same').length };
	});

	function swapDiff() { const t = diffA; diffA = diffB; diffB = t; }

	// ═══════════════════════════════════════════════════════════════════════════
	// COOKIE INSPECTOR
	// ═══════════════════════════════════════════════════════════════════════════
	let cookieRaw = $state('');
	let cookieRaw2 = $state('');

	interface ParsedCookie { name: string; value: string; decoded: string }

	function parseCookieString(raw: string): ParsedCookie[] {
		return raw.split(';').map(p => p.trim()).filter(Boolean).map(p => {
			const eq = p.indexOf('=');
			const name = eq === -1 ? p : p.slice(0, eq).trim();
			const value = eq === -1 ? '' : p.slice(eq + 1).trim();
			let decoded = value;
			try { decoded = decodeURIComponent(value); } catch {}
			return { name, value, decoded };
		});
	}

	let parsedA = $derived(parseCookieString(cookieRaw));
	let parsedB = $derived(parseCookieString(cookieRaw2));

	// diff: find added/removed/changed names between A and B
	let cookieDiff = $derived.by(() => {
		const mapA = new Map(parsedA.map(c => [c.name, c.value]));
		const mapB = new Map(parsedB.map(c => [c.name, c.value]));
		const allNames = [...new Set([...mapA.keys(), ...mapB.keys()])];
		return allNames.map(name => {
			const va = mapA.get(name);
			const vb = mapB.get(name);
			if (va === undefined) return { name, status: 'added' as const, a: '', b: vb! };
			if (vb === undefined) return { name, status: 'removed' as const, a: va, b: '' };
			if (va !== vb) return { name, status: 'changed' as const, a: va, b: vb };
			return { name, status: 'same' as const, a: va, b: vb };
		});
	});

	// ═══════════════════════════════════════════════════════════════════════════
	// STRING TOOLS
	// ═══════════════════════════════════════════════════════════════════════════
	let strInput = $state('');

	let strStats = $derived.by(() => ({
		chars: strInput.length,
		lines: strInput ? strInput.split('\n').length : 0,
		words: strInput.trim() ? strInput.trim().split(/\s+/).length : 0,
		bytes: new TextEncoder().encode(strInput).length,
	}));

	let strLower = $derived(strInput.toLowerCase());
	let strUpper = $derived(strInput.toUpperCase());
	let strTrimmed = $derived(strInput.trim());
	let strReversed = $derived(strInput.split('').reverse().join(''));
	let strNoWhitespace = $derived(strInput.replace(/\s+/g, ''));
	let strB64 = $derived.by(() => { try { return btoa(unescape(encodeURIComponent(strInput))); } catch { return '⚠ error'; } });
	let strUrlEnc = $derived(encodeURIComponent(strInput));
	let strLines = $derived(strInput ? strInput.split('\n').filter(l => l.trim()).length + ' non-empty lines' : '—');
	let strUniqLines = $derived.by(() => {
		if (!strInput) return '—';
		const lines = strInput.split('\n').filter(l => l.trim());
		return new Set(lines).size + ' unique / ' + lines.length + ' total';
	});

	// ═══════════════════════════════════════════════════════════════════════════
	// REGEX TESTER
	// ═══════════════════════════════════════════════════════════════════════════
	let regexPattern = $state('');
	let regexFlags = $state('g');
	let regexInput = $state('');
	let regexError = $state('');

	interface RegexMatch { index: number; full: string; groups: string[] }
	let regexMatches = $state<RegexMatch[]>([]);

	let regexHighlighted = $state('');

	$effect(() => {
		const pattern = regexPattern;
		const flags = regexFlags;
		const input = regexInput;
		regexError = '';
		regexMatches = [];
		regexHighlighted = '';
		if (!pattern) return;
		try {
			const re = new RegExp(pattern, flags.includes('g') ? flags : flags + 'g');
			const matches: RegexMatch[] = [];
			let m: RegExpExecArray | null;
			while ((m = re.exec(input)) !== null) {
				matches.push({ index: m.index, full: m[0], groups: m.slice(1) });
				if (matches.length >= 200) break;
				if (!flags.includes('g')) break;
				// Guard against infinite loop on zero-length matches (e.g. /a*/)
				if (m[0].length === 0) re.lastIndex++;
			}
			regexMatches = matches;
			// Build highlighted HTML
			const re2 = new RegExp(pattern, flags.includes('g') ? flags : flags + 'g');
			regexHighlighted = input.replace(re2, match =>
				`<mark class="bg-primary/30 text-foreground rounded-sm">${match.replace(/</g,'&lt;').replace(/>/g,'&gt;')}</mark>`
			);
		} catch (e: any) {
			regexError = e.message;
		}
	});

	const TAB_LABELS: Record<Tab, string> = {
		encode:  'Encode/Decode',
		diff:    'Diff',
		cookies: 'Cookies',
		strings: 'Strings',
		regex:   'Regex',
	};
</script>

<div class="flex flex-col h-full bg-surface text-foreground overflow-hidden text-[11px]">
	<!-- Tab bar -->
	<div class="flex items-center gap-0 border-b border-border shrink-0 px-1 pt-0.5">
		{#each Object.entries(TAB_LABELS) as [id, label]}
			<button
				class="px-2.5 py-1 text-[10px] rounded-t transition-colors {activeTab === id ? 'text-foreground font-semibold border-b-2 border-primary -mb-px' : 'text-muted-foreground hover:text-foreground'}"
				onclick={() => activeTab = id as Tab}
			>{label}</button>
		{/each}
	</div>

	<!-- ════════════════════════════════════════════════════════════════════════ -->
	<!-- ENCODE / DECODE -->
	<!-- ════════════════════════════════════════════════════════════════════════ -->
	{#if activeTab === 'encode'}
	<div class="flex flex-col gap-2 p-2 overflow-y-auto flex-1">
		<!-- Mode selector -->
		<div class="flex gap-1 flex-wrap">
			{#each ENC_MODES as mode}
				<button
					class="px-2 py-0.5 text-[10px] rounded border {encMode === mode.value ? 'border-primary bg-primary/10 text-foreground' : 'border-border text-muted-foreground hover:border-muted-foreground'}"
					onclick={() => encMode = mode.value}
				>{mode.label}</button>
			{/each}
		</div>

		<!-- Input -->
		<div>
			<div class="flex justify-between mb-0.5">
				<span class="text-[9px] text-muted-foreground uppercase tracking-wider">Input</span>
				<button class="text-[9px] text-muted-foreground hover:text-foreground" onclick={() => encInput = ''}>Clear</button>
			</div>
			<textarea
				rows={4}
				bind:value={encInput}
				placeholder="Paste text, token, or cookie value here..."
				class="w-full skeu-input text-[10px] font-mono resize-y"
			></textarea>
		</div>

		{#if encMode !== 'jwt' && encMode !== 'sha256'}
		<!-- Encoded -->
		<div>
			<div class="flex justify-between mb-0.5">
				<span class="text-[9px] text-muted-foreground uppercase tracking-wider">Encoded ({encMode})</span>
				<button onclick={() => copyText(encEncoded, 'enc')} class="text-[9px] text-muted-foreground hover:text-foreground flex items-center gap-0.5">
					{#if copiedKey === 'enc'}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if} Copy
				</button>
			</div>
			<div class="skeu-input text-[10px] font-mono select-text whitespace-pre-wrap break-all min-h-[36px] max-h-[100px] overflow-y-auto">{encEncoded || '—'}</div>
		</div>

		<!-- Decoded -->
		<div>
			<div class="flex justify-between mb-0.5">
				<span class="text-[9px] text-muted-foreground uppercase tracking-wider">Decoded ({encMode})</span>
				<button onclick={() => copyText(encDecoded, 'dec')} class="text-[9px] text-muted-foreground hover:text-foreground flex items-center gap-0.5">
					{#if copiedKey === 'dec'}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if} Copy
				</button>
			</div>
			<div class="skeu-input text-[10px] font-mono select-text whitespace-pre-wrap break-all min-h-[36px] max-h-[100px] overflow-y-auto">{encDecoded || '—'}</div>
		</div>
		{/if}

		{#if encMode === 'sha256'}
		<div>
			<div class="flex justify-between mb-0.5">
				<span class="text-[9px] text-muted-foreground uppercase tracking-wider">SHA-256</span>
				<button onclick={() => copyText(sha256Result, 'sha')} class="text-[9px] text-muted-foreground hover:text-foreground flex items-center gap-0.5">
					{#if copiedKey === 'sha'}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if} Copy
				</button>
			</div>
			<div class="skeu-input text-[10px] font-mono select-text break-all min-h-[28px]">{sha256Result || '—'}</div>
		</div>
		<!-- MD5 via pure JS (no subtle crypto) -->
		<div class="text-[9px] text-muted-foreground/60 mt-1">
			SHA-256 via WebCrypto. MD5 not available in browser sandbox — use a regex block with <code class="font-mono">md5({'{'}input{'}'}</code> if needed.
		</div>
		{/if}

		{#if encMode === 'jwt'}
		<div>
			<div class="flex justify-between mb-0.5">
				<span class="text-[9px] text-muted-foreground uppercase tracking-wider">JWT Decoded (header + payload)</span>
				<button onclick={() => copyText(encDecoded, 'jwt')} class="text-[9px] text-muted-foreground hover:text-foreground flex items-center gap-0.5">
					{#if copiedKey === 'jwt'}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if} Copy
				</button>
			</div>
			<div class="skeu-input text-[10px] font-mono select-text whitespace-pre-wrap break-all min-h-[80px] max-h-[300px] overflow-y-auto">{encDecoded || '— paste a JWT token in the input above —'}</div>
		</div>
		{/if}
	</div>
	{/if}

	<!-- ════════════════════════════════════════════════════════════════════════ -->
	<!-- TEXT DIFF -->
	<!-- ════════════════════════════════════════════════════════════════════════ -->
	{#if activeTab === 'diff'}
	<div class="flex flex-col flex-1 overflow-hidden p-2 gap-2">
		<!-- Stats -->
		<div class="flex gap-3 text-[10px] shrink-0">
			<span class="text-green">+{diffStats.adds} added</span>
			<span class="text-red">-{diffStats.dels} removed</span>
			<span class="text-yellow">~{diffStats.changes} changed</span>
			<span class="text-muted-foreground">{diffStats.same} same</span>
			<button class="ml-auto text-muted-foreground hover:text-foreground flex items-center gap-0.5" onclick={swapDiff} title="Swap A and B">
				<ArrowLeftRight size={10} /> Swap
			</button>
		</div>

		<!-- Input row -->
		<div class="grid grid-cols-2 gap-2 shrink-0">
			<div>
				<div class="text-[9px] text-muted-foreground uppercase tracking-wider mb-0.5">A (original)</div>
				<textarea rows={5} bind:value={diffA} placeholder="Paste first value..." class="w-full skeu-input text-[10px] font-mono resize-y"></textarea>
			</div>
			<div>
				<div class="text-[9px] text-muted-foreground uppercase tracking-wider mb-0.5">B (modified)</div>
				<textarea rows={5} bind:value={diffB} placeholder="Paste second value..." class="w-full skeu-input text-[10px] font-mono resize-y"></textarea>
			</div>
		</div>

		<!-- Diff output -->
		<div class="flex-1 overflow-y-auto rounded border border-border bg-background min-h-0">
			{#if diffLines.length === 0}
				<div class="p-2 text-[10px] text-muted-foreground">Enter text in both fields to see diff.</div>
			{:else}
				<table class="w-full text-[10px] font-mono border-collapse">
					<thead>
						<tr class="text-[9px] text-muted-foreground uppercase border-b border-border">
							<th class="text-left px-2 py-0.5 w-6">#</th>
							<th class="text-left px-2 py-0.5">A</th>
							<th class="text-left px-2 py-0.5">B</th>
						</tr>
					</thead>
					<tbody>
						{#each diffLines as line, i}
							<tr class="{line.type === 'add' ? 'bg-green/10' : line.type === 'del' ? 'bg-red/10' : line.type === 'change' ? 'bg-yellow/8' : ''}">
								<td class="px-1 text-muted-foreground/50 select-none text-right border-r border-border/30 w-6">{i+1}</td>
								<td class="px-2 py-px whitespace-pre-wrap break-all select-text {line.type === 'del' ? 'text-red line-through' : line.type === 'change' ? 'text-yellow' : 'text-foreground'}">{line.a}</td>
								<td class="px-2 py-px whitespace-pre-wrap break-all select-text {line.type === 'add' ? 'text-green' : line.type === 'change' ? 'text-yellow' : 'text-foreground'}">{line.b}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</div>
	</div>
	{/if}

	<!-- ════════════════════════════════════════════════════════════════════════ -->
	<!-- COOKIE INSPECTOR -->
	<!-- ════════════════════════════════════════════════════════════════════════ -->
	{#if activeTab === 'cookies'}
	<div class="flex flex-col flex-1 overflow-hidden p-2 gap-2">
		<div class="grid grid-cols-2 gap-2 shrink-0">
			<div>
				<div class="text-[9px] text-muted-foreground uppercase tracking-wider mb-0.5">Cookie String A</div>
				<textarea rows={3} bind:value={cookieRaw} placeholder="name=value; other=val2; ..." class="w-full skeu-input text-[10px] font-mono resize-y"></textarea>
			</div>
			<div>
				<div class="text-[9px] text-muted-foreground uppercase tracking-wider mb-0.5">Cookie String B (optional)</div>
				<textarea rows={3} bind:value={cookieRaw2} placeholder="Compare against a second set..." class="w-full skeu-input text-[10px] font-mono resize-y"></textarea>
			</div>
		</div>

		<div class="flex-1 overflow-y-auto min-h-0 space-y-1.5">
			{#if cookieRaw2.trim()}
				<!-- Diff view -->
				<div class="text-[9px] text-muted-foreground uppercase tracking-wider">Cookie Diff</div>
				<div class="rounded border border-border bg-background overflow-x-auto">
					<table class="w-full text-[10px] font-mono border-collapse min-w-[400px]">
						<thead>
							<tr class="text-[9px] text-muted-foreground border-b border-border">
								<th class="text-left px-2 py-0.5">Name</th>
								<th class="text-left px-2 py-0.5">A</th>
								<th class="text-left px-2 py-0.5">B</th>
								<th class="text-left px-2 py-0.5 w-14">Status</th>
							</tr>
						</thead>
						<tbody>
							{#each cookieDiff as row}
								<tr class="{row.status === 'added' ? 'bg-green/10' : row.status === 'removed' ? 'bg-red/10' : row.status === 'changed' ? 'bg-yellow/8' : ''}">
									<td class="px-2 py-px text-primary font-semibold select-text">{row.name}</td>
									<td class="px-2 py-px text-foreground/80 break-all max-w-[150px] truncate select-text" title={row.a}>{row.a || '—'}</td>
									<td class="px-2 py-px text-foreground/80 break-all max-w-[150px] truncate select-text" title={row.b}>{row.b || '—'}</td>
									<td class="px-2 py-px text-[9px] {row.status === 'added' ? 'text-green' : row.status === 'removed' ? 'text-red' : row.status === 'changed' ? 'text-yellow' : 'text-muted-foreground'}">{row.status}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<!-- Single parse view with decode -->
				<div class="text-[9px] text-muted-foreground uppercase tracking-wider">{parsedA.length} cookie{parsedA.length !== 1 ? 's' : ''} parsed</div>
				<div class="rounded border border-border bg-background overflow-x-auto">
					<table class="w-full text-[10px] font-mono border-collapse">
						<thead>
							<tr class="text-[9px] text-muted-foreground border-b border-border">
								<th class="text-left px-2 py-0.5">Name</th>
								<th class="text-left px-2 py-0.5">Value (raw)</th>
								<th class="text-left px-2 py-0.5">Decoded</th>
								<th class="px-1 py-0.5 w-8"></th>
							</tr>
						</thead>
						<tbody>
							{#each parsedA as c}
								<tr class="hover:bg-secondary/30 group">
									<td class="px-2 py-px text-primary font-semibold select-text">{c.name}</td>
									<td class="px-2 py-px text-foreground/80 break-all max-w-[160px] truncate select-text" title={c.value}>{c.value || '—'}</td>
									<td class="px-2 py-px text-muted-foreground break-all max-w-[160px] truncate select-text" title={c.decoded}>{c.decoded !== c.value ? c.decoded : '—'}</td>
									<td class="px-1 py-px">
										<button
											class="opacity-0 group-hover:opacity-100 p-0.5 text-muted-foreground hover:text-foreground"
											onclick={() => copyText(c.value, 'ck-' + c.name)}
											title="Copy value"
										>{#if copiedKey === 'ck-' + c.name}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if}</button>
									</td>
								</tr>
							{/each}
							{#if parsedA.length === 0}
								<tr><td colspan="4" class="px-2 py-2 text-muted-foreground">No cookies — paste a cookie string above.</td></tr>
							{/if}
						</tbody>
					</table>
				</div>
			{/if}
		</div>
	</div>
	{/if}

	<!-- ════════════════════════════════════════════════════════════════════════ -->
	<!-- STRING TOOLS -->
	<!-- ════════════════════════════════════════════════════════════════════════ -->
	{#if activeTab === 'strings'}
	<div class="flex flex-col gap-2 p-2 overflow-y-auto flex-1">
		<div>
			<div class="flex justify-between mb-0.5">
				<span class="text-[9px] text-muted-foreground uppercase tracking-wider">Input</span>
				<span class="text-[9px] text-muted-foreground">{strStats.chars}ch · {strStats.bytes}B · {strStats.lines} lines · {strStats.words} words</span>
			</div>
			<textarea rows={4} bind:value={strInput} placeholder="Paste any text..." class="w-full skeu-input text-[10px] font-mono resize-y"></textarea>
		</div>
		<div class="grid grid-cols-1 gap-1">
			{#each [
				{ label: 'Lowercase',       key: 'lower',    val: strLower },
				{ label: 'Uppercase',       key: 'upper',    val: strUpper },
				{ label: 'Trimmed',         key: 'trim',     val: strTrimmed },
				{ label: 'Reversed',        key: 'rev',      val: strReversed },
				{ label: 'No whitespace',   key: 'nows',     val: strNoWhitespace },
				{ label: 'Base64',          key: 'sb64',     val: strB64 },
				{ label: 'URL Encoded',     key: 'surl',     val: strUrlEnc },
				{ label: 'Non-empty lines', key: 'sl',       val: strLines },
				{ label: 'Unique lines',    key: 'su',       val: strUniqLines },
			] as row}
				<div class="flex items-center gap-1 group">
					<span class="text-[9px] text-muted-foreground w-[90px] shrink-0">{row.label}</span>
					<span class="flex-1 text-[10px] font-mono text-foreground truncate select-text" title={row.val}>{row.val || '—'}</span>
					{#if row.val && row.val !== '—'}
					<button
						class="opacity-0 group-hover:opacity-100 p-0.5 text-muted-foreground hover:text-foreground shrink-0"
						onclick={() => copyText(row.val, row.key)}
						title="Copy"
					>{#if copiedKey === row.key}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if}</button>
					{/if}
				</div>
			{/each}
		</div>
	</div>
	{/if}

	<!-- ════════════════════════════════════════════════════════════════════════ -->
	<!-- REGEX TESTER -->
	<!-- ════════════════════════════════════════════════════════════════════════ -->
	{#if activeTab === 'regex'}
	<div class="flex flex-col gap-2 p-2 overflow-y-auto flex-1">
		<div class="flex gap-1">
			<div class="flex-1">
				<div class="text-[9px] text-muted-foreground uppercase tracking-wider mb-0.5">Pattern</div>
				<input type="text" bind:value={regexPattern} placeholder="(https?://[^\s]+)" class="w-full skeu-input text-[10px] font-mono" />
			</div>
			<div class="w-[60px]">
				<div class="text-[9px] text-muted-foreground uppercase tracking-wider mb-0.5">Flags</div>
				<input type="text" bind:value={regexFlags} placeholder="gim" class="w-full skeu-input text-[10px] font-mono" />
			</div>
		</div>
		{#if regexError}
			<div class="text-[10px] text-red font-mono">⚠ {regexError}</div>
		{/if}

		<div>
			<div class="flex justify-between mb-0.5">
				<span class="text-[9px] text-muted-foreground uppercase tracking-wider">Test String</span>
				<span class="text-[9px] {regexMatches.length > 0 ? 'text-green' : 'text-muted-foreground'}">{regexMatches.length} match{regexMatches.length !== 1 ? 'es' : ''}</span>
			</div>
			<textarea rows={4} bind:value={regexInput} placeholder="Paste text to test against..." class="w-full skeu-input text-[10px] font-mono resize-y"></textarea>
		</div>

		{#if regexHighlighted && regexMatches.length > 0}
		<div>
			<div class="text-[9px] text-muted-foreground uppercase tracking-wider mb-0.5">Highlighted</div>
			<div class="skeu-input text-[10px] font-mono whitespace-pre-wrap break-all select-text max-h-[100px] overflow-y-auto">{@html regexHighlighted}</div>
		</div>

		<div>
			<div class="text-[9px] text-muted-foreground uppercase tracking-wider mb-0.5">Matches</div>
			<div class="space-y-0.5 max-h-[120px] overflow-y-auto">
				{#each regexMatches as m, mi}
					<div class="flex items-start gap-1 group">
						<span class="text-[9px] text-muted-foreground w-5 text-right shrink-0 mt-px">{mi+1}</span>
						<div class="flex-1 min-w-0">
							<code class="text-[10px] text-green font-mono select-text break-all">{m.full}</code>
							{#if m.groups.length > 0}
								<div class="flex gap-1 flex-wrap mt-0.5">
									{#each m.groups as g, gi}
										<span class="text-[9px] bg-secondary rounded px-1 text-muted-foreground">group {gi+1}: <code class="text-foreground select-text">{g ?? '—'}</code></span>
									{/each}
								</div>
							{/if}
						</div>
						<button
							class="opacity-0 group-hover:opacity-100 p-0.5 text-muted-foreground hover:text-foreground shrink-0"
							onclick={() => copyText(m.full, 'rx-' + mi)}
							title="Copy match"
						>{#if copiedKey === 'rx-' + mi}<Check size={9} class="text-green" />{:else}<Copy size={9} />{/if}</button>
					</div>
				{/each}
			</div>
		</div>
		{/if}
	</div>
	{/if}
</div>
