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
