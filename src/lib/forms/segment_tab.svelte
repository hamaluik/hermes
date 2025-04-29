<script lang="ts">
  import {
    getMessageTriggerEvent,
    parseMessageSegment,
    renderMessageSegment,
    type SegmentData,
  } from "../../backend/data";
  import {
    type Field,
    type SegmentSchema,
    fieldId as _fieldId,
  } from "../../backend/schema";
  import InputField from "./input_field.svelte";

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

  let triggerEvent: string | null = $state(null);
  $effect(() => {
    if (message) {
      getMessageTriggerEvent(message)
        .then((event) => {
          triggerEvent = event;
        })
        .catch((error: string) => {
          console.error("Error getting message trigger event:", error);
          triggerEvent = null;
        });
    }
  });

  // groupMembership encodes group names to the list of members by their field ID
  let groupMembership: Record<string, Field[]> = $derived.by(() => {
    const groupMembership: Record<string, Field[]> = {};
    for (const field of schema) {
      // skip fields that are filtered out by trigger_filter
      if (field.trigger_filter) {
        if (triggerEvent != field.trigger_filter) {
          continue;
        }
      }
      if (field.group) {
        groupMembership[field.group] = groupMembership[field.group] || [];
        groupMembership[field.group].push(field);
      } else {
        groupMembership[fieldId(field)] = [field];
      }
    }
    return groupMembership;
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

  const oninput = (_event: Event) => {
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
    return _fieldId(segment, field);
  };
</script>

<div class="segment-form">
  <form>
    {#each Object.entries(groupMembership) as [groupName, fields]}
      {#if fields.length > 1}
        <fieldset>
          <legend>{groupName}</legend>
          {#each fields as field}
            <InputField
              {segment}
              {field}
              bind:data={data.fields[fieldId(field)]!}
              {oninput}
              {onfocus}
              {onblur}
            />
          {/each}
        </fieldset>
      {:else}
        <InputField
          {segment}
          field={fields[0]}
          bind:data={data.fields[fieldId(fields[0])]!}
          {oninput}
          {onfocus}
          {onblur}
        />
      {/if}
    {/each}
  </form>
</div>

<style>
  form {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: flex-start;
    align-items: baseline;
    gap: 1ch;

    &::after {
      content: "";
      flex-grow: 1;
    }
  }
</style>
