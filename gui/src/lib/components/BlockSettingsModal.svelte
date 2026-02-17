<script lang="ts">
	import { app, getEditingBlock, pushUndo } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import type { Block, KeyCondition, Keychain } from '$lib/types';
	import * as Dialog from '$lib/components/ui/dialog';
	import SkeuSelect from './SkeuSelect.svelte';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';

	let block = $derived(getEditingBlock());
	let open = $derived(app.editingBlockId !== null);

	function onOpenChange(v: boolean) {
		if (!v) app.editingBlockId = null;
	}

	function updateBlock(updates: Partial<Block>) {
		if (!block) return;
		pushUndo();
		const updated = { ...block, ...updates };
		send('update_block', updated);
	}

	function updateSettings(key: string, value: unknown) {
		if (!block) return;
		pushUndo();
		const updated = { ...block, settings: { ...block.settings, [key]: value } };
		send('update_block', updated);
	}

	// --- Shared input class strings ---
	const inputCls = "w-full skeu-input font-mono mt-0.5";
	const labelCls = "text-[10px] uppercase tracking-wider text-muted-foreground";
	const smallInputCls = "skeu-input text-[10px] font-mono";

	// --- Shared option arrays ---
	const COMPARISON_OPTIONS = [
		{value:'Contains',label:'Contains'},{value:'NotContains',label:'Not Contains'},
		{value:'EqualTo',label:'Equals'},{value:'NotEqualTo',label:'Not Equals'},
		{value:'MatchesRegex',label:'Regex'},{value:'GreaterThan',label:'Greater'},
		{value:'LessThan',label:'Less'},{value:'Exists',label:'Exists'},{value:'NotExists',label:'Not Exists'},
	];

	const CONVERSION_TYPE_OPTIONS = [
		{value:'String',label:'String'},{value:'Int',label:'Integer'},{value:'Float',label:'Float'},
		{value:'Bool',label:'Boolean'},{value:'Hex',label:'Hex'},{value:'Base64',label:'Base64'},
	];

	// --- HTTP Request: raw header helpers ---
	function headersToRaw(headers: [string, string][]): string {
		return headers.map(([k, v]) => `${k}: ${v}`).join('\n');
	}

	function rawToHeaders(raw: string): [string, string][] {
		return raw.split('\n')
			.map(line => line.trim())
			.filter(line => line.length > 0)
			.map(line => {
				const idx = line.indexOf(':');
				if (idx === -1) return [line, ''] as [string, string];
				return [line.slice(0, idx).trim(), line.slice(idx + 1).trim()] as [string, string];
			});
	}

	let httpTab = $state<'headers' | 'body' | 'options'>('headers');
	let rawHeaders = $state('');

	// Sync raw headers from block when block changes
	$effect(() => {
		if (block && block.settings.type === 'HttpRequest') {
			rawHeaders = headersToRaw(block.settings.headers);
		}
	});

	function commitRawHeaders() {
		if (!block || block.settings.type !== 'HttpRequest') return;
		updateSettings('headers', rawToHeaders(rawHeaders));
	}

	// --- KeyCheck helpers ---
	function addKeychain() {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains, {
			result: 'Success',
			conditions: [{ source: 'data.RESPONSECODE', comparison: 'EqualTo', value: '200' }]
		}];
		updateSettings('keychains', keychains);
	}

	function removeKeychain(ki: number) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains = block.settings.keychains.filter((_: Keychain, i: number) => i !== ki);
		updateSettings('keychains', keychains);
	}

	function addCondition(ki: number) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains];
		const conditions: KeyCondition[] = [...keychains[ki].conditions, { source: '', comparison: 'Contains', value: '' }];
		keychains[ki] = { ...keychains[ki], conditions };
		updateSettings('keychains', keychains);
	}

	function removeCondition(ki: number, ci: number) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains];
		const conditions = keychains[ki].conditions.filter((_: KeyCondition, i: number) => i !== ci);
		keychains[ki] = { ...keychains[ki], conditions };
		updateSettings('keychains', keychains);
	}

	function updateKeychainResult(ki: number, value: string) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains];
		keychains[ki] = { ...keychains[ki], result: value as any };
		updateSettings('keychains', keychains);
	}

	function updateConditionField(ki: number, ci: number, field: keyof KeyCondition, value: string) {
		if (!block || block.settings.type !== 'KeyCheck') return;
		const keychains: Keychain[] = [...block.settings.keychains];
		const conditions: KeyCondition[] = [...keychains[ki].conditions];
		conditions[ci] = { ...conditions[ci], [field]: value };
		keychains[ki] = { ...keychains[ki], conditions };
		updateSettings('keychains', keychains);
	}

	// --- String Function param labels ---
	const STRING_FUNCTIONS = [
		{ value: 'Replace', label: 'Replace', p1: 'Find', p2: 'Replace with' },
		{ value: 'Substring', label: 'Substring', p1: 'Start index', p2: 'Length' },
		{ value: 'Trim', label: 'Trim', p1: '', p2: '' },
		{ value: 'ToUpper', label: 'To Upper', p1: '', p2: '' },
		{ value: 'ToLower', label: 'To Lower', p1: '', p2: '' },
		{ value: 'URLEncode', label: 'URL Encode', p1: '', p2: '' },
		{ value: 'URLDecode', label: 'URL Decode', p1: '', p2: '' },
		{ value: 'Base64Encode', label: 'Base64 Encode', p1: '', p2: '' },
		{ value: 'Base64Decode', label: 'Base64 Decode', p1: '', p2: '' },
		{ value: 'HTMLEncode', label: 'HTML Encode', p1: '', p2: '' },
		{ value: 'HTMLDecode', label: 'HTML Decode', p1: '', p2: '' },
		{ value: 'Split', label: 'Split', p1: 'Separator', p2: '' },
		{ value: 'RandomString', label: 'Random String', p1: 'Length', p2: 'Charset (abc123...)' },
	];

	function getStringFuncMeta(ft: string) {
		return STRING_FUNCTIONS.find(f => f.value === ft) || STRING_FUNCTIONS[0];
	}

	// --- List Function options ---
	const LIST_FUNCTIONS = [
		{ value: 'Join', label: 'Join', param: 'Separator' },
		{ value: 'Sort', label: 'Sort', param: '' },
		{ value: 'Shuffle', label: 'Shuffle', param: '' },
		{ value: 'Add', label: 'Add Item', param: 'Item' },
		{ value: 'Remove', label: 'Remove Item', param: 'Item' },
		{ value: 'Deduplicate', label: 'Deduplicate', param: '' },
		{ value: 'RandomItem', label: 'Random Item', param: '' },
		{ value: 'Length', label: 'Length', param: '' },
	];

	function getListFuncMeta(ft: string) {
		return LIST_FUNCTIONS.find(f => f.value === ft) || LIST_FUNCTIONS[0];
	}

	// --- Crypto Function options ---
	const CRYPTO_FUNCTIONS = [
		{ value: 'MD5', label: 'MD5', needsKey: false },
		{ value: 'SHA1', label: 'SHA-1', needsKey: false },
		{ value: 'SHA256', label: 'SHA-256', needsKey: false },
		{ value: 'SHA512', label: 'SHA-512', needsKey: false },
		{ value: 'HMAC', label: 'HMAC', needsKey: true },
		{ value: 'Base64', label: 'Base64', needsKey: false },
		{ value: 'AES', label: 'AES Encrypt', needsKey: true },
	];

	function getCryptoFuncMeta(ft: string) {
		return CRYPTO_FUNCTIONS.find(f => f.value === ft) || CRYPTO_FUNCTIONS[0];
	}
</script>

<Dialog.Root {open} onOpenChange={onOpenChange}>
	<Dialog.Content class="bg-surface border-border max-w-[520px] max-h-[80vh] p-0 gap-0 overflow-hidden flex flex-col">
		{#if block}
			<!-- Header -->
			<div class="px-3 py-2 border-b border-border shrink-0 panel-raised">
				<div class="flex items-center gap-2">
					<span class="text-sm font-medium text-foreground">{block.label}</span>
					<span class="text-[9px] uppercase tracking-wider text-muted-foreground bg-background px-1.5 py-px rounded border border-border">{block.block_type}</span>
				</div>
				<div class="flex gap-3 mt-1">
					<label class="flex items-center gap-1 text-[10px] text-muted-foreground">
						<input type="checkbox" checked={block.disabled} onchange={() => updateBlock({ disabled: !block!.disabled })} class="skeu-checkbox" />
						Disabled
					</label>
					<label class="flex items-center gap-1 text-[10px] text-muted-foreground">
						<input type="checkbox" checked={block.safe_mode} onchange={() => updateBlock({ safe_mode: !block!.safe_mode })} class="skeu-checkbox" />
						Safe Mode
					</label>
				</div>
			</div>

			<!-- Settings based on block type -->
			<div class="flex-1 overflow-y-auto p-2 space-y-1.5 panel-inset">

				<!-- ===================== HTTP REQUEST ===================== -->
				{#if block.settings.type === 'HttpRequest'}
					<!-- URL bar -->
					<div class="flex gap-1.5">
						<SkeuSelect
							value={block.settings.method}
							onValueChange={(v) => updateSettings('method', v)}
							options={[{value:'GET',label:'GET'},{value:'POST',label:'POST'},{value:'PUT',label:'PUT'},{value:'DELETE',label:'DELETE'},{value:'PATCH',label:'PATCH'},{value:'HEAD',label:'HEAD'},{value:'OPTIONS',label:'OPTIONS'}]}
							class="text-[11px] w-20"
						/>
						<input type="text" value={block.settings.url} placeholder="https://example.com/api/endpoint"
							class="flex-1 skeu-input text-[11px] font-mono"
							oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
					</div>

					<!-- Tabbed sub-panel -->
					<div class="flex border-b border-border">
						{#each ['headers', 'body', 'options'] as tab}
							<button
								class="px-2 py-0.5 text-[11px] capitalize {httpTab === tab ? 'text-foreground font-medium' : 'text-muted-foreground hover:text-foreground'} transition-colors"
								onclick={() => { httpTab = tab as typeof httpTab; }}
							>{tab}</button>
						{/each}
					</div>

					{#if httpTab === 'headers'}
						<!-- Raw headers textarea -->
						<textarea
							class="w-full skeu-input text-[11px] font-mono min-h-[120px] resize-y"
							placeholder="Content-Type: application/json&#10;Accept: */*&#10;Authorization: Bearer <token>"
							bind:value={rawHeaders}
							onblur={commitRawHeaders}
						></textarea>
						<p class="text-[10px] text-muted-foreground">One header per line: <code class="text-foreground/70">Name: Value</code></p>
					{:else if httpTab === 'body'}
						<div class="flex items-center gap-2 mb-1">
							<span class={labelCls}>Mode</span>
							<SkeuSelect value={block.settings.body_type}
								onValueChange={(v) => updateSettings('body_type', v)}
								options={[{value:'None',label:'None'},{value:'Standard',label:'Form'},{value:'Raw',label:'Raw'},{value:'Multipart',label:'Multipart'}]}
								class="text-[10px]"
							/>
						</div>
						{#if block.settings.body_type !== 'None'}
							<textarea value={block.settings.body} placeholder="Request body..."
								class="w-full skeu-input text-[11px] font-mono min-h-[100px] resize-y"
								oninput={(e) => updateSettings('body', (e.target as HTMLTextAreaElement).value)}></textarea>
						{/if}
					{:else if httpTab === 'options'}
						<div class="space-y-1.5">
							<label class="flex items-center gap-2 text-[11px] text-foreground">
								<input type="checkbox" checked={block.settings.follow_redirects} onchange={() => updateSettings('follow_redirects', !block!.settings.follow_redirects)} class="skeu-checkbox" />
								Follow redirects
							</label>
							<div class="flex items-center gap-2">
								<span class="text-[11px] text-muted-foreground">Timeout:</span>
								<input type="number" value={block.settings.timeout_ms}
									class="w-20 skeu-input text-[11px]"
									oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} />
								<span class="text-[10px] text-muted-foreground">ms</span>
							</div>
						</div>
					{/if}

				<!-- ===================== PARSE LR ===================== -->
				{:else if block.settings.type === 'ParseLR'}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>Left delimiter</label>
							<input type="text" value={block.settings.left} class={inputCls}
								oninput={(e) => updateSettings('left', (e.target as HTMLInputElement).value)} />
						</div>
						<div>
							<label class={labelCls}>Right delimiter</label>
							<input type="text" value={block.settings.right} class={inputCls}
								oninput={(e) => updateSettings('right', (e.target as HTMLInputElement).value)} />
						</div>
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
						<label class="flex items-center gap-2 text-[11px] text-foreground">
							<input type="checkbox" checked={block.settings.recursive} onchange={() => updateSettings('recursive', !block!.settings.recursive)} class="skeu-checkbox" />
							Recursive
						</label>
						<label class="flex items-center gap-2 text-[11px] text-foreground">
							<input type="checkbox" checked={block.settings.case_insensitive} onchange={() => updateSettings('case_insensitive', !block!.settings.case_insensitive)} class="skeu-checkbox" />
							Case insensitive
						</label>
					</div>

				<!-- ===================== PARSE JSON ===================== -->
				{:else if block.settings.type === 'ParseJSON'}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>JSON Path</label>
							<input type="text" value={block.settings.json_path} placeholder="e.g. user.token" class={inputCls}
								oninput={(e) => updateSettings('json_path', (e.target as HTMLInputElement).value)} />
						</div>
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
					</div>

				<!-- ===================== PARSE REGEX ===================== -->
				{:else if block.settings.type === 'ParseRegex'}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>Pattern</label>
							<input type="text" value={block.settings.pattern} class={inputCls}
								oninput={(e) => updateSettings('pattern', (e.target as HTMLInputElement).value)} />
						</div>
						<div>
							<label class={labelCls}>Output format</label>
							<input type="text" value={block.settings.output_format} placeholder="$1" class={inputCls}
								oninput={(e) => updateSettings('output_format', (e.target as HTMLInputElement).value)} />
						</div>
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
						<label class="flex items-center gap-2 text-[11px] text-foreground">
							<input type="checkbox" checked={block.settings.multi_line} onchange={() => updateSettings('multi_line', !block!.settings.multi_line)} class="skeu-checkbox" />
							Multi-line mode
						</label>
					</div>

				<!-- ===================== PARSE CSS ===================== -->
				{:else if block.settings.type === 'ParseCSS'}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>CSS Selector</label>
							<input type="text" value={block.settings.selector} placeholder="div.content > a" class={inputCls}
								oninput={(e) => updateSettings('selector', (e.target as HTMLInputElement).value)} />
						</div>
						<div>
							<label class={labelCls}>Attribute (empty = text content)</label>
							<input type="text" value={block.settings.attribute} placeholder="href" class={inputCls}
								oninput={(e) => updateSettings('attribute', (e.target as HTMLInputElement).value)} />
						</div>
						<div>
							<label class={labelCls}>Element index (0-based, -1 = all)</label>
							<input type="number" value={block.settings.index}
								class="w-20 skeu-input text-[11px] mt-0.5"
								oninput={(e) => updateSettings('index', parseInt((e.target as HTMLInputElement).value))} />
						</div>
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
					</div>

				<!-- ===================== PARSE XPATH ===================== -->
				{:else if block.settings.type === 'ParseXPath'}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>XPath expression</label>
							<input type="text" value={block.settings.xpath} placeholder="//div[@class='result']/text()" class={inputCls}
								oninput={(e) => updateSettings('xpath', (e.target as HTMLInputElement).value)} />
						</div>
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
					</div>

				<!-- ===================== KEYCHECK ===================== -->
				{:else if block.settings.type === 'KeyCheck'}
					<div class="space-y-1.5">
						<div class="flex items-center justify-between">
							<span class={labelCls}>Keychains</span>
							<button class="text-[10px] text-primary hover:underline" onclick={addKeychain}>+ Add Keychain</button>
						</div>
						{#each block.settings.keychains as keychain, ki}
							<div class="bg-background rounded p-2 border border-border">
								<div class="flex items-center gap-2 mb-1.5">
									<span class="text-[10px] text-muted-foreground">Result:</span>
									<SkeuSelect value={keychain.result}
										onValueChange={(v) => updateKeychainResult(ki, v)}
										options={[{value:'Success',label:'SUCCESS'},{value:'Fail',label:'FAIL'},{value:'Ban',label:'BAN'},{value:'Retry',label:'RETRY'},{value:'Custom',label:'CUSTOM'}]}
										class="text-[10px]"
									/>
									<div class="flex-1"></div>
									<button class="text-[10px] text-red hover:underline" onclick={() => removeKeychain(ki)}>Remove</button>
								</div>
								{#each keychain.conditions as cond, ci}
									<div class="flex gap-1 mb-1 items-center">
										<input type="text" value={cond.source} placeholder="data.SOURCE" class="flex-1 {smallInputCls}"
											oninput={(e) => updateConditionField(ki, ci, 'source', (e.target as HTMLInputElement).value)} />
										<SkeuSelect value={cond.comparison}
											onValueChange={(v) => updateConditionField(ki, ci, 'comparison', v)}
											options={COMPARISON_OPTIONS}
											class="text-[10px]"
										/>
										<input type="text" value={cond.value} placeholder="value" class="flex-1 {smallInputCls}"
											oninput={(e) => updateConditionField(ki, ci, 'value', (e.target as HTMLInputElement).value)} />
										<button class="text-muted-foreground hover:text-red px-0.5 shrink-0" onclick={() => removeCondition(ki, ci)}>x</button>
									</div>
								{/each}
								<button class="text-[10px] text-primary hover:underline mt-1" onclick={() => addCondition(ki)}>+ Add Condition</button>
							</div>
						{/each}
					</div>

				<!-- ===================== STRING FUNCTION ===================== -->
				{:else if block.settings.type === 'StringFunction'}
					{@const meta = getStringFuncMeta(block.settings.function_type)}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>Function</label>
							<SkeuSelect value={block.settings.function_type}
								onValueChange={(v) => updateSettings('function_type', v)}
								options={STRING_FUNCTIONS.map(f => ({value: f.value, label: f.label}))}
								class="w-full mt-0.5"
							/>
						</div>
						<div>
							<label class={labelCls}>Input variable</label>
							<input type="text" value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
								oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						</div>
						{#if meta.p1}
							<div>
								<label class={labelCls}>{meta.p1}</label>
								<input type="text" value={block.settings.param1} class={inputCls}
									oninput={(e) => updateSettings('param1', (e.target as HTMLInputElement).value)} />
							</div>
						{/if}
						{#if meta.p2}
							<div>
								<label class={labelCls}>{meta.p2}</label>
								<input type="text" value={block.settings.param2} class={inputCls}
									oninput={(e) => updateSettings('param2', (e.target as HTMLInputElement).value)} />
							</div>
						{/if}
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
					</div>

				<!-- ===================== LIST FUNCTION ===================== -->
				{:else if block.settings.type === 'ListFunction'}
					{@const meta = getListFuncMeta(block.settings.function_type)}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>Function</label>
							<SkeuSelect value={block.settings.function_type}
								onValueChange={(v) => updateSettings('function_type', v)}
								options={LIST_FUNCTIONS.map(f => ({value: f.value, label: f.label}))}
								class="w-full mt-0.5"
							/>
						</div>
						<div>
							<label class={labelCls}>Input variable (list)</label>
							<input type="text" value={block.settings.input_var} placeholder="@myList" class={inputCls}
								oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						</div>
						{#if meta.param}
							<div>
								<label class={labelCls}>{meta.param}</label>
								<input type="text" value={block.settings.param1} class={inputCls}
									oninput={(e) => updateSettings('param1', (e.target as HTMLInputElement).value)} />
							</div>
						{/if}
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
					</div>

				<!-- ===================== CRYPTO FUNCTION ===================== -->
				{:else if block.settings.type === 'CryptoFunction'}
					{@const meta = getCryptoFuncMeta(block.settings.function_type)}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>Algorithm</label>
							<SkeuSelect value={block.settings.function_type}
								onValueChange={(v) => updateSettings('function_type', v)}
								options={CRYPTO_FUNCTIONS.map(f => ({value: f.value, label: f.label}))}
								class="w-full mt-0.5"
							/>
						</div>
						<div>
							<label class={labelCls}>Input variable</label>
							<input type="text" value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
								oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						</div>
						{#if meta.needsKey}
							<div>
								<label class={labelCls}>Key / Secret</label>
								<input type="text" value={block.settings.key} class={inputCls}
									oninput={(e) => updateSettings('key', (e.target as HTMLInputElement).value)} />
							</div>
						{/if}
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
					</div>

				<!-- ===================== CONVERSION FUNCTION ===================== -->
				{:else if block.settings.type === 'ConversionFunction'}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>Input variable</label>
							<input type="text" value={block.settings.input_var} class={inputCls}
								oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
						</div>
						<div class="flex gap-2 items-end">
							<div class="flex-1">
								<label class={labelCls}>From type</label>
								<SkeuSelect value={block.settings.from_type}
									onValueChange={(v) => updateSettings('from_type', v)}
									options={CONVERSION_TYPE_OPTIONS}
									class="w-full mt-0.5"
								/>
							</div>
							<div class="flex items-center pb-1 text-muted-foreground">
								<ArrowRight size={14} />
							</div>
							<div class="flex-1">
								<label class={labelCls}>To type</label>
								<SkeuSelect value={block.settings.to_type}
									onValueChange={(v) => updateSettings('to_type', v)}
									options={CONVERSION_TYPE_OPTIONS}
									class="w-full mt-0.5"
								/>
							</div>
						</div>
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
					</div>

				<!-- ===================== IF/ELSE ===================== -->
				{:else if block.settings.type === 'IfElse'}
					<div class="space-y-1.5">
						<span class={labelCls}>Condition</span>
						<div class="flex gap-1">
							<input type="text" value={block.settings.condition.source} placeholder="data.SOURCE" class="flex-1 {smallInputCls}"
								oninput={(e) => updateSettings('condition', { ...block!.settings.condition, source: (e.target as HTMLInputElement).value })} />
							<SkeuSelect value={block.settings.condition.comparison}
								onValueChange={(v) => updateSettings('condition', { ...block!.settings.condition, comparison: v })}
								options={COMPARISON_OPTIONS}
								class="text-[10px]"
							/>
							<input type="text" value={block.settings.condition.value} placeholder="value" class="flex-1 {smallInputCls}"
								oninput={(e) => updateSettings('condition', { ...block!.settings.condition, value: (e.target as HTMLInputElement).value })} />
						</div>
						<div class="border border-border rounded p-2 bg-background">
							<span class="text-[10px] text-green-400 uppercase">True branch</span>
							<p class="text-[10px] text-muted-foreground mt-1">
								{block.settings.true_blocks.length} block{block.settings.true_blocks.length !== 1 ? 's' : ''}
							</p>
						</div>
						<div class="border border-border rounded p-2 bg-background">
							<span class="text-[10px] text-red uppercase">False branch</span>
							<p class="text-[10px] text-muted-foreground mt-1">
								{block.settings.false_blocks.length} block{block.settings.false_blocks.length !== 1 ? 's' : ''}
							</p>
						</div>
					</div>

				<!-- ===================== LOOP ===================== -->
				{:else if block.settings.type === 'Loop'}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>Loop type</label>
							<SkeuSelect value={block.settings.loop_type}
								onValueChange={(v) => updateSettings('loop_type', v)}
								options={[{value:'ForEach',label:'For Each'},{value:'Repeat',label:'Repeat N times'}]}
								class="w-full mt-0.5"
							/>
						</div>
						{#if block.settings.loop_type === 'ForEach'}
							<div>
								<label class={labelCls}>List variable</label>
								<input type="text" value={block.settings.list_var} placeholder="@myList" class={inputCls}
									oninput={(e) => updateSettings('list_var', (e.target as HTMLInputElement).value)} />
							</div>
							<div>
								<label class={labelCls}>Item variable name</label>
								<input type="text" value={block.settings.item_var} placeholder="@item" class={inputCls}
									oninput={(e) => updateSettings('item_var', (e.target as HTMLInputElement).value)} />
							</div>
						{:else}
							<div>
								<label class={labelCls}>Repeat count</label>
								<input type="number" value={block.settings.count} min="1"
									class="w-24 skeu-input text-[11px] mt-0.5"
									oninput={(e) => updateSettings('count', parseInt((e.target as HTMLInputElement).value))} />
							</div>
						{/if}
						<div class="border border-border rounded p-2 bg-background">
							<span class="text-[10px] text-yellow-400 uppercase">Loop body</span>
							<p class="text-[10px] text-muted-foreground mt-1">
								{block.settings.blocks.length} block{block.settings.blocks.length !== 1 ? 's' : ''}
							</p>
						</div>
					</div>

				<!-- ===================== DELAY ===================== -->
				{:else if block.settings.type === 'Delay'}
					<div class="flex gap-2">
						<div>
							<label class={labelCls}>Min (ms)</label>
							<input type="number" value={block.settings.min_ms} class="w-24 skeu-input text-[11px] mt-0.5"
								oninput={(e) => updateSettings('min_ms', parseInt((e.target as HTMLInputElement).value))} />
						</div>
						<div>
							<label class={labelCls}>Max (ms)</label>
							<input type="number" value={block.settings.max_ms} class="w-24 skeu-input text-[11px] mt-0.5"
								oninput={(e) => updateSettings('max_ms', parseInt((e.target as HTMLInputElement).value))} />
						</div>
					</div>

				<!-- ===================== SCRIPT ===================== -->
				{:else if block.settings.type === 'Script'}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>Script code</label>
							<textarea value={block.settings.code} placeholder="// JavaScript code here..."
								class="w-full skeu-input text-[11px] font-mono min-h-[120px] resize-y mt-0.5"
								oninput={(e) => updateSettings('code', (e.target as HTMLTextAreaElement).value)}></textarea>
						</div>
						<div class="flex gap-2">
							<div class="flex-1">
								<label class={labelCls}>Output var</label>
								<input type="text" value={block.settings.output_var} class={inputCls}
									oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
							</div>
							<label class="flex items-center gap-1 text-xs text-foreground pt-4">
								<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
								CAP
							</label>
						</div>
					</div>

				<!-- ===================== LOG ===================== -->
				{:else if block.settings.type === 'Log'}
					<div>
						<label class={labelCls}>Message</label>
						<input type="text" value={block.settings.message} placeholder="Log message with <variables>" class={inputCls}
							oninput={(e) => updateSettings('message', (e.target as HTMLInputElement).value)} />
					</div>

				<!-- ===================== SET VARIABLE ===================== -->
				{:else if block.settings.type === 'SetVariable'}
					<div class="space-y-1.5">
						<div>
							<label class={labelCls}>Variable name</label>
							<input type="text" value={block.settings.name} class={inputCls}
								oninput={(e) => updateSettings('name', (e.target as HTMLInputElement).value)} />
						</div>
						<div>
							<label class={labelCls}>Value</label>
							<input type="text" value={block.settings.value} class={inputCls}
								oninput={(e) => updateSettings('value', (e.target as HTMLInputElement).value)} />
						</div>
						<label class="flex items-center gap-2 text-[11px] text-foreground">
							<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
							Capture (save to results)
						</label>
					</div>

				<!-- ===================== CLEAR COOKIES ===================== -->
				{:else if block.settings.type === 'ClearCookies'}
					<div class="text-[11px] text-muted-foreground">
						Clears all cookies in the current session. No additional settings required.
					</div>

				{:else}
					<div class="text-[11px] text-muted-foreground">
						Settings editor for {block.block_type} blocks
					</div>
				{/if}
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
