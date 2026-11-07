<script lang="ts">
  import * as Card from "$lib/components/ui/card/index.js";
  import { Input } from "$lib/components/ui/input";
  import { Checkbox } from "$lib/components/ui/checkbox/index.js";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import Search from "@lucide/svelte/icons/search";
  import RefreshCcw from "@lucide/svelte/icons/refresh-ccw";
  import XIcon from "@lucide/svelte/icons/x";
  import { projectDataStore } from "$lib/stores/project-data";
  import { selectedStringsStore } from "$lib/stores/selected-strings";
  import { panelStateStore } from "$lib/stores/panel-state";
  import type { StringStatusRow } from "$lib/types/strings";

  let searchQuery = $state("");
  let contentElement: HTMLDivElement;
  let searchInputElement: HTMLInputElement;
  let contentHeight = $state(0);
  
  // Subscribe to panel state
  const isExpanded = $derived($panelStateStore === 'strings');

  // Subscribe to stores
  const projectData = $derived($projectDataStore);
  const selectedStrings = $derived($selectedStringsStore);
  
  // Calculate stats
  const totalCount = $derived(() => projectData?.strings?.length || 0);
  const selectedCount = $derived(() => selectedStrings.size);
  
  const filteredStrings = $derived((projectData?.strings || []).filter((string: StringStatusRow) => {
    // When search is empty, show only selected strings
    if (searchQuery.length === 0) {
      return selectedStrings.has(string.name);
    }
    // When searching, show only matching strings
    return string.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
           string.defaultValue.toLowerCase().includes(searchQuery.toLowerCase());
  }));

  function toggleStringSelection(key: string) {
    selectedStringsStore.toggle(key);
  }

  // Calculate missing languages count for a string
  function getMissingLanguagesCount(string: StringStatusRow): number {
    if (!projectData?.languages) return 0;
    
    const totalLanguages = projectData.languages.length;
    const translatedLanguages = Object.values(string.translationStatus).filter(status => status === "translated").length;
    return totalLanguages - translatedLanguages;
  }

  function measureContentHeight() {
    if (contentElement) {
      contentHeight = contentElement.scrollHeight;
    }
  }

  // Measure height when content changes
  $effect(() => {
    if (projectData.strings.length > 0) {
      setTimeout(measureContentHeight, 0);
    }
  });

  // Re-measure height when search results change
  $effect(() => {
    // Re-measure whenever search query or filtered results change
    searchQuery; // Track search query changes
    filteredStrings.length; // Track result count changes
    setTimeout(measureContentHeight, 0);
  });

  // Focus search input when panel expands
  $effect(() => {
    if (isExpanded && searchInputElement) {
      // Use setTimeout to ensure the panel animation completes before focusing
      setTimeout(() => {
        searchInputElement.focus();
      }, 200); // Match the CSS transition duration
    }
  });
</script>

<div class="rounded-lg border bg-card text-card-foreground shadow-sm">
  <div class="flex flex-col">
    <button 
      class="flex items-center justify-between w-full p-6 h-18 hover:bg-muted/50 transition-colors rounded-lg" 
      onclick={() => panelStateStore.toggle('strings')}
    >
      <div class="flex items-center gap-3">
        <RefreshCcw class="w-5 h-5 text-primary" />
        <h2 class="text-base font-semibold leading-none">Update Strings</h2>
      </div>
      <div class="flex items-center gap-3 text-muted-foreground">
        {#if !isExpanded && selectedStrings.size > 0}
          <span class="text-xs">{selectedStrings.size} selected</span>
        {/if}
        <ChevronDown class={`w-5 h-5 transform transition-transform ${isExpanded ? 'rotate-180' : '-rotate-90'}`} />
      </div>
    </button>

    <div 
      class="panel-content"
      style="height: {isExpanded ? contentHeight + 'px' : '0px'}"
    >
      <div class="panel-inner" bind:this={contentElement}>
        <div class="flex flex-col gap-4">


            <!-- Hint text to exaplin user what this does -->
            <div class="text-sm text-muted-foreground">
              <p>Search and select strings to update.</p>
            </div>

          <div class="flex gap-3">
            <div class="relative flex-1">
              <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <input
                type="text"
                placeholder="Search by key or value..."
                class="flex h-14 w-full rounded-md border border-input bg-background px-3 py-2 pl-9 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring/50 focus-visible:ring-offset-1 disabled:cursor-not-allowed disabled:opacity-50"
                bind:value={searchQuery}
                bind:this={searchInputElement}
              />
            </div>
            
            <button 
              class="flex items-center gap-2 px-4 h-14 text-left hover:bg-muted/50 transition-colors rounded-md bg-muted/20 border border-border whitespace-nowrap disabled:opacity-50 disabled:cursor-not-allowed"
              onclick={() => selectedStringsStore.clear()}
              disabled={selectedCount() === 0}
            >
              <XIcon class="w-4 h-4 text-muted-foreground" />
              <span class="text-sm font-medium text-foreground">Deselect ({selectedCount()})</span>
            </button>
          </div>

          {#if filteredStrings.length > 0}
            <div class="flex flex-col max-h-[400px] overflow-y-auto pr-2 results-container">
              {#each filteredStrings as string}
              <button 
                class="flex items-center gap-4 pl-4 pr-4 pt-2 pb-2 w-full text-left hover:bg-muted/50 transition-colors"
                onclick={() => toggleStringSelection(string.name)}
              >
                <Checkbox
                  checked={selectedStrings.has(string.name)}
                  onchange={(e) => {
                    e.stopPropagation();
                    toggleStringSelection(string.name);
                  }}
                />
                <div class="flex flex-col gap-0.5 flex-1">
                  <div class="font-mono text-sm font-medium">{string.name}</div>
                  <div class="text-sm text-muted-foreground">{string.defaultValue}</div>
                  {#if getMissingLanguagesCount(string) > 0}
                    <span class="text-xs text-amber-500">{getMissingLanguagesCount(string)} missing</span>
                  {/if}
                </div>
              </button>
            {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .panel-content {
    overflow: hidden;
    transition: height 0.4s cubic-bezier(0.4, 0, 0.2, 1);
    max-height: 600px;
  }
  
  .panel-inner {
    padding: 0.75rem;
    padding-top: 0;
  }

  .results-container::-webkit-scrollbar {
    width: 8px;
  }

  .results-container::-webkit-scrollbar-track {
    background: var(--background);
    border-radius: 4px;
  }

  .results-container::-webkit-scrollbar-thumb {
    background: var(--muted);
    border-radius: 4px;
  }

</style>
