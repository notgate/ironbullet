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
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
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
		{#if block.settings.use_tls}
		<label class="flex items-center gap-2 text-[11px] text-muted-foreground ml-4">
			<input type="checkbox" checked={block.settings.ssl_verify !== false}
				onchange={() => updateSettings('ssl_verify', block.settings.ssl_verify === false ? true : false)}
				class="skeu-checkbox" /> Verify TLS certificate
			{#if block.settings.ssl_verify === false}<span class="text-[9px] text-orange">⚠ insecure</span>{/if}
		</label>
		{/if}
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

<!-- ===================== FTP REQUEST ===================== -->
{:else if block.settings.type === 'FtpRequest'}
	<div class="space-y-1.5">
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Host</label><VariableInput value={block.settings.host} placeholder="ftp.example.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
			<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
		</div>
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Username</label><VariableInput value={block.settings.username} placeholder="<USER>" class={inputCls} oninput={(e) => updateSettings('username', (e.target as HTMLInputElement).value)} /></div>
			<div class="flex-1"><label class={labelCls}>Password</label><VariableInput value={block.settings.password} placeholder="<PASS>" class={inputCls} oninput={(e) => updateSettings('password', (e.target as HTMLInputElement).value)} /></div>
		</div>
		<div>
			<label class={labelCls}>Action</label>
			<SkeuSelect value={block.settings.command || 'LIST'}
				onValueChange={(v) => updateSettings('command', v)}
				options={[
					{value:'LIST',label:'LIST — List directory'},
					{value:'RETR',label:'RETR — Download file'},
					{value:'STOR',label:'STOR — Upload file'},
					{value:'DELE',label:'DELE — Delete file'},
					{value:'MKD',label:'MKD — Create directory'},
					{value:'RMD',label:'RMD — Remove directory'},
					{value:'PWD',label:'PWD — Print working directory'},
					{value:'CWD',label:'CWD — Change directory'},
				]}
				class="w-full mt-0.5"
			/>
		</div>
		{#if ['RETR','STOR','DELE','MKD','RMD','CWD'].includes(block.settings.command || '')}
			<div class="relative">
				<label class={labelCls}>Remote Path</label>
				<VariableInput value={block.settings.remote_path || ''} placeholder="/path/to/remote/file" class={inputCls}
					oninput={(e) => updateSettings('remote_path', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.remote_path)}
			</div>
		{/if}
		{#if block.settings.command === 'RETR'}
			<div class="relative">
				<label class={labelCls}>Save To Directory</label>
				<VariableInput value={block.settings.output_dir || ''} placeholder="./downloads" class={inputCls}
					oninput={(e) => updateSettings('output_dir', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.output_dir)}
				<p class="text-[9px] text-muted-foreground mt-0.5">Directory where the downloaded file will be saved. Filename is taken from remote path.</p>
			</div>
		{/if}
		{#if block.settings.command === 'STOR'}
			<div class="relative">
				<label class={labelCls}>Local File to Upload</label>
				<VariableInput value={block.settings.local_path || ''} placeholder="/path/to/local/file.txt" class={inputCls}
					oninput={(e) => updateSettings('local_path', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.local_path)}
			</div>
		{/if}
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Output var</label><VariableInput value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
		</div>
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>

<!-- ===================== SSH REQUEST ===================== -->
{:else if block.settings.type === 'SshRequest'}
	<div class="space-y-1.5">
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Host</label><VariableInput value={block.settings.host} placeholder="ssh.example.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
			<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
		</div>
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Username</label><VariableInput value={block.settings.username} placeholder="<USER>" class={inputCls} oninput={(e) => updateSettings('username', (e.target as HTMLInputElement).value)} /></div>
			<div class="flex-1"><label class={labelCls}>Password</label><VariableInput value={block.settings.password} placeholder="<PASS>" class={inputCls} oninput={(e) => updateSettings('password', (e.target as HTMLInputElement).value)} /></div>
		</div>
		<div>
			<label class={labelCls}>Action</label>
			<SkeuSelect value={block.settings.command || 'banner'}
				onValueChange={(v) => updateSettings('command', v)}
				options={[
					{value:'banner',label:'Banner Grab — Read SSH version string'},
					{value:'auth_check',label:'Auth Check — Attempt login handshake'},
					{value:'exec',label:'Execute Command — Banner exchange + command'},
				]}
				class="w-full mt-0.5"
			/>
		</div>
		{#if block.settings.command === 'exec'}
			<div class="relative">
				<label class={labelCls}>Shell Command</label>
				<VariableInput value={block.settings.ssh_cmd || ''} placeholder="whoami" class={inputCls}
					oninput={(e) => updateSettings('ssh_cmd', (e.target as HTMLInputElement).value)} />
				<p class="text-[9px] text-muted-foreground mt-0.5">Note: Full execution requires ssh2 crate. Banner exchange is performed.</p>
			</div>
		{/if}
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Output var</label><VariableInput value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
		</div>
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>

<!-- ===================== IMAP REQUEST ===================== -->
{:else if block.settings.type === 'ImapRequest'}
	<div class="space-y-1.5">
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Host</label><VariableInput value={block.settings.host} placeholder="imap.gmail.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
			<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
		</div>
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Username</label><VariableInput value={block.settings.username} placeholder="<USER>" class={inputCls} oninput={(e) => updateSettings('username', (e.target as HTMLInputElement).value)} /></div>
			<div class="flex-1"><label class={labelCls}>Password</label><VariableInput value={block.settings.password} placeholder="<PASS>" class={inputCls} oninput={(e) => updateSettings('password', (e.target as HTMLInputElement).value)} /></div>
		</div>
		<label class="flex items-center gap-2 text-[11px] text-foreground"><input type="checkbox" checked={block.settings.use_tls} onchange={() => updateSettings('use_tls', !block!.settings.use_tls)} class="skeu-checkbox" /> Use TLS</label>
		{#if block.settings.use_tls}
		<label class="flex items-center gap-2 text-[11px] text-muted-foreground ml-4">
			<input type="checkbox" checked={block.settings.ssl_verify !== false}
				onchange={() => updateSettings('ssl_verify', block.settings.ssl_verify === false ? true : false)}
				class="skeu-checkbox" /> Verify TLS certificate
			{#if block.settings.ssl_verify === false}<span class="text-[9px] text-orange">⚠ insecure</span>{/if}
		</label>
		{/if}
		<div>
			<label class={labelCls}>Action</label>
			<SkeuSelect value={block.settings.command || 'LOGIN'}
				onValueChange={(v) => updateSettings('command', v)}
				options={[
					{value:'LOGIN',label:'Login — Verify credentials'},
					{value:'SELECT',label:'SELECT — Open mailbox'},
					{value:'FETCH',label:'FETCH — Retrieve message'},
					{value:'LIST "" "*"',label:'LIST — List all mailboxes'},
					{value:'SEARCH ALL',label:'SEARCH — Search all messages'},
				]}
				class="w-full mt-0.5"
			/>
		</div>
		{#if ['SELECT','FETCH','SEARCH ALL'].includes(block.settings.command || '')}
			<div>
				<label class={labelCls}>Mailbox</label>
				<VariableInput value={block.settings.mailbox || 'INBOX'} placeholder="INBOX" class={inputCls}
					oninput={(e) => updateSettings('mailbox', (e.target as HTMLInputElement).value)} />
			</div>
		{/if}
		{#if block.settings.command === 'FETCH'}
			<div>
				<label class={labelCls}>Message #</label>
				<input type="number" value={block.settings.message_num || 1} min="1" class={inputCls}
					oninput={(e) => updateSettings('message_num', parseInt((e.target as HTMLInputElement).value))} />
			</div>
		{/if}
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Output var</label><VariableInput value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
		</div>
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>

<!-- ===================== SMTP REQUEST ===================== -->
{:else if block.settings.type === 'SmtpRequest'}
	<div class="space-y-1.5">
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Host</label><VariableInput value={block.settings.host} placeholder="smtp.gmail.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
			<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
		</div>
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Username</label><VariableInput value={block.settings.username} placeholder="<USER>" class={inputCls} oninput={(e) => updateSettings('username', (e.target as HTMLInputElement).value)} /></div>
			<div class="flex-1"><label class={labelCls}>Password</label><VariableInput value={block.settings.password} placeholder="<PASS>" class={inputCls} oninput={(e) => updateSettings('password', (e.target as HTMLInputElement).value)} /></div>
		</div>
		<label class="flex items-center gap-2 text-[11px] text-foreground"><input type="checkbox" checked={block.settings.use_tls} onchange={() => updateSettings('use_tls', !block!.settings.use_tls)} class="skeu-checkbox" /> Use TLS</label>
		{#if block.settings.use_tls}
		<label class="flex items-center gap-2 text-[11px] text-muted-foreground ml-4">
			<input type="checkbox" checked={block.settings.ssl_verify !== false}
				onchange={() => updateSettings('ssl_verify', block.settings.ssl_verify === false ? true : false)}
				class="skeu-checkbox" /> Verify TLS certificate
			{#if block.settings.ssl_verify === false}<span class="text-[9px] text-orange">⚠ insecure</span>{/if}
		</label>
		{/if}
		<div>
			<label class={labelCls}>Action</label>
			<SkeuSelect value={block.settings.action || 'VERIFY'}
				onValueChange={(v) => updateSettings('action', v)}
				options={[
					{value:'VERIFY',label:'Verify Credentials — Check login only'},
					{value:'SEND_EMAIL',label:'Send Email — Full mail delivery'},
				]}
				class="w-full mt-0.5"
			/>
		</div>
		{#if (block.settings.action || 'VERIFY') === 'SEND_EMAIL'}
			<div class="relative">
				<label class={labelCls}>From</label>
				<VariableInput value={block.settings.from || ''} placeholder="sender@example.com or leave blank for username" class={inputCls}
					oninput={(e) => updateSettings('from', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.from)}
			</div>
			<div class="relative">
				<label class={labelCls}>To (comma separated)</label>
				<VariableInput value={block.settings.to || ''} placeholder="recipient@example.com" class={inputCls}
					oninput={(e) => updateSettings('to', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.to)}
			</div>
			<div class="relative">
				<label class={labelCls}>Subject</label>
				<VariableInput value={block.settings.subject || ''} placeholder="Email subject" class={inputCls}
					oninput={(e) => updateSettings('subject', (e.target as HTMLInputElement).value)} />
				{@render embedBadge(block.settings.subject)}
			</div>
			<div class="relative">
				<label class={labelCls}>Body</label>
				<textarea value={block.settings.body_template || ''} placeholder="Email body content..."
					class="w-full skeu-input text-[11px] font-mono min-h-[80px] resize-y mt-0.5"
					oninput={(e) => updateSettings('body_template', (e.target as HTMLTextAreaElement).value)}></textarea>
				{@render embedBadge(block.settings.body_template)}
			</div>
		{/if}
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Output var</label><VariableInput value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
		</div>
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>

<!-- ===================== POP REQUEST ===================== -->
{:else if block.settings.type === 'PopRequest'}
	<div class="space-y-1.5">
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Host</label><VariableInput value={block.settings.host} placeholder="pop.gmail.com" class={inputCls} oninput={(e) => updateSettings('host', (e.target as HTMLInputElement).value)} /></div>
			<div class="w-20"><label class={labelCls}>Port</label><input type="number" value={block.settings.port} class={inputCls} oninput={(e) => updateSettings('port', parseInt((e.target as HTMLInputElement).value))} /></div>
		</div>
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Username</label><VariableInput value={block.settings.username} placeholder="<USER>" class={inputCls} oninput={(e) => updateSettings('username', (e.target as HTMLInputElement).value)} /></div>
			<div class="flex-1"><label class={labelCls}>Password</label><VariableInput value={block.settings.password} placeholder="<PASS>" class={inputCls} oninput={(e) => updateSettings('password', (e.target as HTMLInputElement).value)} /></div>
		</div>
		<label class="flex items-center gap-2 text-[11px] text-foreground"><input type="checkbox" checked={block.settings.use_tls} onchange={() => updateSettings('use_tls', !block!.settings.use_tls)} class="skeu-checkbox" /> Use TLS</label>
		{#if block.settings.use_tls}
		<label class="flex items-center gap-2 text-[11px] text-muted-foreground ml-4">
			<input type="checkbox" checked={block.settings.ssl_verify !== false}
				onchange={() => updateSettings('ssl_verify', block.settings.ssl_verify === false ? true : false)}
				class="skeu-checkbox" /> Verify TLS certificate
			{#if block.settings.ssl_verify === false}<span class="text-[9px] text-orange">⚠ insecure</span>{/if}
		</label>
		{/if}
		<div>
			<label class={labelCls}>Action</label>
			<SkeuSelect value={block.settings.command || 'STAT'}
				onValueChange={(v) => updateSettings('command', v)}
				options={[
					{value:'STAT',label:'STAT — Mailbox statistics'},
					{value:'LIST',label:'LIST — List messages'},
					{value:'RETR',label:'RETR — Retrieve message'},
					{value:'DELE',label:'DELE — Delete message'},
					{value:'NOOP',label:'NOOP — Keep-alive ping'},
					{value:'RSET',label:'RSET — Reset deletions'},
				]}
				class="w-full mt-0.5"
			/>
		</div>
		{#if ['RETR','DELE'].includes(block.settings.command || '')}
			<div>
				<label class={labelCls}>Message #</label>
				<input type="number" value={block.settings.message_num || 1} min="1" class={inputCls}
					oninput={(e) => updateSettings('message_num', parseInt((e.target as HTMLInputElement).value))} />
			</div>
		{/if}
		<div class="flex gap-2">
			<div class="flex-1"><label class={labelCls}>Output var</label><VariableInput value={block.settings.output_var} class={inputCls} oninput={(e) => updateSettings('output_var', (e.target as HTMLInputElement).value)} /></div>
			<label class="flex items-center gap-1 text-xs text-foreground pt-4"><input type="checkbox" checked={block.settings.capture} onchange={() => updateSettings('capture', !block!.settings.capture)} class="skeu-checkbox" /> CAP</label>
		</div>
		<div class="flex items-center gap-2"><span class="text-[11px] text-muted-foreground">Timeout:</span><input type="number" value={block.settings.timeout_ms} class="w-20 skeu-input text-[11px]" oninput={(e) => updateSettings('timeout_ms', parseInt((e.target as HTMLInputElement).value))} /><span class="text-[10px] text-muted-foreground">ms</span></div>
	</div>
{/if}
