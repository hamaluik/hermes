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

  let { data }: PageProps = $props();

  let toolbarHeight: string | undefined = $state(undefined);

  let hostname: string = $state("");
  let port: number = $state(2575);
  let message: string = $state("");
  let autoCID: boolean = $state(false);
  let autoTimestamp: boolean = $state(false);
  let messageLog: string = $state("");
  let receivedMessage: string = $state("");

  onMount(() => {
    message = get(data.message);
  });
</script>

<Toolbar bind:toolbarHeight>
  <ToolbarButton
    title="Edit Message"
    onclick={() => {
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
    <form class="send-form">
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
          />
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
          />
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
    </form>
    <div class="send">
      <h2>Outbound Message</h2>
      <button class="send-button" type="button" onclick={() => {}}>
        <IconSend />
        Send
      </button>
    </div>
    <MessageEditor {message} readonly={true} --message-editor-flex="0" />
    <h2>Received Message</h2>
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
  <div></div>
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
        display: grid;
        grid-template-columns: 1fr auto;
        align-items: stretch;
        justify-content: flex-start;
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
