<!--
  Status Bar Component

  Displays contextual information at the bottom of the application window.

  Content (left to right):
  1. HL7 cursor position - path (e.g., "PID.5.1"), field name, and specification
  2. Validation summary (right side) - clickable error/warning counts that toggle the validation panel
  3. Current file path - full path to the open file, truncated from the left if needed

  HL7 Cursor Flow:
  1. Parent passes message text and cursor position
  2. Backend locateCursor() parses HL7 structure to determine position
  3. Field metadata looked up from segment schemas
  4. Backend loadSpec() fetches human-readable description
  5. All three pieces rendered on the left side

  Validation Summary:
  - Shows error and warning counts from light/full validation
  - Clicking toggles the validation panel visibility
  - Only visible when there are validation issues

  File Path Display:
  - Only shown when a file is open (currentFilePath is defined)
  - Truncates from the left to keep the filename visible when space is limited
  - Gradient fade on left edge indicates hidden content
  - Full path available in tooltip on hover
-->
<script lang="ts">
  import { locateCursor, type LocatedCursor } from "./cursor";
  import { loadSpec } from "./description";
  import type { SegmentSchemas } from "$lib/shared/schema";
  import type { ValidationResult } from "$lib/validation/validate";

  let {
    message,
    cursorPos,
    oncursorlocated,
    segmentSchemas,
    currentFilePath,
    validationResult,
    onvalidationclick,
  }: {
    message?: string;
    cursorPos?: number;
    segmentSchemas?: SegmentSchemas;
    oncursorlocated?: (locatedCursor: LocatedCursor | null) => void;
    currentFilePath?: string;
    validationResult?: ValidationResult | null;
    onvalidationclick?: () => void;
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
  <div class="hl7-info">
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
  <div class="status-right">
    {#if validationResult && validationResult.issues.length > 0}
      <button type="button" class="validation-summary" onclick={onvalidationclick}>
        {#if validationResult.summary.errors > 0}
          <span class="validation-count error">{validationResult.summary.errors} error{validationResult.summary.errors !== 1 ? 's' : ''}</span>
        {/if}
        {#if validationResult.summary.warnings > 0}
          <span class="validation-count warning">{validationResult.summary.warnings} warning{validationResult.summary.warnings !== 1 ? 's' : ''}</span>
        {/if}
      </button>
    {/if}
    {#if currentFilePath}
      <div class="file-path" title={currentFilePath}>
        <span>{currentFilePath}</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .cursor-description {
    font-size: small;
    color: var(--col-text);

    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: 2ch;
  }
  .hl7-info {
    flex-shrink: 0;

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
  .status-right {
    display: flex;
    align-items: center;
    gap: 1ch;
    min-width: 0;
    flex-shrink: 1;
  }

  .validation-summary {
    display: flex;
    align-items: center;
    gap: 0.5ch;
    background: none;
    border: none;
    cursor: pointer;
    font-size: small;
    padding: 0;
    flex-shrink: 0;

    &:hover {
      text-decoration: underline;
    }
  }

  .validation-count {
    font-size: small;
    font-weight: 500;

    &.error {
      color: var(--col-love);
    }

    &.warning {
      color: var(--col-gold);
    }
  }

  .file-path {
    font-size: small;
    color: var(--col-subtle);
    overflow: hidden;
    min-width: 0;
    flex-shrink: 1;
    display: flex;
    justify-content: flex-end;
    position: relative;

    span {
      white-space: nowrap;
    }

    /* Gradient fade on left edge to indicate truncation */
    &::before {
      content: "";
      position: absolute;
      left: 0;
      top: 0;
      bottom: 0;
      width: 2ch;
      background: linear-gradient(to right, var(--col-surface), transparent);
      pointer-events: none;
    }
  }
</style>
