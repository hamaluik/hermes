<script lang="ts">
  import { goto } from "$app/navigation";
  import IconEditMessage from "$lib/icons/IconEditMessage.svelte";
  import IconHelp from "$lib/icons/IconHelp.svelte";
  import Toolbar from "$lib/toolbar.svelte";
  import ToolbarButton from "$lib/toolbar_button.svelte";
  import ToolbarSpacer from "$lib/toolbar_spacer.svelte";
  import { get } from "svelte/store";
  import type { PageProps } from "./$types";
  import { onMount } from "svelte";
  import MessageEditor from "$lib/message_editor.svelte";
  import ToggleSwitch from "$lib/forms/toggle_switch.svelte";
  import IconSend from "$lib/icons/IconSend.svelte";
  import { sendMessage, type SendRequest } from "../../backend/send_receive";
  import { message as messageDialog } from "@tauri-apps/plugin-dialog";
  import IconSpinner from "$lib/icons/IconSpinner.svelte";

  let { data }: PageProps = $props();

  let toolbarHeight: string | undefined = $state(undefined);

  let hostname: string = $state(data.settings.lastHostname);
  let port: number = $state(data.settings.lastPort);
  let message: string = $state("");
  let autoCID: boolean = $state(data.settings.lastTransformControlId);
  let autoTimestamp: boolean = $state(data.settings.lastTransformTimestamp);
  let waitTime: number = $state(data.settings.lastWaitTimeoutSeconds);
  let messageLog: string = $state("");
  let receivedMessage: string = $state("");
  let sending: boolean = $state(false);

  onMount(() => {
    message = get(data.message);
  });

  const onfocus = (event: Event) => {
    const popover = (event.target as HTMLElement)
      .closest("div")
      ?.querySelector(".popover");
    if (popover) {
      (popover as HTMLElement).classList.add("show");
    }
  };

  const onblur = (event: Event) => {
    const popover = (event.target as HTMLElement)
      .closest("div")
      ?.querySelector(".popover");
    if (popover) {
      (popover as HTMLElement).classList.remove("show");
    }
  };

  const updateSettings = async () => {
    data.settings.lastHostname = hostname;
    data.settings.lastPort = port;
    data.settings.lastTransformControlId = autoCID;
    data.settings.lastTransformTimestamp = autoTimestamp;
    data.settings.lastWaitTimeoutSeconds = waitTime;
    await data.settings.save();
  };

  const onSendSubmit = async (event: Event) => {
    event.preventDefault();
    sending = true;

    updateSettings();

    messageLog = "";
    receivedMessage = "";
    const request: SendRequest = {
      host: hostname,
      port: port,
      message: message,
      transformations: {
        control_id: autoCID,
        timestamp: autoTimestamp,
      },
      wait_timeout_seconds: waitTime,
    };

    let response: string | null = null;
    try {
      response = await sendMessage(request, (log: string) => {
        messageLog += log + "\n\n";
      });

      if (response) {
        receivedMessage = response;
      } else {
        receivedMessage = "No response received.";
      }
    } catch (error) {
      console.error("Error sending message:", error);
      messageLog += "Error: " + error + "\n\n";
      await messageDialog(String(error), {
        title: "Error Sending Message",
        kind: "error",
      });
    } finally {
      sending = false;
    }
  };
</script>

<Toolbar bind:toolbarHeight>
  <ToolbarButton
    title="Edit Message"
    onclick={async () => {
      await updateSettings();
      if (!document.startViewTransition) {
        goto("/");
        return;
      }
      document.startViewTransition(() => {
        goto("/");
      });
    }}
  >
    <IconEditMessage />
  </ToolbarButton>
  <ToolbarSpacer />
  <ToolbarButton title="Help">
    <IconHelp />
  </ToolbarButton>
</Toolbar>
<main class="main" style="--toolbar-height: {toolbarHeight ?? '1px'}">
  <div class="send-receive">
    <h1>Send/Receive</h1>
    <form class="send-form" id="send-form" onsubmit={onSendSubmit}>
      <fieldset>
        <legend>MLLP Connection</legend>

        <div class="form-group">
          <label for="hostname">Hostname</label>
          <input
            type="text"
            id="hostname"
            bind:value={hostname}
            minlength="1"
            maxlength="1024"
            placeholder="localhost"
            required
            pattern="^[a-zA-Z0-9._-]+$"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            {onfocus}
            {onblur}
          />
          <p class="popover">
            The hostname or IP address of the server to connect to.
          </p>
        </div>

        <div class="form-group">
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
            {onfocus}
            {onblur}
          />
          <p class="popover">The port number to connect to.</p>
        </div>

        <div class="switch-group">
          <label for="TLS">TLS</label>
          <ToggleSwitch id="TLS" checked={false} disabled={true} />
        </div>
      </fieldset>
      <fieldset class="message-options">
        <legend>Message Options</legend>
        <label for="auto-cid">Auto-Set Control ID</label>
        <ToggleSwitch id="auto-cid" bind:checked={autoCID} />
        <label for="auto-timestamp">Auto-Set Timestamp</label>
        <ToggleSwitch id="auto-timestamp" bind:checked={autoTimestamp} />
      </fieldset>
      <fieldset>
        <legend>Send Options</legend>
        <div class="form-group">
          <label for="wait-time">Wait Time (s)</label>
          <input
            type="number"
            id="wait-time"
            bind:value={waitTime}
            min="0"
            max="600"
            step="0.1"
            maxlength="5"
            placeholder="10"
            required
            inputmode="numeric"
            pattern={'"^\\d+(\\.\\d{1,1})?$"'}
            {onfocus}
            {onblur}
          />
          <p class="popover">
            The number of seconds to wait for an ACK response before timing out.
            If set to 0, quit immediately after sending the message without
            waiting for a response.
          </p>
        </div>
      </fieldset>
    </form>
    <div class="send">
      <h2>Outbound Message</h2>
      <button
        class="send-button"
        form="send-form"
        type="submit"
        disabled={sending}
      >
        {#if sending}
          <IconSpinner />
          Sending...
        {:else}
          <IconSend />
          Send
        {/if}
      </button>
    </div>
    <MessageEditor {message} readonly={true} --message-editor-flex="0" />
    <div class="receive">
      <h2>Received Message</h2>
      {#if receivedMessage}
        <button
          class="receive-button"
          onclick={async () => {
            data.message.set(receivedMessage);
            data.currentFilePath.set(undefined);
            await updateSettings();
            goto("/");
          }}
        >
          <IconEditMessage /> Edit
        </button>
      {/if}
    </div>
    <MessageEditor
      message={receivedMessage}
      readonly={true}
      --message-editor-flex="0"
      placeholder=""
    />
    <h2>Message Log</h2>
    <textarea class="message-log" bind:value={messageLog} readonly={true}
    ></textarea>
  </div>
</main>

<style>
  .main {
    display: flex;
    min-height: calc(100vh - var(--toolbar-height, 0px));

    flex-direction: column;
    gap: 1lh;
    @media (min-aspect-ratio: 5/4) {
      flex-direction: row;
      gap: 2ch;
    }

    > * {
      flex: 1;

      &:first-child {
        flex: 1.5;
        border-right: 1px solid var(--col-highlightMed);
      }
    }

    .send-receive {
      display: flex;
      flex-direction: column;
      justify-content: stretch;
      align-items: stretch;
      padding: 1rem;
      gap: 1rem;
      overflow-y: auto;

      form.send-form {
        display: flex;
        flex-direction: row;
        align-items: stretch;
        justify-content: stretch;
        gap: 1ch;

        fieldset {
          align-items: flex-start;
        }

        input[type="text"] {
          min-width: calc(16ch + 1em);
        }
        input[type="number"] {
          min-width: calc(6ch + 1em);
        }
        input::-webkit-inner-spin-button {
          appearance: none;
        }

        .form-group {
          position: relative;
          .popover {
            display: none;
            position: absolute;
            top: calc(100% + 0.25rem);
            left: -3ch;
            right: -3ch;
            color: var(--col-text);
            background-color: var(--col-overlay);
            padding: 0.5ch;
            border: 1px solid var(--col-highlightHigh);
            z-index: 1000;
            border-radius: 4px;
            font-size: smaller;
            white-space: pre-line;
            :global(&.show) {
              display: block;
            }
          }
        }

        .switch-group {
          align-self: center;
          display: flex;
          flex-direction: row;
          gap: 1ch;
          align-items: center;
          justify-content: flex-start;
        }

        .message-options {
          display: grid;
          grid-template-columns: auto auto;
          gap: 0.5lh 1ch;
          align-items: center;
          align-content: start;
          position: relative;
        }
      }

      .send {
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: center;

        .send-button {
          display: flex;
          flex-direction: row;
          align-items: center;
          justify-content: center;
          gap: 1ch;
          font-weight: 700;
          background-color: var(--col-pine);
          color: var(--col-text);
          border-radius: 0.5rem;
          border: none;
          cursor: pointer;
          font-size: inherit;
          padding: 0.5em 1ch;

          &:hover,
          &:active,
          &:focus {
            background-color: var(--col-gold);
            color: var(--col-surface);
          }

          &:disabled {
            background-color: transparent;
            color: var(--col-iris);
            cursor: not-allowed;
          }
        }
      }

      .receive {
        display: flex;
        align-items: center;
        justify-content: flex-start;
        gap: 1ch;

        .receive-button {
          display: flex;
          flex-direction: row;
          align-items: center;
          justify-content: center;
          gap: 1ch;
          font-weight: 700;
          background-color: var(--col-overlay);
          color: var(--col-text);
          border-radius: 0.5rem;
          border: none;
          cursor: pointer;
          font-size: inherit;
          padding: 0.5em 1ch;

          &:hover,
          &:active,
          &:focus {
            background-color: var(--col-gold);
            color: var(--col-surface);
          }
        }
      }

      .message-log {
        flex: 1;
        padding: 1rem;
        resize: none;
        border-radius: 0.5rem;
        background-color: var(--col-surface);
        border: 1px solid var(--col-highlightMed);
        color: var(--col-text);
        font-family: ui-monospace, Menlo, Monaco, "Cascadia Mono",
          "Segoe UI Mono", "Roboto Mono", "Oxygen Mono", "Ubuntu Mono",
          "Source Code Pro", "Fira Mono", "Droid Sans Mono", "Consolas",
          "Courier New", monospace;
        line-height: 1.5;
        tab-size: 2;
        min-height: calc(3lh + 2rem);
      }
    }
  }
</style>
