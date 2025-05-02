<script lang="ts">
  import { fieldId as _fieldId, type Field } from "../../backend/schema";

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
