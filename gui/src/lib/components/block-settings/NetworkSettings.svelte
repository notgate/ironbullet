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

<!-- ===================== WEBHOOK ===================== -->
{#if block.settings.type === 'Webhook'}
	<div class="space-y-1.5">
		<div class="flex gap-1.5">
			<SkeuSelect
				value={block.settings.method}
				onValueChange={(v) => updateSettings('method', v)}
				options={[{value:'POST',label:'POST'},{value:'GET',label:'GET'},{value:'PUT',label:'PUT'},{value:'PATCH',label:'PATCH'}]}
				class="text-[11px] w-20"
			/>
			<div class="relative flex-1">
				<VariableInput value={block.settings.url} placeholder="https://discord.com/api/webhooks/..."
					class="w-full skeu-input text-[11px] font-mono"
					oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.url)}
			</div>
		</div>
		<div>
			<label class={labelCls}>Content-Type</label>
			<VariableInput value={block.settings.content_type} class={inputCls}
				oninput={(e) => updateSettings('content_type', (e.target as HTMLInputElement).value)} />
		</div>
		<div class="relative">
			<label class={labelCls}>Body template</label>
			<textarea value={block.settings.body_template} placeholder={'{"content": "Hit: <USER>:<PASS>"}'}
				class="w-full skeu-input text-[11px] font-mono min-h-[80px] resize-y mt-0.5"
				oninput={(e) => updateSettings('body_template', (e.target as HTMLTextAreaElement).value)}></textarea>
			{@render embedBadge(block.settings.body_template)}
			<p class="text-[9px] text-muted-foreground mt-0.5">Use <code class="text-foreground/70">&lt;VAR&gt;</code> for variable interpolation</p>
		</div>
		<div class="relative">
			<label class={labelCls}>Custom cookies</label>
			<textarea value={block.settings.custom_cookies || ''} placeholder="session_id=abc123&#10;auth=<TOKEN>"
				class="w-full skeu-input text-[11px] font-mono min-h-[50px] resize-y mt-0.5"
				oninput={(e) => updateSettings('custom_cookies', (e.target as HTMLTextAreaElement).value)}></textarea>
			{@render embedBadge(block.settings.custom_cookies)}
			<p class="text-[9px] text-muted-foreground mt-0.5">One per line: <code class="text-foreground/70">name=value</code>. Sent as <code class="text-foreground/70">Cookie</code> header.</p>
		</div>
	</div>

<!-- ===================== WEBSOCKET ===================== -->
{:else if block.settings.type === 'WebSocket'}
	<div class="space-y-1.5">
		<div class="relative">
			<label class={labelCls}>URL</label>
			<VariableInput value={block.settings.url} placeholder="wss://example.com/ws" class={inputCls}
				oninput={(e) => updateSettings('url', (e.target as HTMLInputElement).value)} />
			{@render embedBadge(block.settings.url)}
		</div>
		<div>
			<label class={labelCls}>Action</label>
			<SkeuSelect value={block.settings.action}
				onValueChange={(v) => updateSettings('action', v)}
				options={[{value:'connect',label:'Connect'},{value:'send',label:'Send Message'},{value:'receive',label:'Receive'},{value:'close',label:'Close'}]}
				class="w-full mt-0.5"
			/>
		</div>
		{#if block.settings.action === 'send'}
			<div class="relative">
				<label class={labelCls}>Message</label>
				<textarea value={block.settings.message} placeholder="Message payload..."
					class="w-full skeu-input text-[11px] font-mono min-h-[60px] resize-y mt-0.5"
					oninput={(e) => updateSettings('message', (e.target as HTMLTextAreaElement).value)}></textarea>
				{@render embedBadge(block.settings.message)}
			</div>
		{/if}
		{#if block.settings.action === 'receive'}
			<div class="flex gap-2">
				<div class="flex-1">
					<label class={labelCls}>Output var</label>
					<VariableInput value={block.settings.output_var} class={inputCls}
						oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} />
				</div>
			</div>
		{/if}
		<div class="flex items-center gap-2">
			<span class="text-[11px] text-muted-foreground">Timeout:</span>
			<input type="number" value={block.settings.timeout_ms}
				class="w-20 skeu-input text-[11px]"
				oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} />
			<span class="text-[10px] text-muted-foreground">ms</span>
		</div>
	</div>

<!-- ===================== TCP REQUEST ===================== -->
{:else if block.settings.type === 'TcpRequest'}
	<div class="space-y-1.5">
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Host</label><VariableInput value={block.settings.host} placeholder="example.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
			<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
		</div>
		<div><label class={labelCls}>Data to send</label><textarea value={block.settings.data} placeholder="Raw data..." class="w-full skeu-input text-[11px] font-mono min-h-[60px] resize-y mt-0.5" oninput={(e) => updateSettings('data', (e.target as HTMLTextAreaElement).value)}></textarea></div>
		<label class="flex items-center gap-2 text-[11px] text-foreground"><input type="checkbox" checked={block.settings.use_tls} onchange={() => updateSettings('use_tls', !block!.settings.use_tls)} class="skeu-checkbox" /> Use TLS</label>
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Output var</label><VariableInput value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
		</div>
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>

<!-- ===================== UDP REQUEST ===================== -->
{:else if block.settings.type === 'UdpRequest'}
	<div class="space-y-1.5">
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Host</label><VariableInput value={block.settings.host} placeholder="example.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
			<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
		</div>
		<div><label class={labelCls}>Data to send</label><textarea value={block.settings.data} placeholder="Raw data..." class="w-full skeu-input text-[11px] font-mono min-h-[60px] resize-y mt-0.5" oninput={(e) => updateSettings('data', (e.target as HTMLTextAreaElement).value)}></textarea></div>
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Output var</label><VariableInput value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
		</div>
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>

<!-- ===================== FTP / SSH / IMAP / SMTP / POP ===================== -->
{:else if block.settings.type === 'FtpRequest' || block.settings.type === 'SshRequest' || block.settings.type === 'ImapRequest' || block.settings.type === 'SmtpRequest' || block.settings.type === 'PopRequest'}
	<div class="space-y-1.5">
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Host</label><VariableInput value={block.settings.host} placeholder="mail.example.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
			<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
		</div>
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Username</label><VariableInput value={block.settings.username} placeholder="input.USER" class={inputCls} oninput={(e) => updateSettings('username', (e.target as HTMLInputElement).value)} /></div>
			<div class="flex-1"><label class={labelCls}>Password</label><VariableInput value={block.settings.password} placeholder="input.PASS" class={inputCls} oninput={(e) => updateSettings('password', (e.target as HTMLInputElement).value)} /></div>
		</div>
		<div><label class={labelCls}>Command</label><VariableInput value={block.settings.command} class={inputCls} oninput={(e) => updateSettings('command', (e.target as HTMLInputElement).value)} /></div>
		{'use_tls' in block.settings ? '' : ''}
		{#if 'use_tls' in block.settings}
			<label class="flex items-center gap-2 text-[11px] text-foreground"><input type="checkbox" checked={block.settings.use_tls} onchange={() => updateSettings('use_tls', !(block!.settings as any).use_tls)} class="skeu-checkbox" /> Use TLS</label>
		{/if}
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Output var</label><VariableInput value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
		</div>
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>
{/if}
