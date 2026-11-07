<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import Loader2 from '@lucide/svelte/icons/loader-2';
  import CheckCircle from '@lucide/svelte/icons/check-circle';
  import XCircle from '@lucide/svelte/icons/x-circle';
  import { appSettingsStore, MODELS, saveAppSettings, testApiKey, removeApiKey } from '$lib/stores/app-settings';

  let { onSaved = undefined }: { onSaved?: () => void } = $props();

  const hasKey = $derived($appSettingsStore?.has_api_key ?? false);
  const keyFromEnv = $derived($appSettingsStore?.api_key_from_env ?? false);

  let apiKey = $state('');
  let model = $state($appSettingsStore?.model ?? MODELS[0].id);
  let saving = $state(false);
  let result: { success: boolean; message: string } | null = $state(null);

  // Saving needs a new key unless one is already configured (then only the model changes)
  const canSave = $derived(!saving && (apiKey.trim() !== '' || hasKey));

  async function handleSave() {
    saving = true;
    result = null;
    try {
      const newKey = apiKey.trim() || null;
      const test = await testApiKey(newKey, model);
      result = test;
      if (test.success) {
        await saveAppSettings(newKey, model);
        apiKey = '';
        onSaved?.();
      }
    } catch (e) {
      result = { success: false, message: String(e) };
    } finally {
      saving = false;
    }
  }

  async function handleRemove() {
    await removeApiKey();
    result = null;
  }
</script>

<form class="api-key-form" onsubmit={(e) => { e.preventDefault(); handleSave(); }}>
  <div class="field">
    <Label for="api-key">Anthropic API key</Label>
    <Input
      id="api-key"
      type="password"
      autocomplete="off"
      spellcheck={false}
      placeholder={hasKey ? `Saved (${$appSettingsStore?.api_key_hint})${keyFromEnv ? ' from ANTHROPIC_API_KEY' : ''}. Paste a new key to replace it` : 'sk-ant-...'}
      bind:value={apiKey}
    />
    <p class="hint">
      Stored only on this computer. Create a key in the
      <button type="button" class="link" onclick={() => openUrl('https://console.anthropic.com/settings/keys')}>Claude Console</button>.
      {#if hasKey && !keyFromEnv}
        · <button type="button" class="link" onclick={handleRemove}>Remove saved key</button>
      {/if}
    </p>
  </div>

  <div class="field">
    <Label>Model</Label>
    <div class="models" role="radiogroup">
      {#each MODELS as m (m.id)}
        <button
          type="button"
          role="radio"
          aria-checked={model === m.id}
          class="model-option"
          class:selected={model === m.id}
          onclick={() => (model = m.id)}
        >
          <span class="model-name">{m.name}</span>
          <span class="model-note">{m.note}</span>
        </button>
      {/each}
    </div>
    <p class="hint">Prices are per million input / output tokens, billed to your Anthropic account.</p>
  </div>

  {#if result}
    <div class="result" class:error={!result.success}>
      {#if result.success}
        <CheckCircle class="h-4 w-4" /> Saved. Your key works.
      {:else}
        <XCircle class="h-4 w-4" /> {result.message}
      {/if}
    </div>
  {/if}

  <Button type="submit" disabled={!canSave} class="w-full">
    {#if saving}
      <Loader2 class="h-4 w-4 mr-2 animate-spin" /> Checking key...
    {:else}
      Save
    {/if}
  </Button>
</form>

<style>
  .api-key-form {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    width: 100%;
    text-align: left;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .hint {
    margin: 0;
    font-size: 0.75rem;
    line-height: 1.1rem;
    color: var(--muted-foreground);
  }

  .link {
    background: none;
    border: none;
    padding: 0;
    color: var(--primary);
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
    font-size: inherit;
  }

  .models {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .model-option {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    background: transparent;
    color: var(--foreground);
    font-size: 0.875rem;
    text-align: left;
  }

  .model-option.selected {
    border-color: var(--primary);
    background: color-mix(in oklch, var(--primary) 10%, transparent);
  }

  .model-name {
    font-weight: 500;
  }

  .model-note {
    font-size: 0.75rem;
    color: var(--muted-foreground);
  }

  .result {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8125rem;
    color: var(--success, #16a34a);
  }

  .result.error {
    color: var(--destructive);
  }
</style>
