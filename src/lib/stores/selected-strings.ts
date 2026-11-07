import { writable } from 'svelte/store';

function createSelectedStringsStore() {
  const { subscribe, update, set } = writable(new Set<string>());

  return {
    subscribe,
    toggle: (key: string) => update(strings => {
      const newStrings = new Set(strings);
      if (newStrings.has(key)) {
        newStrings.delete(key);
      } else {
        newStrings.add(key);
      }
      return newStrings;
    }),
    clear: () => set(new Set()),
    set
  };
}

export const selectedStringsStore = createSelectedStringsStore();
