<!--
  Main Page Component - Hermes HL7 Message Editor

  This is the primary UI for the Hermes application, providing a complete environment
  for composing, editing, and managing HL7 messages. The component orchestrates several
  key subsystems:

  ## Architecture

  ### Tab-Based Segment Editing
  Each HL7 segment (MSH, PID, PV1, etc.) gets its own tab, allowing users to focus on
  one segment at a time with dedicated form fields. Tabs are dynamically generated based
  on the segments present in the message. This separation prevents overwhelming users with
  the full complexity of a multi-segment message at once.

  ### Resizable Split-Pane Layout
  The UI is split between:
  - Top: Tab-based segment forms (flexible height)
  - Middle: Resize handle (user can drag to adjust proportions)
  - Bottom: Raw HL7 text editor (fixed height, user-adjustable)

  The resize system uses pointer capture to track drags smoothly, even when the cursor
  moves outside the handle. Heights are constrained to prevent unusable layouts (too small
  to read or too large to see tabs).

  ### Two-Way Synchronization
  Changes in the form tabs update the raw message text, and vice versa. This is managed
  through reactive effects that parse/render between the two representations. This allows
  users to work in whichever mode is most comfortable for their task.

  ### Undo/Redo History
  All message edits flow through `updateMessage()`, which records state in a history
  manager before applying changes. This enables undo/redo across all edit sources:
  - Raw text typing (coalesced into single undo entries)
  - Form field changes (discrete entries)
  - Segment additions (discrete entries)

  History is cleared on File > New and File > Open to start fresh. The history manager
  uses Svelte 5 runes for reactive `canUndo`/`canRedo` state that drives toolbar buttons
  and native Edit menu item states.

  ### File Management
  Messages can be opened from and saved to .hl7 files. The component tracks whether
  the current message has unsaved changes to enable/disable the Save button appropriately.

  ### Listening for Incoming Messages
  The application can act as an HL7 MLLP server to receive messages. The listening state
  and unread message count are tracked here to show notifications in the UI (when enabled).
-->
<script lang="ts">
  import MessageEditor from "$lib/editor/message_editor.svelte";
  import CursorDescription from "$lib/editor/cursor_description.svelte";
  import Tabs from "$lib/tabs/tabs.svelte";
  import Tab from "$lib/tabs/tab.svelte";
  import SegmentTab from "$lib/forms/segment_tab.svelte";
  import { onMount } from "svelte";
  import { getAllSegmentSchemas, type SegmentSchemas } from "$lib/shared/schema";
  import {
    message as messageDialog,
    open as openDialog,
    save as saveDialog,
  } from "@tauri-apps/plugin-dialog";
  import {
    generateControlId,
    generateDefaultData,
    generateTemplateMessage,
    getMessageSegmentNames,
    renderMessageSegment,
    getCurrentCellRange,
    getCurrentHl7Timestamp,
  } from "$lib/shared/data";
  import { exportToJson, exportToYaml, exportToToml } from "$lib/editor/export";
  import { importFromJson, importFromYaml, importFromToml } from "$lib/editor/import";
  import {
    getSegmentIndexAtCursor,
    deleteSegment,
    moveSegment,
    duplicateSegment,
  } from "$lib/editor/segment";
  import Toolbar from "$lib/toolbar/toolbar.svelte";
  import ToolbarButton from "$lib/toolbar/toolbar_button.svelte";
  import IconNew from "$lib/icons/IconNew.svelte";
  import IconOpen from "$lib/icons/IconOpen.svelte";
  import IconSave from "$lib/icons/IconSave.svelte";
  import IconSaveAs from "$lib/icons/IconSaveAs.svelte";
  import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
  import type { PageProps } from "./$types";
  import ToolbarSpacer from "$lib/toolbar/toolbar_spacer.svelte";
  import IconSettings from "$lib/icons/IconSettings.svelte";
  import SettingsModal from "$lib/settings/settings_modal.svelte";
  import IconHelp from "$lib/icons/IconHelp.svelte";
  import ToolbarSeparator from "$lib/toolbar/toolbar_separator.svelte";
  import IconSendReceive from "$lib/icons/IconSendReceive.svelte";
  import { get } from "svelte/store";
  import NotificationIcon from "$lib/notification_icon.svelte";
  import { listenToListenResponse } from "$lib/communication/listen";
  import CommunicationDrawer from "$lib/communication/communication_drawer.svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { createHistoryManager } from "$lib/editor/history.svelte";
  import IconUndo from "$lib/icons/IconUndo.svelte";
  import IconRedo from "$lib/icons/IconRedo.svelte";
  import FindReplaceBar from "$lib/find_replace/find_replace_bar.svelte";
  import JumpToFieldModal from "$lib/modals/jump_to_field_modal.svelte";
  import InsertTimestampModal from "$lib/modals/insert_timestamp_modal.svelte";
  import KeyboardShortcutsModal from "$lib/modals/keyboard_shortcuts_modal.svelte";
  import DiffModal from "$lib/diff/diff_modal.svelte";
  import ValidationPanel from "$lib/validation/validation_panel.svelte";
  import type { SearchMatch, ValidationMatch } from "$lib/editor/syntax_highlight";
  import { validateLight, validateFull, type ValidationResult, type ValidationIssue } from "$lib/validation/validate";
  import {
    reloadExtensions,
    getExtensionToolbarButtons,
    getExtensions,
    executeExtensionCommand,
    provideExtensionMessage,
    provideExtensionPatchResult,
    isExtensionRunning,
    type ToolbarButtonInfo,
    type ExtensionStatus,
  } from "$lib/extensions/extensions";
  import type {
    GetMessageRequestPayload,
    PatchMessageRequestPayload,
    MessageFormat,
  } from "$lib/extensions/types";
  import { handleGetMessage, handlePatchMessage } from "$lib/extensions/editor_bridge";
  import DOMPurify from "dompurify";

  let { data }: PageProps = $props();

  // Undo/redo history manager
  const history = createHistoryManager();

  // Core message state
  let message: string = $state("");
  let savedMessage: string = $state(""); // Tracks last saved version to detect unsaved changes
  let cursorPos: number = $state(0);
  let schemas: SegmentSchemas = $state({});
  let messageSegments: string[] = $state([]); // Ordered list of segment names (e.g., ["MSH", "PID", "PV1"])
  let toolbarHeight: string | undefined = $state(undefined);
  let setActiveTab: ((id: string) => void) | undefined = $state(undefined);
  let showSettings = $state(false);
  let currentFilePath: string | undefined = $state(undefined);

  // Communication drawer state (initialized from settings)
  let showCommDrawer = $state(data.settings.commDrawerVisible);
  let commDrawerTab: "send" | "listen" = $state(data.settings.commDrawerTab);

  // Listen server state - tracks whether we're actively listening for incoming HL7 messages
  // and how many received messages haven't been viewed yet
  let listening = $state(false);
  let unreadMessageCount = $state(0);

  // Find/Replace state
  let showFindBar = $state(false);
  let searchMatches: SearchMatch[] = $state([]);
  let currentMatchIndex = $state(0);
  let getEditorSelection: (() => string) | undefined = $state(undefined);
  let editorElement: HTMLTextAreaElement | undefined = $state(undefined);
  let findInitialSelection = $state("");

  // Jump to Field state
  let showJumpToField = $state(false);

  // Insert Timestamp state
  let showInsertTimestampModal = $state(false);

  // Diff modal state
  let showDiffModal = $state(false);

  // Keyboard shortcuts modal state
  let showKeyboardShortcutsModal = $state(false);

  // Extension state
  let extensionButtons: ToolbarButtonInfo[] = $state([]);
  let extensionStatuses: ExtensionStatus[] = $state([]);

  // Validation state
  let validationResult: ValidationResult | null = $state(null);
  let showValidationPanel = $state(false);
  let validationHighlights: ValidationMatch[] = $derived.by(() => {
    if (!validationResult) return [];
    return validationResult.issues
      .filter((issue: ValidationIssue) => issue.range !== null)
      .map((issue: ValidationIssue) => ({
        start: issue.range![0],
        end: issue.range![1],
        severity: issue.severity,
      }));
  });

  /**
   * Message Editor Resize System
   *
   * The editor height is user-adjustable via a drag handle between the tabs and editor.
   * We persist the height to settings so it's remembered across sessions.
   *
   * Constraints prevent unusable layouts:
   * - MIN: 100px ensures the editor is always tall enough to see at least a few lines
   * - MAX: 60% of viewport ensures tab area remains visible and functional
   *
   * The max is dynamic because window resizes could otherwise make the editor consume
   * the entire viewport, hiding the tabs completely.
   */
  let editorHeight = $state(data.settings.editorHeight); // Height in pixels
  let isResizing = $state(false);
  let resizeStartY = 0;
  let resizeStartHeight = 0;
  let windowHeight = $state(window.innerHeight);
  let atMinBound = $state(false); // Visual feedback when user tries to drag beyond min
  let atMaxBound = $state(false); // Visual feedback when user tries to drag beyond max

  const MIN_EDITOR_HEIGHT = 100;
  const MAX_EDITOR_HEIGHT = $derived(windowHeight * 0.6);

  /**
   * Sanitize SVG markup from extensions to prevent XSS attacks
   *
   * Extensions can provide arbitrary SVG icons. This function strips
   * potentially dangerous content like scripts and event handlers while
   * preserving safe SVG markup.
   */
  function sanitizeSvg(svg: string): string {
    return DOMPurify.sanitize(svg, {
      USE_PROFILES: { svg: true, svgFilters: true },
      ADD_TAGS: ['use'], // allow <use> for icon references
      FORBID_TAGS: ['script', 'style'], // prevent script injection
      FORBID_ATTR: ['onerror', 'onload', 'onclick'], // prevent event handlers
    });
  }

  /**
   * Centralized message update function
   *
   * All edits to the message should go through this function to ensure proper
   * history tracking. The coalesce option is used for rapid changes like typing
   * to group them into a single undo entry.
   *
   * @param newMessage - The new message content
   * @param options.coalesce - If true, rapid calls will be merged into one history entry
   */
  function updateMessage(
    newMessage: string,
    options?: { coalesce?: boolean },
  ) {
    history.push(message, options?.coalesce ?? false);
    message = newMessage;
  }

  /**
   * Undo the last message change
   */
  function handleUndo() {
    const previous = history.undo(message);
    if (previous !== null) {
      message = previous;
    }
  }

  /**
   * Redo a previously undone change
   */
  function handleRedo() {
    const next = history.redo(message);
    if (next !== null) {
      message = next;
    }
  }

  /**
   * Resize Handle: Start
   *
   * Uses pointer capture to ensure we receive move/up events even if the cursor
   * leaves the handle element during dragging. This is critical for smooth UX -
   * without it, fast mouse movements would "escape" the drag operation.
   */
  function handleResizeStart(event: PointerEvent) {
    event.preventDefault();
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    isResizing = true;
    resizeStartY = event.clientY;
    resizeStartHeight = editorHeight;
  }

  /**
   * Resize Handle: Move
   *
   * The handle sits between tabs (top) and editor (bottom). Dragging UP should
   * make the editor taller (taking space from tabs), and dragging DOWN should
   * make it shorter. Since clientY increases downward, we invert the delta.
   *
   * Boundary tracking (atMinBound/atMaxBound) provides visual feedback when the
   * user tries to resize beyond the allowed range, helping them understand why
   * the resize stopped.
   */
  function handleResizeMove(event: PointerEvent) {
    if (!isResizing) return;
    event.preventDefault();

    const delta = event.clientY - resizeStartY;
    const newHeight = resizeStartHeight - delta;
    const clampedHeight = Math.max(
      MIN_EDITOR_HEIGHT,
      Math.min(MAX_EDITOR_HEIGHT, newHeight),
    );

    atMinBound =
      clampedHeight === MIN_EDITOR_HEIGHT && newHeight < MIN_EDITOR_HEIGHT;
    atMaxBound =
      clampedHeight === MAX_EDITOR_HEIGHT && newHeight > MAX_EDITOR_HEIGHT;

    editorHeight = clampedHeight;
  }

  /**
   * Resize Handle: End
   *
   * Persists the new height to settings immediately on release. This ensures
   * the user's preferred layout is remembered across app restarts.
   */
  function handleResizeEnd(event: PointerEvent) {
    if (!isResizing) return;
    event.preventDefault();

    const target = event.currentTarget as HTMLElement;
    target.releasePointerCapture(event.pointerId);
    isResizing = false;
    atMinBound = false;
    atMaxBound = false;

    data.settings.editorHeight = editorHeight;
  }

  /**
   * Message Segment Tracking
   *
   * Whenever the message changes, we parse it to extract the ordered list of segment
   * names. This list drives the tab UI - each segment gets its own tab.
   *
   * We parse on every change (rather than only on load) because users can edit the
   * raw message text directly, adding or removing segments. The tab UI needs to stay
   * in sync with whatever segments are actually present in the message.
   */
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

    /**
     * Listen Server Integration
     *
     * Sets up event listener for incoming messages from the Rust backend's MLLP server.
     * Also subscribes to listening state and message stores to track unread count.
     *
     * The unread count is derived by counting messages with `unread: true`. This count
     * would be displayed in the Listen button badge (currently commented out in toolbar).
     */
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

    // Load HL7 schemas from backend - these define the structure of each segment type
    getAllSegmentSchemas()
      .then((_schemas) => {
        console.debug("Schemas loaded:", _schemas);
        schemas = _schemas;
      })
      .catch((error: string) => {
        console.error("Error loading schemas:", error);
        messageDialog(error, { title: "Error Loading Schemas", kind: "error" });
      });

    /**
     * Recent Files Menu Integration
     *
     * Sets up the callback to update the native menu when recent files change.
     * Also initializes the menu with any persisted recent files on startup.
     */
    data.settings.onRecentFilesChanged = (files: string[]) => {
      invoke("update_recent_files_menu", { files });
    };
    // Initialize menu with current recent files (if settings have loaded)
    if (data.settings.recentFiles.length > 0) {
      invoke("update_recent_files_menu", { files: data.settings.recentFiles });
    }

    /**
     * Auto-Save Menu Integration
     *
     * Syncs the Auto-Save checkable menu item with the settings value.
     * The callback is invoked both on initial load and when the setting changes.
     */
    data.settings.onAutoSaveChanged = (enabled: boolean) => {
      invoke("set_auto_save_checked", { checked: enabled });
    };
    // Initialize menu with current auto-save state
    invoke("set_auto_save_checked", { checked: data.settings.autoSaveEnabled });

    /**
     * Theme Application
     *
     * Applies the theme setting to the document's data-theme attribute.
     * The CSS uses this attribute to apply light/dark/auto theme colours.
     */
    const applyTheme = (theme: "light" | "dark" | "auto") => {
      document.documentElement.dataset.theme = theme;
    };
    data.settings.onThemeChanged = applyTheme;
    // Initialize with current theme setting
    applyTheme(data.settings.themeSetting);

    /**
     * Zoom Level Management
     *
     * Uses browser-style non-linear zoom increments for natural scaling.
     * Zoom level is persisted to settings and restored on app startup.
     * The webview's setZoom API takes a scale factor (1.0 = 100%).
     */
    const ZOOM_LEVELS = [0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0];

    const applyZoom = async (zoomLevel: number) => {
      try {
        await getCurrentWebview().setZoom(zoomLevel);
      } catch (error) {
        console.error("Failed to apply zoom level:", error);
      }
    };

    const handleZoomIn = () => {
      const currentIndex = ZOOM_LEVELS.findIndex(
        (z) => z >= data.settings.zoomLevel,
      );
      if (currentIndex < ZOOM_LEVELS.length - 1) {
        const newZoom = ZOOM_LEVELS[currentIndex + 1];
        data.settings.zoomLevel = newZoom;
        applyZoom(newZoom);
      }
    };

    const handleZoomOut = () => {
      const currentIndex = ZOOM_LEVELS.findIndex(
        (z) => z >= data.settings.zoomLevel,
      );
      if (currentIndex > 0) {
        const newZoom = ZOOM_LEVELS[currentIndex - 1];
        data.settings.zoomLevel = newZoom;
        applyZoom(newZoom);
      } else if (currentIndex === -1 && data.settings.zoomLevel > ZOOM_LEVELS[0]) {
        // Current zoom is above max in our list, go to the highest level
        const newZoom = ZOOM_LEVELS[ZOOM_LEVELS.length - 1];
        data.settings.zoomLevel = newZoom;
        applyZoom(newZoom);
      }
    };

    const handleResetZoom = () => {
      data.settings.zoomLevel = 1.0;
      applyZoom(1.0);
    };

    data.settings.onZoomChanged = applyZoom;
    // Initialize with current zoom setting
    applyZoom(data.settings.zoomLevel);

    // Extension system integration
    // When extension configs change (or are loaded from disk), reload the extension host.
    data.settings.onExtensionsChanged = async (extensions) => {
      try {
        await reloadExtensions(extensions);
        await refreshExtensionData();
      } catch (error) {
        console.error("Failed to reload extensions:", error);
      }
    };

    // Helper to refresh extension buttons and statuses
    async function refreshExtensionData() {
      try {
        [extensionButtons, extensionStatuses] = await Promise.all([
          getExtensionToolbarButtons(),
          getExtensions(),
        ]);
      } catch (error) {
        console.error("Failed to refresh extension data:", error);
      }
    }

    // Load extensions on startup with current settings
    reloadExtensions(data.settings.extensions)
      .then(() => refreshExtensionData())
      .catch((error) => {
        console.error("Failed to load extensions on startup:", error);
      });

    // Extension event listeners
    let unlistenExtensionsChanged: UnlistenFn | undefined = undefined;
    let unlistenGetMessageRequest: UnlistenFn | undefined = undefined;
    let unlistenSetMessage: UnlistenFn | undefined = undefined;
    let unlistenPatchMessageRequest: UnlistenFn | undefined = undefined;

    // Listen for extensions-changed events (emitted when extensions start/stop)
    listen("extensions-changed", async () => {
      try {
        await refreshExtensionData();
      } catch (error) {
        console.error("Failed to refresh extension data on extensions-changed:", error);
      }
    }).then((unlisten) => {
      unlistenExtensionsChanged = unlisten;
    });

    // Listen for extension getMessage requests
    listen<GetMessageRequestPayload>("extension-get-message-request", async (event) => {
      const { extensionId, requestId, format } = event.payload;

      try {
        const result = await handleGetMessage(message, format as MessageFormat);

        if (result.error) {
          console.error("Error handling getMessage:", result.error);
          // still provide response with error message
        }

        await provideExtensionMessage(
          extensionId,
          requestId,
          result.message,
          currentFilePath !== undefined,
          currentFilePath
        );
      } catch (error) {
        console.error("Failed to handle getMessage request:", error);
      }
    }).then((unlisten) => {
      unlistenGetMessageRequest = unlisten;
    });

    // Listen for extension setMessage events
    listen<string>("extension-set-message", (event) => {
      const newMessage = event.payload;
      updateMessage(newMessage);
    }).then((unlisten) => {
      unlistenSetMessage = unlisten;
    });

    // Listen for extension patchMessage requests
    listen<PatchMessageRequestPayload>("extension-patch-message-request", async (event) => {
      const { extensionId, requestId, patches } = event.payload;

      try {
        const result = await handlePatchMessage(message, patches);

        // update the editor with patched message
        if (result.success || result.patchesApplied > 0) {
          updateMessage(result.message);
        }

        // send result back to extension
        await provideExtensionPatchResult(
          extensionId,
          requestId,
          result.success,
          result.patchesApplied,
          result.errors
        );
      } catch (error) {
        console.error("Failed to handle patchMessage request:", error);
        // send error response back to extension so it doesn't timeout
        try {
          await provideExtensionPatchResult(
            extensionId,
            requestId,
            false,
            0,
            [{ index: 0, path: "", message: String(error) }]
          );
        } catch (innerError) {
          console.error("Failed to send error response:", innerError);
        }
      }
    }).then((unlisten) => {
      unlistenPatchMessageRequest = unlisten;
    });

    /**
     * Menu Event Listeners
     *
     * The Tauri backend emits events when native menu items are clicked.
     * We listen for these events and trigger the corresponding operations.
     *
     * Organized by menu:
     * - File: New, Open, Save, Save As, Auto-Save, Open Recent, Clear Recent
     * - Edit: Undo, Redo, Find, Find and Replace
     * - View: Zoom In, Zoom Out, Reset Zoom
     * - Tools: Send Message, Listen for Messages
     * - Help: Help window
     */
    let unlistenMenuNew: UnlistenFn | undefined = undefined;
    let unlistenMenuNewFromTemplate: UnlistenFn | undefined = undefined;
    let unlistenMenuOpen: UnlistenFn | undefined = undefined;
    let unlistenMenuSave: UnlistenFn | undefined = undefined;
    let unlistenMenuSaveAs: UnlistenFn | undefined = undefined;
    let unlistenMenuAutoSave: UnlistenFn | undefined = undefined;
    let unlistenMenuUndo: UnlistenFn | undefined = undefined;
    let unlistenMenuRedo: UnlistenFn | undefined = undefined;
    let unlistenMenuFind: UnlistenFn | undefined = undefined;
    let unlistenMenuFindReplace: UnlistenFn | undefined = undefined;
    let unlistenMenuJumpToField: UnlistenFn | undefined = undefined;
    let unlistenMenuOpenRecent: UnlistenFn | undefined = undefined;
    let unlistenMenuClearRecent: UnlistenFn | undefined = undefined;
    let unlistenMenuHelp: UnlistenFn | undefined = undefined;
    let unlistenMenuToolsSend: UnlistenFn | undefined = undefined;
    let unlistenMenuToolsListen: UnlistenFn | undefined = undefined;
    let unlistenMenuToolsGenerateControlId: UnlistenFn | undefined = undefined;
    let unlistenMenuToolsInsertTimestampNow: UnlistenFn | undefined = undefined;
    let unlistenMenuToolsInsertTimestamp: UnlistenFn | undefined = undefined;
    let unlistenMenuToolsCompare: UnlistenFn | undefined = undefined;
    let unlistenMenuToolsValidate: UnlistenFn | undefined = undefined;
    let unlistenMenuZoomIn: UnlistenFn | undefined = undefined;
    let unlistenMenuZoomOut: UnlistenFn | undefined = undefined;
    let unlistenMenuResetZoom: UnlistenFn | undefined = undefined;
    let unlistenMenuKeyboardShortcuts: UnlistenFn | undefined = undefined;
    let unlistenMenuExportJson: UnlistenFn | undefined = undefined;
    let unlistenMenuExportYaml: UnlistenFn | undefined = undefined;
    let unlistenMenuExportToml: UnlistenFn | undefined = undefined;
    let unlistenMenuImportJson: UnlistenFn | undefined = undefined;
    let unlistenMenuImportYaml: UnlistenFn | undefined = undefined;
    let unlistenMenuImportToml: UnlistenFn | undefined = undefined;
    let unlistenMenuDeleteSegment: UnlistenFn | undefined = undefined;
    let unlistenMenuMoveSegmentUp: UnlistenFn | undefined = undefined;
    let unlistenMenuMoveSegmentDown: UnlistenFn | undefined = undefined;
    let unlistenMenuDuplicateSegment: UnlistenFn | undefined = undefined;

    listen("menu-file-new", () => handleNew()).then((fn) => {
      unlistenMenuNew = fn;
    });
    listen<string>("menu-new-from-template", async (event) => {
      try {
        const templateMessage = await generateTemplateMessage(event.payload);
        history.clear();
        message = templateMessage;
        savedMessage = message;
        currentFilePath = undefined;
      } catch (error) {
        console.error("Failed to generate template message:", error);
        messageDialog(`Failed to generate template: ${error}`, {
          title: "Template Error",
          kind: "error",
        });
      }
    }).then((fn) => {
      unlistenMenuNewFromTemplate = fn;
    });
    listen("menu-file-open", () => handleOpenFile()).then((fn) => {
      unlistenMenuOpen = fn;
    });
    listen("menu-file-save", () => handleSave?.()).then((fn) => {
      unlistenMenuSave = fn;
    });
    listen("menu-file-save-as", () => handleSaveAs()).then((fn) => {
      unlistenMenuSaveAs = fn;
    });
    listen("menu-file-export-json", () => handleExport("json")).then((fn) => {
      unlistenMenuExportJson = fn;
    });
    listen("menu-file-export-yaml", () => handleExport("yaml")).then((fn) => {
      unlistenMenuExportYaml = fn;
    });
    listen("menu-file-export-toml", () => handleExport("toml")).then((fn) => {
      unlistenMenuExportToml = fn;
    });
    listen("menu-file-import-json", () => handleImport("json")).then((fn) => {
      unlistenMenuImportJson = fn;
    });
    listen("menu-file-import-yaml", () => handleImport("yaml")).then((fn) => {
      unlistenMenuImportYaml = fn;
    });
    listen("menu-file-import-toml", () => handleImport("toml")).then((fn) => {
      unlistenMenuImportToml = fn;
    });
    listen("menu-file-auto-save", () => {
      // Toggle the auto-save setting when menu item is clicked
      data.settings.autoSaveEnabled = !data.settings.autoSaveEnabled;
    }).then((fn) => {
      unlistenMenuAutoSave = fn;
    });
    listen("menu-edit-undo", () => handleUndo()).then((fn) => {
      unlistenMenuUndo = fn;
    });
    listen("menu-edit-redo", () => handleRedo()).then((fn) => {
      unlistenMenuRedo = fn;
    });
    listen("menu-edit-find", () => handleFind()).then((fn) => {
      unlistenMenuFind = fn;
    });
    listen("menu-edit-find-replace", () => handleFind()).then((fn) => {
      unlistenMenuFindReplace = fn;
    });
    listen("menu-edit-jump-to-field", () => {
      showJumpToField = true;
    }).then((fn) => {
      unlistenMenuJumpToField = fn;
    });
    // Segment operations
    listen("menu-edit-delete-segment", () => handleDeleteSegment()).then((fn) => {
      unlistenMenuDeleteSegment = fn;
    });
    listen("menu-edit-move-segment-up", () => handleMoveSegmentUp()).then((fn) => {
      unlistenMenuMoveSegmentUp = fn;
    });
    listen("menu-edit-move-segment-down", () => handleMoveSegmentDown()).then((fn) => {
      unlistenMenuMoveSegmentDown = fn;
    });
    listen("menu-edit-duplicate-segment", () => handleDuplicateSegment()).then((fn) => {
      unlistenMenuDuplicateSegment = fn;
    });
    // View menu: Zoom controls
    listen("menu-view-zoom-in", () => handleZoomIn()).then((fn) => {
      unlistenMenuZoomIn = fn;
    });
    listen("menu-view-zoom-out", () => handleZoomOut()).then((fn) => {
      unlistenMenuZoomOut = fn;
    });
    listen("menu-view-reset-zoom", () => handleResetZoom()).then((fn) => {
      unlistenMenuResetZoom = fn;
    });
    listen("menu-view-keyboard-shortcuts", () => {
      showKeyboardShortcutsModal = true;
    }).then((fn) => {
      unlistenMenuKeyboardShortcuts = fn;
    });
    listen<string>("menu-open-recent", (event) => {
      handleOpenRecentFile(event.payload);
    }).then((fn) => {
      unlistenMenuOpenRecent = fn;
    });
    listen("menu-clear-recent", () => {
      data.settings.clearRecentFiles();
    }).then((fn) => {
      unlistenMenuClearRecent = fn;
    });
    listen("menu-help", () => {
      invoke("open_help_window");
    }).then((fn) => {
      unlistenMenuHelp = fn;
    });
    // Tools menu: Open communication drawer with appropriate tab
    // Provides keyboard shortcuts (Cmd+T, Cmd+L) for quick access during testing workflows
    listen("menu-tools-send", () => {
      showCommDrawer = true;
      commDrawerTab = "send";
    }).then((fn) => {
      unlistenMenuToolsSend = fn;
    });
    listen("menu-tools-listen", () => {
      showCommDrawer = true;
      commDrawerTab = "listen";
    }).then((fn) => {
      unlistenMenuToolsListen = fn;
    });
    listen("menu-tools-generate-control-id", async () => {
      try {
        const result = await generateControlId(message);
        updateMessage(result.message);
        // Select the new control ID in the editor
        setTimeout(() => {
          if (editorElement) {
            editorElement.focus();
            editorElement.setSelectionRange(result.range.start, result.range.end);
          }
        }, 0);
      } catch (error) {
        console.error("Failed to generate control ID:", error);
      }
    }).then((fn) => {
      unlistenMenuToolsGenerateControlId = fn;
    });
    // Tools menu: Insert timestamp at current cursor position
    listen("menu-tools-insert-timestamp-now", async () => {
      try {
        const cursor = editorElement?.selectionStart ?? 0;
        const range = await getCurrentCellRange(message, cursor);
        if (!range) {
          console.warn("No valid field at cursor position for timestamp insertion");
          return;
        }
        // Generate current timestamp with UTC offset
        const timestamp = await getCurrentHl7Timestamp(true);
        // Replace the current cell with the timestamp
        const newMessage =
          message.slice(0, range.start) + timestamp + message.slice(range.end);
        updateMessage(newMessage);
        // Select the inserted timestamp
        setTimeout(() => {
          if (editorElement) {
            editorElement.focus();
            editorElement.setSelectionRange(range.start, range.start + timestamp.length);
          }
        }, 0);
      } catch (error) {
        console.error("Failed to insert timestamp:", error);
      }
    }).then((fn) => {
      unlistenMenuToolsInsertTimestampNow = fn;
    });
    listen("menu-tools-insert-timestamp", () => {
      showInsertTimestampModal = true;
    }).then((fn) => {
      unlistenMenuToolsInsertTimestamp = fn;
    });
    listen("menu-tools-compare", () => {
      showDiffModal = true;
    }).then((fn) => {
      unlistenMenuToolsCompare = fn;
    });
    listen("menu-tools-validate", async () => {
      // run full validation on demand
      if (message) {
        validationResult = await validateFull(message);
        showValidationPanel = true;
      }
    }).then((fn) => {
      unlistenMenuToolsValidate = fn;
    });

    /**
     * Window Resize Handling
     *
     * When the window shrinks, the max editor height (60% of viewport) also shrinks.
     * If the editor is currently taller than the new max, we clamp it down to prevent
     * the editor from consuming more than 60% of the viewport.
     *
     * This prevents scenarios where a user resizes on a large monitor, then moves to
     * a smaller screen and finds the editor takes up the entire window.
     */
    const handleWindowResize = () => {
      windowHeight = window.innerHeight;
      if (editorHeight > MAX_EDITOR_HEIGHT) {
        editorHeight = MAX_EDITOR_HEIGHT;
        data.settings.editorHeight = editorHeight;
      }
    };
    window.addEventListener("resize", handleWindowResize);

    return () => {
      unlisten?.();
      unlistenMenuNew?.();
      unlistenMenuNewFromTemplate?.();
      unlistenMenuOpen?.();
      unlistenMenuSave?.();
      unlistenMenuSaveAs?.();
      unlistenMenuAutoSave?.();
      unlistenMenuUndo?.();
      unlistenMenuRedo?.();
      unlistenMenuFind?.();
      unlistenMenuFindReplace?.();
      unlistenMenuJumpToField?.();
      unlistenMenuDeleteSegment?.();
      unlistenMenuMoveSegmentUp?.();
      unlistenMenuMoveSegmentDown?.();
      unlistenMenuDuplicateSegment?.();
      unlistenMenuZoomIn?.();
      unlistenMenuZoomOut?.();
      unlistenMenuResetZoom?.();
      unlistenMenuKeyboardShortcuts?.();
      unlistenMenuOpenRecent?.();
      unlistenMenuClearRecent?.();
      unlistenMenuHelp?.();
      unlistenMenuToolsSend?.();
      unlistenMenuToolsListen?.();
      unlistenMenuToolsGenerateControlId?.();
      unlistenMenuToolsInsertTimestampNow?.();
      unlistenMenuToolsInsertTimestamp?.();
      unlistenMenuToolsCompare?.();
      unlistenMenuToolsValidate?.();
      unlistenMenuExportJson?.();
      unlistenMenuExportYaml?.();
      unlistenMenuExportToml?.();
      unlistenMenuImportJson?.();
      unlistenMenuImportYaml?.();
      unlistenMenuImportToml?.();
      unlistenExtensionsChanged?.();
      unlistenGetMessageRequest?.();
      unlistenSetMessage?.();
      unlistenPatchMessageRequest?.();
      window.removeEventListener("resize", handleWindowResize);
    };
  });

  /**
   * Helper for counting segment repetitions
   *
   * HL7 messages can have multiple instances of the same segment (e.g., multiple OBX
   * segments for different observations). This counts how many times a given segment
   * appears *before* the specified index, which is used for numbering tab labels.
   */
  const segmentRepeat = (segment: string, index: number): number => {
    return messageSegments.slice(0, index).filter((s) => s === segment).length;
  };

  /**
   * Tab Label Generation
   *
   * If a segment appears only once, its tab is labelled with just the segment name (e.g., "MSH").
   * If a segment appears multiple times, tabs are numbered to distinguish them (e.g., "OBX (1)", "OBX (2)").
   *
   * This numbering is essential because clicking a tab needs to uniquely identify which
   * segment instance to display in the form. The count is 1-indexed for user-friendliness.
   */
  const tabLabel = (index: number): string => {
    const segment = messageSegments[index];
    const count = messageSegments.filter((s) => s === segment).length;
    if (count > 1) {
      return `${segment} (${segmentRepeat(segment, index) + 1})`;
    }
    return segment;
  };

  /**
   * File Operations
   *
   * New: Creates a fresh message with a minimal MSH segment and default data.
   * Clears the current file path since this is a new unsaved message.
   *
   * Open: Loads an .hl7 file from disk. Resets currentFilePath temporarily to ensure
   * proper state transitions if the user cancels the dialog.
   *
   * Save: Only enabled when there's both a file path AND unsaved changes. Uses $derived
   * to reactively return undefined (disabling the button) when save isn't applicable.
   * Updates savedMessage on success to track that we're in sync with disk.
   *
   * Save As: Always prompts for a new file path, even if we already have one. Useful
   * for creating copies or moving messages to different locations.
   *
   * All three operations update savedMessage to reflect the persisted state, which is
   * compared against the current message to determine if unsaved changes exist.
   */
  function handleNew() {
    history.clear();
    message = "MSH|^~\\&|";
    currentFilePath = undefined;
    const defaultData = generateDefaultData("MSH", schemas["MSH"] ?? {});
    renderMessageSegment(message, "MSH", 0, defaultData).then((newMessage) => {
      if (newMessage) {
        message = newMessage;
        savedMessage = message;
      }
    });
  }

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

    await openFileByPath(filePath);
  }

  /**
   * Opens a file directly by path (used by Open Recent menu)
   */
  async function handleOpenRecentFile(filePath: string) {
    try {
      await openFileByPath(filePath);
    } catch (error) {
      console.error("Error opening recent file:", error);
      messageDialog(`Could not open file: ${filePath}`, {
        title: "Error Opening File",
        kind: "error",
      });
    }
  }

  /**
   * Opens a file by its path and adds it to recent files
   */
  async function openFileByPath(filePath: string) {
    history.clear();
    currentFilePath = undefined;
    message = await readTextFile(filePath);
    savedMessage = message;
    currentFilePath = filePath;
    data.settings.addRecentFile(filePath);
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

  // Sync the Save menu item enabled state with the toolbar save button
  $effect(() => {
    invoke("set_save_enabled", { enabled: handleSave !== undefined });
  });

  // Sync the Undo/Redo menu item enabled states with the history state
  $effect(() => {
    invoke("set_undo_enabled", { enabled: history.canUndo });
  });

  $effect(() => {
    invoke("set_redo_enabled", { enabled: history.canRedo });
  });

  // Sync the Insert Timestamp menu items enabled state based on cursor position
  // Menu items are enabled only when cursor is within a valid field/component
  // Track cursorPos (reactive) so this re-runs when cursor moves
  $effect(() => {
    // Access cursorPos to track it as a dependency
    const cursor = cursorPos;
    // Also track message changes
    const currentMessage = message;
    const checkEnabled = async () => {
      const range = await getCurrentCellRange(currentMessage, cursor);
      invoke("set_insert_timestamp_enabled", { enabled: range !== null });
    };
    checkEnabled();
  });

  // Sync communication drawer state to settings
  $effect(() => {
    data.settings.commDrawerVisible = showCommDrawer;
  });

  $effect(() => {
    data.settings.commDrawerTab = commDrawerTab;
  });

  /**
   * Auto-Save Effect
   *
   * When auto-save is enabled and there's an open file with unsaved changes,
   * automatically save after a short debounce. The debounce (500ms) aligns with
   * the undo coalescing delay, so saves happen after the user stops typing.
   */
  let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    // Access reactive values to track them
    const autoSaveEnabled = data.settings.autoSaveEnabled;
    const hasUnsavedChanges = currentFilePath && message !== savedMessage;

    // Clear any existing timer
    if (autoSaveTimer) {
      clearTimeout(autoSaveTimer);
      autoSaveTimer = null;
    }

    // Only auto-save if enabled, has file path, and has unsaved changes
    if (autoSaveEnabled && hasUnsavedChanges) {
      autoSaveTimer = setTimeout(() => {
        handleSave?.();
      }, 500); // 500ms debounce to match undo coalescing
    }

    return () => {
      if (autoSaveTimer) {
        clearTimeout(autoSaveTimer);
        autoSaveTimer = null;
      }
    };
  });

  /**
   * Passive Validation Effect
   *
   * Runs light validation (required fields, parse errors) when the message changes.
   * This provides immediate feedback without the overhead of full validation.
   * Debounced to avoid running on every keystroke.
   */
  let validationTimer: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    const currentMessage = message;

    // clear any existing timer
    if (validationTimer) {
      clearTimeout(validationTimer);
      validationTimer = null;
    }

    if (currentMessage) {
      validationTimer = setTimeout(async () => {
        validationResult = await validateLight(currentMessage);
      }, 500); // 500ms debounce
    } else {
      validationResult = null;
    }

    return () => {
      if (validationTimer) {
        clearTimeout(validationTimer);
        validationTimer = null;
      }
    };
  });

  /**
   * Handle extension toolbar button click.
   *
   * Executes the command registered by the extension and displays any
   * error messages to the user.
   */
  async function handleExtensionButtonClick(command: string) {
    try {
      const result = await executeExtensionCommand(command);
      if (!result.success && result.message) {
        await messageDialog(result.message, { title: "Extension Error", kind: "error" });
      }
    } catch (error) {
      console.error("Extension command failed:", error);
      await messageDialog(String(error), { title: "Extension Error", kind: "error" });
    }
  }

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
        data.settings.addRecentFile(filePath);
      })
      .catch((error) => {
        console.error("Error saving file:", error);
        messageDialog(error, { title: "Error Saving File", kind: "error" });
      });
  };

  /**
   * Exports the current message to a different format (JSON, YAML, or TOML).
   *
   * Shows a save dialog with the appropriate file extension filter, converts
   * the message using the backend, and writes to the selected file.
   */
  const handleExport = async (format: "json" | "yaml" | "toml") => {
    const formatConfig = {
      json: { name: "JSON Files", extension: "json", title: "Export as JSON" },
      yaml: { name: "YAML Files", extension: "yaml", title: "Export as YAML" },
      toml: { name: "TOML Files", extension: "toml", title: "Export as TOML" },
    };

    const config = formatConfig[format];

    const filePath = await saveDialog({
      filters: [{ name: config.name, extensions: [config.extension] }],
      title: config.title,
    });

    if (!filePath) {
      return;
    }

    try {
      let exported: string;
      switch (format) {
        case "json":
          exported = await exportToJson(message);
          break;
        case "yaml":
          exported = await exportToYaml(message);
          break;
        case "toml":
          exported = await exportToToml(message);
          break;
      }

      await writeTextFile(filePath, exported, { append: false, create: true });
    } catch (error) {
      console.error(`Error exporting to ${format}:`, error);
      messageDialog(`${error}`, { title: `Export Error`, kind: "error" });
    }
  };

  /**
   * Imports a message from a different format (JSON, YAML, or TOML).
   *
   * Shows an open dialog with the appropriate file extension filter, reads
   * the file, converts it using the backend, and loads the result as a new message.
   */
  const handleImport = async (format: "json" | "yaml" | "toml") => {
    const formatConfig = {
      json: { name: "JSON Files", extension: "json", title: "Import from JSON" },
      yaml: { name: "YAML Files", extension: "yaml", title: "Import from YAML" },
      toml: { name: "TOML Files", extension: "toml", title: "Import from TOML" },
    };

    const config = formatConfig[format];

    const filePath = await openDialog({
      filters: [{ name: config.name, extensions: [config.extension] }],
      title: config.title,
      multiple: false,
    });

    if (!filePath) {
      return;
    }

    try {
      const content = await readTextFile(filePath);
      let imported: string;
      switch (format) {
        case "json":
          imported = await importFromJson(content);
          break;
        case "yaml":
          imported = await importFromYaml(content);
          break;
        case "toml":
          imported = await importFromToml(content);
          break;
      }

      // treat imported message as a new unsaved message
      history.clear();
      message = imported;
      savedMessage = message;
      currentFilePath = undefined;
    } catch (error) {
      console.error(`Error importing from ${format}:`, error);
      messageDialog(`${error}`, { title: `Import Error`, kind: "error" });
    }
  };

  /**
   * Opens the find/replace bar
   *
   * If there's a selection in the editor, it's used to pre-populate the search query.
   */
  function handleFind() {
    // Get current selection from editor if available
    findInitialSelection = getEditorSelection?.() ?? "";
    showFindBar = true;
  }

  /**
   * Handles find/replace match changes by updating state
   * Does NOT steal focus - that's handled separately by navigation actions
   */
  function handleMatchesChange(matches: SearchMatch[], currentIndex: number) {
    searchMatches = matches;
    currentMatchIndex = currentIndex;
  }

  /**
   * Scrolls to and selects the current match in the editor
   * Called explicitly when user navigates between matches
   */
  function scrollToCurrentMatch() {
    if (searchMatches.length > 0 && editorElement) {
      const match = searchMatches[currentMatchIndex];
      editorElement.setSelectionRange(match.start, match.end);
    }
  }

  /**
   * Handles replace operations from the find bar
   */
  function handleFindReplace(newMessage: string) {
    updateMessage(newMessage);
  }

  /**
   * Handles find bar close by returning focus to editor
   */
  function handleFindClose() {
    searchMatches = [];
    currentMatchIndex = 0;
    editorElement?.focus();
  }

  /**
   * Loads a received message into the editor
   * Used by the Listen tab's "Load to Editor" button
   */
  function handleLoadToEditor(receivedMessage: string) {
    updateMessage(receivedMessage);
    // Optionally close the drawer after loading
    // showCommDrawer = false;
  }

  /**
   * Applies the result of a segment operation: updates message and cursor
   */
  function applySegmentOperation(result: { message: string; cursor: number } | null) {
    if (!result) return;
    updateMessage(result.message);
    // position cursor at the result location
    setTimeout(() => {
      if (editorElement) {
        editorElement.focus();
        editorElement.setSelectionRange(result.cursor, result.cursor);
      }
    }, 0);
  }

  /**
   * Deletes the segment under the cursor (Cmd+Shift+K)
   */
  async function handleDeleteSegment() {
    const segmentIndex = await getSegmentIndexAtCursor(message, cursorPos);
    if (segmentIndex === null || segmentIndex === 0) return; // cannot delete MSH
    const result = await deleteSegment(message, segmentIndex);
    applySegmentOperation(result);
  }

  /**
   * Moves the segment under the cursor up (Cmd+Shift+↑)
   */
  async function handleMoveSegmentUp() {
    const segmentIndex = await getSegmentIndexAtCursor(message, cursorPos);
    if (segmentIndex === null || segmentIndex <= 1) return; // cannot move MSH or into MSH position
    const result = await moveSegment(message, segmentIndex, "up");
    applySegmentOperation(result);
  }

  /**
   * Moves the segment under the cursor down (Cmd+Shift+↓)
   */
  async function handleMoveSegmentDown() {
    const segmentIndex = await getSegmentIndexAtCursor(message, cursorPos);
    if (segmentIndex === null || segmentIndex === 0) return; // cannot move MSH
    const result = await moveSegment(message, segmentIndex, "down");
    applySegmentOperation(result);
  }

  /**
   * Duplicates the segment under the cursor (Cmd+Shift+D)
   */
  async function handleDuplicateSegment() {
    const segmentIndex = await getSegmentIndexAtCursor(message, cursorPos);
    if (segmentIndex === null || segmentIndex === 0) return; // cannot duplicate MSH
    const result = await duplicateSegment(message, segmentIndex);
    applySegmentOperation(result);
  }
</script>

<Toolbar bind:toolbarHeight>
  <ToolbarButton title="New" onclick={handleNew}>
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
  <ToolbarButton title="Undo" onclick={history.canUndo ? handleUndo : undefined}>
    <IconUndo />
  </ToolbarButton>
  <ToolbarButton title="Redo" onclick={history.canRedo ? handleRedo : undefined}>
    <IconRedo />
  </ToolbarButton>
  <ToolbarSeparator />
  <ToolbarButton
    title="Communication"
    onclick={() => {
      showCommDrawer = !showCommDrawer;
    }}
  >
    <NotificationIcon count={unreadMessageCount}>
      <span class={showCommDrawer ? "commActive" : listening ? "listening" : "notListening"}>
        <IconSendReceive />
      </span>
    </NotificationIcon>
  </ToolbarButton>
  <!-- Extension toolbar buttons -->
  {#if extensionButtons.length > 0}
    <ToolbarSeparator />
    {#each extensionButtons as { extensionId, button }}
      {@const status = extensionStatuses.find(s => s.id === extensionId)}
      {@const isRunning = status?.state === "running"}
      <ToolbarButton
        title={button.label}
        onclick={isRunning ? () => handleExtensionButtonClick(button.command) : undefined}
      >
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <span class="extension-icon" class:disabled={!isRunning}>
          {@html sanitizeSvg(button.icon)}
        </span>
      </ToolbarButton>
    {/each}
  {/if}
  <ToolbarSpacer />
  <ToolbarButton title="Help" onclick={() => invoke("open_help_window")}>
    <IconHelp />
  </ToolbarButton>
  <ToolbarButton
    title="Settings"
    onclick={() => {
      showSettings = true;
    }}
  >
    <IconSettings />
  </ToolbarButton>
</Toolbar>
<div class="app-content" style="--toolbar-height: {toolbarHeight ?? '1px'};">
<main
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
                  const newMessageWithSegment = message + `\n${key}|`;
                  const data = generateDefaultData(key, schemas[key] ?? {});
                  renderMessageSegment(newMessageWithSegment, key, 0, data).then(
                    (renderedMessage) => {
                      if (renderedMessage) {
                        updateMessage(renderedMessage);
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
          >
            <SegmentTab
              segment={key}
              segmentRepeat={segmentRepeat(key, index)}
              schema={schemas[key]}
              {message}
              onchange={(m) => {
                updateMessage(m);
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

  <FindReplaceBar
    bind:show={showFindBar}
    {message}
    initialSelection={findInitialSelection}
    onmatcheschange={handleMatchesChange}
    onnavigate={scrollToCurrentMatch}
    onreplace={handleFindReplace}
    onclose={handleFindClose}
  />
  <MessageEditor
    {message}
    {searchMatches}
    {currentMatchIndex}
    {validationHighlights}
    height={editorHeight}
    onchange={(m, coalesce) => {
      updateMessage(m, { coalesce });
    }}
    oncursorchange={(pos) => {
      cursorPos = pos;
    }}
    onctrlenter={() => {
      showCommDrawer = true;
      commDrawerTab = "send";
    }}
    onmovesegmentup={() => handleMoveSegmentUp()}
    onmovesegmentdown={() => handleMoveSegmentDown()}
    getSelection={(fn) => {
      getEditorSelection = fn;
    }}
    bind:editElement={editorElement}
  />
  <!--
    Tabs Follow Cursor Feature

    When enabled in settings, moving the cursor in the message editor automatically
    switches to the tab corresponding to the segment at the cursor position.

    This helps users understand the structure of the message: as they navigate through
    the raw text, the relevant segment form is displayed. It creates a connection between
    the two representations of the message (raw text vs. structured form).

    The feature is opt-in because some users prefer manual tab control while editing.
  -->
  <CursorDescription
    {message}
    {cursorPos}
    segmentSchemas={schemas}
    {currentFilePath}
    {validationResult}
    onvalidationclick={() => {
      showValidationPanel = !showValidationPanel;
    }}
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
  <ValidationPanel
    result={validationResult}
    bind:show={showValidationPanel}
    onNavigate={(issue: ValidationIssue) => {
      // navigate to the field in the editor and select the range
      if (issue.range && editorElement) {
        const el = editorElement;
        const range = issue.range;
        // use setTimeout to ensure panel state has settled
        setTimeout(() => {
          el.focus();
          el.setSelectionRange(range[0], range[1]);
          // scroll the selection into view by briefly blurring and refocusing
          el.blur();
          el.focus();
          el.setSelectionRange(range[0], range[1]);
        }, 0);
      }
    }}
  />
</main>
<CommunicationDrawer
  settings={data.settings}
  {message}
  listening={data.listening}
  listenedMessages={data.listenedMessages}
  bind:expanded={showCommDrawer}
  bind:activeTab={commDrawerTab}
  height={data.settings.commDrawerHeight}
  onLoadToEditor={handleLoadToEditor}
/>
</div>
<SettingsModal settings={data.settings} bind:show={showSettings} />
<JumpToFieldModal
  bind:show={showJumpToField}
  {message}
  onJump={(start, end) => {
    // Use setTimeout to ensure modal has fully closed before focusing editor
    setTimeout(() => {
      if (editorElement) {
        editorElement.focus();
        editorElement.setSelectionRange(start, end);
        // Scroll the selection into view by briefly blurring and refocusing
        // this triggers the browser's native scroll-to-caret behaviour
        editorElement.blur();
        editorElement.focus();
        editorElement.setSelectionRange(start, end);
      }
    }, 0);
  }}
/>
<InsertTimestampModal
  bind:show={showInsertTimestampModal}
  onInsert={async (timestamp) => {
    // Get the current cell range at cursor position
    const cursor = editorElement?.selectionStart ?? cursorPos;
    const range = await getCurrentCellRange(message, cursor);
    if (!range) {
      console.warn("No valid field at cursor position for timestamp insertion");
      return;
    }
    // Replace the current cell with the timestamp
    const newMessage =
      message.slice(0, range.start) + timestamp + message.slice(range.end);
    updateMessage(newMessage);
    // Select the inserted timestamp
    setTimeout(() => {
      if (editorElement) {
        editorElement.focus();
        editorElement.setSelectionRange(range.start, range.start + timestamp.length);
      }
    }, 0);
  }}
/>
<DiffModal bind:show={showDiffModal} editorMessage={message} />
<KeyboardShortcutsModal bind:show={showKeyboardShortcutsModal} />

<style>
  .app-content {
    display: flex;
    flex-direction: column;
    height: calc(100vh - var(--toolbar-height, 0px));
    max-height: calc(100vh - var(--toolbar-height, 0px));
    overflow: hidden;
  }

  main {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;
    gap: 1rem;
    flex: 1;
    min-height: 0;

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

  span.commActive {
    color: var(--col-iris);
  }

  span.listening {
    color: var(--col-pine);
  }

  span.extension-icon {
    display: flex;
    align-items: center;
    justify-content: center;

    &.disabled {
      opacity: 0.4;
    }

    /* ensure extension SVG icons inherit the current colour */
    :global(svg) {
      width: 1.25rem;
      height: 1.25rem;
    }
  }

  span.notListening {
    color: var(--col-subtle);
  }
</style>
