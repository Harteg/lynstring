<!-- Dialog Description -->
<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";

  type $$Props = DialogPrimitive.DescriptionProps;

  let className: $$Props["class"] | null = null;
  export { className as class };
</script>

<DialogPrimitive.Description
  class={cn("text-sm text-muted-foreground select-none", className)}
  {...$$restProps}
>
  <slot />
</DialogPrimitive.Description>