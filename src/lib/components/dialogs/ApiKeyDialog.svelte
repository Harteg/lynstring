<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import ApiKeyForm from '$lib/components/ApiKeyForm.svelte';

  let { open = $bindable(false) }: { open?: boolean } = $props();
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="dialog-standard sm:max-w-[460px]">
    <Dialog.Header class="dialog-standard-header !pb-3">
      <Dialog.Title class="dialog-standard-title">API key &amp; model</Dialog.Title>
      <Dialog.Description class="dialog-standard-description">
        LynString calls the Anthropic API directly with your key.
      </Dialog.Description>
    </Dialog.Header>
    <div class="px-6 pb-6">
      {#if open}
        <ApiKeyForm onSaved={() => (open = false)} />
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
