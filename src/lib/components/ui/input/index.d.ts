import { SvelteComponentTyped } from 'svelte';

export class Input extends SvelteComponentTyped<{
  type?: string;
  value?: string;
  placeholder?: string;
  class?: string;
}> {}
