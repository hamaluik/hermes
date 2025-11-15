<!--
  Message Editor Component

  A specialized text editor for HL7 messages with syntax highlighting and field navigation.

  Architecture:
  This component uses a "textarea overlay" pattern to achieve syntax highlighting in a
  truly editable text area. Native textareas don't support syntax highlighting, so we:
  1. Layer a transparent textarea over a syntax-highlighted <div>
  2. Synchronize scroll positions between the two elements
  3. Make the textarea text transparent while keeping the caret visible

  This approach preserves all native textarea behaviors (selection, clipboard, IME support)
  while providing visual feedback through the highlighted overlay.

  Key Features:
  - Real-time syntax highlighting via backend Tauri command
  - Tab/Shift+Tab navigation between HL7 fields (using backend cursor tracking)
  - Ctrl/Cmd+Enter shortcut for quick message sending
  - Document-level cursor tracking for cross-component coordination
  - One-click copy-to-clipboard with visual feedback

  Flow for New Developers:
  1. User types in textarea → handleInput fires
  2. handleInput calls backend syntaxHighlight command
  3. Highlighted HTML is injected into overlay div
  4. Scroll synchronization keeps overlay aligned with textarea
  5. Cursor changes trigger document-level events to update field descriptions elsewhere
-->
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

  // Auto-reset the "copied" state after 2 seconds to provide temporary visual feedback
  // This allows the copy button icon to revert from checkmark back to clipboard
  $effect(() => {
    if (copied) {
      setTimeout(() => {
        copied = false;
      }, 2000);
    }
  });

  let selectionListener: () => void;

  // Sync external message updates to both the textarea and highlighting overlay
  // This effect runs when the parent component updates the message prop (e.g., loading a file)
  // We update both elements to maintain the textarea overlay pattern
  $effect(() => {
    if (message) {
      (editElement as HTMLTextAreaElement).value = message;
      syntaxHighlight(message).then((highlighted) => {
        highlightElement.innerHTML = highlighted;
      });
    }
  });

  /**
   * Handles user input in the textarea
   *
   * This is called on every keystroke and performs three critical operations:
   * 1. Notifies parent component of the change (for state management)
   * 2. Re-highlights the message to reflect syntax changes
   * 3. Syncs scroll position (in case content height changed)
   */
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

  /**
   * Synchronizes scroll position between textarea and highlighting overlay
   *
   * Critical for maintaining the overlay illusion - if these get out of sync,
   * users will see highlighted text not aligned with what they're typing.
   */
  function handleScroll() {
    highlightElement.scrollTop = editElement.scrollTop;
    highlightElement.scrollLeft = editElement.scrollLeft;
  }

  /**
   * Handles keyboard shortcuts for improved HL7 message editing workflow
   *
   * Tab/Shift+Tab: Navigate between HL7 fields
   *   - Uses backend cursor.ts to understand HL7 structure and find field boundaries
   *   - Prevents default tab behavior (which would move focus to next element)
   *   - Automatically selects the entire field content for easy replacement
   *   - Shift+Tab navigates backwards through fields
   *
   * Ctrl/Cmd+Enter: Quick send shortcut
   *   - Common pattern in messaging UIs (Slack, Discord, etc.)
   *   - Allows sending without leaving the keyboard to click "Send"
   */
  async function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Tab") {
      event.preventDefault();
      if (!message || !editElement) {
        return;
      }
      // Determine direction based on Shift key
      let range: { start: number; end: number } | undefined | null;
      if (event.shiftKey) {
        range = await getRangeOfPreviousField(message, _cursorPos);
      } else {
        range = await getRangeOfNextField(message, _cursorPos);
      }
      if (!range) {
        return;
      }
      // Select the entire field content so user can immediately type to replace it
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

  /**
   * Tracks cursor position changes and notifies parent component
   *
   * Only fires when this textarea has focus to avoid interference from other editors.
   * The parent uses this to update the field description panel in real-time.
   */
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
    // Initialize scroll sync on mount
    highlightElement.scrollTop = editElement.scrollTop;
    highlightElement.scrollLeft = editElement.scrollLeft;

    // Use document-level selectionchange event instead of element-level events
    // because selectionchange is more reliable and fires for keyboard, mouse, and
    // programmatic selection changes (unlike mouseup/keyup which miss some cases)
    selectionListener = () => {
      if (document.activeElement === editElement) {
        handleCursorChange();
      }
    };
    document.addEventListener("selectionchange", selectionListener);
  });

  onDestroy(() => {
    // Clean up document-level event listener to prevent memory leaks
    document.removeEventListener("selectionchange", selectionListener);
  });

  // Calculate editor height based on content (minimum 3 lines for empty messages)
  // This allows the editor to grow naturally with content when no explicit height is set
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
  <!-- Highlighting overlay positioned behind the textarea via CSS z-index -->
  <div
    class="highlighting"
    aria-hidden="true"
    bind:this={highlightElement}
  ></div>
  <!-- Copy-to-clipboard button (appears on hover via CSS) -->
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

    /* Textarea and highlighting div share identical positioning and typography
       to maintain perfect alignment for the overlay pattern */
    .editor,
    .highlighting {
      margin: 0;
      padding: 0;
      border: 0;
      width: calc(100% - 1rem);
      height: calc(100% - 1rem);
      /* Monospace font ensures character widths are predictable and consistent */
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

    /* Textarea layer: transparent text but visible caret */
    .editor {
      z-index: 1;
      color: transparent; /* Hide the text itself */
      background: transparent; /* See through to highlighting below */
      caret-color: var(--col-iris); /* But keep cursor visible */
      resize: none;

      &:focus {
        outline: none;
      }
    }

    /* Highlighting layer: provides colored syntax feedback behind textarea */
    .highlighting {
      z-index: 0;

      /* Syntax highlighting color classes (applied by backend syntax_highlight.rs) */
      :global(.msh) {
        color: var(--col-pine); /* MSH header segment (special) */
      }
      :global(.seps) {
        color: var(--col-subtle); /* MSH separator declaration (^~\&) */
      }
      :global(.seg) {
        color: var(--col-foam); /* Segment identifiers (PID, ORC, OBR, etc.) */
      }
      :global(.sep) {
        color: var(--col-muted); /* Field/component/subcomponent separators (|^~\&) */
      }
      :global(.cell) {
        color: var(--col-text); /* Field/component/subcomponent content */
      }
      :global(.temp) {
        color: var(--col-gold); /* Template placeholders (e.g., <timestamp>) */
      }
      :global(.ts) {
        color: var(--col-iris); /* Timestamps */
      }
      :global(.err) {
        color: var(--col-love) !important; /* Parse errors */
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
