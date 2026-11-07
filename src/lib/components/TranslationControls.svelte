<script lang="ts">
  import { translationSelectionStore } from '$lib/stores/translation-selection';
  import { selectedStringsStore } from '$lib/stores/selected-strings';
  import { projectDataStore, filteredStrings, translationBatch } from '$lib/stores/project-data';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Checkbox } from '$lib/components/ui/checkbox';

  // Subscribe to stores
  const { filter, selectedLanguages } = $derived($translationSelectionStore);
  const selectedStrings = $derived($selectedStringsStore);
  const { languages } = $derived($projectDataStore);
  const filteredStringsData = $derived($filteredStrings);
  const batch = $derived($translationBatch);

  // Local state for search term
  let searchTerm = $state('');

  // Helper functions
  function updateFilter(newFilter: Partial<typeof filter>) {
    translationSelectionStore.updateFilter(newFilter);
  }

  function selectMissingStrings() {
    const missingLanguages = languages.filter(lang => 
      filteredStringsData.some(row => row.translationStatus[lang] === 'missing')
    );
    
    // Clear current selection and select missing strings
    selectedStringsStore.clear();
    filteredStringsData.forEach(row => {
      if (missingLanguages.some(lang => row.translationStatus[lang] === 'missing')) {
        selectedStringsStore.toggle(row.name);
      }
    });
  }

  function clearAllSelections() {
    selectedStringsStore.clear();
    translationSelectionStore.clearLanguageSelection();
  }

  // Update search term when it changes
  $effect(() => {
    if (searchTerm !== filter.searchTerm) {
      console.log('Search term changed:', searchTerm);
      updateFilter({ searchTerm });
    }
  });

  // Update local search term when filter changes
  $effect(() => {
    searchTerm = filter.searchTerm;
  });
</script>

<div class="translation-controls">
  <div class="controls-section">
    <h3>Filter & Search</h3>
    
    <div class="search-row">
      <Input 
        placeholder="Search strings..."
        bind:value={searchTerm}
      />
    </div>

    <div class="filter-options">
      <label class="filter-option">
        <Checkbox 
          checked={filter.showMissing}
          onCheckedChange={(checked) => updateFilter({ showMissing: checked })}
        />
        <span>Show Missing Translations</span>
      </label>

      <label class="filter-option">
        <Checkbox 
          checked={filter.showTranslated}
          onCheckedChange={(checked) => updateFilter({ showTranslated: checked })}
        />
        <span>Show Translated</span>
      </label>

      <label class="filter-option">
        <Checkbox 
          checked={filter.showNonTranslatable}
          onCheckedChange={(checked) => updateFilter({ showNonTranslatable: checked })}
        />
        <span>Show Non-Translatable</span>
      </label>
    </div>
  </div>

  <div class="controls-section">
    <h3>Language Selection</h3>
    
    <div class="language-chips">
      {#each languages as language}
        <button 
          class="language-chip {selectedLanguages.has(language) ? 'selected' : ''}"
          onclick={() => translationSelectionStore.toggleLanguageSelection(language)}
        >
          {language}
          {#if selectedLanguages.has(language)}
            <span class="check">✓</span>
          {/if}
        </button>
      {/each}
    </div>

    <div class="language-actions">
      <Button variant="outline" size="sm" onclick={() => translationSelectionStore.selectAllLanguages()}>
        {() => "Select All"}
      </Button>
      <Button variant="outline" size="sm" onclick={() => translationSelectionStore.clearLanguageSelection()}>
        {() => "Clear All"}
      </Button>
    </div>
  </div>

  <div class="controls-section">
    <h3>String Selection</h3>
    
    <div class="selection-stats">
      <span>Selected: {selectedStrings.size} strings</span>
      <span>Filtered: {filteredStringsData.length} strings</span>
    </div>

    <div class="selection-actions">
      <Button variant="outline" size="sm" onclick={() => {
        selectedStringsStore.clear();
        filteredStringsData.forEach(row => selectedStringsStore.toggle(row.name));
      }}>
        {() => "Select All Filtered"}
      </Button>
      <Button variant="outline" size="sm" onclick={selectMissingStrings}>
        {() => "Select Missing"}
      </Button>
      <Button variant="outline" size="sm" onclick={() => selectedStringsStore.clear()}>
        {() => "Clear Selection"}
      </Button>
    </div>
  </div>

  <div class="controls-section">
    <h3>Translation Batch</h3>
    
    {#if batch.selections.length > 0}
      <div class="batch-info">
        <div class="batch-stat">
          <strong>{batch.selections.length}</strong> strings selected
        </div>
        <div class="batch-stat">
          <strong>{batch.targetLanguages.length}</strong> languages selected
        </div>
        <div class="batch-stat">
          <strong>{batch.estimatedTokens?.toFixed(0)}</strong> estimated tokens
        </div>
        <div class="batch-stat">
          <strong>${batch.estimatedCost?.toFixed(4)}</strong> estimated cost
        </div>
      </div>

      <div class="batch-actions">
        <Button variant="default" size="sm" class="translate-btn">
          {() => `Translate Selected (${batch.selections.length} strings)`}
        </Button>
        <Button variant="outline" size="sm" onclick={clearAllSelections}>
          {() => "Clear All"}
        </Button>
      </div>
    {:else}
      <div class="no-selection">
        <p>Select strings and languages to create a translation batch</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .translation-controls {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding: 1rem;
    background: #2a2a2a;
    border-radius: 8px;
    border: 1px solid #404040;
  }

  .controls-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .controls-section h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: #e0e0e0;
  }

  .search-row {
    width: 100%;
  }

  .filter-options {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .filter-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: #ccc;
    cursor: pointer;
  }

  .language-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .language-chip {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.375rem 0.75rem;
    background: #404040;
    border: 1px solid #555;
    border-radius: 16px;
    font-size: 0.875rem;
    color: #ccc;
    cursor: pointer;
    transition: all 0.2s;
  }

  .language-chip:hover {
    background: #505050;
    border-color: #666;
  }

  .language-chip.selected {
    background: #007acc;
    border-color: #0098ff;
    color: white;
  }

  .language-chip .check {
    font-weight: bold;
  }

  .language-actions,
  .selection-actions,
  .batch-actions {
    display: flex;
    gap: 0.5rem;
  }

  .selection-stats {
    display: flex;
    gap: 1rem;
    font-size: 0.875rem;
    color: #888;
  }

  .batch-info {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 0.5rem;
  }

  .batch-stat {
    text-align: center;
    padding: 0.5rem;
    background: #404040;
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .batch-stat strong {
    color: #007acc;
  }

  .no-selection {
    text-align: center;
    color: #888;
    font-style: italic;
  }

  .translate-btn {
    background: #007acc;
    color: white;
  }

  .translate-btn:hover {
    background: #0098ff;
  }
</style>
