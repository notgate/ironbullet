<script lang="ts">
	import type { Block } from '$lib/types';
	import VariableInput from '../VariableInput.svelte';
	import SkeuSelect from '../SkeuSelect.svelte';
	import { inputCls, labelCls, hasVars } from './shared';

	const PARSE_MODES = [
		{ value: 'LR', label: 'LR — Left/Right delimiter' },
		{ value: 'Regex', label: 'Regex — Regular expression' },
		{ value: 'Json', label: 'JSON — JSON path' },
		{ value: 'Css', label: 'CSS — CSS selector' },
		{ value: 'XPath', label: 'XPath — XML/HTML path' },
		{ value: 'Cookie', label: 'Cookie — Extract cookie value' },
		{ value: 'Lambda', label: 'Lambda — Transform expression' },
	];

	let { block, updateSettings, embedBadge }: {
		block: Block;
		updateSettings: (key: string, value: unknown) => void;
		embedBadge: import('svelte').Snippet<[string | undefined]>;
	} = $props();
</script>

<!-- ===================== PARSE LR ===================== -->
{#if block.settings.type === 'ParseLR'}
	<div class="space-y-1.5">
		<div class="relative">
			<label class={labelCls}>Input variable</label>
			<VariableInput value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
				oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.input_var)}
		</div>
		<div class="relative">
			<label class={labelCls}>Left delimiter</label>
			<VariableInput value={block.settings.left} class={inputCls}
				oninput={(e) => updateSettings('left', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.left)}
		</div>
		<div class="relative">
			<label class={labelCls}>Right delimiter</label>
			<VariableInput value={block.settings.right} class={inputCls}
				oninput={(e) => updateSettings('right', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.right)}
		</div>
		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Output var</label>
				<VariableInput value={block.settings.output_var} class={inputCls}
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
		<div class="relative">
			<label class={labelCls}>Input variable</label>
			<VariableInput value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
				oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.input_var)}
		</div>
		<div class="relative">
			<label class={labelCls}>JSON Path</label>
			<VariableInput value={block.settings.json_path} placeholder="e.g. user.token" class={inputCls}
				oninput={(e) => updateSettings('json_path', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.json_path)}
		</div>
		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Output var</label>
				<VariableInput value={block.settings.output_var} class={inputCls}
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
		<div class="relative">
			<label class={labelCls}>Input variable</label>
			<VariableInput value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
				oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.input_var)}
		</div>
		<div class="relative">
			<label class={labelCls}>Pattern</label>
			<VariableInput value={block.settings.pattern} class={inputCls}
				oninput={(e) => updateSettings('pattern', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.pattern)}
		</div>
		<div>
			<label class={labelCls}>Output format</label>
			<VariableInput value={block.settings.output_format} placeholder="$1" class={inputCls}
				oninput={(e) => updateSettings('output_format', (e.target as HTMLInputElement).value)} />
		</div>
		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Output var</label>
				<VariableInput value={block.settings.output_var} class={inputCls}
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
		<div class="relative">
			<label class={labelCls}>Input variable</label>
			<VariableInput value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
				oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.input_var)}
		</div>
		<div class="relative">
			<label class={labelCls}>CSS Selector</label>
			<VariableInput value={block.settings.selector} placeholder="div.content > a" class={inputCls}
				oninput={(e) => updateSettings('selector', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.selector)}
		</div>
		<div>
			<label class={labelCls}>Attribute (empty = text content)</label>
			<VariableInput value={block.settings.attribute} placeholder="href" class={inputCls}
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
				<VariableInput value={block.settings.output_var} class={inputCls}
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
		<div class="relative">
			<label class={labelCls}>Input variable</label>
			<VariableInput value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
				oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.input_var)}
		</div>
		<div class="relative">
			<label class={labelCls}>XPath expression</label>
			<VariableInput value={block.settings.xpath} placeholder="//div[@class='result']/text()" class={inputCls}
				oninput={(e) => updateSettings('xpath', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.xpath)}
		</div>
		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Output var</label>
				<VariableInput value={block.settings.output_var} class={inputCls}
					oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
			</div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4">
				<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
				CAP
			</label>
		</div>
	</div>

<!-- ===================== PARSE COOKIE ===================== -->
{:else if block.settings.type === 'ParseCookie'}
	<div class="space-y-1.5">
		<div class="relative">
			<label class={labelCls}>Input variable</label>
			<VariableInput value={block.settings.input_var} placeholder="data.SOURCE.COOKIES" class={inputCls}
				oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.input_var)}
		</div>
		<div class="relative">
			<label class={labelCls}>Cookie name</label>
			<VariableInput value={block.settings.cookie_name} placeholder="session_id" class={inputCls}
				oninput={(e) => updateSettings('cookie_name', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.cookie_name)}
		</div>
		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Output var</label>
				<VariableInput value={block.settings.output_var} class={inputCls}
					oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
			</div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4">
				<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
				CAP
			</label>
		</div>
	</div>

<!-- ===================== LAMBDA PARSER ===================== -->
{:else if block.settings.type === 'LambdaParser'}
	<div class="space-y-1.5">
		<div class="relative">
			<label class={labelCls}>Input variable</label>
			<VariableInput value={block.settings.input_var} placeholder="data.SOURCE" class={inputCls}
				oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.input_var)}
		</div>
		<div class="relative">
			<label class={labelCls}>Lambda expression</label>
			<VariableInput value={block.settings.lambda_expression} placeholder="x => x.split(',')[0]" class={inputCls}
				oninput={(e) => updateSettings('lambda_expression', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.lambda_expression)}
		</div>
		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Output var</label>
				<VariableInput value={block.settings.output_var} class={inputCls}
					oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
			</div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4">
				<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
				CAP
			</label>
		</div>
	</div>

<!-- ===================== UNIFIED PARSE ===================== -->
{:else if block.settings.type === 'Parse'}
	{@const mode = block.settings.parse_mode || 'LR'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Parse Mode</label>
			<SkeuSelect value={mode} onValueChange={(v) => updateSettings('parse_mode', v)}
				options={PARSE_MODES} class="w-full mt-0.5" />
		</div>

		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Input variable</label>
				<VariableInput value={block.settings.input_var || ''} placeholder="data.SOURCE" class={inputCls}
					oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
			</div>
			<div class="flex-1">
				<label class={labelCls}>Output var</label>
				<VariableInput value={block.settings.output_var || 'PARSED'} class={inputCls}
					oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
			</div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4">
				<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
				CAP
			</label>
		</div>

		{#if mode === 'LR'}
			<div class="relative">
				<label class={labelCls}>Left delimiter</label>
				<VariableInput value={block.settings.left || ''} placeholder='e.g. "token":"' class={inputCls}
					oninput={(e) => updateSettings('left', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.left)}
			</div>
			<div class="relative">
				<label class={labelCls}>Right delimiter</label>
				<VariableInput value={block.settings.right || ''} placeholder='"' class={inputCls}
					oninput={(e) => updateSettings('right', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.right)}
			</div>
			<div class="flex gap-3">
				<label class="flex items-center gap-2 text-[11px] text-foreground">
					<input type="checkbox" checked={block.settings.recursive} onchange={() => updateSettings('recursive', !block!.settings.recursive)} class="skeu-checkbox" />
					Recursive (all matches)
				</label>
				<label class="flex items-center gap-2 text-[11px] text-foreground">
					<input type="checkbox" checked={block.settings.case_insensitive} onchange={() => updateSettings('case_insensitive', !block!.settings.case_insensitive)} class="skeu-checkbox" />
					Case insensitive
				</label>
			</div>
		{:else if mode === 'Regex'}
			<div class="relative">
				<label class={labelCls}>Pattern <span class="text-muted-foreground/60">(regex)</span></label>
				<VariableInput value={block.settings.pattern || ''} placeholder="(\w+)" class={`${inputCls} font-mono`}
					oninput={(e) => updateSettings('pattern', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.pattern)}
			</div>
			<div>
				<label class={labelCls}>Output format</label>
				<VariableInput value={block.settings.output_format || '$1'} placeholder="$1" class={`${inputCls} font-mono`}
					oninput={(e) => updateSettings('output_format', (e.target as HTMLInputElement).value)} />
				<p class="text-[9px] text-muted-foreground mt-0.5">Use $1, $2 for capture groups</p>
			</div>
			<label class="flex items-center gap-2 text-[11px] text-foreground">
				<input type="checkbox" checked={block.settings.multi_line} onchange={() => updateSettings('multi_line', !block!.settings.multi_line)} class="skeu-checkbox" />
				Multi-line mode
			</label>
		{:else if mode === 'Json'}
			<div>
				<label class={labelCls}>JSON path <span class="text-muted-foreground/60">(dot notation)</span></label>
				<VariableInput value={block.settings.json_path || ''} placeholder="data.token" class={`${inputCls} font-mono`}
					oninput={(e) => updateSettings('json_path', (e.target as HTMLInputElement).value)} />
				<p class="text-[9px] text-muted-foreground mt-0.5">e.g. <code class="text-foreground/70">user.balance</code> → &lbrace;"user": &lbrace;"balance": 123&rbrace;&rbrace;</p>
			</div>
		{:else if mode === 'Css'}
			<div>
				<label class={labelCls}>CSS Selector</label>
				<VariableInput value={block.settings.selector || ''} placeholder="div.token-value" class={`${inputCls} font-mono`}
					oninput={(e) => updateSettings('selector', (e.target as HTMLInputElement).value)} />
			</div>
			<div class="flex gap-2">
				<div class="flex-1">
					<label class={labelCls}>Attribute <span class="text-muted-foreground/60">(or innerText)</span></label>
					<VariableInput value={block.settings.attribute || 'innerText'} placeholder="innerText" class={inputCls}
						oninput={(e) => updateSettings('attribute', (e.target as HTMLInputElement).value)} />
				</div>
				<div class="w-16">
					<label class={labelCls}>Index</label>
					<input type="number" value={block.settings.index ?? 0} class={inputCls}
						oninput={(e) => updateSettings('index', parseInt((e.target as HTMLInputElement).value))} />
				</div>
			</div>
		{:else if mode === 'XPath'}
			<div>
				<label class={labelCls}>XPath expression</label>
				<VariableInput value={block.settings.xpath || ''} placeholder="//div[@class='token']/text()" class={`${inputCls} font-mono`}
					oninput={(e) => updateSettings('xpath', (e.target as HTMLInputElement).value)} />
			</div>
		{:else if mode === 'Cookie'}
			<div>
				<label class={labelCls}>Cookie name</label>
				<VariableInput value={block.settings.cookie_name || ''} placeholder="session_token" class={inputCls}
					oninput={(e) => updateSettings('cookie_name', (e.target as HTMLInputElement).value)} />
				<p class="text-[9px] text-muted-foreground mt-0.5">Parses Set-Cookie or Cookie header string from input variable</p>
			</div>
		{:else if mode === 'Lambda'}
			<div class="relative">
				<label class={labelCls}>Lambda expression</label>
				<VariableInput value={block.settings.lambda_expression || "x => x.split(',')[0]"} placeholder="x => x.split(',')[0]" class={`${inputCls} font-mono`}
					oninput={(e) => updateSettings('lambda_expression', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.lambda_expression)}
				<p class="text-[9px] text-muted-foreground mt-0.5">Supported: .split(',')[0] .trim() .toUpperCase() .toLowerCase() .replace() .indexOf() .substring()</p>
			</div>
		{/if}
	</div>
{/if}
