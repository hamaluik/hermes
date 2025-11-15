/**
 * Toolbar Button Component
 *
 * Button for toolbar actions with icon support. Automatically disables when no
 * onclick handler is provided, which is used for conditionally-enabled actions
 * like Save (only enabled when there are unsaved changes).
 *
 * The button contains a snippet (typically an icon component) rather than text,
 * providing a compact visual interface. Tooltips (via title attribute) show the
 * action name on hover.
 */
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
