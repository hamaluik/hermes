<!--
  Tab Component

  Individual tab within a Tabs container. Registers itself with the parent via
  Svelte's context API and conditionally renders its content based on active state.

  ## Props

  - id: Unique identifier for this tab
  - label: Display text shown in the tab button
  - index: Position of this tab (0-indexed), used to maintain correct ordering

  ## Reactive Registration Pattern

  The tab uses a reactive $effect to register/update itself in the parent's store
  whenever its id, label, or index props change. This handles cases where Svelte
  reuses a component instance with different props (e.g., when segments change).

  The effect:
  1. Detects when id changes and cleans up the old registration
  2. Updates existing registration if label or position changed
  3. Creates new registration at the correct index if this id isn't registered
  4. Auto-activates the first tab if none is active

  The index prop is critical for maintaining correct tab order when the underlying
  data changes. Without it, tabs would appear in registration order rather than
  the order they appear in the DOM.

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
    index,
    children,
  }: {
    id: string;
    label: string;
    index: number;
    children: Snippet;
  } = $props();

  const activeId: Writable<string | null> = getContext("activeId");
  const items: Writable<{ id: string; label: string }[]> = getContext("tabs");

  // track previous id to detect changes and clean up old registration
  let previousId: string | null = null;

  /**
   * Reactive Tab Registration
   *
   * Keeps the parent's tab store in sync whenever id, label, or index props
   * change. This handles component reuse scenarios where Svelte updates props
   * on an existing instance rather than creating a new one.
   *
   * The index prop ensures tabs are inserted at the correct position to match
   * the order of segments in the message, even when segments are reordered.
   */
  $effect(() => {
    if (previousId !== null && previousId !== id) {
      // id changed - remove old registration
      items.set(get(items).filter((item) => item.id !== previousId));
    }

    const currentItems = get(items);
    const existingIndex = currentItems.findIndex((item) => item.id === id);

    if (existingIndex >= 0) {
      // already registered - update label if changed, and fix position if needed
      const existing = currentItems[existingIndex];
      if (existing.label !== label || existingIndex !== index) {
        const filtered = currentItems.filter((item) => item.id !== id);
        const updated = [
          ...filtered.slice(0, index),
          { id, label },
          ...filtered.slice(index),
        ];
        items.set(updated);
      }
    } else {
      // new registration - insert at correct position
      const updated = [
        ...currentItems.slice(0, index),
        { id, label },
        ...currentItems.slice(index),
      ];
      items.set(updated);
      if (get(activeId) === null) {
        activeId.set(id);
      }
    }

    previousId = id;
  });

  /**
   * Cleanup on Unmount
   *
   * Removes this tab from the parent's store and activates another tab if
   * this one was active.
   */
  onMount(() => {
    return () => {
      items.set(get(items).filter((item) => item.id !== id));
      const remaining = get(items);
      if (remaining.length > 0) {
        activeId.set(remaining[0].id);
      } else {
        activeId.set(null);
      }
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
