<script lang="ts">
	import type { Block, KeyCondition, Keychain } from '$lib/types';
	import SkeuSelect from '../SkeuSelect.svelte';
	import { smallInputCls, labelCls, hasVars, COMPARISON_OPTIONS } from './shared';

	let { block, updateSettings, embedBadge }: {
		block: Block;
		updateSettings: (key: string, value: unknown) => void;
		embedBadge: import('svelte').Snippet<[string | undefined]>;
	} = $props();

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
</script>

{#if block.settings.type === 'KeyCheck'}
	<div class="space-y-1.5">
		<div class="flex items-center justify-between">
			<span class={labelCls}>Keychains</span>
			<button class="text-[10px] text-primary hover:underline" onclick={addKeychain}>+ Add Keychain</button>
		</div>
		{#each block.settings.keychains as keychain, ki}
			<div class="bg-background rounded p-2 border border-border overflow-hidden">
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
					<div class="flex gap-1 mb-1 items-center min-w-0">
						<div class="relative flex-1 min-w-0">
							<VariableInput value={cond.source} placeholder="data.SOURCE" class="w-full {smallInputCls}"
								oninput={(e) => updateConditionField(ki, ci, 'source', (e.target as HTMLInputElement).value)} />
							{@render embedBadge(cond.source)}
						</div>
						<SkeuSelect value={cond.comparison}
							onValueChange={(v) => updateConditionField(ki, ci, 'comparison', v)}
							options={COMPARISON_OPTIONS}
							class="text-[10px] shrink-0"
						/>
						<div class="relative flex-1 min-w-0">
							<VariableInput value={cond.value} placeholder="value" class="w-full {smallInputCls}"
								oninput={(e) => updateConditionField(ki, ci, 'value', (e.target as HTMLInputElement).value)} />
							{@render embedBadge(cond.value)}
						</div>
						<button class="text-muted-foreground hover:text-red px-0.5 shrink-0" onclick={() => removeCondition(ki, ci)}>x</button>
					</div>
				{/each}
				<button class="text-[10px] text-primary hover:underline mt-1" onclick={() => addCondition(ki)}>+ Add Condition</button>
			</div>
		{/each}
	</div>
{/if}
