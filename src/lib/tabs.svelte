<script lang="ts">
  import { setContext, type Snippet } from "svelte";
  import { writable, type Writable } from "svelte/store";

  let {
    children,
  }: {
    children: Snippet;
  } = $props();

  const tabs: Writable<{ id: symbol; label: string }[]> = writable<
    { id: symbol; label: string }[]
  >([]);
  const activeId = writable<symbol | null>(null);

  setContext("tabs", tabs);
  setContext("activeId", activeId);

  const transitionTab = (id: symbol) => {
    activeId.set(id);
    // TODO: better view transitions
    /*if (!document.startViewTransition) {
      activeId.set(id);
      return;
    }

    document.startViewTransition(() => {
      activeId.set(id);
    });*/
  };
</script>

<div class="tabs">
  <ul>
    {#each $tabs as { id, label }}
      <li class="tab" class:active={id === $activeId}>
        <button
          onclick={() => {
            transitionTab(id);
          }}
        >
          {label}
        </button>
      </li>
    {/each}
  </ul>
  {@render children?.()}
</div>

<style>
  .tabs {
    display: flex;
    flex-direction: column;
    gap: 0;
    align-items: stretch;
    justify-content: flex-start;

    ul {
      display: flex;
      list-style: none;
      padding: 0;
      margin: 0 0 0 1ch;
      gap: 1ch;
    }

    .tab {
      border: 1px solid var(--col-highlightHigh);
      border-bottom: none;
      border-radius: 4px 4px 0 0;
      background-color: var(--col-surface);
    }

    .tab.active {
      --active-color: var(--col-surface);
      background-color: var(--active-color);
      border-color: var(--col-muted);
      position: relative;

      &::after {
        content: " ";
        position: absolute;
        bottom: -1px;
        left: 0;
        right: 0;
        border-bottom: 1px solid var(--active-color);
        z-index: 1;
      }
    }

    .tab button {
      background-color: transparent;
      color: var(--col-text);
      border: none;
      padding: 0.5rem 1rem;
      cursor: pointer;
    }

    .tab.active button {
      background-color: var(--col-primary);
      color: white;
    }
  }
</style>
