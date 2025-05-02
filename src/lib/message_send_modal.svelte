<script lang="ts">
  import { onMount } from "svelte";
  import IconClose from "./icons/IconClose.svelte";
  import type { Settings } from "../settings";
  import IconSend from "./icons/IconSend.svelte";
  import IconSpinner from "./icons/IconSpinner.svelte";
  import { sendMessage, type SendRequest } from "../backend/send_receive";
  import MessageEditor from "./message_editor.svelte";
  import IconSendError from "./icons/IconSendError.svelte";

  type ActionState = "Form" | "Sending" | "Results" | "Error";

  let {
    show = $bindable(false),
    settings,
    message,
  }: { show: boolean; settings: Settings; message: string } = $props();

  let dialogElement: HTMLDialogElement | null = $state(null);
  let hostname: string = $state(settings.sendHostname);
  let port: number = $state(settings.sendPort);

  let actionState: ActionState = $state("Form" as ActionState);

  let logElement: HTMLElement | null = $state(null);
  let log: string = $state("");

  let response: string | null = $state(null);
  let error: string | null = $state(null);

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

  onMount(() => {
    dialogElement?.showModal();
    const close = () => {
      if (dialogElement) {
        dialogElement.close();
      }
      show = false;
    };

    dialogElement?.addEventListener("close", () => {
      close();
    });

    return () => {
      close();
    };
  });

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

  $effect(() => {
    if (!logElement) {
      return;
    }
    logElement.scrollTop = logElement.scrollHeight;
  });
</script>

<dialog class="modal" closedby="any" bind:this={dialogElement}>
  <header>
    <h1>
      {#if actionState === "Form"}Send Message{/if}
      {#if actionState === "Sending"}<IconSpinner /> Sending to
        <span class="target">{hostname}:{port}</span>{/if}
      {#if actionState === "Results"}Response from <span class="target"
          >{hostname}:{port}</span
        >{/if}
      {#if actionState === "Error"}<IconSendError /> Error sending to
        <span class="target">{hostname}:{port}</span>{/if}
    </h1>
    <button class="close" onclick={() => (show = false)}>
      <IconClose />
    </button>
  </header>
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
</dialog>

<style>
  .modal {
    display: none;
    &[open] {
      display: flex;
    }

    isolation: isolate;
    z-index: 2000;

    background: var(--col-overlay);
    border: 1px solid var(--col-highlightHigh);
    outline: none;
    color: var(--col-text);
    border-radius: 0.5em;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.5);
    padding: 0;
    margin: 0;

    &::backdrop {
      background: rgba(0, 0, 0, 0.1);
      backdrop-filter: blur(5px);
    }

    position: fixed;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);

    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;

    header {
      width: 100%;
      border-radius: 8px 8px 0 0;
      display: flex;
      flex-direction: row;
      align-items: stretch;
      justify-content: space-between;
      margin: 0;
      padding: 0;
      background: none;

      h1 {
        font-size: medium;
        font-weight: 700;
        padding: 0.5em 1ch;

        display: inline-flex;
        flex-direction: row;
        align-items: center;
        gap: 1ch;

        .target {
          background: var(--col-highlightMed);
          color: var(--col-text);
          padding: 0.0625em 0.5ch;
          border-radius: 4px;
          border: 1px solid var(--col-highlightHigh);
        }
      }

      button.close {
        background: transparent;
        border: none;
        cursor: pointer;
        color: var(--col-text);
        padding: 0.25em 1ch;

        &:hover {
          color: var(--col-love);
        }
      }
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
          color: var(--col-text);
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
  }
</style>
