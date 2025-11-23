<!--
  Segment Tab Component

  Provides a form-based UI for editing a single HL7 message segment with structured fields.

  Purpose:
  While the message editor shows raw HL7 text, this component provides a more user-friendly
  form interface where each field is a labelled input. This is especially helpful for:
  - Users unfamiliar with HL7 syntax
  - Populating common segments (PID, ORC, OBR, etc.)
  - Ensuring field values are placed in the correct positions

  Two-Way Sync:
  This component maintains bidirectional sync between the raw HL7 message and form fields:
  1. Parse: When message changes, extract this segment's data into form fields
  2. Render: When user edits form, reconstruct segment and update message

  Trigger Event Filtering:
  Some segments have fields that only apply to certain message types (trigger events).
  For example, ORC segments have different fields for orders vs results.
  The component filters displayed fields based on the message's trigger event (MSH.9.2).

  Group Membership:
  Related fields are grouped together in fieldsets (e.g., all Patient Name components).
  The groupMembership derived value organises fields by their group for rendering.
-->
<script lang="ts">
  import {
    getMessageTriggerEvent,
    parseMessageSegment,
    renderMessageSegment,
    type SegmentData,
  } from "$lib/shared/data";
  import {
    type Field,
    type SegmentSchema,
    fieldId as _fieldId,
  } from "$lib/shared/schema";
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

  // Extract trigger event from message to enable conditional field filtering
  // Trigger event (e.g., "O01", "R01") is stored in MSH.9.2 and determines
  // which fields are relevant for this message type
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

  /**
   * Groups fields by their logical grouping for display
   *
   * Fields can be organised in two ways:
   * 1. Group: Multiple related fields (e.g., Patient Name components)
   *    - Rendered as a fieldset with legend
   * 2. Standalone: Single field with no group
   *    - Rendered as individual input
   *
   * Trigger filtering: Fields with trigger_filter property are only included
   * if they match the current message's trigger event. This prevents showing
   * irrelevant fields (e.g., order-specific fields in result messages).
   *
   * Returns: Map of group name → array of fields in that group
   */
  let groupMembership: Record<string, Field[]> = $derived.by(() => {
    const groupMembership: Record<string, Field[]> = {};
    for (const field of schema) {
      // Skip fields that don't match the current trigger event
      if (field.trigger_filter) {
        if (triggerEvent != field.trigger_filter) {
          continue;
        }
      }
      if (field.group) {
        // Add to existing group or create new group
        groupMembership[field.group] = groupMembership[field.group] || [];
        groupMembership[field.group].push(field);
      } else {
        // Standalone fields get their own single-member "group" for uniform rendering
        groupMembership[fieldId(field)] = [field];
      }
    }
    return groupMembership;
  });

  /**
   * Parse effect: Message → Form Data
   *
   * When the message changes (from file load, external edit, or another tab),
   * this effect extracts the current segment's data and populates the form fields.
   *
   * This is the "message-to-form" direction of the two-way sync.
   */
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

  /**
   * Render handler: Form Data → Message
   *
   * When user edits any field, reconstruct this segment with the new data
   * and update the full message.
   *
   * Uses $state.snapshot to get a static copy of reactive data for the
   * backend Tauri command (which can't handle Svelte proxies).
   *
   * This is the "form-to-message" direction of the two-way sync.
   */
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

  /**
   * Shows field description popover when input is focused
   *
   * Popovers are rendered in the DOM but hidden by default. This finds
   * the popover within the same form-group and makes it visible.
   */
  const onfocus = (event: Event) => {
    const popover = (event.target as HTMLElement)
      .closest(".form-group")
      ?.querySelector(".popover");
    if (popover) {
      (popover as HTMLElement).classList.add("show");
    }
  };

  /**
   * Hides field description popover when input loses focus
   */
  const onblur = (event: Event) => {
    const popover = (event.target as HTMLElement)
      .closest(".form-group")
      ?.querySelector(".popover");
    if (popover) {
      (popover as HTMLElement).classList.remove("show");
    }
  };

  // Local wrapper to bind segment context to fieldId utility function
  const fieldId = (field: Field): string => {
    return _fieldId(segment, field);
  };
</script>

<div class="segment-form">
  <form>
    <!-- Render fields grouped by their logical grouping -->
    {#each Object.entries(groupMembership) as [groupName, fields]}
      {#if fields.length > 1}
        <!-- Multi-field group: render as fieldset with legend -->
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
        <!-- Single field: render without fieldset wrapper -->
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
