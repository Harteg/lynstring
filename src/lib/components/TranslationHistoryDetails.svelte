<script lang="ts">
  import type { TranslationSession } from '$lib/stores/translation-history';
  import * as Accordion from '$lib/components/ui/accordion';

  let { session }: { session: TranslationSession } = $props();

  type TranslationResult = {
    language: string;
    text: string | null;
    missing: boolean;
  };

  const totalTranslations = session.sourceStrings.length * session.targetLanguages.length;
  const translatedCount = Array.from(session.results.values()).reduce((sum, langMap) => sum + langMap.size, 0);
  const failedCount = totalTranslations - translatedCount;
  
  // Show partial status if any strings failed OR if status is cancelled/failed
  const showPartialStatus =
    (session.status === 'completed' && failedCount > 0) ||
    (session.status !== 'completed' && translatedCount < totalTranslations);
  
  const translatedPercent = totalTranslations > 0 ? Math.round((translatedCount / totalTranslations) * 100) : 0;

  // Get translations for a specific string key including missing entries
  function getTranslationsForString(stringKey: string): TranslationResult[] {
    return session.targetLanguages
      .map(language => {
        const langMap = session.results.get(language);
        const text = langMap?.get(stringKey) ?? null;
        const missing = !text || text.trim().length === 0;
        return { language, text, missing };
      })
      .sort((a, b) => a.language.localeCompare(b.language));
  }
</script>

<div class="history-details">
  {#if session.error || showPartialStatus}
    <div class={`status-card ${session.error ? 'error' : ''} ${showPartialStatus ? 'partial' : ''}`}>
      {#if session.error}
        <div class="status-row">
          <span class="status-label">Error:</span>
          <span class="status-message">{session.error}</span>
        </div>
      {/if}

      {#if showPartialStatus}
        <div class="status-row spaced">
          <span class="status-label">
            {session.status === 'completed' && failedCount > 0 ? 'Partial success' : 'Progress before cancellation'}
          </span>
          <span class="status-value">{translatedPercent}%</span>
        </div>
        <div class="status-progress">
          <div class="status-progress-fill" style={`width: ${translatedPercent}%`}></div>
        </div>
        <div class="status-subtext">
          {translatedCount.toLocaleString()} / {totalTranslations.toLocaleString()} translations completed
        </div>
      {/if}

      {#if failedCount > 0 && (session.status === 'completed' || session.status === 'failed')}
        <div class="status-row">
          <span class="status-label">
            {failedCount} string{failedCount === 1 ? '' : 's'} failed validation:
          </span>
          <span class="status-message">
            These strings may have placeholder mismatches or parsing issues. Try translating them individually.
          </span>
        </div>
      {/if}
    </div>
  {/if}

  <div class="translations-section">
    <Accordion.Root type="multiple">
      {#each session.sourceStrings as sourceString, index (sourceString.name)}
        {@const translations = getTranslationsForString(sourceString.name)}
        {@const missingCount = translations.filter(t => t.missing).length}
        <Accordion.Item
          value={`string-${index}`}
          class={missingCount > 0 ? 'incomplete' : undefined}
        >
          <Accordion.Trigger>
            <div class="string-content">
              <div class="string-info">
                <code class="string-key">{sourceString.name}</code>
                <span class="string-default">{sourceString.text}</span>
                {#if missingCount > 0}
                  <span class="string-warning">
                    {missingCount} missing translation{missingCount === 1 ? '' : 's'}
                  </span>
                {/if}
              </div>
            </div>
          </Accordion.Trigger>
          
          <Accordion.Content>
            {#each translations as translation (translation.language)}
              <div class="translation-row" class:missing={translation.missing}>
                <code class="language-code">{translation.language}</code>
                {#if translation.missing}
                  <span class="translation-text missing-text">Not translated</span>
                {:else}
                  <span class="translation-text">{translation.text}</span>
                {/if}
              </div>
            {/each}
          </Accordion.Content>
        </Accordion.Item>
      {/each}
    </Accordion.Root>
  </div>

</div>

<style>
  .history-details {
    border-top: 1px solid var(--border);
  }

  .status-card {
    margin: 1rem 1.5rem;
    padding: 1rem 1.25rem;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .status-card.error {
    border: 1px solid color-mix(in oklch, var(--destructive) 50%, transparent);
    background: color-mix(in oklch, var(--destructive) 10%, transparent);
  }

  .status-card.partial {
    border: 1px solid color-mix(in oklch, var(--warning, var(--primary)) 40%, transparent);
    background: color-mix(in oklch, var(--warning, var(--primary)) 8%, transparent);
  }

  .status-row {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }

  .status-row.spaced {
    justify-content: space-between;
    align-items: center;
  }

  .status-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--foreground);
  }

  .status-message {
    font-size: 0.875rem;
    color: var(--foreground);
  }

  .status-value {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--accent-foreground);
  }

  .status-progress {
    width: 100%;
    height: 6px;
    border-radius: 999px;
    background: color-mix(in oklch, var(--warning, var(--primary)) 15%, transparent);
    overflow: hidden;
  }

  .status-progress-fill {
    height: 100%;
    background: var(--warning, var(--primary));
    border-radius: inherit;
    transition: width 0.3s ease;
  }

  .status-subtext {
    font-size: 0.8rem;
    color: var(--accent-foreground);
    opacity: 0.85;
  }

  .translations-section {
    padding: 1rem 1.5rem;
  }

  .translations-section :global([data-slot="accordion"]) {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .translations-section :global([data-slot="accordion-item"]) {
    border: 1px solid color-mix(in oklch, var(--border) 80%, transparent) !important;
    border-radius: 12px !important;
    padding: 0 !important;
    background: color-mix(in oklch, var(--foreground) 2%, transparent) !important;
    overflow: hidden;
    transition: border-color 0.2s ease, box-shadow 0.2s ease;
  }

  .translations-section :global([data-slot="accordion-item"].incomplete) {
    border-color: color-mix(in oklch, var(--destructive) 25%, var(--border)) !important;
  }

  .translations-section :global([data-slot="accordion-item"]:is(:hover, :focus-within)) {
    border-color: color-mix(in oklch, var(--primary) 30%, var(--border)) !important;
    box-shadow: 0 0 0 1px color-mix(in oklch, var(--primary) 20%, transparent);
  }

  .translations-section :global([data-slot="accordion-trigger"]) {
    display: flex !important;
    align-items: center !important;
    gap: 0.75rem !important;
    padding: 1rem 1.25rem !important;
    width: 100%;
    border: none !important;
    background: transparent !important;
    cursor: pointer;
    transition: background-color 0.2s ease;
    text-decoration: none !important;
  }

  .translations-section :global([data-slot="accordion-trigger"]:hover) {
    background: color-mix(in oklch, var(--foreground) 4%, transparent) !important;
    text-decoration: none !important;
  }

  .string-content {
    flex: 1;
    min-width: 0;
  }

  .string-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  .string-warning {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--destructive);
  }

  .string-key {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--foreground);
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    word-break: break-word;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    line-height: 1.4;
  }

  .string-default {
    font-size: 0.875rem;
    color: var(--muted-foreground);
    font-weight: 400;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    line-height: 1.4;
    -webkit-line-clamp: 1;
    line-clamp: 1;
    max-height: 1.4em;
  }

  :global([data-slot="accordion-item"][data-state="open"] .string-default) {
    -webkit-line-clamp: 5;
    line-clamp: 5;
    max-height: calc(1.4em * 5);
  }

  /* Style the default accordion chevron */
  .translations-section :global([data-slot="accordion-trigger"] svg) {
    width: 1.25rem;
    height: 1.25rem;
    color: var(--muted-foreground);
    flex-shrink: 0;
    transition: transform 0.2s ease;
    margin-left: auto;
  }

  .translations-section :global([data-slot="accordion-content"]) {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: 0 1.25rem 1.25rem !important;
    background: transparent !important;
  }

  .translation-row {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    padding: 0.85rem 0.25rem;
    border-bottom: 1px solid color-mix(in oklch, var(--border) 70%, transparent);
  }

  .translation-row:first-child {
    padding-top: 0;
  }

  .translation-row:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .translation-row.missing {
    border-left: 3px solid color-mix(in oklch, var(--destructive) 50%, transparent);
    padding-left: 0.75rem;
  }

  .language-code {
    flex-shrink: 0;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--muted-foreground);
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    min-width: 2rem;
  }

  .translation-text {
    flex: 1;
    font-size: 0.875rem;
    color: var(--foreground);
    font-weight: 400;
    line-height: 1.5;
  }

  .translation-text.missing-text {
    color: var(--muted-foreground);
    font-style: italic;
  }

</style>

