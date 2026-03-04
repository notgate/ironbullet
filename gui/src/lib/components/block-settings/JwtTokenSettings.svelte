<script lang="ts">
	import SkeuSelect from '$lib/components/SkeuSelect.svelte';
	import type { Block } from '$lib/types/block';
	import type { JwtTokenSettings } from '$lib/types/block-settings';

	let { block = $bindable() }: { block: Block & { settings: { type: 'JwtToken' } & JwtTokenSettings } } = $props();

	const labelCls = 'text-[10px] uppercase tracking-wider text-muted-foreground block mb-0.5';

	function update(key: keyof JwtTokenSettings, value: unknown) {
		(block.settings as any)[key] = value;
	}
</script>

<div class="space-y-2 p-2 text-xs">
	<!-- Action -->
	<div>
		<label class={labelCls}>Mode</label>
		<SkeuSelect
			value={block.settings.action || 'Sign'}
			onValueChange={(v) => update('action', v)}
			options={[
				{ value: 'Sign', label: 'Sign — generate a signed JWT from claims' },
				{ value: 'Decode', label: 'Decode — verify and extract claims from a JWT' },
			]}
		/>
	</div>

	<!-- Secret -->
	<div>
		<label class={labelCls}>Secret</label>
		<input
			type="text"
			value={block.settings.secret || ''}
			oninput={(e) => update('secret', (e.target as HTMLInputElement).value)}
			placeholder="HMAC secret (supports {VAR} interpolation)"
			class="skeu-input w-full font-mono text-[11px]"
		/>
	</div>

	<!-- Algorithm -->
	<div>
		<label class={labelCls}>Algorithm</label>
		<SkeuSelect
			value={block.settings.algorithm || 'HS256'}
			onValueChange={(v) => update('algorithm', v)}
			options={[
				{ value: 'HS256', label: 'HS256' },
				{ value: 'HS384', label: 'HS384' },
				{ value: 'HS512', label: 'HS512' },
			]}
		/>
	</div>

	{#if (block.settings.action || 'Sign') === 'Sign'}
		<!-- Claims -->
		<div>
			<label class={labelCls}>Claims (JSON)</label>
			<textarea
				value={block.settings.claims || ''}
				oninput={(e) => update('claims', (e.target as HTMLTextAreaElement).value)}
				placeholder={'{"sub": "{USER}", "role": "admin"}'}
				rows={4}
				class="skeu-input w-full font-mono text-[11px] resize-y"
			></textarea>
			<p class="text-[9px] text-muted-foreground mt-0.5">Variable interpolation supported in values. <code>iat</code> injected automatically.</p>
		</div>

		<!-- Expires In -->
		<div>
			<label class={labelCls}>Expires In (seconds)</label>
			<input
				type="number"
				min="0"
				value={block.settings.expires_in_secs ?? 0}
				oninput={(e) => update('expires_in_secs', parseInt((e.target as HTMLInputElement).value) || 0)}
				placeholder="0 = no expiry"
				class="skeu-input w-full text-[11px]"
			/>
			<p class="text-[9px] text-muted-foreground mt-0.5">0 = no <code>exp</code> claim added.</p>
		</div>

		<!-- Output variable -->
		<div>
			<label class={labelCls}>Output Variable</label>
			<input
				type="text"
				value={block.settings.output_var || 'JWT'}
				oninput={(e) => update('output_var', (e.target as HTMLInputElement).value)}
				placeholder="JWT"
				class="skeu-input w-full font-mono text-[11px]"
			/>
			<p class="text-[9px] text-muted-foreground mt-0.5">Signed token stored as <code>{'{'}OUTPUT_VAR{'}'}</code>.</p>
		</div>
	{:else}
		<!-- Token input -->
		<div>
			<label class={labelCls}>Token Input</label>
			<input
				type="text"
				value={block.settings.token_input || ''}
				oninput={(e) => update('token_input', (e.target as HTMLInputElement).value)}
				placeholder="{TOKEN}"
				class="skeu-input w-full font-mono text-[11px]"
			/>
			<p class="text-[9px] text-muted-foreground mt-0.5">Raw JWT string to decode. Supports variable interpolation.</p>
		</div>

		<!-- Verify -->
		<div class="flex items-center gap-2">
			<input
				type="checkbox"
				id="jwt-verify"
				checked={block.settings.verify_on_decode ?? true}
				onchange={(e) => update('verify_on_decode', (e.target as HTMLInputElement).checked)}
				class="skeu-checkbox"
			/>
			<label for="jwt-verify" class="text-[11px] text-foreground cursor-pointer">Verify signature and expiry</label>
		</div>
		<p class="text-[9px] text-muted-foreground">If enabled, block fails on invalid signature or expired token. Claims extracted as <code>CLAIM_&lt;KEY&gt;</code> variables.</p>
	{/if}
</div>
