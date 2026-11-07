<script lang="ts">
  let className = '';
  export { className as class };
</script>

<div class={`rounded-lg border bg-card text-card-foreground shadow-sm ${className}`}>
  <slot />
</div>
