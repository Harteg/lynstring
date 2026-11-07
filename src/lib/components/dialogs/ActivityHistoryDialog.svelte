<script lang="ts">
  import * as Dialog from '$lib/components/ui/dialog';
  import { currentAllActivitySessions } from '$lib/stores/translation-history';
  import { formatRelativeTimeShort, formatDuration } from '$lib/utils/translation-history';

  interface Props {
    open?: boolean;
  }
  let { open = $bindable(false) }: Props = $props();

  const allSessions = $derived($currentAllActivitySessions);

  function getSessionParts(session: (typeof allSessions)[number]) {
    const n = session.sourceStrings.length;
    const langs = session.targetLanguages.length;
    const time = formatRelativeTimeShort(session.timestamp);
    const duration = session.duration != null ? formatDuration(session.duration, false) : null;
    const status =
      session.status === 'cancelled' ? 'Cancelled' :
      session.status === 'failed' ? 'Failed' : 'Translated';
    const progress =
      session.status === 'cancelled' && session.progress.completedStrings > 0
        ? `${session.progress.completedStrings}/${session.progress.totalStrings} done`
        : null;
    const counts = `${n} string${n !== 1 ? 's' : ''} → ${langs} lang${langs !== 1 ? 's' : ''}`;
    return { status, counts, progress, time, duration };
  }

</script>

<Dialog.Root bind:open>
  <Dialog.Content class="dialog-standard sm:max-w-[520px]">
    <Dialog.Header class="dialog-standard-header !pb-3">
      <Dialog.Title class="dialog-standard-title">Activity history</Dialog.Title>
      <Dialog.Description class="dialog-standard-description">
        Translation sessions for the current project
      </Dialog.Description>
    </Dialog.Header>

    <div class="activity-list-container">
      {#if allSessions.length > 0}
        <div class="activity-list">
        {#each allSessions as session}
          {@const parts = getSessionParts(session)}
          <div class="activity-item">
            <div class="flex flex-wrap items-center gap-2 gap-y-3 text-[0.8125rem] leading-snug">
              <span class="font-semibold">{parts.status}</span>
              <span>{parts.counts}</span>
              {#if parts.progress}
                <span class="text-xs text-muted-foreground bg-muted/50 px-2 py-0.5 rounded">
                  {parts.progress}
                </span>
              {/if}
              {#if parts.duration}
                <span class="text-muted-foreground tabular-nums">{parts.duration}</span>
              {/if}
              <span class="ml-auto text-xs text-muted-foreground">{parts.time}</span>
            </div>
          </div>
        {/each}
        </div>
      {:else}
        <div class="activity-empty">No translation activity yet</div>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>

<style>
  .activity-list-container {
    margin: 0.5rem 0 1rem 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .activity-list {
    max-height: 360px;
    overflow-y: auto;
  }

  .activity-item {
    padding: 0.625rem 1rem;
    border-bottom: 1px solid var(--border);
    color: var(--foreground);
  }

  .activity-item:last-child {
    border-bottom: none;
  }

  .activity-empty {
    padding: 2rem;
    text-align: center;
    font-size: 0.875rem;
    color: var(--muted-foreground);
  }
</style>
