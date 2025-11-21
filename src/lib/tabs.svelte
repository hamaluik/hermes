<!--
  Tabs Component

  Container component for managing tabbed navigation. Child Tab components register
  themselves automatically via Svelte's context API, enabling a declarative API where
  tabs are defined in markup rather than via props.

  ## Context API Usage

  We provide two contexts to child components:
  - "tabs": Writable store of tab metadata (id, label, wizard callback)
  - "activeId": Writable store of currently active tab ID

  Child Tab components subscribe to activeId to know when they're active, and push
  their metadata into the tabs store. This creates a two-way communication channel
  without requiring explicit prop drilling through intermediate components.

  Why use context instead of props? Because tab content can be arbitrarily nested
  (e.g., inside conditional blocks, each blocks, etc.), and manually threading
  tab registration callbacks through all that structure would be cumbersome and
  error-prone.

  ## Add Menu Pattern

  The optional addMenu snippet provides UI for adding new segments to the message.
  When provided, a "+" button appears after all tabs. Clicking it reveals a dropdown
  (provided by the parent via snippet) showing available segment types.

  The closeMenu callback is passed to the snippet so segment selection can dismiss
  the menu automatically. This keeps the implementation of the menu content in the
  parent (which knows what segments are available) while the tabs component handles
  the show/hide logic.

  ## Active Tab Tracking

  We track whether the active tab ID exists in the current tab list. This handles
  edge cases where:
  - User edits the raw message and deletes a segment
  - The previously active tab no longer exists
  - We need to show a fallback message instead of blank content

  The subscription pattern ensures activeTabIsMissing updates whenever either the
  tab list or active ID changes.
-->
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

  /**
   * Context API Setup
   *
   * Child Tab components will access these contexts to:
   * 1. Register themselves in the tabs list
   * 2. Know when they're the active tab (to show/hide content)
   */
  setContext("tabs", tabs);
  setContext("activeId", activeId);

  const transitionTab = (id: string) => {
    activeId.set(id);
  };

  /**
   * External Tab Activation
   *
   * Exposes a setactive function that parent components can call to programmatically
   * change tabs (e.g., "tabs follow cursor" feature). The setTimeout avoids selection
   * issues when tab changes happen during text input events.
   */
  $effect(() => {
    setactive = (id: string) => {
      setTimeout(() => {
        activeId.set(id);
      }, 0);
    };
  });

  /**
   * Missing Tab Detection
   *
   * Tracks whether the currently active tab actually exists in the tab list. This
   * can happen when segments are deleted from the raw message - the tab disappears
   * but activeId still references it.
   *
   * We show a fallback message instead of leaving the content area blank, which
   * would be confusing to users.
   */
  activeId.subscribe((id) => {
    const tabList = get(tabs);
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
