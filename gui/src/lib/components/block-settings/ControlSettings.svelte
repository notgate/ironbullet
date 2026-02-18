<script lang="ts">
	import type { Block } from '$lib/types';
	import SkeuSelect from '../SkeuSelect.svelte';
	import VariableInput from '../VariableInput.svelte';
	import { inputCls, labelCls, smallInputCls, hasVars, COMPARISON_OPTIONS } from './shared';

	let { block, updateSettings, embedBadge }: {
		block: Block;
		updateSettings: (key: string, value: unknown) => void;
		embedBadge: import('svelte').Snippet<[string | undefined]>;
	} = $props();
</script>

<!-- ===================== IF/ELSE ===================== -->
{#if block.settings.type === 'IfElse'}
	<div class="space-y-1.5">
		<span class={labelCls}>Condition</span>
		<div class="flex gap-1">
			<VariableInput value={block.settings.condition.source} placeholder="data.SOURCE" class="flex-1 {smallInputCls}"
				oninput={(e) => updateSettings('condition', { ...block!.settings.condition, source: (e.target as HTMLInputElement).value })} />
			<SkeuSelect value={block.settings.condition.comparison}
				onValueChange={(v) => updateSettings('condition', { ...block!.settings.condition, comparison: v })}
				options={COMPARISON_OPTIONS}
				class="text-[10px]"
			/>
			<VariableInput value={block.settings.condition.value} placeholder="value" class="flex-1 {smallInputCls}"
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
				<VariableInput value={block.settings.list_var} placeholder="@myList" class={inputCls}
					oninput={(e) => updateSettings('list_var', (e.target as HTMLInputElement).value)} />
			</div>
			<div>
				<label class={labelCls}>Item variable name</label>
				<VariableInput value={block.settings.item_var} placeholder="@item" class={inputCls}
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
				<VariableInput value={block.settings.output_var} class={inputCls}
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
	<div class="relative">
		<label class={labelCls}>Message</label>
		<VariableInput value={block.settings.message} placeholder="Log message with <variables>" class={inputCls}
			oninput={(e) => updateSettings('message', (e.target as HTMLInputElement).value)} />
		{@render embedBadge(block.settings.message)}
	</div>

<!-- ===================== SET VARIABLE ===================== -->
{:else if block.settings.type === 'SetVariable'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Variable name</label>
			<VariableInput value={block.settings.name} class={inputCls}
				oninput={(e) => updateSettings('name', (e.target as HTMLInputElement).value)} />
		</div>
		<div class="relative">
			<label class={labelCls}>Value</label>
			<VariableInput value={block.settings.value} class={inputCls}
				oninput={(e) => updateSettings('value', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.value)}
		</div>
		<label class="flex items-center gap-2 text-[11px] text-foreground">
			<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
			Capture (save to hit output)
		</label>
	</div>

<!-- ===================== CLEAR COOKIES ===================== -->
{:else if block.settings.type === 'ClearCookies'}
	<div class="text-[11px] text-muted-foreground">
		Clears all cookies in the current session. No additional settings required.
	</div>
{/if}
