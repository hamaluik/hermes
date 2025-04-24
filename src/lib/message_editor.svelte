<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { syntaxHighlight } from "../backend/syntax_highlight";

  let {
    message,
    onchange,
    oncursorchange,
  }: {
    message?: string;
    onchange?: (message: string) => void;
    oncursorchange?: (cursorPos: number) => void;
  } = $props();

  let editElement: HTMLElement;
  let highlightElement: HTMLElement;

  let selectionListener: () => void;

  $effect(() => {
    if (message) {
      console.debug("message changed");
      (editElement as HTMLTextAreaElement).value = message;
      console.debug("message changed, highlighting...");
      syntaxHighlight(message).then((highlighted) => {
        highlightElement.innerHTML = highlighted;
      });
    }
  });

  async function handleInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    let message = target.value;
    if (message[message.length - 1] == "\n") {
      message += " ";
    }
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

  function handleCursorChange() {
    if (document.activeElement === editElement) {
      const cursorPos = (editElement as HTMLTextAreaElement).selectionStart;
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
</script>

<div class="message-editor">
  <textarea
    placeholder="MSH|^~\&|…"
    class="editor"
    oninput={handleInput}
    onscroll={handleScroll}
    bind:this={editElement}
  ></textarea>
  <div
    class="highlighting"
    aria-hidden="true"
    bind:this={highlightElement}
  ></div>
</div>

<style>
  .message-editor {
    flex: var(--message-editor-flex, 1);
    width: 100%;
    height: 100%;
    min-height: 3lh;
    padding: 1rem;
    background-color: var(--col-surface);
    border: 1px solid var(--col-highlightHigh);
    border-radius: 4px;
    position: relative;

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
  }
</style>
