<script lang="ts">
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';
	import type { Block } from '$lib/types/block';
	import type { HeaderSpoofSettings } from '$lib/types/block-settings';

	let { block = $bindable() }: { block: Block & { settings: { type: 'HeaderSpoof' } & HeaderSpoofSettings } } = $props();

	const labelCls = 'text-[10px] uppercase tracking-wider text-muted-foreground block mb-0.5';

	function update(key: keyof HeaderSpoofSettings, value: unknown) {
		(block.settings as any)[key] = value;
	}

	let strategy = $derived(block.settings.strategy || 'RandomPublic');
</script>

<div class="space-y-2 p-2 text-xs">
	<p class="text-[9px] text-muted-foreground">Injects proxy-detection bypass headers into the next HTTP Request block automatically. No header wiring needed.</p>

	<!-- Strategy -->
	<div>
		<label class={labelCls}>IP Strategy</label>
		<SkeuSelect
			value={strategy}
			onValueChange={(v) => update('strategy', v)}
			options={[
				{ value: 'RandomPublic', label: 'Random Public IPv4 — new random IP each request' },
				{ value: 'FixedList', label: 'Fixed List — rotate through a custom IP list' },
				{ value: 'FromProxy', label: 'From Proxy — use the current proxy\'s IP' },
				{ value: 'Manual', label: 'Manual — static value (supports {VAR})' },
			]}
		/>
	</div>

	{#if strategy === 'FixedList'}
		<div>
			<label class={labelCls}>IP List (one per line)</label>
			<textarea
				value={block.settings.fixed_ips || ''}
				oninput={(e) => update('fixed_ips', (e.target as HTMLTextAreaElement).value)}
				placeholder={"1.2.3.4\n5.6.7.8"}
				rows={4}
				class="skeu-input w-full font-mono text-[11px] resize-y"
			></textarea>
		</div>
	{/if}

	{#if strategy === 'Manual'}
		<div>
			<label class={labelCls}>Manual Value</label>
			<input
				type="text"
				value={block.settings.manual_value || ''}
				oninput={(e) => update('manual_value', (e.target as HTMLInputElement).value)}
				placeholder="1.2.3.4 or {VAR}"
				class="skeu-input w-full font-mono text-[11px]"
			/>
		</div>
	{/if}

	<!-- Headers to inject -->
	<div>
		<label class={labelCls}>Headers to Inject</label>
		<div class="space-y-1">
			{#each [
				{ key: 'inject_xff', label: 'X-Forwarded-For' },
				{ key: 'inject_x_real_ip', label: 'X-Real-IP' },
				{ key: 'inject_cf_connecting_ip', label: 'CF-Connecting-IP (Cloudflare)' },
				{ key: 'inject_true_client_ip', label: 'True-Client-IP (Akamai/CF Enterprise)' },
				{ key: 'inject_proto', label: 'X-Forwarded-Proto: https' },
				{ key: 'inject_host', label: 'X-Forwarded-Host (mirrors Host header)' },
			] as h}
				<div class="flex items-center gap-2">
					<input
						type="checkbox"
						id="hs-{h.key}"
						checked={(block.settings as any)[h.key] ?? false}
						onchange={(e) => update(h.key as keyof HeaderSpoofSettings, (e.target as HTMLInputElement).checked)}
						class="skeu-checkbox"
					/>
					<label for="hs-{h.key}" class="text-[11px] text-foreground cursor-pointer">{h.label}</label>
				</div>
			{/each}
		</div>
	</div>

	<!-- Output variable -->
	<div>
		<label class={labelCls}>Store IP As Variable (optional)</label>
		<input
			type="text"
			value={block.settings.output_var || ''}
			oninput={(e) => update('output_var', (e.target as HTMLInputElement).value)}
			placeholder="SPOOF_IP (leave blank to skip)"
			class="skeu-input w-full font-mono text-[11px]"
		/>
		<p class="text-[9px] text-muted-foreground mt-0.5">Chosen IP stored as this variable for downstream use.</p>
	</div>
</div>
