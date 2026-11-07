import { writable } from 'svelte/store';

/**
 * Trigger for unused strings scan. When requestScan() is called,
 * UnusedStringsManager (mounted in layout) responds by running the scan.
 * consumeScanRequest() resets the trigger so project switch etc. won't re-open the dialog.
 */
const { subscribe, set, update } = writable(0);

export const unusedStringsScanRequestStore = { subscribe };

export function requestUnusedStringsScan() {
	update((n) => n + 1);
}

export function consumeScanRequest() {
	set(0);
}
