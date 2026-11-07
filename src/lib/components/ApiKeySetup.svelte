<script lang="ts">
  import KeyRound from '@lucide/svelte/icons/key-round';
  import ApiKeyForm from './ApiKeyForm.svelte';
</script>

<div class="setup-section">
  <div class="setup-wrapper">
    <div class="setup-icon">
      <KeyRound />
    </div>
    <h1 class="setup-title">Add your Anthropic API key</h1>
    <p class="setup-text">
      LynString translates with Claude using your own API key. You only pay Anthropic for what you
      translate.
    </p>
    <ApiKeyForm />
  </div>
</div>

<style>
  .setup-section {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100%;
    width: 100%;
    padding: 3rem 1.5rem;
    background: var(--background);
    overflow-y: auto;
  }

  .setup-wrapper {
    width: 100%;
    max-width: 420px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1.25rem;
  }

  .setup-icon {
    color: var(--primary);
    opacity: 0.9;
  }

  .setup-icon :global(svg) {
    width: 3rem;
    height: 3rem;
  }

  .setup-title {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--foreground);
    margin: 0;
  }

  .setup-text {
    font-size: 0.95rem;
    line-height: 1.5;
    color: var(--muted-foreground);
    margin: 0;
  }
</style>
