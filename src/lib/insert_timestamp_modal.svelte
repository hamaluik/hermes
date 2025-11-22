<!--
  Insert Timestamp Modal

  Modal dialog for inserting a specific date/time as an HL7-formatted timestamp.
  Users select a date, time, and optionally a UTC offset using native pickers.

  ## Why Native Pickers?

  Using native HTML date/time inputs provides several benefits:
  - Platform-appropriate UI (calendar widgets, time spinners)
  - Accessibility built-in (keyboard navigation, screen readers)
  - No additional dependencies or custom date picker libraries
  - Consistent behavior across the application

  ## HL7 Timestamp Format

  Outputs timestamps in HL7 DTM format: YYYYMMDDHHmmss[+/-ZZZZ]
  - Without offset: 20250115143000
  - With offset: 20250115143000-0500

  ## UTC Offset Handling

  The offset dropdown includes common timezones plus "Local timezone". When the user
  selects a specific offset (not "Local"), the datetime string sent to the backend
  includes that offset (e.g., "2025-01-15T14:30:00-08:00"). This ensures the generated
  timestamp uses the selected offset, not the system's local timezone.

  If the UTC offset is omitted (checkbox unchecked), the timestamp should be interpreted
  in the context of the message header timestamp offset (MSH.7). If MSH.7 also lacks an
  offset, the timestamp is assumed to be in the local time of the receiver. This note is
  displayed to users in the modal.

  ## Keyboard Shortcuts

  - Enter: Insert timestamp (same as clicking Insert button)
  - Escape: Close modal without inserting
-->
<script lang="ts">
  import Modal from "./components/modal.svelte";
  import ModalHeader from "./components/modal_header.svelte";
  import ModalFooter from "./components/modal_footer.svelte";
  import { formatDatetimeToHl7 } from "../backend/data";

  let {
    show = $bindable(false),
    onInsert,
  }: {
    show: boolean;
    onInsert: (timestamp: string) => void;
  } = $props();

  // Form state - default to current date/time
  let dateValue: string = $state("");
  let timeValue: string = $state("");
  let offsetValue: string = $state("local");
  let includeOffset: boolean = $state(true);
  let errorMessage: string = $state("");

  let dateInput: HTMLInputElement | null = $state(null);

  // Common UTC offsets for the dropdown
  const utcOffsets = [
    { label: "Local timezone", value: "local" },
    { label: "UTC-12:00", value: "-12:00" },
    { label: "UTC-11:00", value: "-11:00" },
    { label: "UTC-10:00 (Hawaii)", value: "-10:00" },
    { label: "UTC-09:00 (Alaska)", value: "-09:00" },
    { label: "UTC-08:00 (Pacific)", value: "-08:00" },
    { label: "UTC-07:00 (Mountain)", value: "-07:00" },
    { label: "UTC-06:00 (Central)", value: "-06:00" },
    { label: "UTC-05:00 (Eastern)", value: "-05:00" },
    { label: "UTC-04:00 (Atlantic)", value: "-04:00" },
    { label: "UTC-03:00", value: "-03:00" },
    { label: "UTC-02:00", value: "-02:00" },
    { label: "UTC-01:00", value: "-01:00" },
    { label: "UTC+00:00 (GMT)", value: "+00:00" },
    { label: "UTC+01:00 (CET)", value: "+01:00" },
    { label: "UTC+02:00 (EET)", value: "+02:00" },
    { label: "UTC+03:00", value: "+03:00" },
    { label: "UTC+04:00", value: "+04:00" },
    { label: "UTC+05:00", value: "+05:00" },
    { label: "UTC+05:30 (IST)", value: "+05:30" },
    { label: "UTC+06:00", value: "+06:00" },
    { label: "UTC+07:00", value: "+07:00" },
    { label: "UTC+08:00 (China)", value: "+08:00" },
    { label: "UTC+09:00 (Japan)", value: "+09:00" },
    { label: "UTC+10:00 (Sydney)", value: "+10:00" },
    { label: "UTC+11:00", value: "+11:00" },
    { label: "UTC+12:00", value: "+12:00" },
    { label: "UTC+13:00", value: "+13:00" },
    { label: "UTC+14:00", value: "+14:00" },
  ];

  // Reset state and set to current date/time when modal opens
  $effect(() => {
    if (show) {
      const now = new Date();
      dateValue = now.toISOString().split("T")[0]; // YYYY-MM-DD
      timeValue = now.toTimeString().slice(0, 5); // HH:MM
      offsetValue = "local";
      includeOffset = true;
      errorMessage = "";
      // Focus date input after dialog renders
      setTimeout(() => dateInput?.focus(), 0);
    }
  });

  const handleInsert = async () => {
    if (!dateValue) {
      errorMessage = "Please select a date";
      return;
    }
    if (!timeValue) {
      errorMessage = "Please select a time";
      return;
    }

    try {
      // Build ISO datetime string
      let datetime: string;
      if (offsetValue === "local") {
        // Use local timezone - backend will handle conversion
        datetime = `${dateValue}T${timeValue}:00`;
      } else {
        // Include explicit offset
        datetime = `${dateValue}T${timeValue}:00${offsetValue}`;
      }

      const timestamp = await formatDatetimeToHl7(datetime, includeOffset);
      onInsert(timestamp);
      show = false;
    } catch (error) {
      errorMessage = `Failed to format timestamp: ${error}`;
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Enter") {
      event.preventDefault();
      handleInsert();
    }
  };

  const handleClose = () => {
    show = false;
  };
</script>

<Modal bind:show maxWidth="450px">
  <ModalHeader onclose={handleClose}>Insert Timestamp</ModalHeader>
  <main>
    <form method="dialog" onsubmit={(e) => e.preventDefault()}>
      <div class="row">
        <div class="field">
          <label for="date">Date</label>
          <input
            type="date"
            id="date"
            bind:value={dateValue}
            bind:this={dateInput}
            onkeydown={handleKeyDown}
          />
        </div>
        <div class="field">
          <label for="time">Time</label>
          <input
            type="time"
            id="time"
            bind:value={timeValue}
            onkeydown={handleKeyDown}
          />
        </div>
      </div>

      <div class="field">
        <label for="offset">UTC Offset</label>
        <select id="offset" bind:value={offsetValue} onkeydown={handleKeyDown}>
          {#each utcOffsets as offset}
            <option value={offset.value}>{offset.label}</option>
          {/each}
        </select>
      </div>

      <div class="checkbox-row">
        <input
          type="checkbox"
          id="includeOffset"
          bind:checked={includeOffset}
        />
        <label for="includeOffset">Include UTC offset in timestamp</label>
      </div>

      {#if errorMessage}
        <p class="error">{errorMessage}</p>
      {/if}

      <p class="hint">
        If the UTC offset is omitted, the timestamp should be interpreted using
        the message header timestamp offset (MSH.7), or local time if none exists.
      </p>
    </form>
  </main>
  <ModalFooter>
    {#snippet right()}
      <button class="cancel" onclick={handleClose}>Cancel</button>
      <button
        class="apply"
        onclick={handleInsert}
        disabled={!dateValue || !timeValue}
      >
        Insert
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
      gap: 0.75lh;
    }

    .row {
      display: flex;
      flex-direction: row;
      gap: 1ch;

      .field {
        flex: 1;
      }
    }

    .field {
      display: flex;
      flex-direction: column;
      gap: 0.25lh;
    }

    label {
      font-weight: 600;
      font-size: 0.9em;
    }

    input[type="date"],
    input[type="time"],
    select {
      padding: 0.5em 1ch;
      border: 1px solid var(--col-highlightHigh);
      border-radius: 4px;
      background: var(--col-surface);
      color: var(--col-text);
      font-family: inherit;
      font-size: 1em;

      &:focus {
        outline: 2px solid var(--col-iris);
        outline-offset: -1px;
      }
    }

    select {
      cursor: pointer;
    }

    .checkbox-row {
      display: flex;
      flex-direction: row;
      align-items: center;
      gap: 0.5ch;

      input[type="checkbox"] {
        width: 1.25em;
        height: 1.25em;
        cursor: pointer;
        accent-color: var(--col-iris);
      }

      label {
        font-weight: normal;
        cursor: pointer;
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
      line-height: 1.4;
    }
  }
</style>
