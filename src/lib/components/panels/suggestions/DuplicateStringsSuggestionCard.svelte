<script lang="ts">
  import { duplicateKeysStore } from '$lib/stores/project-data';
  import AlertTriangle from '@lucide/svelte/icons/alert-triangle';
  import SuggestionCard from './SuggestionCard.svelte';

  const duplicateKeys = $derived($duplicateKeysStore);
  const hasDuplicates = $derived(duplicateKeys.length > 0);

  function formatLanguageName(lang: string): string {
    return lang === 'default' ? 'Default (values/)' : lang;
  }
</script>

{#if hasDuplicates}
  <SuggestionCard
    icon={AlertTriangle}
    title="Duplicate string keys ({duplicateKeys.length})"
    description="Fix duplicates before translating"
    variant="blocker"
  >
    {#snippet children()}
      <div class="duplicate-details">
        {#each duplicateKeys as duplicate}
          <div class="duplicate-item">
            <code class="key">{duplicate.key}</code>
            <span class="langs">in: {duplicate.languages.map(formatLanguageName).join(', ')}</span>
          </div>
        {/each}
      </div>
    {/snippet}
  </SuggestionCard>
{/if}

<style>
  .duplicate-details {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .duplicate-item {
    font-size: 0.8125rem;
    color: var(--muted-foreground);
  }
  .key {
    font-family: ui-monospace, monospace;
    background: color-mix(in oklch, var(--destructive) 20%, transparent);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
  }
  .langs {
    margin-left: 0.5rem;
  }
</style>
