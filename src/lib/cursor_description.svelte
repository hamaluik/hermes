<script lang="ts">
  import {
    getSpecAndDescription,
    locateCursor,
    type LocatedCursor,
  } from "../backend";

  let {
    message,
    cursorPos,
    oncursorlocated,
  }: {
    message?: string | null;
    cursorPos?: number | null;
    oncursorlocated?: (locatedCursor: LocatedCursor | null) => void;
  } = $props();

  let _path = $state("");
  let _description = $state("");
  let _spec = $state("");

  function renderLocatedCursor(locatedCursor: LocatedCursor | null) {
    if (!locatedCursor?.segment) {
      _path = "";
      _description = "";
      _spec = "";
      return;
    }
    const { segment, field, repeat, component, subcomponent } = locatedCursor;

    const fieldStr = field !== null ? `.${field}` : "";
    const repeatStr = repeat !== null ? `[${repeat}]` : "";
    const componentStr = component !== null ? `.${component}` : "";
    const subcomponentStr = subcomponent !== null ? `.${subcomponent}` : "";
    _path = `${segment}${fieldStr}${repeatStr}${componentStr}${subcomponentStr}`;
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
            return getSpecAndDescription(
              locatedCursor.segment,
              locatedCursor.field,
              locatedCursor.component,
            );
          } else {
            return { spec: null, description: null };
          }
        })
        .then(({ spec, description }) => {
          _spec = spec ?? "";
          _description = description ?? "";
        });
    } else {
      _path = "";
      _spec = "";
      _description = "";
    }
  });
</script>

<div class="cursor-description">
  {#if _path}
    <p>
      {#if _path}
        <span class="path">{_path}</span>
      {/if}
      {#if _spec}
        <span class="spec">({_spec})</span>
      {/if}
    </p>
  {/if}
  {#if _description}
    <p class="description">
      {_description}
    </p>
  {/if}
</div>

<style>
  .cursor-description {
    font-size: 0.9em;
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
  .spec {
    font-size: 0.8em;
    color: var(--col-subtle);
  }
  .description {
    font-size: 0.8em;
    color: var(--col-text);
    white-space: pre-line;
  }
</style>
