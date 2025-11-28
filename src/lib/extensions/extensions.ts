/**
 * TypeScript bridge for the extension system.
 *
 * Provides functions to interact with the Rust extension host via Tauri commands.
 * Extensions are third-party executables that communicate with Hermes over stdio
 * using JSON-RPC 2.0.
 *
 * @see EXTENSIONPLAN.md for the complete extension system design
 * @see extensions/api-docs/ for the extension API specification
 */

import { invoke } from "@tauri-apps/api/core";
import type { ExtensionConfig } from "../../settings";

// Re-export types for convenience
export type { ExtensionConfig } from "../../settings";
export type {
  MessageFormat,
  GetMessageRequestPayload,
  GetMessageResult,
  PatchMessageRequestPayload,
  Patch,
  PatchError,
  PatchMessageResult,
  SetMessagePayload,
} from "./types";

// ============================================================================
// Types
// ============================================================================

/**
 * Extension lifecycle state.
 *
 * Mirrors the Rust `ExtensionState` enum from `extensions/types.rs`.
 */
export type ExtensionState =
  | "starting"
  | "initializing"
  | "running"
  | "shutting_down"
  | "stopped"
  | { failed: string };

/**
 * Status information for an extension.
 *
 * Returned by `getExtensions()` to display extension status in the settings UI.
 */
export interface ExtensionStatus {
  /** Unique identifier (derived from executable path). */
  id: string;

  /** Original path from configuration (used for matching). */
  path: string;

  /** Human-readable name from extension metadata. */
  name: string;

  /** Version string from extension metadata. */
  version: string;

  /** Current lifecycle state. */
  state: ExtensionState;

  /** Error message if state is "failed". */
  error?: string;
}

/**
 * Toolbar button information from an extension.
 *
 * Used to render extension buttons in the toolbar.
 */
export interface ToolbarButtonInfo {
  /** ID of the extension that registered this button. */
  extensionId: string;

  /** Button definition. */
  button: {
    /** Unique identifier for the button. */
    id: string;

    /** Tooltip/label text. */
    label: string;

    /** SVG icon markup (should use currentColor for theme compatibility). */
    icon: string;

    /** Command ID to execute when clicked. */
    command: string;

    /** Optional visual grouping. */
    group?: string;
  };
}


/**
 * Log level for extension events.
 */
export type LogLevel = "info" | "warn" | "error";

/**
 * Message event type for sync_editor_message command.
 *
 * Used to inform extensions about message lifecycle events.
 */
export type MessageEvent =
  | { type: "opened"; isNew: boolean }
  | { type: "saved"; saveAs: boolean };

/**
 * Log entry from an extension.
 */
export interface ExtensionLog {
  /** Timestamp of the log entry (ISO 8601 format). */
  timestamp: string;

  /** Log level. */
  level: LogLevel;

  /** Log message. */
  message: string;
}

// ============================================================================
// Tauri Command Bridges
// ============================================================================

/**
 * Reload all extensions with the provided configuration.
 *
 * Shuts down existing extensions and restarts them with the new configuration.
 * This should be called whenever extension settings change, including on app
 * startup after settings are loaded.
 *
 * @param configs - Extension configurations from Settings.extensions
 */
export async function reloadExtensions(
  configs: ExtensionConfig[],
): Promise<void> {
  return invoke("reload_extensions", { configs });
}

/**
 * Get status information for all extensions.
 *
 * Returns a list of extension statuses including ID, name, version, state,
 * and any error messages for failed extensions.
 */
export async function getExtensions(): Promise<ExtensionStatus[]> {
  return invoke("get_extensions");
}

/**
 * Get all toolbar buttons from all running extensions.
 *
 * Returns buttons with their associated extension IDs, allowing the frontend
 * to display them and route clicks appropriately.
 */
export async function getExtensionToolbarButtons(): Promise<
  ToolbarButtonInfo[]
> {
  return invoke("get_extension_toolbar_buttons");
}

/**
 * Send a command to an extension.
 *
 * This is called when a user clicks an extension toolbar button. The command
 * string identifies which extension and action to invoke.
 */
export async function sendExtensionCommand(command: string): Promise<void> {
  await invoke("send_extension_command", { command });
}

/**
 * Get log entries for a specific extension.
 *
 * Returns the recent log entries (up to 100) for the specified extension.
 *
 * @param extensionId - ID of the extension to get logs for
 */
export async function getExtensionLogs(
  extensionId: string,
): Promise<ExtensionLog[]> {
  return invoke("get_extension_logs", { extensionId });
}

/**
 * Sync the current editor message content to the backend.
 *
 * Called whenever the message changes to keep the backend in sync for extension
 * access. Optionally includes event information for opened/saved notifications.
 *
 * @param message - Current message content
 * @param filePath - Current file path (null for unsaved messages)
 * @param event - Optional event type (opened or saved)
 */
export async function syncEditorMessage(
  message: string,
  filePath: string | null,
  event?: MessageEvent,
): Promise<void> {
  // convert frontend event format to backend format
  const eventParam = event
    ? event.type === "opened"
      ? { opened: { is_new: event.isNew } }
      : { saved: { save_as: event.saveAs } }
    : null;
  return invoke("sync_editor_message", {
    message,
    filePath,
    event: eventParam,
  });
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Check if an extension state represents a running extension.
 */
export function isExtensionRunning(state: ExtensionState): boolean {
  return state === "running";
}

/**
 * Get error message from an extension state, if it's in a failed state.
 */
export function getExtensionError(state: ExtensionState): string | undefined {
  if (typeof state === "object" && "failed" in state) {
    return state.failed;
  }
  return undefined;
}
