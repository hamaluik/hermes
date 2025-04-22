<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let editElement: HTMLElement;
  let highlightElement: HTMLElement;

  async function handleInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    let message = target.value;

    let highlighted: string = await invoke("syntax_highlight", {
      message,
    });
    // Handle final newlines
    if (highlighted[highlighted.length - 1] == "\n") {
      highlighted += " ";
    }
    highlightElement.innerHTML = highlighted;

    handleScroll();
  }

  function handleScroll() {
    highlightElement.scrollTop = editElement.scrollTop;
    highlightElement.scrollLeft = editElement.scrollLeft;
  }
</script>

<div class="message-editor">
  <textarea
    placeholder="MSH|^~\&|…"
    class="editor"
    on:input={handleInput}
    on:scroll={handleScroll}
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
      /* Both elements need the same text and space styling so they are directly on top of each other */
      margin: 0;
      padding: 0;
      border: 0;
      width: calc(100% - 1rem);
      height: calc(100% - 1rem);
    }
    .editor,
    .highlighting {
      font-size: 16px;
      font-family: ui-monospace, Menlo, Monaco, "Cascadia Mono", "Segoe UI Mono",
        "Roboto Mono", "Oxygen Mono", "Ubuntu Mono", "Source Code Pro",
        "Fira Mono", "Droid Sans Mono", "Consolas", "Courier New", monospace;
      line-height: 1.5;
      tab-size: 2;
    }

    .editor,
    .highlighting {
      /* In the same place */
      position: absolute;
      top: 0.5rem;
      left: 0.5rem;
    }

    /* Move the .editor in front of the result */

    .editor {
      z-index: 1;
    }
    .highlighting {
      z-index: 0;
    }

    /* Make .editor almost completely transparent */

    .editor {
      color: transparent;
      background: transparent;
      caret-color: var(--col-iris);
    }

    /* Can be scrolled */
    .editor,
    .highlighting {
      overflow: auto;
      white-space: pre; /* Allows .editor to scroll horizontally */
    }

    /* No resize on .editor */
    .editor {
      resize: none;
    }

    .editor:focus {
      outline: none;
    }

    .highlighting {
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
      :global(.err) {
        color: var(--col-love) !important;
      }
    }
  }
</style>
