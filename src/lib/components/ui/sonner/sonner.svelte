<script lang="ts">
	import { Toaster as Sonner, type ToasterProps as SonnerProps } from "svelte-sonner";

	let { ...restProps }: SonnerProps = $props();
</script>

<Sonner
	class="toaster group"
	position="bottom-left"
	offset={{ bottom: "2rem", left: "1rem" }}
	theme="system"
	{...restProps}
/>
