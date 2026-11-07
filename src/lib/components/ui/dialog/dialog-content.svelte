<!-- Dialog Content -->
<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import { scale } from "svelte/transition";
  import { XIcon } from "@lucide/svelte";

  type $$Props = DialogPrimitive.ContentProps & {
    showClose?: boolean;
  };

  let className: $$Props["class"] | null = null;
  export { className as class };
  export let showClose = true;
</script>

<DialogPrimitive.Portal>
  <div transition:scale={{ duration: 100 }}>
    <DialogPrimitive.Overlay
      class="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm"
    />
  </div>
  <div transition:scale>
    <DialogPrimitive.Content
      class={cn(
        "fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border border-border/50 bg-background/95 p-6 shadow-lg backdrop-blur-sm duration-150 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 sm:rounded-lg",
        className
      )}
      tabindex={0}
      {...$$restProps}
  >
    <slot />
    {#if showClose}
      <DialogPrimitive.Close
        class="absolute right-4 top-4 rounded-sm opacity-70 hover:opacity-100 focus:outline-none disabled:pointer-events-none"
      >
        <XIcon class="h-5 w-5" />
        <span class="sr-only">Close</span>
      </DialogPrimitive.Close>
    {/if}
    </DialogPrimitive.Content>
  </div>
</DialogPrimitive.Portal>