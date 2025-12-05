<!--
  Datetime Picker Popover

  Compact popover component for picking dates and datetimes for HL7 fields.
  Uses the HTML Popover API for positioning near the trigger element.

  ## Modes

  - date: Shows only date input, outputs YYYYMMDD format
  - datetime: Shows date + time + UTC offset dropdown + "include offset" checkbox

  ## Pre-population

  When value prop contains an existing HL7 timestamp, the component parses it
  using parseHl7Timestamp() and pre-populates the inputs. This allows users to
  edit existing values rather than always starting from scratch.

  ## Output Format

  The component calls formatDatetimeToHl7() to convert the selected date/time
  back to HL7 format (YYYYMMDDHHmmss or YYYYMMDD depending on mode).
-->
<script lang="ts">
  import { parseHl7Timestamp, formatDatetimeToHl7 } from "$lib/shared/data";

  interface Props {
    /** Unique ID for the popover element */
    id: string;
    /** Reference to trigger button for positioning (WebKit workaround) */
    anchor?: HTMLElement;
    /** Whether to show date-only or date+time+offset */
    mode: "date" | "datetime";
    /** Current HL7 timestamp value to pre-populate from */
    value: string;
    /** Callback when user applies selection */
    onselect: (hl7Value: string) => void;
    /** Optional callback when popover closes */
    onclose?: () => void;
  }

  let { id, anchor, mode, value, onselect, onclose }: Props = $props();

  // form state
  let dateValue: string = $state("");
  let timeValue: string = $state("");
  let offsetValue: string = $state("local");
  let includeOffset: boolean = $state(true);

  // common UTC offsets for the dropdown
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

  // pre-populate from existing value or default to current date/time
  $effect(() => {
    // capture current props to detect staleness after async call
    const currentValue = value;
    const currentMode = mode;

    (async () => {
      if (currentValue && currentValue.trim()) {
        // parse existing HL7 value
        const parsed = await parseHl7Timestamp(currentValue, currentMode);

        // check for stale response if props changed during async call
        if (value !== currentValue || mode !== currentMode) return;

        if (parsed.valid) {
          dateValue = parsed.date || "";
          timeValue = parsed.time || "";
          if (parsed.offset) {
            offsetValue = parsed.offset;
            includeOffset = true;
          } else {
            offsetValue = "local";
            includeOffset = false;
          }
        } else {
          // invalid existing value, fall back to current date/time
          initializeWithCurrent();
        }
      } else {
        // no existing value, use current date/time
        initializeWithCurrent();
      }
    })();
  });

  function initializeWithCurrent() {
    const now = new Date();
    dateValue = now.toISOString().split("T")[0]; // YYYY-MM-DD
    if (mode === "datetime") {
      timeValue = now.toTimeString().slice(0, 5); // HH:MM
      offsetValue = "local";
      includeOffset = true;
    }
  }

  // position popover near anchor element (WebKit doesn't do this automatically)
  const positionPopover = (popoverElement: HTMLElement) => {
    if (!anchor) return;

    const anchorRect = anchor.getBoundingClientRect();
    const popoverRect = popoverElement.getBoundingClientRect();
    const viewportHeight = window.innerHeight;
    const viewportWidth = window.innerWidth;

    // default: position below the anchor with small gap
    let top = anchorRect.bottom + 4;
    let left = anchorRect.left;

    // flip above if would overflow bottom of viewport
    if (top + popoverRect.height > viewportHeight - 8) {
      top = anchorRect.top - popoverRect.height - 4;
    }

    // keep within horizontal viewport bounds
    if (left + popoverRect.width > viewportWidth - 8) {
      left = viewportWidth - popoverRect.width - 8;
    }
    if (left < 8) left = 8;

    popoverElement.style.position = "fixed";
    popoverElement.style.top = `${top}px`;
    popoverElement.style.left = `${left}px`;
    popoverElement.style.margin = "0";
  };

  const handleApply = async () => {
    if (!dateValue) return;
    if (mode === "datetime" && !timeValue) return;

    try {
      let hl7Value: string;
      if (mode === "date") {
        // date-only mode: build ISO date string and format without time
        const datetime = `${dateValue}T00:00:00`;
        const formatted = await formatDatetimeToHl7(datetime, false);
        // extract just the date part (first 8 chars: YYYYMMDD)
        hl7Value = formatted.slice(0, 8);
      } else {
        // datetime mode: build full ISO datetime string
        let datetime: string;
        if (offsetValue === "local") {
          datetime = `${dateValue}T${timeValue}:00`;
        } else {
          datetime = `${dateValue}T${timeValue}:00${offsetValue}`;
        }
        hl7Value = await formatDatetimeToHl7(datetime, includeOffset);
      }

      onselect(hl7Value);
      // close the popover
      const popoverElement = document.getElementById(id);
      popoverElement?.hidePopover();
    } catch (error) {
      console.error("Error formatting datetime:", error);
    }
  };

  const handleClear = () => {
    onselect("");
    const popoverElement = document.getElementById(id);
    popoverElement?.hidePopover();
  };

  const handleAuto = () => {
    onselect("{auto}");
    const popoverElement = document.getElementById(id);
    popoverElement?.hidePopover();
  };

  const handleToggle = (event: Event) => {
    const popoverElement = event.target as HTMLElement;
    if (popoverElement.matches(":popover-open")) {
      // popover just opened - position it near the anchor
      positionPopover(popoverElement);
    } else {
      // popover just closed
      onclose?.();
    }
  };
</script>

<div {id} popover class="datetime-picker" ontoggle={handleToggle}>
  <form method="dialog" onsubmit={(e) => e.preventDefault()}>
    {#if mode === "date"}
      <div class="field">
        <label for="{id}-date">Date</label>
        <input type="date" id="{id}-date" bind:value={dateValue} />
      </div>
    {:else}
      <div class="row">
        <div class="field">
          <label for="{id}-date">Date</label>
          <input type="date" id="{id}-date" bind:value={dateValue} />
        </div>
        <div class="field">
          <label for="{id}-time">Time</label>
          <input type="time" id="{id}-time" bind:value={timeValue} />
        </div>
      </div>

      <div class="field">
        <label for="{id}-offset">UTC Offset</label>
        <select id="{id}-offset" bind:value={offsetValue}>
          {#each utcOffsets as offset}
            <option value={offset.value}>{offset.label}</option>
          {/each}
        </select>
      </div>

      <div class="checkbox-row">
        <input
          type="checkbox"
          id="{id}-includeOffset"
          bind:checked={includeOffset}
        />
        <label for="{id}-includeOffset">Include UTC offset</label>
      </div>
    {/if}

    <div class="button-row">
      <button type="button" class="secondary" onclick={handleClear}>
        Clear
      </button>
      <button type="button" class="secondary" onclick={initializeWithCurrent}>
        Now
      </button>
      <button type="button" class="secondary" onclick={handleAuto}>
        Auto
      </button>
      <button
        type="button"
        class="apply"
        onclick={handleApply}
        disabled={!dateValue || (mode === "datetime" && !timeValue)}
      >
        Apply
      </button>
    </div>
  </form>
</div>

<style>
  .datetime-picker {
    background: var(--col-surface);
    border: 1px solid var(--col-highlightHigh);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    padding: 1ch;
    color: var(--col-text);
    min-width: 280px;
    margin: 0;
    inset: unset; /* clear browser default positioning for JS positioning */

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
      font-size: 0.85em;
    }

    input[type="date"],
    input[type="time"],
    select {
      padding: 0.5em;
      background-color: var(--col-surface);
      border: 1px solid var(--col-muted);
      border-radius: 4px;
      color: var(--col-text);
      font-family: inherit;
      font-size: 0.9em;

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
        width: 1.1em;
        height: 1.1em;
        cursor: pointer;
        accent-color: var(--col-iris);
      }

      label {
        font-weight: normal;
        font-size: 0.9em;
        cursor: pointer;
      }
    }

    .button-row {
      display: flex;
      flex-direction: row;
      gap: 0.5ch;
      justify-content: flex-end;
      padding-top: 0.25lh;
    }

    button {
      padding: 0.4em 1.5ch;
      border: none;
      border-radius: 4px;
      font-family: inherit;
      font-size: 0.9em;
      font-weight: 500;
      cursor: pointer;
      transition: background-color 0.1s;

      &:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }
    }

    .secondary {
      background: transparent;
      color: var(--col-text);

      &:hover:not(:disabled) {
        background: var(--col-highlightHigh);
      }
    }

    .apply {
      background: var(--col-iris);
      color: var(--col-base);

      &:hover:not(:disabled) {
        background: var(--col-gold);
        color: var(--col-base);
      }
    }
  }
</style>
