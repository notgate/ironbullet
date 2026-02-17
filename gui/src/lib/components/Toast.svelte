<script lang="ts">
	import { getToasts, dismiss, type ToastType } from '$lib/toast.svelte';
	import CheckCircle from '@lucide/svelte/icons/check-circle';
	import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
	import AlertCircle from '@lucide/svelte/icons/alert-circle';
	import Info from '@lucide/svelte/icons/info';
	import X from '@lucide/svelte/icons/x';

	let toasts = $derived(getToasts());

	const iconSize = 14;
	const typeColors: Record<ToastType, string> = {
		success: 'var(--green)',
		error: 'var(--red)',
		warning: 'var(--orange)',
		info: 'var(--blue)',
	};
</script>

<div class="fixed bottom-3 right-3 z-[9999] flex flex-col gap-1.5 pointer-events-none" style="max-width: 340px;">
	{#each toasts as t (t.id)}
		<div
			class="toast-item pointer-events-auto flex items-start gap-2 px-3 py-2 rounded border text-[11px] {t.leaving ? 'toast-leave' : 'toast-enter'}"
			style="border-left: 3px solid {typeColors[t.type]};"
		>
			<span class="shrink-0 mt-px" style="color: {typeColors[t.type]};">
				{#if t.type === 'success'}<CheckCircle size={iconSize} />
				{:else if t.type === 'error'}<AlertCircle size={iconSize} />
				{:else if t.type === 'warning'}<AlertTriangle size={iconSize} />
				{:else}<Info size={iconSize} />
				{/if}
			</span>
			<span class="flex-1 text-foreground leading-snug">{t.message}</span>
			<button
				class="shrink-0 text-muted-foreground hover:text-foreground p-0.5"
				onclick={() => dismiss(t.id)}
			>
				<X size={10} />
			</button>
		</div>
	{/each}
</div>

<style>
	.toast-item {
		background: linear-gradient(to bottom, #2e2e33, #28282c);
		border-color: var(--border);
		box-shadow: 0 4px 16px rgba(0,0,0,0.5), 0 1px 3px rgba(0,0,0,0.3);
	}

	.toast-enter {
		animation: toastSlideIn 0.25s ease-out;
	}

	.toast-leave {
		animation: toastSlideOut 0.25s ease-in forwards;
	}

	@keyframes toastSlideIn {
		from { transform: translateX(100%); opacity: 0; }
		to { transform: translateX(0); opacity: 1; }
	}

	@keyframes toastSlideOut {
		from { transform: translateX(0); opacity: 1; }
		to { transform: translateX(100%); opacity: 0; }
	}
</style>
