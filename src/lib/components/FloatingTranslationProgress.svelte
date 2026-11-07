<script lang="ts">
  import { translationService, translationStore } from '$lib/services/translation';
  import { formatDuration } from '$lib/utils/translation-history';
  import LanguagesIcon from '@lucide/svelte/icons/languages';
  import RefreshCcw from '@lucide/svelte/icons/refresh-ccw';

  const translationState = $derived($translationStore);

  // Live stopwatch ticker — updates every 100ms while translating
  let tick = $state(0);
  $effect(() => {
    if (!translationState.isTranslating || !translationState.pendingSession) return;
    const id = setInterval(() => {
      tick++;
    }, 100);
    return () => clearInterval(id);
  });

  // Depend on tick so this re-runs every 100ms and picks up new Date.now()
  const elapsedMs = $derived(
    translationState.isTranslating && translationState.pendingSession && tick >= 0
      ? Date.now() - translationState.pendingSession.timestamp
      : 0
  );
  const elapsedFormatted = $derived(formatDuration(elapsedMs));

  // Build list of active translation(s) - structured for future multi-process support
  const activeTranslations = $derived.by(() => {
    const items: Array<{
      id: string;
      projectPath: string;
      label: string;
      isPreparing: boolean;
      progress: { completed: number; total: number; percentage: number; batchInfo: string } | null;
    }> = [];

    if (!translationState.isTranslating && !translationState.isPreparing) {
      return items;
    }

    const projectPath = translationState.currentProjectPath || '';
    const label =
      translationState.currentProjectName ||
      projectPath.split(/[/\\]/).filter(Boolean).pop() ||
      'Translation';

    if (translationState.isPreparing) {
      items.push({
        id: 'preparing',
        projectPath,
        label,
        isPreparing: true,
        progress: null
      });
    } else if (translationState.isTranslating && translationState.progress) {
      const p = translationState.progress;
      const total = p.current_language_total_strings ?? 0;
      const completed = p.current_language_strings_done ?? 0;
      const percentage = total > 0 ? Math.min(100, (completed / total) * 100) : 0;
      // Batch count is global across entire job (works for both string batching and language batching)
      const batchInfo =
        (p.total_batches ?? 0) > 1
          ? `Batch ${p.current_batch ?? 0}/${p.total_batches ?? 0}`
          : '';

      items.push({
        id: translationState.currentSessionId || 'active',
        projectPath,
        label,
        isPreparing: false,
        progress: {
          completed,
          total,
          percentage,
          batchInfo
        }
      });
    }

    return items;
  });

  function handleCancel() {
    translationService.cancelTranslation();
  }
</script>

{#if activeTranslations.length > 0}
  <div class="floating-progress-container" aria-live="polite">
    {#each activeTranslations as item (item.id)}
      <div class="progress-card" class:has-progress={!!item.progress}>
        <div class="progress-header">
          <div class="progress-icon">
            {#if item.isPreparing}
              <RefreshCcw class="w-4 h-4 animate-spin" />
            {:else}
              <LanguagesIcon class="w-4 h-4" />
            {/if}
          </div>
          <div class="progress-info">
            <span class="progress-label">
              {item.isPreparing ? 'Preparing...' : 'Translating'}
            </span>
            <span class="progress-project">{item.label}</span>
          </div>
          {#if !item.isPreparing}
            <button type="button" class="cancel-btn" onclick={handleCancel}>Cancel</button>
          {/if}
        </div>
        {#if item.progress}
          <div class="progress-body">
            <div class="progress-stats">
              <div class="progress-stats-left">
                <span class="progress-count">
                  {item.progress.completed.toLocaleString()} / {item.progress.total.toLocaleString()}
                </span>
                {#if item.progress.batchInfo}
                  <span class="progress-batch">{item.progress.batchInfo}</span>
                {/if}
              </div>
              <span class="progress-duration" aria-label="Elapsed time">{elapsedFormatted}</span>
            </div>
            <div class="progress-bar-wrap">
              <div class="progress-bar" style="width: {item.progress.percentage}%"></div>
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .floating-progress-container {
    position: fixed;
    bottom: calc(var(--footer-height, 0px) + 1rem);
    right: 1rem;
    z-index: 50;
    display: flex;
    flex-direction: column-reverse;
    gap: 0.5rem;
    max-width: 320px;
    min-width: 260px;
    animation: slideUpFade 0.3s ease-out;
  }

  @media (prefers-reduced-motion: reduce) {
    .floating-progress-container {
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

  .progress-card {
    padding: 0.75rem 1rem;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--card);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
  }

  .progress-card.has-progress {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15), 0 0 0 1px color-mix(in oklch, var(--primary) 15%, transparent);
  }

  .progress-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .progress-icon {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    background: color-mix(in oklch, var(--primary) 12%, transparent);
    color: var(--primary);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .progress-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .progress-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--foreground);
  }

  .progress-project {
    font-size: 0.75rem;
    color: var(--muted-foreground);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-top: -5px;
  }

  .progress-body {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .progress-stats {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    font-size: 0.8125rem;
    color: var(--muted-foreground);
    font-variant-numeric: tabular-nums;
  }

  .progress-stats-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .progress-batch {
    opacity: 0.8;
    flex-shrink: 0;
  }

  .progress-duration {
    font-family: ui-monospace, 'Cascadia Code', 'Source Code Pro', Menlo, monospace;
    font-variant-numeric: tabular-nums;
    opacity: 0.9;
    min-width: 5ch; /* "59.9s" - prevent layout shift */
    flex-shrink: 0;
  }

  .cancel-btn {
    flex-shrink: 0;
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    font-weight: 500;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    background: var(--muted);
    color: var(--muted-foreground);
    transition: background 0.2s, color 0.2s;
  }

  .cancel-btn:hover,
  .cancel-btn:focus-visible {
    background: var(--destructive);
    color: oklch(1 0 0);
  }

  .cancel-btn:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: 2px;
  }

  .progress-bar-wrap {
    height: 6px;
    background: var(--muted);
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: var(--primary);
    border-radius: 999px;
    transition: width 0.2s ease-out;
    position: relative;
  }

  :global(.animate-spin) {
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
