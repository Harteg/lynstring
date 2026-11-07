<script lang="ts">
	import { cn } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	interface Props extends HTMLAttributes<HTMLDivElement> {
		value?: number;
		max?: number;
		class?: string;
		/** 'muted' = grayscale track/fill (e.g. token usage). Red override still applied via progress-over-limit class. */
		variant?: 'default' | 'muted';
	}

	let {
		value = 0,
		max = 100,
		class: className,
		variant = 'default',
		...restProps
	}: Props = $props();

	const percentage = $derived(Math.min(Math.max((value / max) * 100, 0), 100));
	const trackClass = variant === 'muted' ? 'bg-neutral-200 dark:bg-neutral-700' : 'bg-primary/20';
	const indicatorClass = variant === 'muted' ? 'bg-neutral-500 dark:bg-neutral-400' : 'bg-primary';
</script>

<div
	role="progressbar"
	aria-valuemin={0}
	aria-valuemax={max}
	aria-valuenow={value}
	class={cn("relative h-2 w-full overflow-hidden rounded-full", trackClass, className)}
	{...restProps}
>
	<div
		class={cn("h-full w-full flex-1 transition-all", indicatorClass)}
		style="transform: translateX(-{100 - percentage}%)"
	></div>
</div>
