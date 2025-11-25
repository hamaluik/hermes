/**
 * Editor bridge for extension system.
 *
 * Provides helper functions for handling extension requests that require
 * interaction with the editor. These functions are called by the main page
 * component when it receives events from the backend.
 */

import { invoke } from "@tauri-apps/api/core";
import type { MessageFormat, Patch, PatchError } from "./types";

/**
 * Result of the handleGetMessage operation.
 */
export interface GetMessageHandlerResult {
  /** The message content in the requested format. */
  message: string;
  /** Whether any errors occurred during conversion. */
  error?: string;
}

/**
 * Result of the handlePatchMessage operation.
 */
export interface PatchMessageHandlerResult {
  /** The patched message content (HL7 format). */
  message: string;
  /** Whether all patches were applied successfully. */
  success: boolean;
  /** Number of patches applied. */
  patchesApplied: number;
  /** Errors for patches that failed. */
  errors?: PatchError[];
}

/**
 * Convert an HL7 message to the requested format.
 *
 * Uses the existing export commands on the backend for format conversion.
 *
 * @param rawMessage - The raw HL7 message from the editor
 * @param format - The desired output format
 * @returns The message in the requested format, or an error
 */
export async function handleGetMessage(
  rawMessage: string,
  format: MessageFormat,
): Promise<GetMessageHandlerResult> {
  try {
    let message: string;

    switch (format) {
      case "hl7":
        message = rawMessage;
        break;
      case "json":
        message = await invoke<string>("export_to_json", { message: rawMessage });
        break;
      case "yaml":
        message = await invoke<string>("export_to_yaml", { message: rawMessage });
        break;
      case "toml":
        message = await invoke<string>("export_to_toml", { message: rawMessage });
        break;
      default:
        return { message: "", error: `unsupported format: ${format}` };
    }

    return { message };
  } catch (error) {
    return { message: "", error: String(error) };
  }
}

/**
 * Apply patches to an HL7 message.
 *
 * Uses the backend patch handling for proper HL7 path parsing and manipulation.
 *
 * @param rawMessage - The raw HL7 message from the editor
 * @param patches - The patches to apply
 * @returns The patched message and result information
 */
export async function handlePatchMessage(
  rawMessage: string,
  patches: Patch[],
): Promise<PatchMessageHandlerResult> {
  try {
    // call backend to apply patches
    const result = await invoke<{
      message: string;
      success: boolean;
      patchesApplied: number;
      errors?: PatchError[];
    }>("apply_extension_patches", {
      message: rawMessage,
      patches,
    });

    return result;
  } catch (error) {
    // backend call failed entirely (not a patch-specific error)
    // return a generic error with sentinel values since we don't know which patch failed
    return {
      message: rawMessage,
      success: false,
      patchesApplied: 0,
      errors: [{ index: 0, path: "", message: String(error) }],
    };
  }
}
