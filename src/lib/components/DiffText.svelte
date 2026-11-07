<script lang="ts">
  import type { DiffSegment } from '$lib/utils/diff';
  import { computeSeparateDiffs } from '$lib/utils/diff';

  interface Props {
    oldText: string;
    newText: string;
    type: 'old' | 'new';
  }

  let { oldText, newText, type }: Props = $props();

  const segments = $derived.by(() => {
    const { oldSegments, newSegments } = computeSeparateDiffs(oldText, newText);
    return type === 'old' ? oldSegments : newSegments;
  });
</script>

<span class="diff-text">
  {#each segments as segment (segment)}
    {#if segment.type === 'equal'}
      <span class="diff-equal">{segment.text}</span>
    {:else if segment.type === 'delete'}
      <span class="diff-delete">{segment.text}</span>
    {:else if segment.type === 'insert'}
      <span class="diff-insert">{segment.text}</span>
    {/if}
  {/each}
</span>

<style>
  .diff-text {
    white-space: pre-wrap;
    word-break: break-word;
  }

  .diff-equal {
    /* No special styling for equal parts */
  }

  .diff-delete {
    background: color-mix(in oklch, var(--destructive) 30%, transparent);
    text-decoration: line-through;
    color: var(--destructive);
  }

  .diff-insert {
    background: color-mix(in oklch, var(--primary) 20%, transparent);
    color: var(--foreground);
    font-weight: 600;
  }
</style>

