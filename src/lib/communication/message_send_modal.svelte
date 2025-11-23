<!--
  Message Send Modal

  Modal dialog for sending HL7 messages to a remote MLLP server and receiving responses.
  This component provides the UI for configuring connection settings (host/port) and
  displays real-time progress during the send operation.

  ## State Machine Flow

  The component progresses through four states (ActionState):
  1. Form: User enters/confirms hostname and port
  2. Sending: Connection in progress, live log updates displayed
  3. Results: Success - displays the received HL7 response message
  4. Error: Failure - displays the error message

  The state machine is one-way: users must close the modal to return to Form state.
  This prevents confusion about whether they're looking at old results or starting fresh.

  ## Why Debounced Settings?

  The hostname and port inputs auto-save to persistent settings as the user types.
  Debouncing prevents excessive writes to the Tauri store on every keystroke, which
  could cause performance issues or wear on storage. The 500ms delay ensures we only
  save after the user has paused typing.
-->
<script lang="ts">
  import type { Settings } from "../../settings";
  import IconSend from "$lib/icons/IconSend.svelte";
  import IconSpinner from "$lib/icons/IconSpinner.svelte";
  import { sendMessage, type SendRequest } from "./send_receive";
  import MessageEditor from "$lib/editor/message_editor.svelte";
  import IconSendError from "$lib/icons/IconSendError.svelte";
  import Modal from "$lib/components/modal.svelte";
  import ModalHeader from "$lib/components/modal_header.svelte";

  /**
   * ActionState: Modal State Machine
   *
   * - Form: Initial state, shows hostname/port inputs and Send button
   * - Sending: Active connection, streaming log updates from backend
   * - Results: Successful response received, displays HL7 message
   * - Error: Failed connection or timeout, displays error details
   *
   * Transitions: Form → (Submit) → Sending → (Success) → Results
   *                                        → (Failure) → Error
   */
  type ActionState = "Form" | "Sending" | "Results" | "Error";

  let {
    show = $bindable(false),
    settings,
    message,
  }: { show: boolean; settings: Settings; message: string } = $props();

  let hostname: string = $state(settings.sendHostname);
  let port: number = $state(settings.sendPort);

  let actionState: ActionState = $state("Form" as ActionState);

  let logElement: HTMLElement | null = $state(null);
  let log: string = $state("");

  let response: string | null = $state(null);
  let error: string | null = $state(null);

  /**
   * Debounced Settings Persistence
   *
   * As users type in the hostname/port fields, we save to settings after 500ms of
   * inactivity. This balances two concerns:
   * - UX: Settings persist immediately from the user's perspective
   * - Performance: We don't write to Tauri store on every keystroke (which could
   *   be dozens of times per second during fast typing)
   *
   * The reactive effect ensures any changes to hostname or port trigger the
   * debounced save, even if changed programmatically.
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
  const saveSettings = debounce((_host: string, _port: number) => {
    settings.sendHostname = _host;
    settings.sendPort = _port;
  }, 500);
  $effect(() => {
    saveSettings(hostname, port);
  });

  const handleClose = () => {
    show = false;
  };

  /**
   * Form Submission: Send Message Flow
   *
   * 1. Transition to "Sending" state immediately for UI feedback
   * 2. Reset log/response/error from any previous send attempt
   * 3. Call sendMessage() which:
   *    - Opens TCP connection to host:port
   *    - Sends message with MLLP framing (VT...FS+CR)
   *    - Waits for response with timeout
   *    - Emits progress events that we capture in the log callback
   * 4. On success: Store response and transition to "Results"
   * 5. On failure: Store error and transition to "Error"
   *
   * The log callback is invoked multiple times during the send operation as the
   * backend emits progress events (connecting, sending, waiting, receiving). This
   * provides real-time feedback during potentially slow network operations.
   */
  const onsubmit = async (event: Event) => {
    event.preventDefault();
    actionState = "Sending";

    log = "Captain's log, stardate " + new Date().toISOString() + ":\n\n";
    response = null;
    error = null;
    const request: SendRequest = {
      host: hostname,
      port: port,
      message: message,
      wait_timeout_seconds: settings.sendWaitTimeoutSeconds,
    };

    try {
      response = await sendMessage(request, (thislog: string) => {
        log += thislog + "\n\n";
      });
      response = response?.trim() ?? null;
      actionState = "Results";
    } catch (_error) {
      console.error("Error sending message:", _error);
      error = String(_error);
      actionState = "Error";
    }
  };

  /**
   * Log Auto-Scroll
   *
   * During the "Sending" state, new log entries are appended to the log string.
   * This effect keeps the log scrolled to the bottom so users always see the
   * most recent updates without manual scrolling.
   *
   * This is important for long-running operations (e.g., slow network, long timeout)
   * where multiple log entries accumulate and the user needs to see progress.
   */
  $effect(() => {
    if (!logElement) {
      return;
    }
    logElement.scrollTop = logElement.scrollHeight;
  });
</script>

<Modal bind:show>
  <ModalHeader onclose={handleClose}>
    {#if actionState === "Form"}Send Message{/if}
    {#if actionState === "Sending"}<IconSpinner /> Sending to
      <span class="target">{hostname}:{port}</span>{/if}
    {#if actionState === "Results"}Response from <span class="target"
        >{hostname}:{port}</span
      >{/if}
    {#if actionState === "Error"}<IconSendError /> Error sending to
      <span class="target">{hostname}:{port}</span>{/if}
  </ModalHeader>
  {#if actionState === "Form"}
    <main>
      <form {onsubmit}>
        <div class="form-item">
          <label for="hostname">Hostname</label>
          <!-- svelte-ignore a11y_autofocus -->
          <input
            type="text"
            id="hostname"
            autofocus
            bind:value={hostname}
            minlength="1"
            maxlength="1024"
            placeholder="localhost"
            required
            pattern="^[a-zA-Z0-9._-]+$"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
          />
        </div>
        <div class="form-item">
          <label for="port">Port</label>
          <input
            type="number"
            id="port"
            bind:value={port}
            min="1"
            max="65535"
            step="1"
            maxlength="5"
            placeholder="2575"
            required
            inputmode="numeric"
            pattern="\d*"
          />
        </div>
        <button type="submit" class="submit">
          <IconSend />
          Send
        </button>
      </form>
    </main>
  {:else if actionState === "Sending"}
    <main>
      <pre class="log" bind:this={logElement}><code>{log}</code></pre>
    </main>
  {:else if actionState === "Results"}
    <main class="results">
      <MessageEditor
        message={response ?? undefined}
        readonly={true}
        placeholder="¯\_(ツ)_/¯"
      />
    </main>
  {:else if actionState === "Error"}
    <main>
      <div class="log" bind:this={logElement}>
        {error}
      </div>
    </main>
  {/if}
</Modal>

<style>
  .target {
    background: var(--col-highlightMed);
    color: var(--col-text);
    padding: 0.0625em 0.5ch;
    border-radius: 4px;
    border: 1px solid var(--col-highlightHigh);
  }

  main {
    padding: 1rem;

      form {
        display: flex;
        flex-direction: row;
        align-items: flex-end;
        justify-content: stretch;
        gap: 1ch;

        input[type="text"] {
          min-width: calc(16ch + 1em);
        }
        input[type="number"] {
          min-width: calc(5ch + 1em);
        }
        input::-webkit-inner-spin-button {
          appearance: none;
        }

        .form-item {
          flex: 1 1 auto;
          width: min-content;
          max-width: 30ch;

          display: grid;
          grid-template-columns: 1fr;
          grid-template-rows: auto auto;
          grid-template-areas: "label" "input";
          gap: 0;
          align-items: stretch;
          position: relative;

          label {
            margin-bottom: 0.1lh;
            font-size: small;
            color: var(--col-text);
            white-space: nowrap;
          }
        }

        button.submit {
          background: var(--col-pine);
          color: var(--col-base);
          border: none;
          border-radius: 4px;
          padding: 0.5em 1ch;
          cursor: pointer;
          font-weight: 700;
          transition:
            color 0.2s ease-in-out,
            background-color 0.2s ease-in-out;

          display: inline-flex;
          flex-direction: row;
          align-items: center;
          gap: 1ch;
          font-size: medium;

          &:hover,
          &:focus,
          &:active {
            background-color: var(--col-gold);
            color: var(--col-base);
            outline: none;
          }
        }
      }

      pre.log {
        code {
          font-family: ui-monospace, Menlo, Monaco, "Cascadia Mono",
            "Segoe UI Mono", "Roboto Mono", "Oxygen Mono", "Ubuntu Mono",
            "Source Code Pro", "Fira Mono", "Droid Sans Mono", "Consolas",
            "Courier New", monospace;
        }
      }

      .log {
        max-width: min(80vw, calc(80ch + 2rem));
        max-height: min(80vh, calc(25lh + 2rem));
        overflow: auto;

        background: var(--col-surface);
        color: var(--col-text);
        border: 1px solid var(--col-highlightMed);
        border-radius: 4px;
        padding: 1rem;
        margin: 0;
        font-size: small;
        white-space: pre-line;

        code {
          white-space: pre-line;
          margin: 0;
          padding: 0;
        }
      }

    &.results {
      min-width: min(80vw, calc(80ch + 2rem));
    }
  }

  :global(html[data-theme="dark"]) button.submit {
    color: var(--col-text);
  }

  @media (prefers-color-scheme: dark) {
    :global(html[data-theme="auto"]) button.submit {
      color: var(--col-text);
    }
  }
</style>
