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
