<script lang="ts">
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';
	import VariableInput from '$lib/components/VariableInput.svelte';
	import type { Block } from '$lib/types/block';
	import type { HeaderSpoofSettings } from '$lib/types/block-settings';

	let { block, updateSettings, embedBadge }: {
		block: Block;
		updateSettings: (key: string, value: unknown) => void;
		embedBadge: import('svelte').Snippet<[string | undefined]>;
	} = $props();

	const s = $derived(block.settings as { type: 'HeaderSpoof' } & HeaderSpoofSettings);
	const labelCls = 'text-[10px] uppercase tracking-wider text-muted-foreground block mb-0.5';
	const strategy = $derived(s.strategy || 'RandomPublic');
</script>

<div class="space-y-2 p-2 text-xs">
	<p class="text-[9px] text-muted-foreground">Injects proxy-detection bypass headers into the next HTTP Request block. No manual header wiring required.</p>

	<!-- IP Strategy -->
	<div>
		<label class={labelCls}>IP Strategy</label>
		<SkeuSelect
			value={strategy}
			onValueChange={(v) => updateSettings('strategy', v)}
			options={[
				{ value: 'RandomPublic', label: 'Random Public IPv4 — new random IP each request' },
				{ value: 'FixedList',    label: 'Fixed List — rotate through a custom IP list' },
				{ value: 'FromProxy',    label: 'From Proxy — forward the current proxy IP' },
				{ value: 'Manual',       label: 'Manual — static value (supports variables)' },
			]}
		/>
	</div>

	{#if strategy === 'FixedList'}
		<div>
			<label class={labelCls}>IP List (one per line)</label>
			<div class="relative">
				<textarea
					value={s.fixed_ips || ''}
					oninput={(e) => updateSettings('fixed_ips', (e.target as HTMLTextAreaElement).value)}
					placeholder={"1.2.3.4\n5.6.7.8"}
					rows={4}
					class="skeu-input w-full font-mono text-[11px] resize-y"
				></textarea>
				{@render embedBadge(s.fixed_ips)}
			</div>
		</div>
	{/if}

	{#if strategy === 'Manual'}
		<div>
			<label class={labelCls}>Manual Value</label>
			<VariableInput
				value={s.manual_value || ''}
				placeholder="1.2.3.4"
				class="flex-1 skeu-input text-[11px] font-mono"
				oninput={(e) => updateSettings('manual_value', (e.target as HTMLInputElement).value)}
			/>
		</div>
	{/if}

	<!-- Headers to inject -->
	<div>
		<label class={labelCls}>Headers to Inject</label>
		<div class="space-y-1 mt-0.5">
			{#each [
				{ key: 'inject_xff',              label: 'X-Forwarded-For',                   default: true  },
				{ key: 'inject_x_real_ip',        label: 'X-Real-IP',                         default: false },
				{ key: 'inject_cf_connecting_ip', label: 'CF-Connecting-IP (Cloudflare)',      default: false },
				{ key: 'inject_true_client_ip',   label: 'True-Client-IP (Akamai/CF Ent.)',   default: false },
				{ key: 'inject_proto',            label: 'X-Forwarded-Proto: https',           default: true  },
				{ key: 'inject_host',             label: 'X-Forwarded-Host (mirrors Host)',    default: false },
			] as h}
				<label class="flex items-center gap-2 cursor-pointer">
					<input
						type="checkbox"
						checked={(s as any)[h.key] ?? h.default}
						onchange={(e) => updateSettings(h.key, (e.target as HTMLInputElement).checked)}
						class="skeu-checkbox"
					/>
					<span class="text-[11px] text-foreground">{h.label}</span>
				</label>
			{/each}
		</div>
	</div>

	<!-- Output variable -->
	<div>
		<label class={labelCls}>Store Chosen IP As Variable</label>
		<VariableInput
			value={s.output_var || ''}
			placeholder="SPOOF_IP (leave blank to skip)"
			class="flex-1 skeu-input text-[11px] font-mono"
			oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)}
		/>
		<p class="text-[9px] text-muted-foreground mt-0.5">Chosen IP stored as this variable for downstream blocks.</p>
	</div>
</div>
