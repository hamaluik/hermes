<script lang="ts">
  import MessageEditor from "$lib/message_editor.svelte";
  import CursorDescription from "$lib/cursor_description.svelte";
  import Tabs from "$lib/tabs.svelte";
  import Tab from "$lib/tab.svelte";
  import SegmentTab from "$lib/forms/segment_tab.svelte";
  import { onMount } from "svelte";
  import { getAllSegmentSchemas, type SegmentSchemas } from "../backend/schema";
  import { message as messageDialog } from "@tauri-apps/plugin-dialog";
  import {
    generateDefaultData,
    getMessageSegmentNames,
    renderMessageSegment,
  } from "../backend/data";
  import Toolbar from "$lib/toolbar.svelte";
  import ToolbarButton from "$lib/toolbar_button.svelte";
  import IconNew from "$lib/icons/IconNew.svelte";
  import IconOpen from "$lib/icons/IconOpen.svelte";
  import IconSave from "$lib/icons/IconSave.svelte";
  import IconSaveAs from "$lib/icons/IconSaveAs.svelte";

  let message: string = $state("MSH|^~\\&|");
  let cursorPos: number = $state(0);
  let schemas: SegmentSchemas = $state({});
  let messageSegments: string[] = $state([]);

  $effect(() => {
    if (!message) {
      return;
    }

    getMessageSegmentNames(message)
      .then((segments) => {
        messageSegments = segments;
      })
      .catch((error: string) => {
        console.error("Error loading message segments:", error);
        messageDialog(error, {
          title: "Error Loading Message Segments",
          kind: "error",
        });
      });
  });

  onMount(() => {
    getAllSegmentSchemas()
      .then((_schemas) => {
        console.debug("Schemas loaded:", _schemas);
        schemas = _schemas;
      })
      .catch((error: string) => {
        console.error("Error loading schemas:", error);
        messageDialog(error, { title: "Error Loading Schemas", kind: "error" });
      });
  });

  const segmentRepeat = (segment: string, index: number): number => {
    return messageSegments.slice(0, index).filter((s) => s === segment).length;
  };

  const tabLabel = (index: number): string => {
    const segment = messageSegments[index];
    const count = messageSegments.filter((s) => s === segment).length;
    if (count > 1) {
      return `${segment} (${segmentRepeat(segment, index) + 1})`;
    }
    return segment;
  };
</script>

<Toolbar>
  <ToolbarButton
    title="New"
    onclick={() => {
      message = "MSH|^~\\&|";
    }}
  >
    <IconNew />
  </ToolbarButton>
  <ToolbarButton title="Open">
    <IconOpen />
  </ToolbarButton>
  <ToolbarButton title="Save">
    <IconSave />
  </ToolbarButton>
  <ToolbarButton title="Save As">
    <IconSaveAs />
  </ToolbarButton>
</Toolbar>
<main>
  <Tabs>
    {#snippet addMenu(closeMenu)}
      <ul class="add-menu">
        {#each Object.keys(schemas) as key}
          <li>
            <button
              onclick={() => {
                message = message + `\n${key}|`;
                const data = generateDefaultData(key, schemas[key] ?? {});
                renderMessageSegment(message, key, 0, data).then(
                  (newMessage) => {
                    if (newMessage) {
                      message = newMessage;
                    }
                  },
                );
                closeMenu.closeMenu();
              }}
            >
              {key}
            </button>
          </li>
        {/each}
      </ul>
    {/snippet}
    {#each messageSegments as key, index}
      {#if schemas[key]}
        <Tab label={tabLabel(index)}>
          <SegmentTab
            segment={key}
            segmentRepeat={segmentRepeat(key, index)}
            schema={schemas[key]}
            {message}
            onchange={(m) => {
              message = m;
            }}
          />
        </Tab>
      {/if}
    {/each}
  </Tabs>

  <MessageEditor
    --message-editor-flex="1"
    {message}
    onchange={(m) => {
      message = m;
    }}
    oncursorchange={(pos) => {
      cursorPos = pos;
    }}
  />
  <CursorDescription {message} {cursorPos} />
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;
    gap: 1rem;

    padding: 1rem;

    isolation: isolate;
    z-index: 0;
  }

  .add-menu {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 0.25lh;
    white-space: nowrap;
    padding: 0.5em 0;

    li {
      margin: 0;
      padding: 0;
    }

    button {
      width: 100%;
      background-color: transparent;
      outline: none;
      border: none;
      cursor: pointer;
      color: var(--col-text);
      padding: 0.25em 1ch;

      &:hover {
        background-color: var(--col-pine);
        color: var(--col-surface);
      }
    }
  }
</style>
