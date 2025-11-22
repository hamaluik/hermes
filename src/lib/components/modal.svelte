<script lang="ts">
  import { onMount } from "svelte";
  import type { Snippet } from "svelte";

  let {
    show = $bindable(false),
    maxWidth,
    maxHeight,
    width,
    height,
    children,
  }: {
    show: boolean;
    maxWidth?: string;
    maxHeight?: string;
    width?: string;
    height?: string;
    children: Snippet<[]>;
  } = $props();

  let dialogElement: HTMLDialogElement | null = $state(null);

  const close = () => {
    if (dialogElement) {
      dialogElement.close();
    }
    show = false;
  };

  onMount(() => {
    if (show) {
      dialogElement?.showModal();
    }

    dialogElement?.addEventListener("close", () => {
      close();
    });

    return () => {
      close();
    };
  });

  // Watch for show prop changes after mount
  $effect(() => {
    if (show && dialogElement && !dialogElement.open) {
      dialogElement.showModal();
    } else if (!show && dialogElement && dialogElement.open) {
      dialogElement.close();
    }
  });
</script>

<dialog
  class="modal"
  closedby="any"
  bind:this={dialogElement}
  style:max-width={maxWidth}
  style:max-height={maxHeight}
  style:width={width}
  style:height={height}
>
  {@render children()}
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
    inset: 0;
    margin: auto;

    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;
  }
</style>
