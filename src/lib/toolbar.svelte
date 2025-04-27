<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    children,
    toolbarHeight = $bindable(),
  }: { children: Snippet; toolbarHeight?: string } = $props();

  let toolbarElement: HTMLElement | null = null;
  let spacerElement: HTMLElement | null = null;

  $effect(() => {
    if (toolbarElement && spacerElement) {
      toolbarHeight = getComputedStyle(toolbarElement).height;
      spacerElement.style.height = toolbarHeight;
    } else {
      toolbarHeight = undefined;
    }
  });
</script>

<div class="toolbar" bind:this={toolbarElement}>
  {@render children?.()}
</div>
<div class="toolbar-spacer" bind:this={spacerElement}></div>

<style>
  .toolbar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    background-color: var(--col-overlay);
    border-bottom: 1px solid var(--col-highlightHigh);
    padding: 0.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    isolation: isolate;
    z-index: 1000;

    display: flex;
    flex-direction: row;
    align-items: baseline;
    justify-content: flex-start;
    gap: 1ch;
  }
</style>
