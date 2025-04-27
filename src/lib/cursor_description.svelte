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

  function renderLocatedCursor(locatedCursor: LocatedCursor | null) {
    if (!locatedCursor?.segment) {
      _path = "";
      _fieldName = "";
      _spec = "";
      return;
    }
    const { segment, field, repeat, component, subcomponent } = locatedCursor;

    const fieldStr = field != null ? `.${field}` : "";
    const repeatStr = repeat != null ? `[${repeat}]` : "";
    const componentStr = component != null ? `.${component}` : "";
    const subcomponentStr = subcomponent != null ? `.${subcomponent}` : "";
    _path = `${segment}${fieldStr}${repeatStr}${componentStr}${subcomponentStr}`;

    const fieldSchema = segmentSchemas?.[segment]?.find(
      (s) => s.field === field && s.component === component,
    );
    _fieldName =
      (fieldSchema?.group ? `${fieldSchema?.group} → ` : "") +
      (fieldSchema?.name ?? "");
  }

  $effect(() => {
    if (message && Number.isFinite(cursorPos)) {
      const loc = locateCursor(message, cursorPos!);
      loc
        .then((locatedCursor) => {
          if (oncursorlocated) {
            oncursorlocated(locatedCursor);
          }
          renderLocatedCursor(locatedCursor);
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
