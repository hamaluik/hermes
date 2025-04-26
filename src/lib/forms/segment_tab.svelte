<script lang="ts">
  import {
    parseMessageSegment,
    renderMessageSegment,
    type SegmentData,
  } from "../../backend/data";
  import type { Field, SegmentSchema } from "../../backend/schema";

  let {
    segment,
    segmentRepeat,
    schema,
    message,
    onchange,
  }: {
    segment: string;
    segmentRepeat: number;
    schema: SegmentSchema;
    message?: string;
    onchange?: (message: string) => void;
  } = $props();

  let data: SegmentData = $state({ fields: {} });

  const datalistId = (field: Field): string | null => {
    if (!field.values) {
      return null;
    }
    return `${fieldId(field)}.datalist`;
  };

  let datalists: Record<string, { value: string; label: string }[]> =
    $derived.by(() => {
      const datalists: Record<string, { value: string; label: string }[]> = {};
      for (const field of schema) {
        if (field.values) {
          datalists[datalistId(field)!] = Object.entries(field.values).map(
            ([value, label]) => {
              return { value, label };
            },
          );
          datalists[datalistId(field)!].sort((a, b) => {
            return a.value.localeCompare(b.value);
          });
        }
      }
      return datalists;
    });

  $effect(() => {
    if (message) {
      parseMessageSegment(message, segment, segmentRepeat)
        .then((parsedSegment) => {
          if (parsedSegment) {
            data = parsedSegment;
          }
        })
        .catch((error: string) => {
          console.error("Error parsing message segment:", error);
        });
    }
  });

  const oninput = () => {
    if (onchange && message) {
      renderMessageSegment(
        message,
        segment,
        segmentRepeat,
        $state.snapshot(data),
      ).then((renderedMessage) => {
        if (renderedMessage) {
          onchange(renderedMessage);
        }
      });
    }
  };

  const onfocus = (event: Event) => {
    const popover = (event.target as HTMLElement)
      .closest(".form-group")
      ?.querySelector(".popover");
    if (popover) {
      (popover as HTMLElement).classList.add("show");
    }
  };

  const onblur = (event: Event) => {
    const popover = (event.target as HTMLElement)
      .closest(".form-group")
      ?.querySelector(".popover");
    if (popover) {
      (popover as HTMLElement).classList.remove("show");
    }
  };

  const fieldId = (field: Field): string => {
    return (
      `${segment}.${field.field}` +
      (Number.isFinite(field.component) ? `.${field.component}` : "")
    );
  };
</script>

<div class="segment-form">
  <form>
    {#each schema as field}
      <div class="form-group">
        <label for={field.name}
          >{field.name} <span class="field-id">{fieldId(field)}</span></label
        >
        <input
          type="text"
          id={field.name}
          name={field.name}
          bind:value={data.fields[fieldId(field)]}
          {oninput}
          {onfocus}
          {onblur}
          minlength={field.minlength}
          maxlength={field.maxlength}
          placeholder={field.placeholder}
          required={field.required}
          pattern={field.pattern}
          list={datalistId(field)}
        />
        {#if field.note}
          <div class="popover">
            <p>{field.note}</p>
          </div>
        {/if}
      </div>
    {/each}
    {#each Object.entries(datalists) as [key, values]}
      <datalist id={key}>
        {#each values as { value, label }}
          <option {value} {label}></option>
        {/each}
      </datalist>
    {/each}
  </form>
</div>

<style></style>
