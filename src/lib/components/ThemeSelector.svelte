<script lang="ts">
  import { preferencesStore, type ThemeMode } from '$lib/stores/preferences';
  import { updatePreferences } from '$lib/stores/preferences';
  import { Button } from '$lib/components/ui/button';
  import MonitorIcon from '@lucide/svelte/icons/monitor';
  import SunIcon from '@lucide/svelte/icons/sun';
  import MoonIcon from '@lucide/svelte/icons/moon';

  // Svelte $ syntax for store subscription
  $: currentTheme = $preferencesStore.theme;

  function setTheme(theme: ThemeMode) {
    updatePreferences(prefs => ({ ...prefs, theme }));
  }
</script>

<div class="rounded-md border flex p-0.5 shrink-0">
  <Button
    variant="ghost"
    size="icon"
    onclick={() => setTheme('system')}
    class="h-7 w-7 min-w-7 p-0 {currentTheme === 'system' ? 'bg-muted' : ''}"
    aria-pressed={currentTheme === 'system'}
    aria-label="System theme"
  >
    <MonitorIcon class="h-3.5 w-3.5" />
  </Button>
  <Button
    variant="ghost"
    size="icon"
    onclick={() => setTheme('light')}
    class="h-7 w-7 min-w-7 p-0 {currentTheme === 'light' ? 'bg-muted' : ''}"
    aria-pressed={currentTheme === 'light'}
    aria-label="Light theme"
  >
    <SunIcon class="h-3.5 w-3.5" />
  </Button>
  <Button
    variant="ghost"
    size="icon"
    onclick={() => setTheme('dark')}
    class="h-7 w-7 min-w-7 p-0 {currentTheme === 'dark' ? 'bg-muted' : ''}"
    aria-pressed={currentTheme === 'dark'}
    aria-label="Dark theme"
  >
    <MoonIcon class="h-3.5 w-3.5" />
  </Button>
</div>
