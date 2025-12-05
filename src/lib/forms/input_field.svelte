<!--
  Input Field Component

  Reusable form input for HL7 field values with integrated validation, help text,
  value suggestions, and date/time pickers. Used throughout segment forms to
  provide a consistent editing experience.

  ## Field ID Display

  Shows the HL7 field path (e.g., "MSH-3.1") next to the field label. This helps
  users correlate form fields with the raw HL7 message structure. Field IDs are
  displayed in a muted colour and parentheses to distinguish them from the
  user-friendly field name.

  ## Date/Time Picker Integration

  Fields with a `datatype` property (`"date"` or `"datetime"`) display a picker
  button next to the input. Clicking it opens a popover with native date/time
  inputs that format values to HL7 DTM format (YYYYMMDD or YYYYMMDDHHmmss±ZZZZ).

  The picker pre-populates from existing HL7 values when present, allowing users
  to edit rather than re-enter timestamps. This uses the `parseHl7Timestamp()`
  bridge to convert HL7 format to ISO for the native inputs.

  Why native inputs: Platform date/time pickers provide familiar UI, accessibility,
  and keyboard navigation without additional dependencies.

  ## Datalist Integration (Value Suggestions)

  When a field has predefined values (e.g., message types, gender codes), we
  generate a datalist element linked to the input. This provides autocomplete
  suggestions as the user types, while still allowing free-form input.

  The datalist ID is derived from the field ID to ensure uniqueness when multiple
  instances of the same segment appear in a message (e.g., multiple OBX segments).

  ## Popover Help Text

  If a field has a "note" property, it's rendered in a popover that appears below
  the input. The popover is initially hidden and can be shown/hidden via CSS class
  manipulation (typically on focus/blur events from the parent component).

  This keeps the UI clean while providing contextual help when needed. The popover
  positioning is relative to the input, extending slightly beyond it (3ch on each side)
  to ensure visibility for longer help text.
-->
<script lang="ts">
  import {
    fieldId as _fieldId,
    DataType,
    type Field,
  } from "$lib/shared/schema";
  import DateTimePicker from "./datetime_picker.svelte";
  import IconCalendar from "$lib/icons/IconCalendar.svelte";
  import IconCalendarClock from "$lib/icons/IconCalendarClock.svelte";

  let {
    segment,
    field,
    data = $bindable(),
    oninput,
    onfocus,
    onblur,
  }: {
    segment: string;
    field: Field;
    data: string;
    oninput?: (event: Event) => void;
    onfocus?: (event: Event) => void;
    onblur?: (event: Event) => void;
  } = $props();

  let fieldId = $derived(_fieldId(segment, field));

  // reference to picker button for positioning the popover
  let pickerAnchor: HTMLElement = $state(null!);

  // Generate unique datalist ID only when the field has predefined values
  let datalistId = $derived.by(() => {
    if (!field.values) {
      return null;
    }
    return `${fieldId}.datalist`;
  });

  // Generate unique picker ID for datetime fields
  let pickerId = $derived(`${fieldId}.picker`);

  const handlePickerSelect = (hl7Value: string) => {
    data = hl7Value;
    // trigger the oninput handler to sync with parent
    const syntheticEvent = new Event("input");
    oninput?.(syntheticEvent);
  };
</script>

<div
  class="form-group"
  style={field.datatype == DataType.DateTime
    ? "flex-basis: 22ch;"
    : field.datatype == DataType.Date
      ? "flex-basis: 12ch;"
      : ""}
>
  <label for={field.name}
    >{field.name} <span class="field-id">{fieldId}</span></label
  >
  {#if field.datatype == DataType.Date || field.datatype == DataType.DateTime}
    <div class="input-with-picker">
      <input
        type="text"
        id={field.name}
        name={field.name}
        bind:value={data}
        {oninput}
        {onfocus}
        {onblur}
        minlength={field.minlength}
        maxlength={field.maxlength}
        placeholder={field.placeholder}
        required={field.required}
        pattern={field.pattern}
        list={datalistId}
      />
      <button
        type="button"
        bind:this={pickerAnchor}
        popovertarget={pickerId}
        title={field.datatype === DataType.Date
          ? "Pick date"
          : "Pick date/time"}
      >
        {#if field.datatype === DataType.Date}
          <IconCalendar />
        {:else}
          <IconCalendarClock />
        {/if}
      </button>
      <DateTimePicker
        id={pickerId}
        anchor={pickerAnchor}
        mode={field.datatype === DataType.Date ? "date" : "datetime"}
        value={data}
        onselect={handlePickerSelect}
      />
    </div>
  {:else}
    <input
      type="text"
      id={field.name}
      name={field.name}
      bind:value={data}
      {oninput}
      {onfocus}
      {onblur}
      minlength={field.minlength}
      maxlength={field.maxlength}
      placeholder={field.placeholder}
      required={field.required}
      pattern={field.pattern}
      list={datalistId}
    />
  {/if}
  {#if field.note}
    <div class="popover">
      <p>{field.note}</p>
    </div>
  {/if}
  {#if datalistId && field.values}
    <datalist id={datalistId}>
      {#each Object.entries(field.values) as [value, label]}
        <option {value} {label}></option>
      {/each}
    </datalist>
  {/if}
</div>

<style>
  .form-group {
    flex: 1 1 auto;
    width: min-content;
    max-width: 30ch;

    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: auto auto;
    grid-template-areas: "label" "input";
    gap: 0;
    align-items: stretch;
    position: relative;

    label {
      margin-bottom: 0.1lh;
      font-size: small;
      color: var(--col-text);
      white-space: nowrap;

      .field-id {
        font-size: x-small;
        color: var(--col-subtle);

        &::before {
          content: "(";
        }

        &::after {
          content: ")";
        }
      }
    }
  }

  .input-with-picker {
    display: flex;
    flex-direction: row;
    gap: 0;

    input {
      flex: 1;
      min-width: 0;
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
      border-right: none;
      padding-right: 0;
      margin-right: 0;
    }

    input:invalid + button {
      color: var(--col-love);
      border-color: var(--col-love);
    }

    input:focus + button {
      outline: none;
      outline-offset: -1px;
      border-color: var(--col-iris);
      box-shadow: 0 0 0 1px var(--col-iris);
    }

    button {
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 0.25em;
      background-color: var(--col-surface);
      border: 1px solid var(--col-muted);
      border-left: none;
      padding-left: 0;
      margin-left: 0;
      border-radius: 0 4px 4px 0;
      color: var(--col-subtle);
      cursor: pointer;
      aspect-ratio: 1;
      height: 100%;

      &:hover {
        color: var(--col-text);
      }

      &:focus {
        outline: none;
        outline-offset: -1px;
        border-color: var(--col-iris);
        box-shadow: 0 0 0 1px var(--col-iris);
      }

      :global(svg) {
        width: 1em;
        height: 1em;
      }
    }
  }

  .popover {
    display: none;
    position: absolute;
    top: calc(100% + 0.25rem);
    left: -3ch;
    right: -3ch;
    color: var(--col-text);
    background-color: var(--col-overlay);
    padding: 0.5ch;
    border: 1px solid var(--col-highlightHigh);
    z-index: 1000;
    border-radius: 4px;
    font-size: smaller;
    white-space: pre-line;

    :global(&.show) {
      display: block;
    }
  }
</style>
