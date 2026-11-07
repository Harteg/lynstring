<script lang="ts">
  import { translationService, translationStore } from '$lib/services/translation';
  import { currentProject } from '$lib/stores/project';
  import AlertCircle from '@lucide/svelte/icons/alert-circle';
  import X from '@lucide/svelte/icons/x';

  const translationState = $derived($translationStore);

  const showError = $derived(
    translationState.error &&
    translationState.error !== 'Translation cancelled' &&
    $currentProject &&
    translationState.currentProjectPath === $currentProject.path
  );

  const displayMessage = $derived.by(() => {
    const err = translationState.error;
    if (!err) return '';
    // Show as-is for API key / rate limit messages (no "Translation failed:" prefix)
    if (
      err.startsWith('No Anthropic API key') ||
      err.startsWith('Invalid Anthropic API key') ||
      err.startsWith('Rate limited')
    ) {
      return err;
    }
    return `Translation failed: ${err}`;
  });

  function handleDismiss() {
    translationService.dismissError();
  }
</script>

{#if showError && displayMessage}
  <div class="floating-error-container" aria-live="assertive" role="alert">
    <div class="error-card">
      <div class="error-icon">
        <AlertCircle class="w-4 h-4" />
      </div>
      <p class="error-message">{displayMessage}</p>
      <button type="button" class="dismiss-btn" onclick={handleDismiss} aria-label="Dismiss">
        <X size={16} />
      </button>
    </div>
  </div>
{/if}

<style>
  .floating-error-container {
    position: fixed;
    bottom: calc(var(--footer-height) + 1rem);
    right: 1.5rem;
    z-index: 50;
    max-width: 400px;
    min-width: 300px;
    animation: slideUpFade 0.35s ease-out;
  }

  @media (prefers-reduced-motion: reduce) {
    .floating-error-container {
      animation: none;
    }
  }

  @keyframes slideUpFade {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .error-card {
    position: relative;
    padding: 1rem 2.5rem 1rem 1.25rem;
    border-radius: 12px;
    border: 1px solid color-mix(in oklch, var(--destructive) 40%, transparent);
    background: color-mix(in oklch, var(--destructive) 12%, var(--card));
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
  }

  .error-icon {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    background: color-mix(in oklch, var(--destructive) 20%, transparent);
    color: var(--destructive);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .error-message {
    flex: 1;
    margin: 0;
    font-size: 0.875rem;
    line-height: 1.4;
    color: var(--foreground);
  }

  .dismiss-btn {
    position: absolute;
    top: 0.625rem;
    right: 0.625rem;
    padding: 0.25rem;
    border: none;
    background: transparent;
    color: var(--muted-foreground);
    cursor: pointer;
    border-radius: 6px;
    opacity: 0.7;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dismiss-btn:hover,
  .dismiss-btn:focus-visible {
    opacity: 1;
    background: color-mix(in oklch, var(--destructive) 15%, transparent);
    color: var(--foreground);
  }

  .dismiss-btn:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: 2px;
  }
</style>
