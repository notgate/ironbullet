<script lang="ts">
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';
	import VariableInput from '$lib/components/VariableInput.svelte';
	import type { Block } from '$lib/types/block';
	import type { JwtTokenSettings } from '$lib/types/block-settings';

	let { block, updateSettings, embedBadge }: {
		block: Block;
		updateSettings: (key: string, value: unknown) => void;
		embedBadge: import('svelte').Snippet<[string | undefined]>;
	} = $props();

	const s = $derived(block.settings as { type: 'JwtToken' } & JwtTokenSettings);
	const labelCls = 'text-[10px] uppercase tracking-wider text-muted-foreground block mb-0.5';
	const action = $derived(s.action || 'Sign');
</script>

<div class="space-y-2 p-2 text-xs">
	<!-- Mode -->
	<div>
		<label class={labelCls}>Mode</label>
		<SkeuSelect
			value={action}
			onValueChange={(v) => updateSettings('action', v)}
			options={[
				{ value: 'Sign',   label: 'Sign — generate a signed JWT from claims' },
				{ value: 'Decode', label: 'Decode — verify and extract claims from a JWT' },
			]}
		/>
	</div>

	<!-- Secret -->
	<div>
		<label class={labelCls}>Secret</label>
		<VariableInput
			value={s.secret || ''}
			placeholder="HMAC secret"
			class="flex-1 skeu-input text-[11px] font-mono"
			oninput={(e) => updateSettings('secret', (e.target as HTMLInputElement).value)}
		/>
	</div>

	<!-- Algorithm -->
	<div>
		<label class={labelCls}>Algorithm</label>
		<SkeuSelect
			value={s.algorithm || 'HS256'}
			onValueChange={(v) => updateSettings('algorithm', v)}
			options={[
				{ value: 'HS256', label: 'HS256' },
				{ value: 'HS384', label: 'HS384' },
				{ value: 'HS512', label: 'HS512' },
			]}
		/>
	</div>

	{#if action === 'Sign'}
		<!-- Claims -->
		<div>
			<label class={labelCls}>Claims (JSON)</label>
			<div class="relative">
				<textarea
					value={s.claims || ''}
					oninput={(e) => updateSettings('claims', (e.target as HTMLTextAreaElement).value)}
					placeholder={'{"sub": "{USER}", "role": "admin"}'}
					rows={4}
					class="skeu-input w-full font-mono text-[11px] resize-y"
				></textarea>
				{@render embedBadge(s.claims)}
			</div>
			<p class="text-[9px] text-muted-foreground mt-0.5">Variable interpolation supported in values. <code>iat</code> injected automatically.</p>
		</div>

		<!-- Expires In -->
		<div>
			<label class={labelCls}>Expires In (seconds)</label>
			<input
				type="number"
				min="0"
				value={s.expires_in_secs ?? 0}
				oninput={(e) => updateSettings('expires_in_secs', parseInt((e.target as HTMLInputElement).value) || 0)}
				class="skeu-input w-full text-[11px]"
			/>
			<p class="text-[9px] text-muted-foreground mt-0.5">0 = no <code>exp</code> claim added.</p>
		</div>

		<!-- Output variable -->
		<div>
			<label class={labelCls}>Output Variable</label>
			<VariableInput
				value={s.output_var || 'JWT'}
				placeholder="JWT"
				class="flex-1 skeu-input text-[11px] font-mono"
				oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)}
			/>
			<p class="text-[9px] text-muted-foreground mt-0.5">Signed token stored as this variable for downstream use.</p>
		</div>
	{:else}
		<!-- Token input -->
		<div>
			<label class={labelCls}>Token Input</label>
			<VariableInput
				value={s.token_input || ''}
				placeholder="{'{TOKEN}'}"
				class="flex-1 skeu-input text-[11px] font-mono"
				oninput={(e) => updateSettings('token_input', (e.target as HTMLInputElement).value)}
			/>
			<p class="text-[9px] text-muted-foreground mt-0.5">Raw JWT string to decode. Supports variable interpolation.</p>
		</div>

		<!-- Output variable (decode) -->
		<div>
			<label class={labelCls}>Output Variable</label>
			<VariableInput
				value={s.output_var || 'JWT'}
				placeholder="JWT"
				class="flex-1 skeu-input text-[11px] font-mono"
				oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)}
			/>
			<p class="text-[9px] text-muted-foreground mt-0.5">Claims extracted as <code>CLAIM_KEY</code> variables. Full decoded JSON stored here.</p>
		</div>

		<!-- Verify -->
		<label class="flex items-center gap-2 cursor-pointer">
			<input
				type="checkbox"
				checked={s.verify_on_decode ?? true}
				onchange={(e) => updateSettings('verify_on_decode', (e.target as HTMLInputElement).checked)}
				class="skeu-checkbox"
			/>
			<span class="text-[11px] text-foreground">Verify signature and expiry</span>
		</label>
		<p class="text-[9px] text-muted-foreground">If enabled, block fails on invalid signature or expired token.</p>
	{/if}
</div>
