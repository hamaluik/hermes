<!--
  Cursor Description Component

  Displays contextual information about the current cursor position in an HL7 message.

  Purpose:
  When editing HL7 messages, users need to know what field they're currently editing.
  This component shows:
  1. The HL7 path (e.g., "PID.5.1" for Patient Name - Family Name)
  2. The human-readable field name (e.g., "Patient Name → Family Name")
  3. The field specification/description from HL7 definitions

  Flow:
  1. Parent component passes message text and cursor position
  2. Component calls backend locateCursor() to parse HL7 structure and determine position
  3. Looks up field metadata from segment schemas
  4. Calls backend loadSpec() to fetch human-readable description
  5. Renders all three pieces of information inline

  This provides real-time contextual help as users navigate through message fields,
  reducing the need to reference external HL7 documentation.
-->
<script lang="ts">
  import { locateCursor, type LocatedCursor } from "../backend/cursor";
  import { loadSpec } from "../backend/description";
  import type { SegmentSchemas } from "../backend/schema";
  let {
    message,
    cursorPos,
    oncursorlocated,
    segmentSchemas,
  }: {
    message?: string;
    cursorPos?: number;
    segmentSchemas?: SegmentSchemas;
    oncursorlocated?: (locatedCursor: LocatedCursor | null) => void;
  } = $props();

  let _path = $state("");
  let _fieldName = $state("");
  let _spec = $state("");

  /**
   * Formats the located cursor information for display
   *
   * Builds the HL7 path notation (e.g., "PID.5[0].1.2") and looks up the
   * human-readable field name from the segment schema.
   *
   * Path format breakdown:
   * - segment: Three-letter segment ID (MSH, PID, ORC, etc.)
   * - field: Field number within segment (1-based)
   * - repeat: Repeat index for repeating fields (0-based, shown in brackets)
   * - component: Component number within field (1-based)
   * - subcomponent: Subcomponent number within component (1-based)
   *
   * Example: PID.5[0].1 = Patient Name (first repeat), Family Name component
   */
  function renderLocatedCursor(locatedCursor: LocatedCursor | null) {
    if (!locatedCursor?.segment) {
      _path = "";
      _fieldName = "";
      _spec = "";
      return;
    }
    const { segment, field, repeat, component, subcomponent } = locatedCursor;

    // Build HL7 path notation
    const fieldStr = field != null ? `.${field}` : "";
    const repeatStr = repeat != null ? `[${repeat}]` : "";
    const componentStr = component != null ? `.${component}` : "";
    const subcomponentStr = subcomponent != null ? `.${subcomponent}` : "";
    _path = `${segment}${fieldStr}${repeatStr}${componentStr}${subcomponentStr}`;

    // Look up field metadata from schema to get human-readable name
    // Schema matching requires both field number and component to handle composite fields
    const fieldSchema = segmentSchemas?.[segment]?.find(
      (s) => s.field === field && s.component === component,
    );
    // Display group hierarchy (e.g., "Patient Name → Family Name") if field belongs to a group
    _fieldName =
      (fieldSchema?.group ? `${fieldSchema?.group} → ` : "") +
      (fieldSchema?.name ?? "");
  }

  /**
   * Reactive effect that updates display when cursor position changes
   *
   * This effect chains three async operations:
   * 1. locateCursor: Parses HL7 message to find structural position (backend Tauri command)
   * 2. renderLocatedCursor: Formats path and looks up field name from schema (local)
   * 3. loadSpec: Fetches human-readable description from HL7 definitions (backend Tauri command)
   *
   * The chaining ensures spec loading only happens if cursor is in a valid segment,
   * avoiding unnecessary backend calls when cursor is between segments or in whitespace.
   */
  $effect(() => {
    if (message && Number.isFinite(cursorPos)) {
      const loc = locateCursor(message, cursorPos!);
      loc
        .then((locatedCursor) => {
          // Notify parent component of cursor location (for tab navigation features)
          if (oncursorlocated) {
            oncursorlocated(locatedCursor);
          }
          renderLocatedCursor(locatedCursor);
          // Only load spec if cursor is positioned in an actual segment
          if (locatedCursor?.segment) {
            return loadSpec(
              locatedCursor.segment,
              locatedCursor.field ?? null,
              locatedCursor.component ?? null,
            );
          } else {
            return null;
          }
        })
        .then((spec) => {
          _spec = spec ?? "";
        });
    } else {
      // Clear display when no valid message or cursor position
      _path = "";
      _spec = "";
    }
  });
</script>

<div class="cursor-description">
  {#if _path}
    <p>
      {#if _path}
        <span class="path">{_path}</span>
      {/if}
      {#if _fieldName}
        <span class="field-name">{_fieldName}</span>
      {/if}
      {#if _spec}
        <span class="spec">({_spec})</span>
      {/if}
    </p>
  {/if}
</div>

<style>
  .cursor-description {
    font-size: small;
    color: var(--col-text);

    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;

    p {
      display: inline-flex;
      gap: 1ch;
      align-items: flex-end;
    }
  }
  .path {
    font-weight: bold;
    color: var(--col-iris);
  }
  .field-name {
    font-size: small;
    color: var(--col-text);
    white-space: pre-line;
  }
  .spec {
    font-size: smaller;
    color: var(--col-subtle);
  }
</style>
