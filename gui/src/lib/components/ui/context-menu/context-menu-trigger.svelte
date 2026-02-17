<script lang="ts">
	import { ContextMenu as ContextMenuPrimitive } from "bits-ui";
	import { cn, type WithoutChild } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: WithoutChild<ContextMenuPrimitive.TriggerProps> = $props();
</script>

<ContextMenuPrimitive.Trigger
	bind:ref
	data-slot="context-menu-trigger"
	class={cn(className)}
	{...restProps}
/>
