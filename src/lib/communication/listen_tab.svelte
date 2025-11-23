<!--
  Listen Tab Component

  Tab content for running an MLLP listen server to receive incoming HL7 messages.
  Used within the Communication Drawer to provide passive message listening.

  Layout:
  ┌──────────────┬────────────────────┬───────────────────────────────┐
  │ Port: [2575] │ Received (5) Clear │ Selected Message              │
  │              │ ────────────────── │ ─────────────────────────────  │
  │ [Start/Stop] │ ● ADT^A01  10:32am │ MSH|^~\&|ADT^A01|...          │
  │              │ ○ ORM^O01  10:31am │ PID|1||12345^^^MRN...         │
  │ Listening... │ ○ ADT^A04  10:30am │                               │
  │              │ ○ ORM^O01  10:29am │ [Load to Editor]              │
  └──────────────┴────────────────────┴───────────────────────────────┘

  Message List:
  - ● = unread (filled circle)
  - ○ = read (empty circle)
  - Clicking a message selects it and marks it as read
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Settings } from "../../settings";
  import type { Writable } from "svelte/store";
  import { startListening, stopListening } from "./listen";
  import IconListen from "$lib/icons/IconListen.svelte";
  import IconSpinner from "$lib/icons/IconSpinner.svelte";
  import MessageEditor from "$lib/editor/message_editor.svelte";

  type ListenedMessage = { message: string; unread: boolean; timestamp?: Date };

  let {
    settings,
    listening,
    messages,
    onLoadToEditor,
  }: {
    settings: Settings;
    listening: Writable<boolean>;
    messages: Writable<ListenedMessage[]>;
    onLoadToEditor?: (message: string) => void;
  } = $props();

  // Local state
  let port: number = $state(settings.listenPort);

  // Register callback to sync state after settings load from disk
  onMount(() => {
    settings.onListenSettingsChanged = (p) => {
      port = p;
    };
  });

  onDestroy(() => {
    settings.onListenSettingsChanged = null;
  });
  let selectedIndex: number | null = $state(null);
  let isStarting: boolean = $state(false);
  let error: string | null = $state(null);

  /**
   * Port validation matching database wizard inputs.
   * Port: Standard TCP port range (1-65535)
   */
  const isPortValid = $derived(port >= 1 && port <= 65535);

  // Derived state
  let isListening: boolean = $state(false);
  let messageList: ListenedMessage[] = $state([]);

  // Subscribe to stores
  $effect(() => {
    const unsubListening = listening.subscribe((value) => {
      isListening = value;
    });
    const unsubMessages = messages.subscribe((value) => {
      messageList = value;
      // Auto-select newest message if nothing is selected
      if (selectedIndex === null && value.length > 0) {
        selectedIndex = value.length - 1;
        markAsRead(selectedIndex);
      }
    });
    return () => {
      unsubListening();
      unsubMessages();
    };
  });

  // Selected message
  let selectedMessage: string | null = $derived(
    selectedIndex !== null && messageList[selectedIndex]
      ? messageList[selectedIndex].message
      : null,
  );

  /**
   * Debounced settings persistence for port.
   */
  const debounce = (callback: (...args: any[]) => void, wait: number) => {
    let timeoutId: number | undefined = undefined;
    return (...args: any[]) => {
      window.clearTimeout(timeoutId);
      timeoutId = window.setTimeout(() => {
        callback.apply(null, args);
      }, wait);
    };
  };

  const savePort = debounce((_port: number) => {
    settings.listenPort = _port;
  }, 500);

  $effect(() => {
    savePort(port);
  });

  /**
   * Start the listen server.
   */
  async function handleStart() {
    if (isListening || isStarting) return;

    isStarting = true;
    error = null;

    try {
      await startListening(null, port, listening);
    } catch (e) {
      console.error("Failed to start listening:", e);
      error = String(e);
    } finally {
      isStarting = false;
    }
  }

  /**
   * Stop the listen server.
   */
  async function handleStop() {
    if (!isListening) return;

    try {
      await stopListening(listening);
    } catch (e) {
      console.error("Failed to stop listening:", e);
      error = String(e);
    }
  }

  /**
   * Select a message and mark it as read.
   */
  function selectMessage(index: number) {
    selectedIndex = index;
    markAsRead(index);
  }

  /**
   * Mark a message as read.
   */
  function markAsRead(index: number) {
    messages.update((msgs) => {
      if (msgs[index] && msgs[index].unread) {
        const updated = [...msgs];
        updated[index] = { ...updated[index], unread: false };
        return updated;
      }
      return msgs;
    });
  }

  /**
   * Clear all messages.
   */
  function clearMessages() {
    messages.set([]);
    selectedIndex = null;
  }

  /**
   * Extract message type from HL7 message (e.g., "ADT^A01").
   */
  function getMessageType(message: string): string {
    const lines = message.split("\n");
    const mshLine = lines.find((l) => l.startsWith("MSH"));
    if (!mshLine) return "Unknown";

    const fields = mshLine.split("|");
    if (fields.length >= 10) {
      return fields[8] || "Unknown"; // MSH.9 is the message type
    }
    return "Unknown";
  }

  /**
   * Format timestamp for display.
   */
  function formatTime(date?: Date): string {
    if (!date) return "";
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }

  // Determine button state
  let canStart: boolean = $derived(
    !isListening && !isStarting && isPortValid,
  );
</script>

<div class="listen-tab">
  <div class="controls">
    <div class="form-row">
      <label for="listen-port">Port</label>
      <input
        type="number"
        id="listen-port"
        bind:value={port}
        min="1"
        max="65535"
        placeholder="2575"
        required
        disabled={isListening}
        class:invalid={!isPortValid}
      />
    </div>

    {#if isListening}
      <button class="listen-button stop" onclick={handleStop}>
        <IconListen />
        Stop Listening
      </button>
    {:else}
      <button
        class="listen-button start"
        onclick={handleStart}
        disabled={!canStart}
      >
        {#if isStarting}
          <IconSpinner />
          Starting...
        {:else}
          <IconListen />
          Start Listening
        {/if}
      </button>
    {/if}

    {#if isListening}
      <div class="status active">
        Listening on port {port}
      </div>
    {/if}

    {#if error}
      <div class="status error">{error}</div>
    {/if}
  </div>

  <div class="message-list-section">
    <div class="message-list-header">
      <span>Received ({messageList.length})</span>
      {#if messageList.length > 0}
        <button class="clear-button" onclick={clearMessages}>Clear</button>
      {/if}
    </div>

    <div class="message-list">
      {#if messageList.length === 0}
        <div class="empty-list">
          {#if isListening}
            Waiting for messages...
          {:else}
            Start listening to receive messages
          {/if}
        </div>
      {:else}
        {#each messageList as msg, i}
          <button
            class="message-item"
            class:selected={selectedIndex === i}
            class:unread={msg.unread}
            onclick={() => selectMessage(i)}
          >
            <span class="unread-indicator">{msg.unread ? "●" : "○"}</span>
            <span class="message-type">{getMessageType(msg.message)}</span>
            <span class="message-time">{formatTime(msg.timestamp)}</span>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <div class="message-panel">
    <div class="panel-header">
      <span>Selected Message</span>
      {#if selectedMessage}
        <div class="panel-actions">
          {#if onLoadToEditor}
            <button
              class="action-button"
              onclick={() => onLoadToEditor?.(selectedMessage!)}
            >
              Load to Editor
            </button>
          {/if}
        </div>
      {/if}
    </div>
    <div class="message-content">
      {#if selectedMessage}
        <MessageEditor
          message={selectedMessage}
          readonly={true}
          placeholder="No message selected"
        />
      {:else}
        <div class="empty-state">
          {#if messageList.length === 0}
            No messages received yet
          {:else}
            Select a message to view
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .listen-tab {
    display: flex;
    flex-direction: row;
    gap: 1rem;
    flex: 1;
    min-height: 0;
    padding: 0.75rem;
  }

  .controls {
    flex: 0 0 auto;
    width: 140px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .message-list-section {
    flex: 0 0 auto;
    width: 180px;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .form-row {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;

    label {
      font-size: 0.75rem;
      color: var(--col-subtle);
    }

    input {
      width: 100%;
      padding: 0.375rem 0.5rem;
      font-size: 0.875rem;
      background: var(--col-surface);
      border: 1px solid var(--col-highlightMed);
      border-radius: 4px;
      color: var(--col-text);

      &:focus {
        outline: none;
        border-color: var(--col-iris);
      }

      &:disabled {
        opacity: 0.5;
      }

      &.invalid {
        border-color: var(--col-love);
      }

      &.invalid:focus {
        border-color: var(--col-love);
      }
    }

    input[type="number"] {
      appearance: textfield;
      -moz-appearance: textfield;

      &::-webkit-inner-spin-button,
      &::-webkit-outer-spin-button {
        appearance: none;
        -webkit-appearance: none;
        margin: 0;
      }
    }
  }

  .listen-button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5ch;
    transition:
      background-color 0.15s ease,
      opacity 0.15s ease;

    &.start {
      background: var(--col-pine);
      color: var(--col-base);

      &:hover:not(:disabled) {
        background: var(--col-gold);
        color: var(--col-base);
      }
    }

    &.stop {
      background: var(--col-love);
      color: var(--col-base);

      &:hover {
        background: var(--col-rose);
      }
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }

  :global(html[data-theme="dark"]) .listen-button.start {
    color: var(--col-text);
  }

  @media (prefers-color-scheme: dark) {
    :global(html[data-theme="auto"]) .listen-button.start {
      color: var(--col-text);
    }
  }

  .status {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;

    &.active {
      background: var(--col-pine);
      color: var(--col-base);
      opacity: 0.8;
    }

    &.error {
      background: var(--col-love);
      color: var(--col-base);
    }
  }

  :global(html[data-theme="dark"]) .status.active {
    color: var(--col-text);
  }

  @media (prefers-color-scheme: dark) {
    :global(html[data-theme="auto"]) .status.active {
      color: var(--col-text);
    }
  }

  .message-list-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 0.375rem;
    font-size: 0.75rem;
    color: var(--col-subtle);
  }

  .clear-button {
    background: none;
    border: none;
    color: var(--col-muted);
    font-size: 0.75rem;
    cursor: pointer;
    padding: 0;

    &:hover {
      color: var(--col-text);
    }
  }

  .message-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .empty-list {
    color: var(--col-muted);
    font-size: 0.75rem;
    text-align: center;
    padding: 1rem 0;
  }

  .message-item {
    display: flex;
    align-items: center;
    gap: 0.5ch;
    padding: 0.25rem 0.5rem;
    background: none;
    border: 1px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    text-align: left;
    font-size: 0.75rem;
    color: var(--col-text);

    &:hover {
      background: var(--col-overlay);
    }

    &.selected {
      background: var(--col-highlightMed);
      border-color: var(--col-iris);
    }

    &.unread {
      font-weight: 600;
    }

    .unread-indicator {
      color: var(--col-iris);
      flex-shrink: 0;
    }

    .message-type {
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .message-time {
      color: var(--col-muted);
      flex-shrink: 0;
    }
  }

  .message-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    border: 1px solid var(--col-highlightMed);
    border-radius: 4px;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--col-subtle);
    background: var(--col-overlay);
    border-bottom: 1px solid var(--col-highlightMed);
  }

  .panel-actions {
    display: flex;
    gap: 0.5rem;
  }

  .action-button {
    background: var(--col-highlightMed);
    border: none;
    border-radius: 4px;
    padding: 0.25rem 0.5rem;
    font-size: 0.625rem;
    color: var(--col-text);
    cursor: pointer;

    &:hover {
      background: var(--col-highlightHigh);
    }
  }

  .message-content {
    flex: 1;
    min-height: 0;
    overflow: hidden;

    :global(.message-editor) {
      height: 100%;
      min-height: 100%;
      border: none;
      border-radius: 0;
    }
  }

  .empty-state {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    color: var(--col-muted);
    font-size: 0.875rem;
    text-align: center;
  }
</style>
