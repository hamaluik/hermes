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

// Re-export ExtensionConfig from settings for convenience
export type { ExtensionConfig } from "../../settings";

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
 * Result of executing an extension command.
 */
export interface CommandExecuteResult {
  /** Whether the command executed successfully. */
  success: boolean;

  /** Optional message (error or informational). */
  message?: string;
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

// TODO: Phase 4 - The following functions will be used for frontend integration:
//
// - getExtensions(): Get status of all extensions for settings UI
// - getExtensionToolbarButtons(): Get toolbar buttons for dynamic rendering
// - executeExtensionCommand(command: string): Execute command when toolbar button clicked
//
// These are already implemented in the Rust backend (commands/extensions/mod.rs)
// but the frontend integration is deferred to Phase 4.

/**
 * Get status information for all extensions.
 *
 * Returns a list of extension statuses including ID, name, version, state,
 * and any error messages for failed extensions.
 *
 * TODO: Phase 4 - Wire up to settings UI
 */
export async function getExtensions(): Promise<ExtensionStatus[]> {
  return invoke("get_extensions");
}

/**
 * Get all toolbar buttons from all running extensions.
 *
 * Returns buttons with their associated extension IDs, allowing the frontend
 * to display them and route clicks appropriately.
 *
 * TODO: Phase 4 - Wire up to toolbar component
 */
export async function getExtensionToolbarButtons(): Promise<
  ToolbarButtonInfo[]
> {
  return invoke("get_extension_toolbar_buttons");
}

/**
 * Execute an extension command.
 *
 * This is called when a user clicks an extension toolbar button. The command
 * string identifies which extension and action to invoke.
 *
 * TODO: Phase 4 - Wire up to toolbar button click handlers
 */
export async function executeExtensionCommand(
  command: string,
): Promise<CommandExecuteResult> {
  return invoke("execute_extension_command", { command });
}
