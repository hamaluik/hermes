<script lang="ts">
  import { onMount } from "svelte";
  import { get, type Writable } from "svelte/store";
  import IconClose from "./icons/IconClose.svelte";
  import { startListening, stopListening } from "../backend/listen";

  let {
    show = $bindable(false),
    listening,
    listenedMessages,
  }: {
    show: boolean;
    listening: Writable<boolean>;
    listenedMessages: Writable<
      {
        message: string;
        unread: boolean;
      }[]
    >;
  } = $props();

  let dialogElement: HTMLDialogElement | null = $state(null);

  onMount(() => {
    if (!get(listening)) {
      startListening(null, 2575, listening)
        .then(() => {
          listening.set(true);
        })
        .catch((error) => {
          console.error("Error starting listening:", error);
          listening.set(false);
        });
    }

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
      if (get(listening)) {
        listening.set(false);
      }
      stopListening(listening);
      close();
    };
  });
</script>

<dialog class="modal" closedby="any" bind:this={dialogElement}>
  <header>
    <h1>Listen</h1>
    <button class="close" onclick={() => (show = false)}>
      <IconClose />
    </button>
  </header>
  <main>
    <form></form>
  </main>
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
    }
  }
</style>
