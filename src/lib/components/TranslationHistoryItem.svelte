<script lang="ts">
  import type { TranslationSession } from '$lib/stores/translation-history';
  import * as Accordion from '$lib/components/ui/accordion';
  import { formatRelativeTime, getTranslationSummary } from '$lib/utils/translation-history';
  import CheckCircle2Icon from "@lucide/svelte/icons/check-circle-2";
  import CircleDashedIcon from "@lucide/svelte/icons/circle-dashed";
  import CircleIcon from "@lucide/svelte/icons/circle";
  import AlertTriangleIcon from "@lucide/svelte/icons/alert-triangle";
  import AlertOctagonIcon from "@lucide/svelte/icons/alert-octagon";
  import TranslationHistoryDetails from './TranslationHistoryDetails.svelte';

  let { session, isCurrent = false, onExpand }: {
    session: TranslationSession;
    isCurrent?: boolean;
    onExpand?: () => void;
  } = $props();

  // Auto-expand if in progress and is current session
  // Keep expanded if current session fails/cancels so user can see error and partial results
  let value = $state<string[]>([]);

  $effect(() => {
    if (session.status === 'in_progress' && isCurrent) {
      // Expand while in progress
      value = [session.id];
    } else if ((session.status === 'failed' || session.status === 'cancelled') && isCurrent) {
      // Keep current session expanded when it fails/cancels so user can see error and history
      value = [session.id];
    } else if (session.status !== 'in_progress' && !isCurrent) {
      // Collapse non-current sessions that are not in progress
      value = [];
    }
    // Note: If current session completes successfully, let user control expansion manually
  });

  function hasSuccessfulTranslations(): boolean {
    if (!session?.results) {
      return false;
    }

    for (const langMap of session.results.values()) {
      if (langMap.size > 0) {
        return true;
      }
    }

    return false;
  }

  function shouldShowFailureIcon(): boolean {
    return session.status === 'failed' || session.status === 'cancelled';
  }
  
  function hasPartialFailure(): boolean {
    if (session.status !== 'completed') return false;
    
    const totalExpected = session.sourceStrings.length * session.targetLanguages.length;
    const totalTranslated = Array.from(session.results.values()).reduce((sum, langMap) => sum + langMap.size, 0);
    
    return totalTranslated < totalExpected;
  }
</script>

<Accordion.Root
  type="multiple"
  bind:value={value}
  class="history-item-wrapper"
>
  <Accordion.Item value={session.id} class={`${isCurrent ? 'current' : ''} ${session.status === 'in_progress' ? 'is-in-progress' : ''}`}>
    <Accordion.Trigger
      disabled={session.status === 'in_progress'}
      data-in-progress={session.status === 'in_progress'}
      onclick={() => {
        if (session.status === 'in_progress') {
          return;
        }
        onExpand?.();
      }}
    >
      <!-- Status Icon -->
      <div class="status-icon">
        {#if session.status === 'completed' && hasPartialFailure()}
          <AlertTriangleIcon class="icon-partial-failed" />
        {:else if session.status === 'completed'}
          <CheckCircle2Icon class="icon-completed" />
        {:else if session.status === 'in_progress'}
          {#if isCurrent}
            <CircleDashedIcon class="icon-in-progress" />
          {:else}
            <CircleIcon class="icon-stalled" />
          {/if}
        {:else if shouldShowFailureIcon()}
          {#if hasSuccessfulTranslations()}
            <AlertTriangleIcon class="icon-partial-failed" />
          {:else}
            <AlertOctagonIcon class="icon-total-failed" />
          {/if}
        {/if}
      </div>

      <!-- Main Content -->
      <div class="history-item-content">
        <div class="history-item-summary">{getTranslationSummary(session)}</div>
        <div class="history-item-time">{formatRelativeTime(session.timestamp)}</div>
        {#if isCurrent && session.status === 'in_progress'}
          <div class="progress-indicator">
            <div class="progress-bar-mini">
              <div 
                class="progress-fill-mini"
                style="width: {session.progress.totalStrings > 0 ? (session.progress.completedStrings / session.progress.totalStrings * 100) : 0}%"
              ></div>
            </div>
            <span class="progress-text-mini">
              {session.progress.completedStrings} / {session.progress.totalStrings}
            </span>
          </div>
        {/if}
      </div>
    </Accordion.Trigger>

    {#if session.status !== 'in_progress'}
      <Accordion.Content>
        <TranslationHistoryDetails {session} />
      </Accordion.Content>
    {/if}
  </Accordion.Item>
</Accordion.Root>

<style>
  :global(.history-item-wrapper [data-slot="accordion-item"]) {
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--background);
    transition: all 0.2s ease;
    overflow: hidden;
  }

  :global(.history-item-wrapper [data-slot="accordion-item"].is-in-progress [data-slot="accordion-trigger"]) {
    cursor: not-allowed !important;
    opacity: 0.7;
  }

  :global(.history-item-wrapper [data-slot="accordion-item"].is-in-progress [data-slot="accordion-trigger"] svg) {
    display: none;
  }

  :global(.history-item-wrapper [data-slot="accordion-item"].current) {
    border-color: var(--primary);
    box-shadow: 0 0 0 1px var(--primary);
  }

  :global(.history-item-wrapper [data-slot="accordion-trigger"]) {
    width: 100%;
    padding: 1.25rem 1.5rem !important;
    display: flex !important;
    align-items: center !important;
    gap: 1rem !important;
    background: transparent !important;
    border: none !important;
    cursor: pointer;
    text-align: left;
    transition: background-color 0.2s ease;
    text-decoration: none !important;
  }

  :global(.history-item-wrapper [data-slot="accordion-trigger"]:hover) {
    background: color-mix(in oklch, var(--muted) 50%, transparent) !important;
    text-decoration: none !important;
  }

  .status-icon {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 50%;
    background: color-mix(in oklch, var(--muted) 50%, transparent);
    line-height: 0;
  }

  .status-icon :global(svg) {
    display: block;
  }

  .status-icon :global(.icon-completed) {
    width: 1.5rem;
    height: 1.5rem;
    color: var(--chart-2);
  }

  .status-icon :global(.icon-in-progress) {
    width: 1.5rem;
    height: 1.5rem;
    color: var(--primary);
    animation: spin 2s linear infinite;
  }

  .status-icon :global(.icon-stalled) {
    width: 1.5rem;
    height: 1.5rem;
    color: var(--muted-foreground);
  }

  .status-icon :global(.icon-partial-failed) {
    width: 1.5rem;
    height: 1.5rem;
    color: var(--warning);
  }

  .status-icon :global(.icon-total-failed) {
    width: 1.5rem;
    height: 1.5rem;
    color: var(--destructive);
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .history-item-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
    min-width: 0;
  }

  .history-item-summary {
    font-size: 1rem;
    font-weight: 500;
    color: var(--foreground);
    line-height: 1.4;
  }

  .history-item-time {
    font-size: 0.875rem;
    color: var(--muted-foreground);
    font-weight: 400;
  }

  .progress-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.25rem;
  }

  .progress-bar-mini {
    width: 80px;
    height: 4px;
    background: var(--muted);
    border-radius: 9999px;
    overflow: hidden;
  }

  .progress-fill-mini {
    height: 100%;
    background: var(--primary);
    border-radius: 9999px;
    transition: width 0.3s ease;
  }

  .progress-text-mini {
    font-size: 0.75rem;
    color: var(--muted-foreground);
    white-space: nowrap;
  }

  /* Style the default accordion chevron */
  :global(.history-item-wrapper [data-slot="accordion-trigger"] svg) {
    width: 1.25rem;
    height: 1.25rem;
    color: var(--muted-foreground);
    flex-shrink: 0;
    transition: transform 0.2s ease;
  }

  :global(.history-item-wrapper [data-slot="accordion-content"]) {
    padding: 0 !important;
  }
</style>


