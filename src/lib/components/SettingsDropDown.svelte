<script lang="ts">
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import SettingsIcon from '@lucide/svelte/icons/settings';
  import InfoIcon from '@lucide/svelte/icons/info';
  import BugIcon from '@lucide/svelte/icons/bug';
  import KeyRoundIcon from '@lucide/svelte/icons/key-round';
  import ThemeSelector from '$lib/components/ThemeSelector.svelte';
  import ApiKeyDialog from '$lib/components/dialogs/ApiKeyDialog.svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { isDevMode } from '$lib/config/dev';
  import { devPreferencesStore, updateDevPreferences } from '$lib/stores/dev-preferences';
  import { appSettingsStore, MODELS } from '$lib/stores/app-settings';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { requestUnusedStringsScan } from '$lib/stores/unused-strings-scan';
  import SearchIcon from '@lucide/svelte/icons/search';
  import { currentProject } from '$lib/stores/project';

  const REPO_URL = 'https://github.com/harteg/lynstring';

  let dropdownOpen = $state(false);
  let apiKeyDialogOpen = $state(false);
  let appVersion = $state<string>('Loading...');
  let devSectionVisible = $state(false);

  getVersion().then(version => {
    appVersion = version;
  }).catch(() => {
    appVersion = 'Unknown';
  });

  const modelName = $derived(
    MODELS.find(m => m.id === $appSettingsStore?.model)?.name ?? $appSettingsStore?.model ?? ''
  );

  function openApiKeyDialog() {
    dropdownOpen = false;
    apiKeyDialogOpen = true;
  }

  const isDev = $derived(isDevMode());

  function toggleDevSection() {
    if (isDev) {
      devSectionVisible = !devSectionVisible;
    }
  }
</script>

<DropdownMenu.Root bind:open={dropdownOpen}>
  <DropdownMenu.Trigger class="settings-trigger">
    <SettingsIcon class="h-4 w-4" />
    <span class="hidden sm:inline">Settings</span>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content class="w-80">

    <!-- Anthropic API -->
    <div class="space-y-2 px-2 py-1.5">
      <h4 class="text-sm font-medium">Anthropic API</h4>
      <div class="flex items-center justify-between text-xs text-muted-foreground">
        <span>Model</span>
        <span class="text-foreground">{modelName}</span>
      </div>
      <div class="flex items-center justify-between text-xs text-muted-foreground">
        <span>API key</span>
        <span class="font-mono text-foreground">
          {#if $appSettingsStore?.has_api_key}
            {$appSettingsStore.api_key_hint}{$appSettingsStore.api_key_from_env ? ' (env)' : ''}
          {:else}
            Not set
          {/if}
        </span>
      </div>
      <DropdownMenu.Item onclick={openApiKeyDialog} class="flex items-center gap-2 cursor-pointer">
        <KeyRoundIcon class="h-4 w-4" />
        Change API key or model
      </DropdownMenu.Item>
    </div>

    <DropdownMenu.Separator />

    <!-- Tools -->
    {#if $currentProject}
      <div class="space-y-1 px-2 py-1.5">
        <h4 class="text-sm font-medium">Tools</h4>
        <DropdownMenu.Item
          onclick={() => {
            requestUnusedStringsScan();
            dropdownOpen = false;
          }}
          class="flex items-center gap-2 cursor-pointer"
        >
          <SearchIcon class="h-4 w-4" />
          Scan for unused strings
        </DropdownMenu.Item>
      </div>
      <DropdownMenu.Separator />
    {/if}

    <!-- Theme Selector -->
    <div class="flex items-center justify-between gap-3 px-2 py-1.5">
      <h4 class="text-sm font-medium">Theme</h4>
      <ThemeSelector />
    </div>

    <DropdownMenu.Separator />

    <!-- Theme Selector -->
    <div class="flex items-center justify-between gap-3 px-2 py-1.5">
      <h4 class="text-sm font-medium">Theme</h4>
      <ThemeSelector />
    </div>

    <DropdownMenu.Separator />

    <!-- Version & About -->
    <div class="space-y-2 px-2 py-1.5">
      <div class="flex items-center justify-between text-sm">
        <div class="flex items-center gap-2 text-muted-foreground">
          <InfoIcon class="h-4 w-4" />
          <span>Version</span>
        </div>
        {#if isDev}
          <button
            type="button"
            class="font-mono text-xs cursor-pointer hover:text-foreground transition-colors bg-transparent border-none p-0 text-left"
            onclick={toggleDevSection}
          >
            {appVersion}
          </button>
        {:else}
          <span class="font-mono text-xs">{appVersion}</span>
        {/if}
      </div>

      <div class="flex items-center gap-2 flex-wrap">
        <button
          type="button"
          class="text-xs text-muted-foreground hover:text-foreground hover:underline underline-offset-2 bg-transparent border-none p-0 cursor-pointer"
          onclick={() => openUrl(REPO_URL)}
        >
          GitHub
        </button>
        <span class="text-muted-foreground">·</span>
        <button
          type="button"
          class="text-xs text-muted-foreground hover:text-foreground hover:underline underline-offset-2 bg-transparent border-none p-0 cursor-pointer inline-flex items-center gap-1"
          onclick={() => openUrl(`${REPO_URL}/issues`)}
        >
          <BugIcon class="h-3 w-3" />
          Report Issue
        </button>
      </div>
    </div>

    <!-- Dev Section (only visible in dev mode and when toggled) -->
    {#if isDev && devSectionVisible}
      <DropdownMenu.Separator />
      <div class="space-y-2 px-2 py-1.5">
        <h4 class="text-sm font-medium text-muted-foreground">Dev Tools</h4>
        <div class="space-y-1 text-xs text-muted-foreground">
          <div>Development mode enabled</div>
          <div>Build: {import.meta.env.MODE}</div>

          <div class="pt-2 border-t space-y-2">
            <div class="flex items-center justify-between">
              <Label for="simulate-translations" class="text-xs cursor-pointer">
                Simulate translations
              </Label>
              <Switch
                id="simulate-translations"
                bind:checked={$devPreferencesStore.simulateTranslations}
                onCheckedChange={(checked) => {
                  updateDevPreferences((prefs) => ({
                    ...prefs,
                    simulateTranslations: checked
                  }));
                }}
              />
            </div>
            <div class="text-xs text-muted-foreground">
              When enabled, copies default strings to target languages instead of calling AI API
            </div>
            
            {#if $devPreferencesStore.simulateTranslations}
              <div class="flex items-center justify-between pt-1">
                <Label for="simulate-delay" class="text-xs cursor-pointer">
                  Simulate delay (5s per batch)
                </Label>
                <Switch
                  id="simulate-delay"
                  bind:checked={$devPreferencesStore.simulateWithDelay}
                  onCheckedChange={(checked) => {
                    updateDevPreferences((prefs) => ({
                      ...prefs,
                      simulateWithDelay: checked
                    }));
                  }}
                />
              </div>
            {/if}
          </div>


        </div>
      </div>
    {/if}

  </DropdownMenu.Content>
</DropdownMenu.Root>

<ApiKeyDialog bind:open={apiKeyDialogOpen} />

<style>
  :global([data-slot="dropdown-menu-trigger"].settings-trigger) {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    height: 2.25rem;
    padding: 0 0.75rem;
    font-size: 0.875rem;
    font-weight: 500;
    border-radius: 0.375rem;
    background: transparent;
    border: none;
    color: var(--foreground);
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  :global([data-slot="dropdown-menu-trigger"].settings-trigger:hover) {
    background: var(--accent);
    color: var(--accent-foreground);
  }
  
  :global([data-slot="dropdown-menu-trigger"].settings-trigger:focus-visible) {
    outline: 2px solid var(--ring);
    outline-offset: 2px;
  }
</style>
