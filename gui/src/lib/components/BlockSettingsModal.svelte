<script lang="ts">
	import { app, getEditingBlock, pushUndo } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import type { Block } from '$lib/types';
	import * as Dialog from '$lib/components/ui/dialog';
	import { BLOCK_DOCS, hasVars } from './block-settings/shared';
	import HttpRequestSettings from './block-settings/HttpRequestSettings.svelte';
	import ParseSettings from './block-settings/ParseSettings.svelte';
	import KeyCheckSettings from './block-settings/KeyCheckSettings.svelte';
	import FunctionSettings from './block-settings/FunctionSettings.svelte';
	import ControlSettings from './block-settings/ControlSettings.svelte';
	import NetworkSettings from './block-settings/NetworkSettings.svelte';
	import BypassSettings from './block-settings/BypassSettings.svelte';
	import BrowserSettings from './block-settings/BrowserSettings.svelte';
	import AdvancedSettings from './block-settings/AdvancedSettings.svelte';

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

	const HTTP_TYPES = ['HttpRequest'];
	const PARSE_TYPES = ['ParseLR', 'ParseJSON', 'ParseRegex', 'ParseCSS', 'ParseXPath', 'ParseCookie', 'LambdaParser'];
	const KEYCHECK_TYPES = ['KeyCheck'];
	const FUNCTION_TYPES = ['StringFunction', 'ListFunction', 'CryptoFunction', 'ConversionFunction', 'ByteArray', 'Constants', 'Dictionary', 'FloatFunction', 'IntegerFunction', 'TimeFunction', 'GenerateGUID', 'PhoneCountry'];
	const CONTROL_TYPES = ['IfElse', 'Loop', 'Delay', 'Script', 'Log', 'SetVariable', 'ClearCookies'];
	const NETWORK_TYPES = ['Webhook', 'WebSocket', 'TcpRequest', 'UdpRequest', 'FtpRequest', 'SshRequest', 'ImapRequest', 'SmtpRequest', 'PopRequest'];
	const BYPASS_TYPES = ['CaptchaSolver', 'CloudflareBypass', 'LaravelCsrf'];
	const BROWSER_TYPES = ['BrowserOpen', 'NavigateTo', 'ClickElement', 'TypeText', 'WaitForElement', 'GetElementText', 'Screenshot', 'ExecuteJs'];
	const ADVANCED_TYPES = ['DateFunction', 'CaseSwitch', 'CookieContainer', 'RandomUserAgent', 'OcrCaptcha', 'RecaptchaInvisible', 'XacfSensor', 'RandomData', 'DataDomeSensor', 'Plugin', 'Group', 'AkamaiV3Sensor'];
</script>

{#snippet embedBadge(val: string | undefined)}
	{#if hasVars(val)}
		<span class="absolute top-0.5 right-1 text-[8px] uppercase tracking-wider font-semibold text-primary/80 bg-primary/10 px-1 py-px rounded select-none pointer-events-none z-10">embed</span>
	{/if}
{/snippet}

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
				{#if HTTP_TYPES.includes(block.settings.type)}
					<HttpRequestSettings {block} {updateSettings} {embedBadge} />
				{:else if PARSE_TYPES.includes(block.settings.type)}
					<ParseSettings {block} {updateSettings} {embedBadge} />
				{:else if KEYCHECK_TYPES.includes(block.settings.type)}
					<KeyCheckSettings {block} {updateSettings} {embedBadge} />
				{:else if FUNCTION_TYPES.includes(block.settings.type)}
					<FunctionSettings {block} {updateSettings} />
				{:else if CONTROL_TYPES.includes(block.settings.type)}
					<ControlSettings {block} {updateSettings} {embedBadge} />
				{:else if NETWORK_TYPES.includes(block.settings.type)}
					<NetworkSettings {block} {updateSettings} {embedBadge} />
				{:else if BYPASS_TYPES.includes(block.settings.type)}
					<BypassSettings {block} {updateSettings} {embedBadge} />
				{:else if BROWSER_TYPES.includes(block.settings.type)}
					<BrowserSettings {block} {updateSettings} {embedBadge} />
				{:else if ADVANCED_TYPES.includes(block.settings.type)}
					<AdvancedSettings {block} {updateSettings} {embedBadge} />
				{:else}
					<div class="text-[11px] text-muted-foreground">
						Settings editor for {block.block_type} blocks
					</div>
				{/if}
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
