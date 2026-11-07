<script lang="ts">
  /** LynString wordmark — lightning bolt + "Lyn"String. Use with brand colors. */
  interface Props {
    class?: string;
    size?: 'sm' | 'md' | 'lg';
  }
  let { class: className = '', size = 'md' }: Props = $props();
  const iconSizes = { sm: 20, md: 28, lg: 32 };
  const textSizes = { sm: 'text-base', md: 'text-xl', lg: 'text-2xl' };
</script>

<div class="lynstring-wordmark {className}">
  <svg
    class="lynstring-icon flex-shrink-0"
    width={iconSizes[size]}
    height={iconSizes[size]}
    viewBox="0 0 32 32"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
  >
    <path d="M18 4L8 18H15L14 28L24 14H17L18 4Z" fill="currentColor" />
    <path
      d="M6 20H10M22 12H26"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      opacity="0.3"
    />
  </svg>
  <span class="lynstring-text {textSizes[size]}">
    <span class="lynstring-lyn">Lyn</span>String
  </span>
</div>

<style>
  .lynstring-wordmark {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--foreground);
  }
  .lynstring-icon {
    color: var(--primary);
  }
  .lynstring-text {
    font-family: 'Space Grotesk', system-ui, sans-serif;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
  }
  .lynstring-lyn {
    color: var(--primary);
  }
</style>
