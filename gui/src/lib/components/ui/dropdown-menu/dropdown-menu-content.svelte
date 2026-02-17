<script lang="ts">
	import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		sideOffset = 4,
		...restProps
	}: DropdownMenuPrimitive.ContentProps = $props();
</script>

<DropdownMenuPrimitive.Portal>
	<DropdownMenuPrimitive.Content
		bind:ref
		{sideOffset}
		data-slot="dropdown-menu-content"
		class={cn(
			"menu-content z-50 max-h-[--radix-dropdown-menu-content-available-height] min-w-[180px] overflow-y-auto overflow-x-hidden rounded p-[3px]",
			className
		)}
		{...restProps}
	/>
</DropdownMenuPrimitive.Portal>
