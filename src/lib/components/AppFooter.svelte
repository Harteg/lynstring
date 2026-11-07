<script lang="ts">
  import { currentProject } from '$lib/stores/project';
  import { currentAllActivitySessions } from '$lib/stores/translation-history';
  import { formatRelativeTimeShort } from '$lib/utils/translation-history';
  import { getVersion } from '@tauri-apps/api/app';
  import ActivityHistoryDialog from './dialogs/ActivityHistoryDialog.svelte';
  import History from '@lucide/svelte/icons/history';

  const allSessions = $derived($currentAllActivitySessions);
  const lastActivity = $derived(allSessions[0] ?? null);

  let activityDialogOpen = $state(false);
  let appVersion = $state('');
  let timeTick = $state(0);

  // Update every 30s so "Less than a minute ago" transitions to "1 minute ago", etc.
  $effect(() => {
    const id = setInterval(() => (timeTick = Date.now()), 30_000);
    return () => clearInterval(id);
  });

  // Derived so activity text re-evaluates when timeTick changes (timeTick is dependency)
  const activityText = $derived.by(() => {
    void timeTick; // dependency: when this updates, we re-compute the relative time
    return lastActivity ? formatLastActivity(lastActivity) : null;
  });

  function formatLastActivity(session: (typeof allSessions)[number]): string {
    const n = session.sourceStrings.length;
    const langs = session.targetLanguages.length;
    const time = formatRelativeTimeShort(session.timestamp);
    const statusLabel =
      session.status === 'cancelled' ? 'Cancelled' :
      session.status === 'failed' ? 'Failed' : 'Translated';
    const progress =
      session.status === 'cancelled' && session.progress.completedStrings > 0
        ? ` (${session.progress.completedStrings}/${session.progress.totalStrings})`
        : '';
    return `${statusLabel} · ${n} string${n !== 1 ? 's' : ''} → ${langs} lang${langs !== 1 ? 's' : ''}${progress} · ${time}`;
  }

  getVersion().then((v) => (appVersion = v));
</script>

<footer class="app-footer">
  <div class="footer-left">
    {#if $currentProject}
      <button
        type="button"
        class="footer-activity"
        onclick={() => (activityDialogOpen = true)}
        title={lastActivity ? 'View all activity' : 'View activity history'}
      >
        <History class="size-3.5" />
        {#if lastActivity}
          <span class="footer-activity-text">{activityText}</span>
        {:else}
          <span class="footer-placeholder">No recent activity</span>
        {/if}
      </button>
    {:else}
      <span class="footer-placeholder">No recent activity</span>
    {/if}
  </div>
  <div class="footer-right">
    {#if appVersion}
      <span class="footer-version">v{appVersion}</span>
    {/if}
  </div>
</footer>

<ActivityHistoryDialog bind:open={activityDialogOpen} />

<style>
  .app-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--footer-height);
    padding: 0 1rem;
    font-size: 0.75rem;
    color: var(--muted-foreground);
    background: var(--footer-background);
    flex-shrink: 0;
    border-top: 1px solid var(--border);
  }

  .footer-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .footer-activity {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.125rem 0;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--muted-foreground);
    font-size: inherit;
    text-align: left;
    max-width: 100%;
    overflow: hidden;
  }

  .footer-activity:hover {
    color: var(--foreground);
  }

  .footer-activity-text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .footer-placeholder {
    opacity: 0.7;
  }

  .footer-right {
    flex-shrink: 0;
  }

  .footer-version {
    opacity: 0.7;
  }
</style>
