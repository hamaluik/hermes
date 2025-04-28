<script lang="ts">
  import { onMount } from "svelte";
  import type { Settings } from "../settings";
  import IconClose from "./icons/IconClose.svelte";
  import IconSave from "./icons/IconSave.svelte";
  import ToggleSwitch from "./forms/toggle_switch.svelte";

  let {
    settings,
    show = $bindable(undefined),
  }: {
    settings: Settings;
    show: ((open: boolean) => void) | undefined;
  } = $props();

  let dialogElement: HTMLDialogElement | null = $state(null);

  let tabsFollowCursor: boolean = $state(settings.tabsFollowCursor);

  const saveSettings = () => {
    // Save settings logic here
    settings.tabsFollowCursor = tabsFollowCursor;
  };

  onMount(() => {
    show = (open: boolean) => {
      if (dialogElement) {
        open ? dialogElement.showModal() : dialogElement.close();
      }
    };

    return () => {
      if (dialogElement) {
        dialogElement.close();
      }
      show = undefined;
    };
  });
</script>

<dialog class="settings" bind:this={dialogElement} closedby="any">
  <header>
    <h1>Settings</h1>
    <button
      class="close"
      onclick={() => {
        show?.(false);
      }}
    >
      <IconClose />
    </button>
  </header>
  <main>
    <form method="dialog">
      <label for="tabsFollowCursor">Tabs Follow Cursor</label>
      <ToggleSwitch
        id="tabsFollowCursor"
        checked={tabsFollowCursor}
        onchange={() => {
          tabsFollowCursor = !tabsFollowCursor;
        }}
      />
    </form>
  </main>
  <footer>
    <button
      class="cancel"
      onclick={() => {
        show?.(false);
      }}
    >
      <IconClose />
      <span>Cancel</span>
    </button>
    <button
      class="save"
      onclick={() => {
        saveSettings();
        show?.(false);
      }}
    >
      <IconSave />
      <span>Save</span>
    </button>
  </footer>
</dialog>

<style>
  .settings {
    display: none;
    &[open] {
      display: flex;
    }
    flex-direction: column;
    gap: 0;
    align-items: stretch;

    background: var(--col-surface);
    border: 1px solid var(--col-highlightLow);
    border-radius: 0.5em;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.5);
    padding: 0;
    margin: 0;

    position: fixed;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);

    /*min-width: 40ch;*/

    &::backdrop {
      background: rgba(0, 0, 0, 0.1);
      backdrop-filter: blur(5px);
    }

    header {
      border-radius: 8px 8px 0 0;
      display: flex;
      flex-direction: row;
      align-items: stretch;
      justify-content: space-between;
      margin: 0;
      padding: 0;

      background: var(--col-overlay);
      color: var(--col-text);

      h1 {
        padding: 0.5rem;
      }

      button {
        background: transparent;
        border: none;
        cursor: pointer;
        padding: 0.5em 1ch;
        color: var(--col-text);
        transition: color 0.2s ease-in-out;

        &:hover {
          color: var(--col-rose);
        }
      }
    }

    main {
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: stretch;
      justify-content: stretch;
      padding: 0.5rem 2ch;
      color: var(--col-text);

      form {
        display: grid;
        grid-template-columns: 1fr auto;
        gap: 0.8lh 1ch;
        align-items: center;
      }
    }

    footer {
      display: flex;
      flex-direction: row;
      align-items: stretch;
      justify-content: flex-end;
      gap: 1ch;
      width: 100%;
      padding: 0.5rem;

      button {
        display: inline-flex;
        flex-direction: row;
        align-items: center;
        gap: 1ch;
        padding: 0.5rem;
        border-radius: 4px;
        background: var(--col-overlay);
        border: 1px solid var(--col-highlightLow);
        color: var(--col-text);
        transition:
          background 0.2s ease-in-out,
          color 0.2s ease-in-out;

        &.save {
          background: var(--col-pine);
          color: var(--col-base);

          &:hover {
            background: var(--col-gold);
            color: var(--col-base);
          }
        }
        @media (prefers-color-scheme: dark) {
          &.save {
            color: var(--col-text);
          }
        }

        &.cancel:hover {
          color: var(--col-rose);
        }
      }
    }
  }
</style>
