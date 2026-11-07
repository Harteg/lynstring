<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { Component } from 'svelte';

  interface Props {
    /** Icon component to render */
    icon: Component;
    /** Title / headline */
    title: string;
    /** Optional description */
    description?: string;
    /** Primary action slot (e.g. Translate button) */
    action?: Snippet;
    /** Optional secondary/extra content slot */
    children?: Snippet;
    /** Visual variant */
    variant?: 'default' | 'warning' | 'blocker';
  }
  let { icon: Icon, title, description, action, children, variant = 'default' }: Props = $props();
</script>

<div class="suggestion-card" class:suggestion-warning={variant === 'warning'} class:suggestion-blocker={variant === 'blocker'}>
  <div class="suggestion-header">
    <div class="icon-wrap">
      <Icon class="w-5 h-5" />
    </div>
    <div class="suggestion-text">
      <h3 class="suggestion-title">{title}</h3>
      {#if description}
        <p class="suggestion-desc">{description}</p>
      {/if}
    </div>
    {#if action}
      <div class="suggestion-action">
        {@render action()}
      </div>
    {/if}
  </div>
  {#if children}
    <div class="suggestion-extra">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .suggestion-card {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: 1rem 1.25rem;
    border-radius: 12px;
    border: 1px solid var(--border);
    background: var(--card);
    transition: border-color 0.2s, background 0.2s;
  }

  .suggestion-warning {
    border-color: color-mix(in oklch, var(--warning) 30%, var(--border));
    background: color-mix(in oklch, var(--warning) 5%, var(--card));
  }

  .suggestion-blocker {
    border-color: color-mix(in oklch, var(--destructive) 25%, var(--border));
    background: color-mix(in oklch, var(--destructive) 8%, var(--card));
  }

  .suggestion-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    width: 100%;
  }

  .icon-wrap {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    background: color-mix(in oklch, var(--primary) 12%, transparent);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--primary);
    flex-shrink: 0;
  }

  .suggestion-warning .icon-wrap {
    background: color-mix(in oklch, var(--warning) 15%, transparent);
    color: var(--warning);
  }

  .suggestion-blocker .icon-wrap {
    background: color-mix(in oklch, var(--destructive) 15%, transparent);
    color: var(--destructive);
  }

  .suggestion-text {
    flex: 1;
    min-width: 0;
  }

  .suggestion-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--foreground);
    margin: 0;
  }

  .suggestion-desc {
    font-size: 0.8125rem;
    color: var(--muted-foreground);
    margin: 0;
  }

  .suggestion-action {
    flex-shrink: 0;
  }

  .suggestion-extra {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }
</style>
