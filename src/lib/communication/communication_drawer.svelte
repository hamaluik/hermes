<!--
  Communication Drawer Component

  A collapsible bottom drawer containing tabs for Send and Listen functionality.
  Provides an integrated, non-modal workflow for HL7 message communication.

  Layout (expanded):
  ┌─────────────────────────────────────────────────────┐
  │ [Send] [Listen (3)]                    [▼ Collapse] │
  ├─────────────────────────────────────────────────────┤
  │                                                     │
  │  Tab content (SendTab or ListenTab)                 │
  │                                                     │
  └─────────────────────────────────────────────────────┘

  Layout (collapsed):
  ┌─────────────────────────────────────────────────────┐
  │ [Send] [Listen (3)]                     [▲ Expand]  │
  └─────────────────────────────────────────────────────┘

  Features:
  - Tab switching between Send and Listen modes
  - Badge on Listen tab shows unread message count
  - Collapse/expand toggle to minimise when not needed
  - Resizable height via drag handle (future enhancement)
-->
<script lang="ts">
  import type { Settings } from "../../settings";
  import type { Writable } from "svelte/store";
  import { onMount } from "svelte";
  import SendTab from "./send_tab.svelte";
  import ListenTab from "./listen_tab.svelte";
  import IconChevronDown from "$lib/icons/IconChevronDown.svelte";
  import IconChevronUp from "$lib/icons/IconChevronUp.svelte";

  type Tab = "send" | "listen";
  type ListenedMessage = { message: string; unread: boolean; timestamp?: Date };

  let {
    settings,
    message,
    listening,
    listenedMessages,
    expanded = $bindable(true),
    activeTab = $bindable<Tab>("send"),
    height = 300,
    onLoadToEditor,
  }: {
    settings: Settings;
    message: string;
    listening: Writable<boolean>;
    listenedMessages: Writable<ListenedMessage[]>;
    expanded?: boolean;
    activeTab?: Tab;
    height?: number;
    onLoadToEditor?: (message: string) => void;
  } = $props();

  // Track unread count from store
  let unreadCount: number = $state(0);

  $effect(() => {
    const unsub = listenedMessages.subscribe((msgs) => {
      unreadCount = msgs.filter((m) => m.unread).length;
    });
    return unsub;
  });

  function toggleExpanded() {
    expanded = !expanded;
  }

  function selectTab(tab: Tab) {
    activeTab = tab;
    if (!expanded) {
      expanded = true;
    }
  }

  function handleBackdropClick() {
    expanded = false;
  }
</script>

<button
  class="drawer-backdrop"
  class:visible={expanded}
  onclick={handleBackdropClick}
  aria-label="Close drawer"
></button>

<div
  class="communication-drawer"
  class:visible={expanded}
  style="--drawer-height: {height}px; transform: translateY({expanded ? '0' : '100%'}); transition: transform 0.2s ease-out;"
>
  <div class="tab-bar">
    <div class="tabs">
      <button
        class="tab"
        class:active={activeTab === "send"}
        onclick={() => selectTab("send")}
      >
        Send
      </button>
      <button
        class="tab"
        class:active={activeTab === "listen"}
        onclick={() => selectTab("listen")}
      >
        Listen
        {#if unreadCount > 0}
          <span class="badge">{unreadCount}</span>
        {/if}
      </button>
    </div>

    <button class="collapse-toggle" onclick={toggleExpanded}>
      <IconChevronDown />
      <span class="toggle-text">Close</span>
    </button>
  </div>

  <div class="tab-content">
    {#if activeTab === "send"}
      <SendTab {settings} {message} />
    {:else}
      <ListenTab
        {settings}
        {listening}
        messages={listenedMessages}
        {onLoadToEditor}
      />
    {/if}
  </div>
</div>

<style>
  .communication-drawer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 100;
    display: flex;
    flex-direction: column;
    background: var(--col-base);
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.25);
    border-radius: 12px 12px 0 0;
    height: var(--drawer-height);
    min-height: var(--drawer-height);
    max-height: 80vh;
    overflow: hidden;
  }

  .tab-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 0.5rem;
    background: var(--col-overlay);
    border-bottom: 1px solid var(--col-highlightMed);
    flex-shrink: 0;
    border-radius: 12px 12px 0 0;
  }

  .tabs {
    display: flex;
    gap: 0;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 0.5ch;
    padding: 0.5rem 1rem;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--col-subtle);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      color 0.15s ease,
      border-color 0.15s ease;

    &:hover {
      color: var(--col-text);
    }

    &.active {
      color: var(--col-text);
      border-bottom-color: var(--col-iris);
    }
  }

  .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.25rem;
    height: 1.25rem;
    padding: 0 0.375rem;
    background: var(--col-love);
    color: var(--col-base);
    font-size: 0.625rem;
    font-weight: 700;
    border-radius: 9999px;
  }

  .collapse-toggle {
    display: flex;
    align-items: center;
    gap: 0.25ch;
    padding: 0.375rem 0.5rem;
    background: none;
    border: none;
    color: var(--col-muted);
    font-size: 0.75rem;
    cursor: pointer;

    &:hover {
      color: var(--col-text);
    }

    .toggle-text {
      /* Hide text on small screens */
      @media (max-width: 600px) {
        display: none;
      }
    }
  }

  .tab-content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* Backdrop */
  .drawer-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: 99;
    border: none;
    cursor: pointer;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.2s ease;
  }

  .drawer-backdrop.visible {
    opacity: 1;
    pointer-events: auto;
  }
</style>
