import { writable } from 'svelte/store';

type NetworkState = {
  isOnline: boolean;
  lastChanged: number;
};

const supportsNavigator = typeof navigator !== 'undefined';

const { subscribe, set } = writable<NetworkState>({
  isOnline: supportsNavigator ? navigator.onLine : true,
  lastChanged: Date.now()
});

let initialized = false;
let cleanupListeners: (() => void) | null = null;

function updateStatus(isOnline: boolean) {
  set({
    isOnline,
    lastChanged: Date.now()
  });
}

export function initializeNetworkWatcher() {
  if (typeof window === 'undefined' || initialized) {
    return cleanupListeners;
  }

  const handleOnline = () => updateStatus(true);
  const handleOffline = () => updateStatus(false);

  window.addEventListener('online', handleOnline);
  window.addEventListener('offline', handleOffline);

  updateStatus(navigator.onLine);

  cleanupListeners = () => {
    window.removeEventListener('online', handleOnline);
    window.removeEventListener('offline', handleOffline);
    initialized = false;
    cleanupListeners = null;
  };

  initialized = true;
  return cleanupListeners;
}

export const networkStatus = { subscribe };

