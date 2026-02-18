<script lang="ts">
	import type { Block } from '$lib/types';
	import SkeuSelect from '../SkeuSelect.svelte';
	import VariableInput from '../VariableInput.svelte';
	import { inputCls, labelCls, hasVars } from './shared';

	let { block, updateSettings, embedBadge }: {
		block: Block;
		updateSettings: (key: string, value: unknown) => void;
		embedBadge: import('svelte').Snippet<[string | undefined]>;
	} = $props();
</script>

<!-- ===================== CAPTCHA SOLVER ===================== -->
{#if block.settings.type === 'CaptchaSolver'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Solver service</label>
			<SkeuSelect value={block.settings.solver_service}
				onValueChange={(v) => updateSettings('solver_service', v)}
				options={[{value:'capsolver',label:'CapSolver'},{value:'2captcha',label:'2Captcha'},{value:'anticaptcha',label:'Anti-Captcha'},{value:'capmonster',label:'CapMonster'},{value:'deathbycaptcha',label:'DeathByCaptcha'}]}
				class="w-full mt-0.5"
			/>
		</div>
		<div>
			<label class={labelCls}>Captcha type</label>
			<SkeuSelect value={block.settings.captcha_type}
				onValueChange={(v) => updateSettings('captcha_type', v)}
				options={[{value:'RecaptchaV2',label:'reCAPTCHA v2'},{value:'HCaptcha',label:'hCaptcha'},{value:'FunCaptcha',label:'FunCaptcha'},{value:'ImageCaptcha',label:'Image Captcha'},{value:'Turnstile',label:'CF Turnstile'}]}
				class="w-full mt-0.5"
			/>
		</div>
		<div>
			<label class={labelCls}>API key</label>
			<VariableInput value={block.settings.api_key} placeholder="Your solver API key" class={inputCls}
				oninput={(e) => updateSettings('api_key', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Site key</label>
			<VariableInput value={block.settings.site_key} placeholder="Target site captcha key" class={inputCls}
				oninput={(e) => updateSettings('site_key', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>Page URL</label>
			<VariableInput value={block.settings.page_url} placeholder="https://example.com/login" class={inputCls}
				oninput={(e) => updateSettings('page_url', (e.target as HTMLInputElement).value)} />
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
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>

<!-- ===================== CLOUDFLARE BYPASS ===================== -->
{:else if block.settings.type === 'CloudflareBypass'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Target URL</label>
			<VariableInput value={block.settings.url} placeholder="https://protected-site.com" class={inputCls}
				oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
		</div>
		<div>
			<label class={labelCls}>FlareSolverr endpoint</label>
			<VariableInput value={block.settings.flaresolverr_url} placeholder="http://localhost:8191/v1" class={inputCls}
				oninput={(e) => updateSettings('flaresolverr_url', (e.target as HTMLInputElement).value)} />
			<p class="text-[9px] text-muted-foreground mt-0.5">Local FlareSolverr instance URL</p>
		</div>
		<div class="flex gap-2">
			<div class="flex-1">
				<label class={labelCls}>Output var (cookies)</label>
				<VariableInput value={block.settings.output_var} class={inputCls}
					oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
			</div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4">
				<input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" />
				CAP
			</label>
		</div>
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.max_timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('max_timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>

<!-- ===================== LARAVEL CSRF ===================== -->
{:else if block.settings.type === 'LaravelCsrf'}
	<div class="space-y-1.5">
		<div>
			<label class={labelCls}>Page URL</label>
			<VariableInput value={block.settings.url} placeholder="https://example.com/login" class={inputCls}
				oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
			<p class="text-[9px] text-muted-foreground mt-0.5">URL to fetch CSRF token from</p>
		</div>
		<div>
			<label class={labelCls}>CSRF selector</label>
			<VariableInput value={block.settings.csrf_selector} placeholder={'input[name="_token"]'} class={inputCls}
				oninput={(e) => updateSettings('csrf_selector', (e.target as HTMLInputElement).value)} />
			<p class="text-[9px] text-muted-foreground mt-0.5">CSS selector for the hidden CSRF input</p>
		</div>
		<div>
			<label class={labelCls}>Cookie name</label>
			<VariableInput value={block.settings.cookie_name} placeholder="XSRF-TOKEN" class={inputCls}
				oninput={(e) => updateSettings('cookie_name', (e.target as HTMLInputElement).value)} />
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
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>
{/if}
