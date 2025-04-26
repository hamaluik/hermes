<script lang="ts">
  import MessageEditor from "$lib/message_editor.svelte";
  import CursorDescription from "$lib/cursor_description.svelte";
  import Tabs from "$lib/tabs.svelte";
  import Tab from "$lib/tab.svelte";
  import SegmentTab from "$lib/forms/segment_tab.svelte";
  import { onMount } from "svelte";
  import { getAllSegmentSchemas, type SegmentSchemas } from "../backend/schema";
  import { message as messageDialog } from "@tauri-apps/plugin-dialog";
  import { getMessageSegmentNames } from "../backend/data";

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

<main>
  <Tabs>
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
    min-height: 100vh;
  }
</style>
