<script lang="ts">
	import { app } from '$lib/state.svelte';
	import { dock, PANEL_LABELS } from '$lib/state/dock.svelte';
	import type { PanelId } from '$lib/state/dock.svelte';

	const ALL_PANEL_IDS: PanelId[] = ['debugger', 'code', 'data', 'jobs', 'network', 'variables', 'inspector'];

	const labelCls = 'text-[10px] font-medium text-muted-foreground uppercase tracking-wide';
</script>

<div class="space-y-5">
	<!-- Dialog confirmations -->
	<div>
		<p class={labelCls}>Confirmations</p>
		<div class="mt-2 space-y-2">
			<label class="flex items-center gap-2 text-[11px] text-foreground cursor-pointer select-none">
				<input
					type="checkbox"
					checked={app.uiPrefs.skipUnsavedDialog}
					onchange={() => app.setUiPref('skipUnsavedDialog', !app.uiPrefs.skipUnsavedDialog)}
					class="skeu-checkbox"
				/>
				<span>Skip "unsaved changes" dialog when closing tabs</span>
			</label>
			<p class="text-[9px] text-muted-foreground ml-5">Tabs with unsaved changes will close immediately without prompting to save.</p>

			<label class="flex items-center gap-2 text-[11px] text-foreground cursor-pointer select-none mt-2">
				<input
					type="checkbox"
					checked={app.uiPrefs.skipCloseConfirm}
					onchange={() => app.setUiPref('skipCloseConfirm', !app.uiPrefs.skipCloseConfirm)}
					class="skeu-checkbox"
				/>
				<span>Skip confirmation when closing the app</span>
			</label>
			<p class="text-[9px] text-muted-foreground ml-5">The app will close immediately even if tabs have unsaved changes.</p>
		</div>
	</div>

	<!-- Bottom panel tab visibility -->
	<div>
		<p class={labelCls}>Bottom Panel Tabs</p>
		<p class="text-[9px] text-muted-foreground mt-0.5 mb-2">Hide tabs you don't use. Hidden tabs can be restored here.</p>
		<div class="space-y-1">
			{#each ALL_PANEL_IDS as panelId}
				{@const hidden = dock.isHidden(panelId)}
				<label class="flex items-center justify-between gap-2 px-2 py-1.5 rounded border border-border hover:bg-accent/10 cursor-pointer select-none">
					<span class="text-[11px] text-foreground">{PANEL_LABELS[panelId]}</span>
					<button
						class="text-[9px] px-2 py-0.5 rounded border {hidden ? 'border-muted-foreground/30 text-muted-foreground' : 'border-primary/40 text-primary bg-primary/5'}"
						onclick={() => dock.toggleHidden(panelId)}
					>{hidden ? 'Hidden' : 'Visible'}</button>
				</label>
			{/each}
		</div>
	</div>

	<!-- Compact mode -->
	<div>
		<p class={labelCls}>Density</p>
		<div class="mt-2">
			<label class="flex items-center gap-2 text-[11px] text-foreground cursor-pointer select-none">
				<input
					type="checkbox"
					checked={app.uiPrefs.compactMode}
					onchange={() => app.setUiPref('compactMode', !app.uiPrefs.compactMode)}
					class="skeu-checkbox"
				/>
				<span>Compact mode — reduce spacing in block list and panels</span>
			</label>
		</div>
	</div>

	<!-- Intellisense -->
	<div>
		<p class={labelCls}>Intellisense</p>
		<div class="mt-2 space-y-1">
			<label class="flex items-center gap-2 text-[11px] text-foreground cursor-pointer select-none">
				<input
					type="checkbox"
					checked={app.uiPrefs.intellisenseEnabled}
					onchange={() => app.setUiPref('intellisenseEnabled', !app.uiPrefs.intellisenseEnabled)}
					class="skeu-checkbox"
				/>
				<span>Enable intellisense autocomplete in block input fields</span>
			</label>
			<p class="text-[9px] text-muted-foreground ml-5">
				Suggests variables, delimiter patterns, and next-word completions from live response data.
				Disable if you prefer plain inputs with no popup.
			</p>
		</div>
	</div>

	<!-- Reset -->
	<div>
		<p class={labelCls}>Reset</p>
		<div class="mt-2 flex gap-2">
			<button
				class="skeu-btn text-[10px]"
				onclick={() => dock.resetLayout()}
			>Reset panel layout</button>
		</div>
	</div>
</div>
