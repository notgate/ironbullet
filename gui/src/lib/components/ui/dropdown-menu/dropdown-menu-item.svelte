<script lang="ts">
	import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui";
	import { cn, type WithoutChild } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		disabled = false,
		children,
		...restProps
	}: WithoutChild<DropdownMenuPrimitive.ItemProps> & { disabled?: boolean } = $props();
</script>

<DropdownMenuPrimitive.Item
	bind:ref
	data-slot="dropdown-menu-item"
	{disabled}
	class={cn(
		"menu-item relative flex cursor-default items-center gap-2 rounded-sm px-3 py-[4px] text-[11px] text-foreground outline-hidden select-none transition-colors data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-40",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</DropdownMenuPrimitive.Item>
