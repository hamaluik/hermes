<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let {
    message,
    cursorPos,
  }: {
    message?: string | null;
    cursorPos?: number | null;
  } = $props();

  let path = $state("");
  let description = $state("");
  let spec = $state("");

  interface LocatedCursor {
    segment: string | null;
    field: number | null;
    repeat: number | null;
    component: number | null;
    subcomponent: number | null;
    description_standard: string | null;
    description_wizard: string | null;
  }

  async function locateCursor(
    message: string,
    cursor: number,
  ): Promise<LocatedCursor | null> {
    return await invoke("locate_cursor", {
      message,
      cursor,
    });
  }

  function renderLocatedCursor(locatedCursor: LocatedCursor | null) {
    if (!locatedCursor?.segment) {
      path = "";
      description = "";
      spec = "";
      return;
    }
    const {
      segment,
      field,
      repeat,
      component,
      subcomponent,
      description_standard,
      description_wizard,
    } = locatedCursor;

    const fieldStr = field !== null ? `.${field}` : "";
    const repeatStr = repeat !== null ? `[${repeat}]` : "";
    const componentStr = component !== null ? `.${component}` : "";
    const subcomponentStr = subcomponent !== null ? `.${subcomponent}` : "";
    path = `${segment}${fieldStr}${repeatStr}${componentStr}${subcomponentStr}`;

    if (description_standard) {
      spec = description_standard;
    } else {
      spec = "";
    }

    if (description_wizard) {
      description = description_wizard;
    } else {
      description = "";
    }
  }

  $effect(() => {
    if (message && Number.isFinite(cursorPos)) {
      const loc = locateCursor(message, cursorPos!);
      loc.then((locatedCursor) => {
        renderLocatedCursor(locatedCursor);
      });
    } else {
      path = "";
      spec = "";
      description = "";
    }
  });
</script>

<div class="cursor-breadcrumbs">
  {#if path}
    <p>
      {#if path}
        <span class="path">{path}</span>
      {/if}
      {#if spec}
        <span class="spec">({spec})</span>
      {/if}
    </p>
  {/if}
  {#if description}
    <p class="description">
      {description}
    </p>
  {/if}
</div>

<style>
  .cursor-breadcrumbs {
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
