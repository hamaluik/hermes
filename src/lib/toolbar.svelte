/**
 * Toolbar Component
 *
 * Fixed-position toolbar at the top of the application. Contains action buttons
 * (New, Open, Save, Settings, etc.) arranged in a horizontal row.
 *
 * ## Fixed Positioning with Spacer
 *
 * The toolbar uses fixed positioning to stay visible during scrolling. However,
 * fixed elements don't occupy space in the document flow, which would cause content
 * to render underneath the toolbar.
 *
 * We solve this with a spacer element: a second div that matches the toolbar's
 * height but uses normal positioning. This spacer pushes content down, creating
 * space for the fixed toolbar.
 *
 * The height is calculated dynamically (via getComputedStyle) rather than hardcoded
 * because the toolbar height can vary based on:
 * - Font size settings
 * - Padding adjustments
 * - Different icon sizes
 * - Zoom levels
 *
 * The bindable toolbarHeight is exposed so parent components can account for it
 * in their layout calculations (e.g., viewport height calculations).
 */
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
    align-items: center;
    justify-content: flex-start;
    gap: 1ch;

    view-transition-name: toolbar;
  }
</style>
