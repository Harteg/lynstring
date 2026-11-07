<script lang="ts">
	import '../app.css';
	import { onMount, onDestroy } from 'svelte';
	import AppHeader from '$lib/components/AppHeader.svelte';
	import FloatingTranslationProgress from '$lib/components/FloatingTranslationProgress.svelte';
	import FloatingTranslationError from '$lib/components/FloatingTranslationError.svelte';
	import Toaster from '$lib/components/ui/sonner/sonner.svelte';
	import UnusedStringsManager from '$lib/components/UnusedStringsManager.svelte';
	import { networkStatus, initializeNetworkWatcher } from '$lib/stores/network';
	import { initializePreferencesStore, cleanupPreferencesStore } from '$lib/stores/preferences';
	import { loadAppSettings } from '$lib/stores/app-settings';

	let { children: pageContent } = $props();

	let cleanupNetwork: (() => void) | null = null;
	let ready = $state(false);

	onMount(async () => {
		await initializePreferencesStore();
		try {
			await loadAppSettings();
		} catch (err) {
			console.error('Failed to load settings:', err);
		}
		ready = true;
		cleanupNetwork = initializeNetworkWatcher();
	});

	onDestroy(() => {
		cleanupNetwork?.();
		cleanupPreferencesStore();
	});
</script>

<div class="app-shell">
	<!-- Dedicated drag region for macOS Overlay title bar (height 0 on Windows/Linux) -->
	<div class="titlebar-drag-region" data-tauri-drag-region></div>
	<AppHeader />

	{#if !$networkStatus.isOnline}
		<div class="offline-banner">
			<span class="offline-title">You're offline</span>
			<span class="offline-copy">Translating needs an internet connection.</span>
		</div>
	{/if}
</div>

<FloatingTranslationProgress />
<FloatingTranslationError />
<Toaster />
<UnusedStringsManager />

{#if ready}
	{@render pageContent()}
{/if}

<style>
	.app-shell {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		z-index: 40;
		display: flex;
		flex-direction: column;
		padding-top: var(--titlebar-height);
	}

	/* macOS Overlay: draggable area in title bar region (height 0 on Windows/Linux) */
	.titlebar-drag-region {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: var(--titlebar-height);
		background: var(--titlebar-background);
	}

	.offline-banner {
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
		padding: 0.5rem 1rem;
		background: color-mix(in oklch, var(--warning) 20%, transparent);
		color: var(--warning-foreground);
		text-align: center;
		font-size: 0.85rem;
	}

	.offline-title {
		font-weight: 600;
	}

	.offline-copy {
		opacity: 0.85;
	}

	/* Dark mode: amber banner with white text for better contrast */
	:global(.dark) .offline-banner,
	:global([data-theme="dark"]) .offline-banner {
		background: color-mix(in oklch, var(--warning) 35%, var(--background));
		color: #ffffff;
	}

	:global(.dark) .offline-banner .offline-copy,
	:global([data-theme="dark"]) .offline-banner .offline-copy {
		opacity: 0.9;
	}
</style>
