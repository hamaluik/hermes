<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { syntaxHighlight } from "../backend/syntax_highlight";
  import {
    getRangeOfNextField,
    getRangeOfPreviousField,
  } from "../backend/cursor";
  import IconClipboard from "./icons/IconClipboard.svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import IconClipboardCheck from "./icons/IconClipboardCheck.svelte";

  let {
    message,
    onchange,
    oncursorchange,
    onctrlenter,
    readonly,
    placeholder,
    height,
  }: {
    message?: string;
    onchange?: (message: string) => void;
    oncursorchange?: (cursorPos: number) => void;
    onctrlenter?: () => void;
    readonly?: boolean;
    placeholder?: string;
    height?: number;
  } = $props();

  let editElement: HTMLElement;
  let highlightElement: HTMLElement;
  let _cursorPos: number = $state(0);
  let copied: boolean = $state(false);

  $effect(() => {
    if (copied) {
      setTimeout(() => {
        copied = false;
      }, 2000);
    }
  });

  let selectionListener: () => void;

  $effect(() => {
    if (message) {
      (editElement as HTMLTextAreaElement).value = message;
      syntaxHighlight(message).then((highlighted) => {
        highlightElement.innerHTML = highlighted;
      });
    }
  });

  async function handleInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    let message = target.value;
    if (onchange) {
      onchange(message);
    }

    const highlighted = await syntaxHighlight(message);
    highlightElement.innerHTML = highlighted;

    handleScroll();
  }

  function handleScroll() {
    highlightElement.scrollTop = editElement.scrollTop;
    highlightElement.scrollLeft = editElement.scrollLeft;
  }

  async function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Tab") {
      event.preventDefault();
      if (!message || !editElement) {
        return;
      }
      // if shift is pressed, move the cursor to the previous field
      let range: { start: number; end: number } | undefined | null;
      if (event.shiftKey) {
        range = await getRangeOfPreviousField(message, _cursorPos);
      } else {
        range = await getRangeOfNextField(message, _cursorPos);
      }
      if (!range) {
        return;
      }
      // select the text in the range in the editElement
      const start = range.start;
      const end = range.end;
      (editElement as HTMLTextAreaElement).setSelectionRange(start, end);
    } else if (
      event.key === "Enter" &&
      (event.ctrlKey || event.metaKey) &&
      onctrlenter
    ) {
      event.preventDefault();
      onctrlenter();
    }
  }

  function handleCursorChange() {
    if (document.activeElement === editElement) {
      const cursorPos = (editElement as HTMLTextAreaElement).selectionStart;
      _cursorPos = cursorPos;
      if (oncursorchange) {
        oncursorchange(cursorPos);
      }
    }
  }

  onMount(() => {
    highlightElement.scrollTop = editElement.scrollTop;
    highlightElement.scrollLeft = editElement.scrollLeft;

    selectionListener = () => {
      if (document.activeElement === editElement) {
        handleCursorChange();
      }
    };
    document.addEventListener("selectionchange", selectionListener);
  });

  onDestroy(() => {
    document.removeEventListener("selectionchange", selectionListener);
  });

  let messageHeight: number = $derived.by(() => {
    const lines = (message?.trim() ?? "").split("\n").length;
    return Math.max(lines, 3);
  });
</script>

<div
  class="message-editor"
  style="--message-height: {messageHeight}lh; {height !== undefined ? `--editor-height: ${height}px;` : ''}"
>
  <textarea
    placeholder={placeholder ?? "MSH|^~\\&|…"}
    class="editor"
    oninput={handleInput}
    onscroll={handleScroll}
    onkeydown={handleKeyDown}
    bind:this={editElement}
    {readonly}
  ></textarea>
  <div
    class="highlighting"
    aria-hidden="true"
    bind:this={highlightElement}
  ></div>
  <div class="copy">
    <button
      class="copy-button"
      disabled={message === undefined || message === "" || copied}
      onclick={async () => {
        try {
          await writeText(message ?? "");
          copied = true;
        } catch (error) {
          console.error("Error copying to clipboard:", error);
          copied = false;
        }
      }}
    >
      {#if copied}
        <IconClipboardCheck />
      {:else}
        <IconClipboard />
      {/if}
    </button>
  </div>
</div>

<style>
  .message-editor {
    flex: var(--message-editor-flex, 1);
    width: 100%;
    /* Use --editor-height if provided, otherwise default to content-based height */
    height: var(--editor-height, calc(var(--message-height) + 1rem));
    min-height: var(--editor-height, calc(var(--message-height) + 1rem));
    padding: 1rem;
    background-color: var(--col-surface);
    border: 1px solid var(--col-highlightHigh);
    border-radius: 4px;
    position: relative;
    isolation: isolate;
    z-index: 0;

    .editor,
    .highlighting {
      margin: 0;
      padding: 0;
      border: 0;
      width: calc(100% - 1rem);
      height: calc(100% - 1rem);
      font-family: ui-monospace, Menlo, Monaco, "Cascadia Mono", "Segoe UI Mono",
        "Roboto Mono", "Oxygen Mono", "Ubuntu Mono", "Source Code Pro",
        "Fira Mono", "Droid Sans Mono", "Consolas", "Courier New", monospace;
      line-height: 1.5;
      tab-size: 2;
      position: absolute;
      top: 0.5rem;
      left: 0.5rem;
      overflow: auto;
      white-space: pre;
    }

    .editor {
      z-index: 1;
      color: transparent;
      background: transparent;
      caret-color: var(--col-iris);
      resize: none;

      &:focus {
        outline: none;
      }
    }

    .highlighting {
      z-index: 0;

      :global(.msh) {
        color: var(--col-pine);
      }
      :global(.seps) {
        color: var(--col-subtle);
      }
      :global(.seg) {
        color: var(--col-foam);
      }
      :global(.sep) {
        color: var(--col-muted);
      }
      :global(.cell) {
        color: var(--col-text);
      }
      :global(.temp) {
        color: var(--col-gold);
      }
      :global(.ts) {
        color: var(--col-iris);
      }
      :global(.err) {
        color: var(--col-love) !important;
      }
    }

    &:hover {
      .copy {
        display: block;
      }
    }

    .copy {
      display: none;
      position: absolute;
      z-index: 2;
      top: 2px;
      right: 4px;

      .copy-button {
        background: var(--col-overlay);
        color: var(--col-subtle);
        border: none;
        cursor: pointer;
        margin: 0;
        padding: 0;
        font-size: medium;

        &:hover,
        &:active,
        &:focus {
          color: var(--col-text);
        }
      }
    }
  }
</style>
