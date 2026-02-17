<script lang="ts">
	import { send } from "$lib/ipc";
	import { app } from "$lib/state.svelte";
	import Workflow from '@lucide/svelte/icons/workflow';
	import Minus from '@lucide/svelte/icons/minus';
	import Square from '@lucide/svelte/icons/square';
	import X from '@lucide/svelte/icons/x';

	function onDrag(e: MouseEvent) {
		if (!(e.target as HTMLElement).closest('.chrome-btns')) {
			send('drag');
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed top-0 left-0 right-0 flex items-center bg-surface h-7 select-none panel-raised z-[9999]"
	onmousedown={onDrag}
>
	<span class="flex-1 px-2 text-[11px] text-muted-foreground tracking-wide flex items-center gap-1.5">
		<Workflow size={12} class="text-muted-foreground" />
		reqflow
		<span class="text-muted-foreground/50 text-[10px]">0.1.0</span>
	</span>
	<div class="chrome-btns flex h-full">
		<button
			class="w-[38px] h-full flex items-center justify-center text-muted-foreground hover:bg-[#3e3e3e] hover:text-foreground transition-colors cursor-pointer"
			onclick={() => send('minimize')}
			title="Minimize"
		>
			<Minus size={12} />
		</button>
		<button
			class="w-[38px] h-full flex items-center justify-center text-muted-foreground hover:bg-[#3e3e3e] hover:text-foreground transition-colors cursor-pointer"
			onclick={() => send('maximize')}
			title="Maximize"
		>
			<Square size={10} />
		</button>
		<button
			class="w-[38px] h-full flex items-center justify-center text-muted-foreground hover:bg-red hover:text-white transition-colors cursor-pointer"
			onclick={() => send('close')}
			title="Close"
		>
			<X size={12} />
		</button>
	</div>
</div>
