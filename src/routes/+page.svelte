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
  import IconHelp from "$lib/icons/IconHelp.svelte";
  import ToolbarSeparator from "$lib/toolbar_separator.svelte";
  import IconSendReceive from "$lib/icons/IconSendReceive.svelte";
  import { get } from "svelte/store";
  import MessageSendModal from "$lib/message_send_modal.svelte";
  import IconListen from "$lib/icons/IconListen.svelte";
  import ListenModal from "$lib/listen_modal.svelte";
  import NotificationIcon from "$lib/notification_icon.svelte";
  import { listenToListenResponse } from "../backend/listen";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import HeaderWizard from "$lib/wizards/header_wizard.svelte";

  let { data }: PageProps = $props();

  let message: string = $state("");
  let savedMessage: string = $state("");
  let cursorPos: number = $state(0);
  let schemas: SegmentSchemas = $state({});
  let messageSegments: string[] = $state([]);
  let toolbarHeight: string | undefined = $state(undefined);
  let setActiveTab: ((id: string) => void) | undefined = $state(undefined);
  let showSettings: ((show: boolean) => void) | undefined = $state(undefined);
  let showSend = $state(false);
  let currentFilePath: string | undefined = $state(undefined);

  let currentWizardModal: string | null = $state(null);

  let showListeningModal = $state(false);

  let listening = $state(false);
  let unreadMessageCount = $state(0);

  // Message editor resize state
  let editorHeight = $state(data.settings.editorHeight); // Height in pixels
  let isResizing = $state(false);
  let resizeStartY = 0;
  let resizeStartHeight = 0;
  let windowHeight = $state(window.innerHeight);
  let atMinBound = $state(false);
  let atMaxBound = $state(false);

  const MIN_EDITOR_HEIGHT = 100; // 100px minimum
  const MAX_EDITOR_HEIGHT = $derived(windowHeight * 0.6); // 60% of viewport

  const WIZARD_SEGMENTS = ["MSH", "PID", "PV1"];
  const WIZARD_COMPONENTS: Record<string, any> = {
    MSH: HeaderWizard,
  };

  function handleResizeStart(event: PointerEvent) {
    event.preventDefault();
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    isResizing = true;
    resizeStartY = event.clientY;
    resizeStartHeight = editorHeight;
  }

  function handleResizeMove(event: PointerEvent) {
    if (!isResizing) return;
    event.preventDefault();

    const delta = event.clientY - resizeStartY;
    // Invert delta because handle is at top: dragging up (negative delta) should increase height
    const newHeight = resizeStartHeight - delta;
    const clampedHeight = Math.max(
      MIN_EDITOR_HEIGHT,
      Math.min(MAX_EDITOR_HEIGHT, newHeight),
    );

    // Track if we're at the bounds for visual feedback
    atMinBound =
      clampedHeight === MIN_EDITOR_HEIGHT && newHeight < MIN_EDITOR_HEIGHT;
    atMaxBound =
      clampedHeight === MAX_EDITOR_HEIGHT && newHeight > MAX_EDITOR_HEIGHT;

    editorHeight = clampedHeight;
  }

  function handleResizeEnd(event: PointerEvent) {
    if (!isResizing) return;
    event.preventDefault();

    const target = event.currentTarget as HTMLElement;
    target.releasePointerCapture(event.pointerId);
    isResizing = false;
    atMinBound = false;
    atMaxBound = false;

    // Save the new height to settings
    data.settings.editorHeight = editorHeight;
  }

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
    message = get(data.message);

    let unlisten: UnlistenFn | undefined = undefined;
    listenToListenResponse(data.listenedMessages).then((_unlisten) => {
      unlisten = _unlisten;
    });

    data.listening.subscribe((value) => {
      listening = value;
    });
    data.listenedMessages.subscribe((value) => {
      unreadMessageCount = value.filter((m) => m.unread).length;
    });

    getAllSegmentSchemas()
      .then((_schemas) => {
        console.debug("Schemas loaded:", _schemas);
        schemas = _schemas;
      })
      .catch((error: string) => {
        console.error("Error loading schemas:", error);
        messageDialog(error, { title: "Error Loading Schemas", kind: "error" });
      });

    // Listen for window resize events to update MAX_EDITOR_HEIGHT
    const handleWindowResize = () => {
      windowHeight = window.innerHeight;
      // Clamp editor height if it exceeds new max
      if (editorHeight > MAX_EDITOR_HEIGHT) {
        editorHeight = MAX_EDITOR_HEIGHT;
        data.settings.editorHeight = editorHeight;
      }
    };
    window.addEventListener("resize", handleWindowResize);

    return () => {
      unlisten?.();
      window.removeEventListener("resize", handleWindowResize);
    };
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
    savedMessage = message;
    currentFilePath = filePath;
  }

  let handleSave = $derived.by(() => {
    if (!currentFilePath || message === savedMessage) {
      return undefined;
    }
    return () => {
      writeTextFile(currentFilePath!, message, {
        append: false,
        create: true,
      })
        .then(() => {
          savedMessage = message;
        })
        .catch((error) => {
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
    })
      .then(() => {
        savedMessage = message;
      })
      .catch((error) => {
        console.error("Error saving file:", error);
        messageDialog(error, { title: "Error Saving File", kind: "error" });
      });
  };

  const handleListen = () => {
    showListeningModal = true;
  };
</script>

<Toolbar bind:toolbarHeight>
  <ToolbarButton
    title="New"
    onclick={() => {
      message = "MSH|^~\\&|";
      currentFilePath = undefined;
      const defaultData = generateDefaultData("MSH", schemas["MSH"] ?? {});
      renderMessageSegment(message, "MSH", 0, defaultData).then(
        (newMessage) => {
          if (newMessage) {
            message = newMessage;
            savedMessage = message;
          }
        },
      );
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
  <ToolbarSeparator />
  <ToolbarButton
    title="Send/Receive"
    onclick={() => {
      showSend = true;
    }}
  >
    <IconSendReceive />
  </ToolbarButton>
  <ToolbarButton title="Listen" onclick={handleListen}>
    <NotificationIcon count={unreadMessageCount}>
      <span class={listening ? "listening" : "notListening"}>
        <IconListen />
      </span>
    </NotificationIcon>
  </ToolbarButton>
  <ToolbarSpacer />
  <ToolbarButton title="Help">
    <IconHelp />
  </ToolbarButton>
  <ToolbarButton
    title="Settings"
    onclick={() => {
      showSettings?.(true);
    }}
  >
    <IconSettings />
  </ToolbarButton>
</Toolbar>
<main
  style="--toolbar-height: {toolbarHeight ?? '1px'};"
  class:resizing={isResizing}
>
  <div class="tabs-scroll-container">
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
          <Tab
            id={key}
            label={tabLabel(index)}
            onWizard={WIZARD_SEGMENTS.includes(key)
              ? () => {
                  currentWizardModal = key;
                }
              : undefined}
          >
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
  </div>

  <div
    class="resize-handle"
    class:resizing={isResizing}
    class:at-min-bound={atMinBound}
    class:at-max-bound={atMaxBound}
    onpointerdown={handleResizeStart}
    onpointermove={handleResizeMove}
    onpointerup={handleResizeEnd}
    role="separator"
    aria-orientation="horizontal"
    aria-label="Resize message editor"
  >
    <div class="resize-grip"></div>
  </div>

  <MessageEditor
    {message}
    height={editorHeight}
    onchange={(m) => {
      message = m;
    }}
    oncursorchange={(pos) => {
      cursorPos = pos;
    }}
    onctrlenter={() => {
      showSend = true;
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
{#if showListeningModal}
  <ListenModal
    bind:show={showListeningModal}
    listening={data.listening}
    listenedMessages={data.listenedMessages}
  />
{/if}
{#if showSend}
  <MessageSendModal bind:show={showSend} settings={data.settings} {message} />
{/if}
{#if currentWizardModal && WIZARD_COMPONENTS[currentWizardModal]}
  {@const WizardComponent = WIZARD_COMPONENTS[currentWizardModal]}
  <WizardComponent
    onclose={() => {
      currentWizardModal = null;
    }}
    {message}
    onchange={(m: string) => {
      message = m;
    }}
    settings={data.settings}
  />
{/if}

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;
    gap: 1rem;
    height: calc(100vh - var(--toolbar-height, 0px));
    max-height: calc(100vh - var(--toolbar-height, 0px));

    padding: 1rem;

    isolation: isolate;
    z-index: 0;
  }

  main.resizing {
    user-select: none;
    cursor: ns-resize;
  }

  .tabs-scroll-container {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  main :global(.message-editor) {
    flex: 0 0 auto;
    overflow: hidden;
  }

  .resize-handle {
    flex: 0 0 auto;
    height: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: ns-resize;
    user-select: none;
    touch-action: none;
    position: relative;
    margin: -4px 0;
    z-index: 10;
  }

  .resize-handle:hover .resize-grip,
  .resize-handle.resizing .resize-grip {
    background-color: var(--col-pine);
    opacity: 1;
  }

  .resize-handle.at-min-bound .resize-grip,
  .resize-handle.at-max-bound .resize-grip {
    background-color: var(--col-love);
    opacity: 1;
    width: 80px;
  }

  .resize-grip {
    width: 60px;
    height: 4px;
    background-color: var(--col-subtle);
    border-radius: 2px;
    opacity: 0.5;
    transition: all 0.15s ease;
    pointer-events: none;
  }

  main :global(.cursor-description) {
    flex: 0 0 auto;
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

  span.listening {
    color: var(--col-pine);
  }

  span.notListening {
    color: var(--col-subtle);
  }
</style>
