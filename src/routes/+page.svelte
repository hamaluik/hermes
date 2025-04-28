<script lang="ts">
  import MessageEditor from "$lib/message_editor.svelte";
  import CursorDescription from "$lib/cursor_description.svelte";
  import Tabs from "$lib/tabs.svelte";
  import Tab from "$lib/tab.svelte";
  import SegmentTab from "$lib/forms/segment_tab.svelte";
  import { onMount } from "svelte";
  import { getAllSegmentSchemas, type SegmentSchemas } from "../backend/schema";
  import {
    message as messageDialog,
    open as openDialog,
    save as saveDialog,
  } from "@tauri-apps/plugin-dialog";
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
  import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
  import type { PageProps } from "./$types";
  import ToolbarSpacer from "$lib/toolbar_spacer.svelte";
  import IconSettings from "$lib/icons/IconSettings.svelte";
  import SettingsModal from "$lib/settings_modal.svelte";

  let { data }: PageProps = $props();

  let currentFilePath: string | undefined = $state(undefined);
  let message: string = $state("MSH|^~\\&|");
  let cursorPos: number = $state(0);
  let schemas: SegmentSchemas = $state({});
  let messageSegments: string[] = $state([]);
  let toolbarHeight: string | undefined = $state(undefined);
  let setActiveTab: ((id: string) => void) | undefined = $state(undefined);
  let showSettings: ((show: boolean) => void) | undefined = $state(undefined);

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

  async function handleOpenFile() {
    const filePath = await openDialog({
      filters: [
        {
          name: "HL7 Messages",
          extensions: ["hl7"],
        },
      ],
      multiple: false,
      title: "Open HL7 Message",
    });

    if (!filePath) {
      return;
    }

    currentFilePath = undefined;
    message = await readTextFile(filePath);
    currentFilePath = filePath;
  }

  let handleSave = $derived.by(() => {
    if (!currentFilePath) {
      return undefined;
    }
    return () => {
      writeTextFile(currentFilePath!, message, {
        append: false,
        create: true,
      }).catch((error) => {
        console.error("Error saving file:", error);
        messageDialog(error, { title: "Error Saving File", kind: "error" });
      });
    };
  });

  const handleSaveAs = async () => {
    const filePath = await saveDialog({
      filters: [
        {
          name: "HL7 Messages",
          extensions: ["hl7"],
        },
      ],
      title: "Save HL7 Message",
    });
    if (!filePath) {
      return;
    }

    currentFilePath = filePath;
    await writeTextFile(filePath, message, {
      append: false,
      create: true,
    }).catch((error) => {
      console.error("Error saving file:", error);
      messageDialog(error, { title: "Error Saving File", kind: "error" });
    });
  };
</script>

<Toolbar bind:toolbarHeight>
  <ToolbarButton
    title="New"
    onclick={() => {
      message = "MSH|^~\\&|";
      currentFilePath = undefined;
      const data = generateDefaultData("MSH", schemas["MSH"] ?? {});
      renderMessageSegment(message, "MSH", 0, data).then((newMessage) => {
        if (newMessage) {
          message = newMessage;
        }
      });
    }}
  >
    <IconNew />
  </ToolbarButton>
  <ToolbarButton title="Open" onclick={handleOpenFile}>
    <IconOpen />
  </ToolbarButton>
  <ToolbarButton title="Save" onclick={handleSave}>
    <IconSave />
  </ToolbarButton>
  <ToolbarButton title="Save As" onclick={handleSaveAs}>
    <IconSaveAs />
  </ToolbarButton>
  <ToolbarSpacer />
  <ToolbarButton
    title="Settings"
    onclick={() => {
      showSettings?.(true);
    }}
  >
    <IconSettings />
  </ToolbarButton>
</Toolbar>
<main style="--toolbar-height: {toolbarHeight ?? '1px'}">
  <Tabs bind:setactive={setActiveTab}>
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
        <Tab id={key} label={tabLabel(index)}>
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
    {message}
    onchange={(m) => {
      message = m;
    }}
    oncursorchange={(pos) => {
      cursorPos = pos;
    }}
  />
  <CursorDescription
    {message}
    {cursorPos}
    segmentSchemas={schemas}
    oncursorlocated={(loc) => {
      if (!data.settings.tabsFollowCursor) {
        return;
      }
      if (loc?.segment && setActiveTab) {
        const setactive: (id: string) => void = setActiveTab;
        setactive(loc.segment);
      }
    }}
  />
</main>
<SettingsModal settings={data.settings} bind:show={showSettings} />

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;
    gap: 1rem;
    min-height: calc(100vh - var(--toolbar-height, 0px));

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
