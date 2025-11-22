<!--
  Jump to Field Modal

  Modal dialog for navigating to a specific HL7 field by path. Users enter a field
  path like "PID.5.1" and the cursor jumps to that field in the message editor.

  ## Query Syntax

  Uses hl7-parser's query syntax:
  - `PID.5` - Fifth field of first PID segment
  - `PID.5.1` - First component of fifth field
  - `PID[2].5` - Fifth field of second PID segment occurrence
  - `PID.5[1].1` - First component of first repeat of fifth field

  ## Error Handling

  If the field path is invalid or the field doesn't exist in the message,
  an error message is displayed. The modal stays open so users can correct
  their input.

  ## Keyboard Shortcuts

  - Enter: Jump to field (same as clicking Jump button)
  - Escape: Close modal without jumping
-->
<script lang="ts">
  import Modal from "./components/modal.svelte";
  import ModalHeader from "./components/modal_header.svelte";
  import ModalFooter from "./components/modal_footer.svelte";
  import { getFieldRange } from "../backend/cursor";

  let {
    show = $bindable(false),
    message,
    onJump,
  }: {
    show: boolean;
    message: string;
    onJump: (start: number, end: number) => void;
  } = $props();

  let fieldPath: string = $state("");
  let errorMessage: string = $state("");
  let inputElement: HTMLInputElement | null = $state(null);

  // Reset state when modal opens and focus input
  $effect(() => {
    if (show) {
      fieldPath = "";
      errorMessage = "";
      // Focus input after dialog renders
      setTimeout(() => inputElement?.focus(), 0);
    }
  });

  const handleJump = async () => {
    if (!fieldPath.trim()) {
      errorMessage = "Please enter a field path";
      return;
    }

    // Convert to uppercase since HL7 segment names are case-sensitive (uppercase)
    const normalizedPath = fieldPath.trim().toUpperCase();
    const range = await getFieldRange(message, normalizedPath);
    if (range === null) {
      errorMessage = `Field "${fieldPath}" not found`;
      return;
    }

    onJump(range.start, range.end);
    show = false;
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Enter") {
      event.preventDefault();
      handleJump();
    }
  };

  const handleClose = () => {
    show = false;
  };
</script>

<Modal bind:show maxWidth="400px">
  <ModalHeader onclose={handleClose}>Jump to Field</ModalHeader>
  <main>
    <form method="dialog" onsubmit={(e) => e.preventDefault()}>
      <label for="fieldPath">Field Path</label>
      <input
        type="text"
        id="fieldPath"
        placeholder="e.g., PID.5.1"
        bind:value={fieldPath}
        bind:this={inputElement}
        onkeydown={handleKeyDown}
        autocomplete="off"
        spellcheck="false"
      />
      {#if errorMessage}
        <p class="error">{errorMessage}</p>
      {/if}
      <p class="hint">
        Examples: PID.5, MSH.9.1, PID[2].3
      </p>
    </form>
  </main>
  <ModalFooter>
    {#snippet right()}
      <button class="cancel" onclick={handleClose}>Cancel</button>
      <button class="apply" onclick={handleJump} disabled={!fieldPath.trim()}>
        Jump
      </button>
    {/snippet}
  </ModalFooter>
</Modal>

<style>
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: stretch;
    padding: 0.5rem 2ch;
    color: var(--col-text);

    form {
      display: flex;
      flex-direction: column;
      gap: 0.5lh;
    }

    label {
      font-weight: 600;
    }

    input {
      padding: 0.5em 1ch;
      border: 1px solid var(--col-highlightHigh);
      border-radius: 4px;
      background: var(--col-surface);
      color: var(--col-text);
      font-family: monospace;
      font-size: 1em;

      &:focus {
        outline: 2px solid var(--col-iris);
        outline-offset: -1px;
      }
    }

    .error {
      color: var(--col-love);
      font-size: 0.9em;
      margin: 0;
    }

    .hint {
      color: var(--col-muted);
      font-size: 0.85em;
      margin: 0;
    }
  }
</style>
