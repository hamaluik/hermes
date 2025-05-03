<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    count,
    children,
  }: {
    count: number;
    children: Snippet;
  } = $props();
</script>

<div class="notification-icon">
  {@render children?.()}
  {#if count > 0}
    <div class="notification-count">{count}</div>
  {/if}
</div>

<style>
  .notification-icon {
    position: relative;
    display: inline-block;
  }

  .notification-count {
    position: absolute;
    top: -5px;
    right: -5px;
    background-color: var(--col-love);
    color: var(--col-text);
    border-radius: 50%;
    aspect-ratio: 1 / 1;
    height: 1lh;
    font-size: x-small;

    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
</style>
