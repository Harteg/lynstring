<!-- Dialog Root -->
<script lang="ts">
  import { Dialog as DialogPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";
  import { fade } from "svelte/transition";

  type $$Props = DialogPrimitive.RootProps;

  export let open = false;
</script>

<DialogPrimitive.Root bind:open {...$$restProps}>
  <slot />
</DialogPrimitive.Root>
