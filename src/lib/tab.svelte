<script lang="ts">
  import { getContext, onMount, type Snippet } from "svelte";
  import { get, type Writable } from "svelte/store";

  let {
    id,
    label,
    children,
  }: {
    id?: symbol;
    label: string;
    children: Snippet;
  } = $props();

  if (!id) {
    id = Symbol();
  }

  const activeId: Writable<symbol | null> = getContext("activeId");
  onMount(() => {
    const items: Writable<{ id: symbol; label: string }[]> = getContext("tabs");
    items.set([...get(items), { id, label }]);

    if (get(activeId) === null) {
      activeId.set(id);
    }

    return () => {
      items.set(get(items).filter((item) => item.id !== id));
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
    height: calc-size(auto, size);
    isolation: isolate;
    z-index: 1;
  }
</style>
