<script lang="ts">
  import { setContext, type Snippet } from "svelte";
  import { get, writable, type Writable } from "svelte/store";
  import IconWizard from "./icons/IconWizard.svelte";

  let {
    setactive = $bindable(),
    addMenu,
    children,
  }: {
    setactive?: (id: string) => void;
    addMenu?: Snippet<[{ closeMenu: () => void }]>;
    children: Snippet;
  } = $props();

  const tabs: Writable<{ id: string; label: string; onWizard?: () => void }[]> =
    writable<{ id: string; label: string; onWizard?: () => void }[]>([]);
  const activeId = writable<string | null>(null);
  let activeTabIsMissing: boolean = $state(false);

  let showAddMenu = $state(false);

  setContext("tabs", tabs);
  setContext("activeId", activeId);

  const transitionTab = (id: string) => {
    activeId.set(id);
  };

  $effect(() => {
    setactive = (id: string) => {
      // so we don't get weird selection issues
      setTimeout(() => {
        activeId.set(id);
      }, 0);
    };
  });

  activeId.subscribe((id) => {
    const tabList = get(tabs);
    // Only show missing tab message if there are tabs and the active one is missing
    activeTabIsMissing =
      tabList.length > 0 && !tabList.some((tab) => tab.id === id);
  });
</script>

<div class="tabs">
  <ul>
    {#each $tabs as { id, label, onWizard }}
      <li class="tab" class:active={id === $activeId}>
        <button
          onclick={() => {
            transitionTab(id);
          }}
        >
          {label}
        </button>
        {#if onWizard && id === $activeId}
          <button
            class="wizard"
            aria-label="Open Wizard"
            title="Open Wizard"
            onclick={() => {
              onWizard?.();
            }}
          >
            <IconWizard />
          </button>
        {/if}
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
  {#if activeTabIsMissing}
    <div class="missing-tab">
      <p>(no configuration found for this message segment)</p>
    </div>
  {/if}
</div>

<style>
  .tabs {
    display: flex;
    flex-direction: column;
    gap: 0;
    align-items: stretch;
    justify-content: flex-start;
    font-size: smaller;
    height: 100%;

    > ul {
      display: flex;
      list-style: none;
      padding: 0;
      margin: 0 0 0 1ch;
      gap: 1ch;
      isolation: isolate;
      z-index: 2;
      flex: 0 0 auto;
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
      color: var(--col-text);
    }

    .tab button.wizard {
      width: 2em;
      height: 2em;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      padding: 0.5rem;
      margin: 0 0.5rem 0 -1rem;
      width: auto;
    }

    .tab button.wizard:hover {
      color: var(--col-gold);
      cursor: pointer;
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

  .missing-tab {
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
