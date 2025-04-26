<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    children,
    title,
    onclick,
  }: { children: Snippet; title: string; onclick?: () => void } = $props();
</script>

<button
  class="toolbar-button"
  aria-label={title}
  {title}
  {onclick}
  disabled={!onclick}
>
  {@render children?.()}
</button>

<style>
  .toolbar-button {
    background: none;
    border: none;
    color: var(--col-text);
    font-size: 1rem;
    padding: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.1s ease-in-out;
  }

  .toolbar-button:hover {
    color: var(--col-gold);
  }

  .toolbar-button:disabled {
    color: var(--col-muted);
    cursor: not-allowed;
  }
</style>
