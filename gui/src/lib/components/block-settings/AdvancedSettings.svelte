<script lang="ts">
	import { app } from '$lib/state.svelte';
	import type { Block } from '$lib/types';
	import SkeuSelect from '../SkeuSelect.svelte';
	import VariableInput from '../VariableInput.svelte';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import Plus from '@lucide/svelte/icons/plus';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import { inputCls, labelCls, hintCls, smallInputCls, hasVars, fieldHint } from './shared';

	let { block, updateSettings, embedBadge }: {
		block: Block;
		updateSettings: (key: string, value: unknown) => void;
		embedBadge: import('svelte').Snippet<[string | undefined]>;
	} = $props();
</script>

<!-- ===================== DATE FUNCTION ===================== -->
{#if block.settings.type === 'DateFunction'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Function</label>
			{#if fieldHint(block, 'function_type')}<p class={hintCls}>{fieldHint(block, 'function_type')}</p>{/if}
			<SkeuSelect value={block.settings.function_type}
				options={[
					{value:'Now',label:'Now — Current date/time'},{value:'UnixTimestamp',label:'Unix Timestamp — Current (seconds)'},
					{value:'CurrentUnixTimeMs',label:'CurrentUnixTime — Current (milliseconds)'},
					{value:'UnixToDate',label:'Unix to Date — Convert timestamp to string'},
					{value:'FormatDate',label:'Format Date — Reformat a date string'},
					{value:'ParseDate',label:'Parse Date — Date string to timestamp'},
					{value:'AddTime',label:'Add Time — Add duration to timestamp'},
					{value:'SubtractTime',label:'Subtract Time — Subtract duration from timestamp'},
					{value:'Compute',label:'Compute — Evaluate arithmetic expression'},
					{value:'Round',label:'Round — Round number to N decimals'},
				]}
				onValueChange={(v) => updateSettings('function_type', v)} />
		</div>
		{#if !['Now','UnixTimestamp','CurrentUnixTimeMs'].includes(block.settings.function_type)}
			<div>
				<label class={labelCls}>Input var</label>
				{#if fieldHint(block, 'input_var')}<p class={hintCls}>{fieldHint(block, 'input_var')}</p>{/if}
				<VariableInput value={block.settings.input_var} class={inputCls} placeholder="data.SOURCE"
					oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
			</div>
		{/if}
		{#if block.settings.function_type === 'Compute'}
			<div>
				<label class={labelCls}>Expression <span class="text-muted-foreground/60">(supports +, -, *, /, %, parens)</span></label>
				<VariableInput value={block.settings.param || ''} placeholder="<BALANCE> * 0.9 + 100" class={`${inputCls} font-mono`}
					oninput={(e) => updateSettings('param', (e.target as HTMLInputElement).value)} />
				<p class="text-[9px] text-muted-foreground mt-0.5">Variables are interpolated before evaluation. e.g. &lt;PRICE&gt; * 2</p>
			</div>
		{:else if block.settings.function_type === 'Round'}
			<div>
				<label class={labelCls}>Decimal places</label>
				<input type="number" value={block.settings.param || '2'} min="0" max="10" class="w-20 skeu-input text-[11px]"
					oninput={(e) => updateSettings('param', (e.target as HTMLInputElement).value)} />
			</div>
		{:else if !['Now','UnixTimestamp','CurrentUnixTimeMs','Compute','Round'].includes(block.settings.function_type)}
			<div>
				<label class={labelCls}>Format</label>
				{#if fieldHint(block, 'format')}<p class={hintCls}>{fieldHint(block, 'format')}</p>{/if}
				<VariableInput value={block.settings.format} class={inputCls} placeholder="%Y-%m-%d %H:%M:%S"
					oninput={(e) => updateSettings('format', (e.target as HTMLInputElement).value)} />
			</div>
		{/if}
		{#if ['AddTime','SubtractTime'].includes(block.settings.function_type)}
			<div class="flex gap-2">
				<div class="flex-1">
					<label class={labelCls}>Amount</label>
					{#if fieldHint(block, 'amount')}<p class={hintCls}>{fieldHint(block, 'amount')}</p>{/if}
					<input type="number" value={block.settings.amount} class={inputCls}
						oninput={(e) => updateSettings('amount', parseInt((e.target as HTMLInputElement).value) || 0)} />
				</div>
				<div class="flex-1">
					<label class={labelCls}>Unit</label>
					{#if fieldHint(block, 'unit')}<p class={hintCls}>{fieldHint(block, 'unit')}</p>{/if}
					<SkeuSelect value={block.settings.unit}
						options={[{value:'seconds',label:'Seconds'},{value:'minutes',label:'Minutes'},{value:'hours',label:'Hours'},{value:'days',label:'Days'}]}
						onValueChange={(v) => updateSettings('unit', v)} />
				</div>
			</div>
		{/if}
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

<!-- ===================== CASE / SWITCH ===================== -->
{:else if block.settings.type === 'CaseSwitch'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Input variable</label>
			{#if fieldHint(block, 'input_var')}<p class={hintCls}>{fieldHint(block, 'input_var')}</p>{/if}
			<VariableInput value={block.settings.input_var} class={inputCls} placeholder="data.RESPONSECODE"
				oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Cases</label>
			{#if fieldHint(block, 'cases')}<p class={hintCls}>{fieldHint(block, 'cases')}</p>{/if}
			{#each block.settings.cases as c, ci}
				<div class="flex gap-1 items-center mt-1">
					<VariableInput value={c.match_value} class="{smallInputCls} flex-1" placeholder="Match value"
						oninput={(e) => {
							const cases = [...block!.settings.cases];
							cases[ci] = { ...cases[ci], match_value: (e.target as HTMLInputElement).value };
							updateSettings('cases', cases);
						}} />
					<ArrowRight size={10} class="text-muted-foreground shrink-0" />
					<VariableInput value={c.result_value} class="{smallInputCls} flex-1" placeholder="Result value"
						oninput={(e) => {
							const cases = [...block!.settings.cases];
							cases[ci] = { ...cases[ci], result_value: (e.target as HTMLInputElement).value };
							updateSettings('cases', cases);
						}} />
					<button class="p-0.5 rounded hover:bg-destructive/20 text-muted-foreground shrink-0"
						onclick={() => {
							const cases = block!.settings.cases.filter((_: unknown, i: number) => i !== ci);
							updateSettings('cases', cases);
						}}>
						<Trash2 size={10} />
					</button>
				</div>
			{/each}
			<button class="skeu-btn text-[10px] mt-1 flex items-center gap-1"
				onclick={() => updateSettings('cases', [...block!.settings.cases, { match_value: '', result_value: '' }])}>
				<Plus size={10} /> Add Case
			</button>
		</div>
		<div>
			<label class={labelCls}>Default value</label>
			{#if fieldHint(block, 'default_value')}<p class={hintCls}>{fieldHint(block, 'default_value')}</p>{/if}
			<VariableInput value={block.settings.default_value} class={inputCls} placeholder="FAIL"
				oninput={(e) => updateSettings('default_value', (e.target as HTMLInputElement).value)} />
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

<!-- ===================== COOKIE CONTAINER ===================== -->
{:else if block.settings.type === 'CookieContainer'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Source type</label>
			{#if fieldHint(block, 'source_type')}<p class={hintCls}>{fieldHint(block, 'source_type')}</p>{/if}
			<SkeuSelect
				value={block.settings.source_type}
				onValueChange={(v) => updateSettings('source_type', v)}
				options={[{value:'text',label:'Raw text'},{value:'file',label:'File path'}]}
				class="w-full mt-0.5"
			/>
		</div>
		<div>
			<label class={labelCls}>{block.settings.source_type === 'file' ? 'File path' : 'Cookie text'}</label>
			{#if fieldHint(block, 'source')}<p class={hintCls}>{fieldHint(block, 'source')}</p>{/if}
			{#if block.settings.source_type === 'file'}
				<VariableInput value={block.settings.source} class={inputCls} placeholder="C:\cookies.txt or <FILE_PATH>"
					oninput={(e) => updateSettings('source', (e.target as HTMLInputElement).value)} />
			{:else}
				<textarea value={block.settings.source}
					placeholder={".example.com\tTRUE\t/\tFALSE\t0\tsession\tabc123\nname=value"}
					class="w-full skeu-input text-[11px] font-mono min-h-[100px] resize-y mt-0.5"
					oninput={(e) => updateSettings('source', (e.target as HTMLTextAreaElement).value)}></textarea>
			{/if}
			<p class="text-[9px] text-muted-foreground mt-0.5">Accepts Netscape format (tab-separated) or simple <code class="text-foreground/70">name=value</code> lines.</p>
		</div>
		<div>
			<label class={labelCls}>Domain filter</label>
			{#if fieldHint(block, 'domain')}<p class={hintCls}>{fieldHint(block, 'domain')}</p>{/if}
			<VariableInput value={block.settings.domain} class={inputCls} placeholder=".example.com (leave empty for all)"
				oninput={(e) => updateSettings('domain', (e.target as HTMLInputElement).value)} />
		</div>
		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Output var</label>
				{#if fieldHint(block, 'output_var')}<p class={hintCls}>{fieldHint(block, 'output_var')}</p>{/if}
				<VariableInput value={block.settings.output_var} class={inputCls}
					oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
			</div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4">
				<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
				CAP
			</label>
		</div>
		<label class="flex items-center gap-2 text-[11px] text-foreground">
			<input type="checkbox" checked={block.settings.save_netscape} onchange={() => updateSettings('save_netscape', !block!.settings.save_netscape)} class="skeu-checkbox" />
			Also save in Netscape format
		</label>
		{#if fieldHint(block, 'save_netscape')}<p class={hintCls}>{fieldHint(block, 'save_netscape')}</p>{/if}
	</div>

<!-- ===================== RANDOM USER AGENT ===================== -->
{:else if block.settings.type === 'RandomUserAgent'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Mode</label>
			<SkeuSelect
				value={block.settings.mode}
				onValueChange={(v) => updateSettings('mode', v)}
				options={[{value:'Random',label:'Random (built-in list)'},{value:'CustomList',label:'Custom List'}]}
				class="text-[11px] w-full"
			/>
		</div>
		{#if block.settings.mode === 'Random'}
			<div>
				<label class={labelCls}>Browser filter</label>
				<div class="flex gap-2 flex-wrap">
					{#each ['Chrome', 'Firefox', 'Safari', 'Edge'] as browser}
						<label class="flex items-center gap-1 text-[10px]">
							<input type="checkbox"
								checked={block.settings.browser_filter.includes(browser)}
								onchange={() => {
									const filters = [...block.settings.browser_filter];
									const idx = filters.indexOf(browser);
									if (idx >= 0) filters.splice(idx, 1); else filters.push(browser);
									updateSettings('browser_filter', filters);
								}}
								class="skeu-checkbox" />
							{browser}
						</label>
					{/each}
				</div>
			</div>
			<div>
				<label class={labelCls}>Platform filter</label>
				<div class="flex gap-2 flex-wrap">
					{#each ['Desktop', 'Mobile', 'Tablet'] as platform}
						<label class="flex items-center gap-1 text-[10px]">
							<input type="checkbox"
								checked={block.settings.platform_filter.includes(platform)}
								onchange={() => {
									const filters = [...block.settings.platform_filter];
									const idx = filters.indexOf(platform);
									if (idx >= 0) filters.splice(idx, 1); else filters.push(platform);
									updateSettings('platform_filter', filters);
								}}
								class="skeu-checkbox" />
							{platform}
						</label>
					{/each}
				</div>
			</div>
		{:else}
			<div>
				<label class={labelCls}>Custom UA list (one per line)</label>
				<textarea value={block.settings.custom_list}
					class="w-full skeu-input text-[11px] font-mono min-h-[80px] resize-y mt-0.5"
					oninput={(e) => updateSettings('custom_list', (e.target as HTMLTextAreaElement).value)}
				></textarea>
			</div>
		{/if}
		<label class="flex items-center gap-2 text-[11px]">
			<input type="checkbox" checked={block.settings.match_tls} class="skeu-checkbox"
				onchange={() => updateSettings('match_tls', !block.settings.match_tls)} />
			Match TLS fingerprint (JA3 + HTTP/2)
		</label>
		{#if block.settings.match_tls}
			<p class={hintCls}>Automatically selects matching TLS profile for the chosen browser</p>
		{/if}
		<div>
			<label class={labelCls}>Output variable</label>
			<VariableInput value={block.settings.output_var} class={inputCls}
				oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
		</div>
		<label class="flex items-center gap-2 text-[11px]">
			<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
				onchange={() => updateSettings('capture', !block.settings.capture)} />
			Capture
		</label>
	</div>

<!-- ===================== OCR CAPTCHA ===================== -->
{:else if block.settings.type === 'OcrCaptcha'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Input variable (base64 image)</label>
			<VariableInput value={block.settings.input_var} class={inputCls}
				oninput={(e) => updateSettings('input_var', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Language</label>
			<VariableInput value={block.settings.language} class={inputCls} placeholder="eng"
				oninput={(e) => updateSettings('language', (e.target as HTMLInputElement).value)} />
			{#if fieldHint(block, 'language')}<p class={hintCls}>{fieldHint(block, 'language')}</p>{/if}
		</div>
		<div>
			<label class={labelCls}>Page Segmentation Mode (PSM)</label>
			<input type="number" value={block.settings.psm} min="0" max="13" class={inputCls}
				oninput={(e) => updateSettings('psm', parseInt((e.target as HTMLInputElement).value) || 7)} />
		</div>
		<div>
			<label class={labelCls}>Whitelist (allowed chars)</label>
			<VariableInput value={block.settings.whitelist} class={inputCls} placeholder="0123456789ABCDEFabcdef"
				oninput={(e) => updateSettings('whitelist', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Output variable</label>
			<VariableInput value={block.settings.output_var} class={inputCls}
				oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
		</div>
		<label class="flex items-center gap-2 text-[11px]">
			<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
				onchange={() => updateSettings('capture', !block.settings.capture)} />
			Capture
		</label>
	</div>

<!-- ===================== RECAPTCHA INVISIBLE ===================== -->
{:else if block.settings.type === 'RecaptchaInvisible'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Site Key</label>
			<VariableInput value={block.settings.sitekey} class={inputCls}
				oninput={(e) => updateSettings('sitekey', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Anchor URL</label>
			<VariableInput value={block.settings.anchor_url} class={inputCls}
				oninput={(e) => updateSettings('anchor_url', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Reload URL</label>
			<VariableInput value={block.settings.reload_url} class={inputCls}
				oninput={(e) => updateSettings('reload_url', (e.target as HTMLInputElement).value)} />
		</div>
		<div class="grid grid-cols-2 gap-1">
			<div>
				<label class={labelCls}>co (base64 origin)</label>
				<VariableInput value={block.settings.co} class={inputCls}
					oninput={(e) => updateSettings('co', (e.target as HTMLInputElement).value)} />
			</div>
			<div>
				<label class={labelCls}>v (JS version)</label>
				<VariableInput value={block.settings.v} class={inputCls}
					oninput={(e) => updateSettings('v', (e.target as HTMLInputElement).value)} />
			</div>
		</div>
		<div class="grid grid-cols-2 gap-1">
			<div>
				<label class={labelCls}>ar</label>
				<VariableInput value={block.settings.ar} class={inputCls}
					oninput={(e) => updateSettings('ar', (e.target as HTMLInputElement).value)} />
			</div>
			<div>
				<label class={labelCls}>hi</label>
				<VariableInput value={block.settings.hi} class={inputCls}
					oninput={(e) => updateSettings('hi', (e.target as HTMLInputElement).value)} />
			</div>
		</div>
		<div class="grid grid-cols-2 gap-1">
			<div>
				<label class={labelCls}>size</label>
				<VariableInput value={block.settings.size} class={inputCls}
					oninput={(e) => updateSettings('size', (e.target as HTMLInputElement).value)} />
			</div>
			<div>
				<label class={labelCls}>action</label>
				<VariableInput value={block.settings.action} class={inputCls}
					oninput={(e) => updateSettings('action', (e.target as HTMLInputElement).value)} />
			</div>
		</div>
		<div>
			<label class={labelCls}>cb</label>
			<VariableInput value={block.settings.cb} class={inputCls}
				oninput={(e) => updateSettings('cb', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>User Agent</label>
			<VariableInput value={block.settings.user_agent} class={inputCls}
				oninput={(e) => updateSettings('user_agent', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Output variable</label>
			<VariableInput value={block.settings.output_var} class={inputCls}
				oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
		</div>
		<label class="flex items-center gap-2 text-[11px]">
			<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
				onchange={() => updateSettings('capture', !block.settings.capture)} />
			Capture
		</label>
	</div>

<!-- ===================== XACF SENSOR ===================== -->
{:else if block.settings.type === 'XacfSensor'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Bundle ID</label>
			<VariableInput value={block.settings.bundle_id} class={inputCls} placeholder="com.example.app"
				oninput={(e) => updateSettings('bundle_id', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Version</label>
			<VariableInput value={block.settings.version} class={inputCls} placeholder="2.1.2"
				oninput={(e) => updateSettings('version', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Output variable</label>
			<VariableInput value={block.settings.output_var} class={inputCls}
				oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
		</div>
		<label class="flex items-center gap-2 text-[11px]">
			<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
				onchange={() => updateSettings('capture', !block.settings.capture)} />
			Capture
		</label>
	</div>

<!-- ===================== RANDOM DATA ===================== -->
{:else if block.settings.type === 'RandomData'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Data type</label>
			<SkeuSelect
				value={block.settings.data_type}
				onValueChange={(v) => updateSettings('data_type', v)}
				options={[
					{value:'String',label:'Random String'},{value:'Uuid',label:'UUID'},{value:'Number',label:'Number'},
					{value:'Email',label:'Email'},{value:'FirstName',label:'First Name'},{value:'LastName',label:'Last Name'},
					{value:'FullName',label:'Full Name'},{value:'StreetAddress',label:'Street Address'},
					{value:'City',label:'City'},{value:'State',label:'State'},{value:'ZipCode',label:'Zip Code'},
					{value:'PhoneNumber',label:'Phone Number'},{value:'Date',label:'Date'},
				]}
				class="text-[11px] w-full"
			/>
		</div>
		{#if block.settings.data_type === 'String'}
			<div>
				<label class={labelCls}>Length</label>
				<input type="number" value={block.settings.string_length} class={inputCls} min="1" max="1024"
					oninput={(e) => updateSettings('string_length', parseInt((e.target as HTMLInputElement).value) || 16)} />
			</div>
			<div>
				<label class={labelCls}>Charset</label>
				<SkeuSelect
					value={block.settings.string_charset}
					onValueChange={(v) => updateSettings('string_charset', v)}
					options={[
						{value:'alphanumeric',label:'Alphanumeric'},{value:'alpha',label:'Alpha only'},
						{value:'hex',label:'Hex'},{value:'numeric',label:'Numeric'},{value:'custom',label:'Custom'},
					]}
					class="text-[11px] w-full"
				/>
			</div>
			{#if block.settings.string_charset === 'custom'}
				<div>
					<label class={labelCls}>Custom characters</label>
					<VariableInput value={block.settings.custom_chars} class={inputCls}
						oninput={(e) => updateSettings('custom_chars', (e.target as HTMLInputElement).value)} />
				</div>
			{/if}
		{:else if block.settings.data_type === 'Number'}
			<div class="grid grid-cols-2 gap-2">
				<div>
					<label class={labelCls}>Min</label>
					<input type="number" value={block.settings.number_min} class={inputCls}
						oninput={(e) => updateSettings('number_min', parseInt((e.target as HTMLInputElement).value) || 0)} />
				</div>
				<div>
					<label class={labelCls}>Max</label>
					<input type="number" value={block.settings.number_max} class={inputCls}
						oninput={(e) => updateSettings('number_max', parseInt((e.target as HTMLInputElement).value) || 100)} />
				</div>
			</div>
			<label class="flex items-center gap-2 text-[11px]">
				<input type="checkbox" checked={block.settings.number_decimal} class="skeu-checkbox"
					onchange={() => updateSettings('number_decimal', !block.settings.number_decimal)} />
				Decimal
			</label>
		{:else if block.settings.data_type === 'Date'}
			<div>
				<label class={labelCls}>Date format</label>
				<VariableInput value={block.settings.date_format} class={inputCls} placeholder="%Y-%m-%d"
					oninput={(e) => updateSettings('date_format', (e.target as HTMLInputElement).value)} />
			</div>
			<div class="grid grid-cols-2 gap-2">
				<div>
					<label class={labelCls}>Min date</label>
					<VariableInput value={block.settings.date_min} class={inputCls} placeholder="1990-01-01"
						oninput={(e) => updateSettings('date_min', (e.target as HTMLInputElement).value)} />
				</div>
				<div>
					<label class={labelCls}>Max date</label>
					<VariableInput value={block.settings.date_max} class={inputCls} placeholder="2025-12-31"
						oninput={(e) => updateSettings('date_max', (e.target as HTMLInputElement).value)} />
				</div>
			</div>
		{/if}
		<div>
			<label class={labelCls}>Output variable</label>
			<VariableInput value={block.settings.output_var} class={inputCls}
				oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
		</div>
		<label class="flex items-center gap-2 text-[11px]">
			<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
				onchange={() => updateSettings('capture', !block.settings.capture)} />
			Capture
		</label>
	</div>

<!-- ===================== DATADOME SENSOR ===================== -->
{:else if block.settings.type === 'DataDomeSensor'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Site URL</label>
			<VariableInput value={block.settings.site_url} class={inputCls} placeholder="https://example.com"
				oninput={(e) => updateSettings('site_url', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>DataDome cookie</label>
			<VariableInput value={block.settings.cookie_datadome} class={inputCls} placeholder="<data.COOKIES>"
				oninput={(e) => updateSettings('cookie_datadome', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>User Agent</label>
			<VariableInput value={block.settings.user_agent} class={inputCls} placeholder="<UA>"
				oninput={(e) => updateSettings('user_agent', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Custom WASM (base64, optional)</label>
			<textarea value={block.settings.custom_wasm_b64}
				class="w-full skeu-input text-[10px] font-mono min-h-[40px] resize-y mt-0.5"
				placeholder="Leave empty for default sensor"
				oninput={(e) => updateSettings('custom_wasm_b64', (e.target as HTMLTextAreaElement).value)}
			></textarea>
		</div>
		<div>
			<label class={labelCls}>Output variable</label>
			<VariableInput value={block.settings.output_var} class={inputCls}
				oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
		</div>
		<label class="flex items-center gap-2 text-[11px]">
			<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
				onchange={() => updateSettings('capture', !block.settings.capture)} />
			Capture
		</label>
	</div>

<!-- ===================== AKAMAI V3 SENSOR ===================== -->
{:else if block.settings.type === 'AkamaiV3Sensor'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Mode</label>
			<SkeuSelect
				value={block.settings.mode}
				onValueChange={(v) => updateSettings('mode', v)}
				options={[{value:'Encrypt',label:'Encrypt'},{value:'Decrypt',label:'Decrypt'},{value:'ExtractCookieHash',label:'Extract Cookie Hash'}]}
				class="text-[11px] w-full"
			/>
		</div>
		<div>
			<label class={labelCls}>{block.settings.mode === 'ExtractCookieHash' ? 'bm_sz cookie variable' : 'Payload variable'}</label>
			<VariableInput value={block.settings.payload_var} class={inputCls}
				placeholder={block.settings.mode === 'ExtractCookieHash' ? 'BM_SZ' : 'SENSOR_PAYLOAD'}
				oninput={(e) => updateSettings('payload_var', (e.target as HTMLInputElement).value)} />
		</div>
		{#if block.settings.mode !== 'ExtractCookieHash'}
			<div>
				<label class={labelCls}>File hash (integer)</label>
				<VariableInput value={block.settings.file_hash} class={inputCls} placeholder="e.g. 123456789"
					oninput={(e) => updateSettings('file_hash', (e.target as HTMLInputElement).value)} />
			</div>
			<div>
				<label class={labelCls}>Cookie hash</label>
				<VariableInput value={block.settings.cookie_hash} class={inputCls} placeholder="8888888"
					oninput={(e) => updateSettings('cookie_hash', (e.target as HTMLInputElement).value)} />
				<p class="text-[9px] text-muted-foreground mt-0.5">From bm_sz cookie. Use Extract Cookie Hash mode to get this value. Default: 8888888</p>
			</div>
		{/if}
		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Output variable</label>
				<VariableInput value={block.settings.output_var} class={inputCls}
					oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
			</div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4">
				<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
				CAP
			</label>
		</div>
		<p class="text-[9px] text-muted-foreground">Algorithm by <a href="https://github.com/glizzykingdreko/akamai-v3-sensor-data-helper" target="_blank" class="underline text-foreground/70 hover:text-foreground">glizzykingdreko</a></p>
	</div>

<!-- ===================== PLUGIN BLOCK ===================== -->
{:else if block.settings.type === 'Plugin'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Plugin type</label>
			<div class="text-[11px] font-mono text-muted-foreground px-2 py-1 bg-background rounded border border-border">
				{block.settings.plugin_block_type || 'Not set'}
			</div>
		</div>
		{#if (() => { const meta = app.pluginBlocks.find(pb => pb.block_type_name === block.settings.plugin_block_type); if (!meta?.settings_schema_json) return false; try { const s = JSON.parse(meta.settings_schema_json); return s?.properties && Object.keys(s.properties).length > 0; } catch { return false; } })()}
			{@const pluginSchema = (() => { const meta = app.pluginBlocks.find(pb => pb.block_type_name === block.settings.plugin_block_type); try { return JSON.parse(meta!.settings_schema_json); } catch { return {}; } })()}
			{@const pluginSettings = (() => { try { return JSON.parse(block.settings.settings_json || '{}'); } catch { return {}; } })()}
			{#each Object.entries(pluginSchema.properties || {}) as [propName, propDef]}
				{@const pd = propDef as any}
				<div>
					<label class={labelCls}>{pd.title || propName}</label>
					{#if pd.enum}
						<select class={inputCls} value={pluginSettings[propName] ?? pd.default ?? ''}
							onchange={(e) => {
								const updated = { ...pluginSettings, [propName]: (e.target as HTMLSelectElement).value };
								updateSettings('settings_json', JSON.stringify(updated));
							}}>
							{#each pd.enum as opt}
								<option value={opt}>{opt}</option>
							{/each}
						</select>
					{:else if pd.type === 'boolean'}
						<label class="flex items-center gap-2 text-[11px]">
							<input type="checkbox" class="skeu-checkbox"
								checked={pluginSettings[propName] ?? pd.default ?? false}
								onchange={() => {
									const updated = { ...pluginSettings, [propName]: !(pluginSettings[propName] ?? pd.default ?? false) };
									updateSettings('settings_json', JSON.stringify(updated));
								}} />
						</label>
					{:else if pd.type === 'number' || pd.type === 'integer'}
						<input type="number" class={inputCls}
							value={pluginSettings[propName] ?? pd.default ?? 0}
							oninput={(e) => {
								const updated = { ...pluginSettings, [propName]: parseFloat((e.target as HTMLInputElement).value) };
								updateSettings('settings_json', JSON.stringify(updated));
							}} />
					{:else}
						<div class="relative">
							<VariableInput class={inputCls}
								value={pluginSettings[propName] ?? pd.default ?? ''}
								oninput={(e) => {
									const updated = { ...pluginSettings, [propName]: (e.target as HTMLInputElement).value };
									updateSettings('settings_json', JSON.stringify(updated));
								}} />
							{@render embedBadge(pluginSettings[propName] ?? pd.default ?? '')}
						</div>
					{/if}
				</div>
			{/each}
		{:else}
			{@const kvEntries = (() => { try { return Object.entries(JSON.parse(block.settings.settings_json || '{}')); } catch { return []; } })() as [string, any][]}
			<div>
				<label class={labelCls}>Settings</label>
				<div class="space-y-1">
					{#each kvEntries as [k, v], i}
						<div class="flex items-center gap-1">
							<VariableInput value={k} class={inputCls + ' flex-1'}
								placeholder="key"
								oninput={(e) => {
									const newKey = (e.target as HTMLInputElement).value;
									const obj: Record<string, any> = {};
									kvEntries.forEach(([ek, ev], j) => { obj[j === i ? newKey : ek] = ev; });
									updateSettings('settings_json', JSON.stringify(obj));
								}} />
							<div class="relative flex-1">
							<VariableInput value={typeof v === 'string' ? v : JSON.stringify(v)} class={inputCls + ' w-full'}
								placeholder="value"
								oninput={(e) => {
									const obj: Record<string, any> = {};
									kvEntries.forEach(([ek, ev], j) => { obj[ek] = j === i ? (e.target as HTMLInputElement).value : ev; });
									updateSettings('settings_json', JSON.stringify(obj));
								}} />
							{@render embedBadge(typeof v === 'string' ? v : JSON.stringify(v))}
						</div>
							<button class="p-0.5 text-muted-foreground hover:text-destructive shrink-0" title="Remove field"
								onclick={() => {
									const obj: Record<string, any> = {};
									kvEntries.forEach(([ek, ev], j) => { if (j !== i) obj[ek] = ev; });
									updateSettings('settings_json', JSON.stringify(obj));
								}}>
								<Trash2 size={11} />
							</button>
						</div>
					{/each}
					<button class="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground mt-0.5"
						onclick={() => {
							const obj: Record<string, any> = {};
							kvEntries.forEach(([ek, ev]) => { obj[ek] = ev; });
							obj[''] = '';
							updateSettings('settings_json', JSON.stringify(obj));
						}}>
						<Plus size={10} /> Add field
					</button>
				</div>
			</div>
		{/if}
		<div>
			<label class={labelCls}>Output variable</label>
			<VariableInput value={block.settings.output_var} class={inputCls}
				oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
		</div>
		<label class="flex items-center gap-2 text-[11px]">
			<input type="checkbox" checked={block.settings.capture} class="skeu-checkbox"
				onchange={() => updateSettings('capture', !block.settings.capture)} />
			Capture
		</label>
	</div>

<!-- ===================== GROUP ===================== -->
{:else if block.settings.type === 'Group'}
	<div class="space-y-1.5">
		<label class="flex items-center gap-2 text-[11px]">
			<input type="checkbox" checked={block.settings.collapsed} class="skeu-checkbox"
				onchange={() => updateSettings('collapsed', !block.settings.collapsed)} />
			Collapsed
		</label>
		<div class="text-[10px] text-muted-foreground">
			{block.settings.blocks?.length || 0} block{(block.settings.blocks?.length || 0) !== 1 ? 's' : ''} in group. Drag blocks into the group container below.
		</div>
	</div>
{/if}
