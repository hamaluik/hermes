<!--
  Input Field Component

  Reusable form input for HL7 field values with integrated validation, help text,
  and value suggestions. Used throughout segment forms to provide a consistent
  editing experience.

  ## Field ID Display

  Shows the HL7 field path (e.g., "MSH-3.1") next to the field label. This helps
  users correlate form fields with the raw HL7 message structure. Field IDs are
  displayed in a muted color and parentheses to distinguish them from the
  user-friendly field name.

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
  import { fieldId as _fieldId, type Field } from "$lib/shared/schema";

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

  // Generate unique datalist ID only when the field has predefined values
  let datalistId = $derived.by(() => {
    if (!field.values) {
      return null;
    }
    return `${fieldId}.datalist`;
  });
</script>

<div class="form-group">
  <label for={field.name}
    >{field.name} <span class="field-id">{fieldId}</span></label
  >
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
