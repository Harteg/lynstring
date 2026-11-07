import { writable } from 'svelte/store';

type PanelId = 'project' | 'status' | 'languages' | 'strings' | 'history' | 'changes';

function createPanelStateStore() {
  const { subscribe, set, update } = writable<PanelId | null>('status');

  return {
    subscribe,
    toggle: (panelId: PanelId) => {
      update(current => {
        // If clicking the same panel, close it
        if (current === panelId) {
          return null;
        }
        // Otherwise, open the clicked panel
        return panelId;
      });
    },
    close: () => set(null),
    set: (panelId: PanelId | null) => set(panelId),
    setStatus: () => update(current => 'status'),
    isExpanded: (panelId: PanelId) => {
      let current: PanelId | null = null;
      subscribe(state => current = state)();
      return current === panelId;
    }
  };
}

export const panelStateStore = createPanelStateStore();
