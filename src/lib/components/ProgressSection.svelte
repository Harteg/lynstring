<script lang="ts">
  import { projectDataStore } from '$lib/stores/project-data';
  import { translationSelectionStore } from '$lib/stores/translation-selection';
  import RefreshButton from './RefreshButton.svelte';

  // Subscribe to stores
  const projectData = $derived($projectDataStore);
  const translationSelection = $derived($translationSelectionStore);


  // Calculate missing strings count for selected languages only
  let missingCount = $state(0);
  let completionPercentage = $state(0);
  let totalPossible = $state(0);
  let completed = $state(0);
  
  $effect(() => {
    if (!projectData.strings.length || !projectData.languages.length) {
      missingCount = 0;
      completionPercentage = 0;
      return;
    }
    
    // Count unique strings that have at least one missing translation in selected languages
    let uniqueMissingStrings = 0;
    let totalMissingTranslations = 0;
    
    for (const string of projectData.strings) {
      let hasMissingTranslation = false;
      for (const language of projectData.languages) {
        if (string.translationStatus[language] === 'missing') {
          totalMissingTranslations++;
          hasMissingTranslation = true;
        }
      }
      if (hasMissingTranslation) {
        uniqueMissingStrings++;
      }
    }
    
    totalPossible = projectData.strings.length * projectData.languages.length;
    completed = totalPossible - totalMissingTranslations;
    completionPercentage = Math.round((completed / totalPossible) * 100);
    missingCount = uniqueMissingStrings;
  });

  // Calculate total strings count
  const totalStrings = $derived(() => projectData.strings.length);
  
  
  const selectedCount = $derived(() => translationSelection.selectedLanguages.size);
  const totalCount = $derived(() => projectData.languages.length);
</script>

<div class="project-status-card">
  <div class="card-header">
    <h2 class="card-title">Project Status</h2>
    <RefreshButton />
  </div>

  <div class="stats-grid">
    <!-- Total Languages -->
    <div class="stat-item">
      <div class="stat-label">Total Languages</div>
      <div class="stat-value">{totalCount()}</div>
    </div>

    <!-- Total Strings -->
    <div class="stat-item">
      <div class="stat-label">Total Strings</div>
      <div class="stat-value">{totalStrings()}</div>
    </div>

    <!-- Missing String Keys -->
    <!-- Missing Translations -->
    <div class="stat-item">
      <div class="stat-label">Missing Translations</div>
      <div class="stat-value" class:text-red={(totalPossible - completed) > 0} class:text-green={(totalPossible - completed) === 0}>
        {totalPossible - completed}
      </div>
    </div>

    <div class="stat-item">
      <div class="stat-label">Missing strings</div>
      <div class="stat-value" class:text-red={missingCount > 0} class:text-green={missingCount === 0}>
        {missingCount}
      </div>
    </div>
  </div>

  <!-- Progress bar -->
  {#if selectedCount() > 0}
    <div class="progress-bar-container">
      <div class="progress-bar">
        <div 
          class="progress-fill"
          style="width: {completionPercentage}%"
        ></div>
      </div>
      <div class="progress-text">
        {completionPercentage}% complete
      </div>
    </div>
  {/if}

</div>

<style>
  .project-status-card {
    background: rgba(30, 41, 59, 0.5);
    border-radius: 16px;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 24px;
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
  }

  .card-title {
    font-size: 1.25rem;
    font-weight: 500;
    color: #E2E8F0;
    margin: 0;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 24px;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .stat-label {
    font-size: 0.875rem;
    color: #94A3B8;
    font-weight: 500;
  }

  .stat-value {
    font-size: 2.5rem;
    font-weight: 600;
    color: #F8FAFC;
    line-height: 1;
    transition: color 0.2s ease;
  }

  .text-red {
    color: #FB923C;
  }

  .text-green {
    color: #10B981;
  }

  .progress-bar-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: rgba(51, 65, 85, 0.5);
    border-radius: 9999px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #10B981, #059669);
    border-radius: 9999px;
    transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .progress-text {
    font-size: 0.875rem;
    color: #94A3B8;
    text-align: right;
    font-weight: 500;
  }

</style>
