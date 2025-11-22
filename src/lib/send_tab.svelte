<!--
  Send Tab Component

  Tab content for sending HL7 messages over MLLP and viewing responses.
  Used within the Communication Drawer to provide a non-modal send/receive workflow.

  Layout:
  ┌─────────────────────┬───────────────────────────────┐
  │ Host: [127.0.0.1]   │ Response                      │
  │ Port: [2575    ]    │ ─────────────────────────────  │
  │ Timeout: [5] sec    │ MSH|^~\&|ACK|...              │
  │                     │                               │
  │ [Send Message]      │                               │
  │                     │                               │
  │ ● Status...         │                               │
  └─────────────────────┴───────────────────────────────┘

  State Machine:
  - Idle: Ready to send, showing last response (if any)
  - Sending: Connection in progress, showing status updates
  - Error: Last send failed, showing error message
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Settings } from "../settings";
  import IconSend from "./icons/IconSend.svelte";
  import IconSpinner from "./icons/IconSpinner.svelte";
  import IconSendError from "./icons/IconSendError.svelte";
  import { sendMessage, type SendRequest } from "../backend/send_receive";
  import MessageEditor from "./message_editor.svelte";

  type SendState = "idle" | "sending" | "error";

  let {
    settings,
    message,
  }: {
    settings: Settings;
    message: string;
  } = $props();

  // Form inputs bound to settings
  let hostname: string = $state(settings.sendHostname);
  let port: number = $state(settings.sendPort);
  let timeout: number = $state(settings.sendWaitTimeoutSeconds);

  // Register callback to sync state after settings load from disk
  onMount(() => {
    settings.onSendSettingsChanged = (h, p, t) => {
      hostname = h;
      port = p;
      timeout = t;
    };
  });

  onDestroy(() => {
    settings.onSendSettingsChanged = null;
  });

  // State machine
  let sendState: SendState = $state("idle" as SendState);
  let statusText: string = $state("");
  let response: string | null = $state(null);
  let error: string | null = $state(null);

  /**
   * Validation patterns matching database wizard inputs.
   * Host: Standard hostname/IP format (with optional port :1433 or instance \\SQLEXPRESS)
   * Port: Standard TCP port range (1-65535)
   */
  const hostPattern = /^[a-zA-Z0-9]([a-zA-Z0-9\-\.:]*[a-zA-Z0-9])?$/;
  const isHostValid = $derived(
    hostname.length >= 1 && hostname.length <= 255 && hostPattern.test(hostname),
  );
  const isPortValid = $derived(port >= 1 && port <= 65535);

  /**
   * Debounced settings persistence.
   * Saves hostname/port/timeout to settings after 500ms of inactivity.
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

  const saveSettings = debounce(
    (_host: string, _port: number, _timeout: number) => {
      settings.sendHostname = _host;
      settings.sendPort = _port;
      settings.sendWaitTimeoutSeconds = _timeout;
    },
    500,
  );

  $effect(() => {
    saveSettings(hostname, port, timeout);
  });

  /**
   * Handles the send button click.
   * Sends the message and updates state based on the result.
   */
  async function handleSend() {
    if (sendState === "sending") return;

    sendState = "sending";
    statusText = `Connecting to ${hostname}:${port}...`;
    response = null;
    error = null;

    const request: SendRequest = {
      host: hostname,
      port: port,
      message: message,
      wait_timeout_seconds: timeout,
    };

    try {
      const result = await sendMessage(request, (log: string) => {
        statusText = log;
      });
      response = result?.trim() ?? null;
      sendState = "idle";
      statusText = response ? "Response received" : "No response received";
    } catch (_error) {
      console.error("Error sending message:", _error);
      error = String(_error);
      sendState = "error";
      statusText = "";
    }
  }

  // Determine if Send button should be disabled
  let canSend: boolean = $derived(
    sendState !== "sending" &&
      isHostValid &&
      isPortValid &&
      message.trim() !== "",
  );
</script>

<div class="send-tab">
  <div class="controls">
    <div class="form-row">
      <label for="send-hostname">Host</label>
      <input
        type="text"
        id="send-hostname"
        bind:value={hostname}
        placeholder="127.0.0.1"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        minlength={1}
        maxlength={255}
        pattern="^[a-zA-Z0-9]([a-zA-Z0-9\-\.:]*[a-zA-Z0-9])?$"
        required
        class:invalid={hostname.length > 0 && !isHostValid}
      />
    </div>

    <div class="form-row">
      <label for="send-port">Port</label>
      <input
        type="number"
        id="send-port"
        bind:value={port}
        min="1"
        max="65535"
        placeholder="2575"
        required
        class:invalid={!isPortValid}
      />
    </div>

    <div class="form-row">
      <label for="send-timeout">Timeout</label>
      <div class="input-with-suffix">
        <input
          type="number"
          id="send-timeout"
          bind:value={timeout}
          min="1"
          max="300"
          placeholder="5"
        />
        <span class="suffix">sec</span>
      </div>
    </div>

    <button
      class="send-button"
      onclick={handleSend}
      disabled={!canSend}
    >
      {#if sendState === "sending"}
        <IconSpinner />
        Sending...
      {:else}
        <IconSend />
        Send
      {/if}
    </button>

    {#if statusText || error}
      <div class="status" class:error={sendState === "error"}>
        {#if sendState === "error"}
          <IconSendError />
        {/if}
        <span class="status-text">{error || statusText}</span>
      </div>
    {/if}
  </div>

  <div class="response-panel">
    <div class="panel-header">Response</div>
    <div class="response-content">
      {#if response}
        <MessageEditor
          message={response}
          readonly={true}
          placeholder="No response"
        />
      {:else}
        <div class="empty-state">
          {#if sendState === "idle" && !error}
            Send a message to see the response here
          {:else if sendState === "sending"}
            Waiting for response...
          {:else if error}
            Send failed - check error above
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .send-tab {
    display: flex;
    flex-direction: row;
    gap: 1rem;
    flex: 1;
    min-height: 0;
    padding: 0.75rem;
  }

  .controls {
    flex: 0 0 auto;
    width: 180px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    overflow-y: auto;
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

  .input-with-suffix {
    display: flex;
    align-items: center;
    gap: 0.5ch;

    input {
      flex: 1;
    }

    .suffix {
      font-size: 0.75rem;
      color: var(--col-subtle);
    }
  }

  .send-button {
    margin-top: 0.5rem;
    padding: 0.5rem 1rem;
    background: var(--col-pine);
    color: var(--col-base);
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

    &:hover:not(:disabled) {
      background: var(--col-gold);
      color: var(--col-base);
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }

  :global(html[data-theme="dark"]) .send-button {
    color: var(--col-text);
  }

  @media (prefers-color-scheme: dark) {
    :global(html[data-theme="auto"]) .send-button {
      color: var(--col-text);
    }
  }

  .status {
    margin-top: 0.25rem;
    display: flex;
    align-items: flex-start;
    gap: 0.5ch;
    font-size: 0.75rem;
    color: var(--col-subtle);

    &.error {
      color: var(--col-love);
    }

    .status-text {
      flex: 1;
      word-break: break-word;
    }
  }

  .response-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    border: 1px solid var(--col-highlightMed);
    border-radius: 4px;
    overflow: hidden;
  }

  .panel-header {
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--col-subtle);
    background: var(--col-overlay);
    border-bottom: 1px solid var(--col-highlightMed);
  }

  .response-content {
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
