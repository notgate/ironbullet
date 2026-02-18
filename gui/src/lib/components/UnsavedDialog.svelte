<script lang="ts">
	import { app, closeTab, markTabSaved, continueAppClose, cancelAppClose } from '$lib/state.svelte';
	import { send } from '$lib/ipc';
	import * as Dialog from '$lib/components/ui/dialog';
	import AlertTriangle from '@lucide/svelte/icons/triangle-alert';

	let open = $derived(app.showUnsavedDialog);
	let pendingTab = $derived(app.configTabs.find(t => t.id === app.pendingCloseTabId));

	function onOpenChange(v: boolean) {
		if (!v) {
			if (app.pendingAppClose) {
				cancelAppClose();
			} else {
				app.showUnsavedDialog = false;
				app.pendingCloseTabId = null;
			}
		}
	}

	function handleSave() {
		if (!app.pendingCloseTabId) return;
		// Save then close
		send('save_pipeline', {});
		markTabSaved();
		if (app.pendingAppClose) {
			// Mark as saved (no longer dirty), then continue to next unsaved tab
			const tab = app.configTabs.find(t => t.id === app.pendingCloseTabId);
			if (tab) tab.isDirty = false;
			app.showUnsavedDialog = false;
			app.pendingCloseTabId = null;
			continueAppClose();
		} else {
			closeTab(app.pendingCloseTabId);
		}
	}

	function handleDiscard() {
		if (!app.pendingCloseTabId) return;
		if (app.pendingAppClose) {
			// Mark as not dirty (discard changes), then continue to next unsaved tab
			const tab = app.configTabs.find(t => t.id === app.pendingCloseTabId);
			if (tab) tab.isDirty = false;
			app.showUnsavedDialog = false;
			app.pendingCloseTabId = null;
			continueAppClose();
		} else {
			closeTab(app.pendingCloseTabId);
		}
	}

	function handleCancel() {
		if (app.pendingAppClose) {
			cancelAppClose();
		} else {
			app.showUnsavedDialog = false;
			app.pendingCloseTabId = null;
		}
	}
</script>

<Dialog.Root {open} {onOpenChange}>
	<Dialog.Content class="sm:max-w-[380px] p-0 gap-0 overflow-hidden" showCloseButton={false}>
		<div class="p-5">
			<div class="flex items-start gap-3">
				<div class="p-2 rounded-md bg-yellow/10 text-yellow shrink-0">
					<AlertTriangle size={18} />
				</div>
				<div>
					<Dialog.Title class="text-sm font-medium text-foreground">Unsaved Changes</Dialog.Title>
					<Dialog.Description class="text-[11px] text-muted-foreground mt-1">
						{#if pendingTab}
							<strong>{pendingTab.name}</strong> has unsaved changes.
						{:else}
							This config has unsaved changes.
						{/if}
						Do you want to save before closing?
					</Dialog.Description>
				</div>
			</div>
		</div>
		<div class="flex items-center gap-2 px-4 py-3 bg-background/50 border-t border-border justify-end">
			<button class="skeu-btn text-[10px] text-muted-foreground" onclick={handleCancel}>Cancel</button>
			<button class="skeu-btn text-[10px] text-red" onclick={handleDiscard}>Don't Save</button>
			<button class="skeu-btn text-[10px] text-green" onclick={handleSave}>Save</button>
		</div>
	</Dialog.Content>
</Dialog.Root>
