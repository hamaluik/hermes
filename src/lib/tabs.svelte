<script lang="ts">
  import { setContext, type Snippet } from "svelte";
  import { writable, type Writable } from "svelte/store";

  let {
    addMenu,
    children,
  }: {
    addMenu?: Snippet<[{ closeMenu: () => void }]>;
    children: Snippet;
  } = $props();

  const tabs: Writable<{ id: symbol; label: string }[]> = writable<
    { id: symbol; label: string }[]
  >([]);
  const activeId = writable<symbol | null>(null);

  let showAddMenu = $state(false);

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
    {#if addMenu}
      <li class="tab">
        <button
          aria-label="Add Segment"
          title="Add Segment"
          onclick={() => {
            showAddMenu = !showAddMenu;
          }}>+</button
        >
        {#if showAddMenu}
          <div class="add-menu">
            {@render addMenu?.({
              closeMenu: () => {
                showAddMenu = false;
              },
            })}
          </div>
        {/if}
      </li>
    {/if}
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
    font-size: smaller;

    > ul {
      display: flex;
      list-style: none;
      padding: 0;
      margin: 0 0 0 1ch;
      gap: 1ch;
      isolation: isolate;
      z-index: 2;
    }

    .tab {
      border: 1px solid var(--col-highlightHigh);
      border-bottom: none;
      border-radius: 4px 4px 0 0;
      background-color: var(--col-surface);
      position: relative;
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

    .add-menu {
      position: absolute;
      top: 75%;
      left: 50%;
      min-width: 10ch;
      background-color: var(--col-surface);
      border: 1px solid var(--col-highlightHigh);
      border-radius: 4px;
      padding: 0.5rem;
      z-index: 3;
    }
  }
</style>
