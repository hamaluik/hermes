<!--
  Tab Component

  Individual tab within a Tabs container. Registers itself with the parent via
  Svelte's context API and conditionally renders its content based on active state.

  ## Self-Registration Pattern

  On mount, the tab:
  1. Retrieves the "tabs" context (writable store from parent)
  2. Adds its own metadata (id, label) to the store
  3. If no tab is currently active, activates itself (first tab wins)

  This allows tabs to be declared in markup without manually managing a tab list
  in the parent component. The parent just renders whatever tabs exist in its
  children, and they handle their own registration.

  ## Cleanup on Unmount

  When a tab is removed (e.g., user deletes a segment from the message), the
  cleanup function:
  1. Removes the tab from the parent's tab list
  2. If this tab was active, activates the first remaining tab

  This ensures the UI stays in a valid state when tabs disappear dynamically.

  ## Conditional Rendering

  Only the active tab renders its content. This is important because tab content
  can contain expensive computations, large forms, or real-time data subscriptions.
  Rendering all tabs but hiding inactive ones via CSS would waste resources.
-->
<script lang="ts">
  import { getContext, onMount, type Snippet } from "svelte";
  import { get, type Writable } from "svelte/store";

  let {
    id,
    label,
    children,
  }: {
    id: string;
    label: string;
    children: Snippet;
  } = $props();

  const activeId: Writable<string | null> = getContext("activeId");

  /**
   * Tab Registration and Lifecycle
   *
   * Registers this tab with the parent Tabs component on mount, and cleans up
   * on unmount. Auto-activates if this is the first tab.
   */
  onMount(() => {
    const items: Writable<{ id: string; label: string }[]> = getContext("tabs");
    items.set([...get(items), { id, label }]);

    if (get(activeId) === null) {
      activeId.set(id);
    }

    return () => {
      items.set(get(items).filter((item) => item.id !== id));
      activeId.set(get(items).length > 0 ? get(items)[0].id : null);
    };
  });
</script>

{#if id === $activeId}
  <div class="tab-content">
    {@render children?.()}
  </div>
{/if}

<style>
  .tab-content {
    padding: 0.5lh 1ch;
    margin: 0;
    border: 1px solid var(--col-muted);
    background-color: var(--col-surface);
    border-radius: 4px;
    view-transition-name: tab-content;
    isolation: isolate;
    z-index: 1;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
</style>
