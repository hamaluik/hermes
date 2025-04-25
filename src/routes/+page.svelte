<script lang="ts">
  import MessageEditor from "$lib/message_editor.svelte";
  import CursorDescription from "$lib/cursor_description.svelte";
  import Tabs from "$lib/tabs.svelte";
  import Tab from "$lib/tab.svelte";
  import SegmentTab from "$lib/forms/segment_tab.svelte";
  import { onMount } from "svelte";
  import { getAllSegmentSchemas, type SegmentSchemas } from "../backend/schema";
  import { message as messageDialog } from "@tauri-apps/plugin-dialog";

  let message: string = $state("MSH|^~\\&|");
  let cursorPos: number = $state(0);
  let schemas: SegmentSchemas = $state({});

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
</script>

<main>
  <Tabs>
    {#each Object.entries(schemas) as [key, schema]}
      <Tab label={key}>
        <SegmentTab
          segment={key}
          {schema}
          {message}
          onchange={(m) => {
            message = m;
          }}
        />
      </Tab>
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
