/**
 * Type definitions for the extension system frontend integration.
 *
 * These types mirror the Rust types in `extensions/types.rs` and define the
 * event payloads used for communication between the frontend and backend
 * during extension operations.
 */

// ============================================================================
// Message Format
// ============================================================================

/**
 * Supported message formats for extension operations.
 */
export type MessageFormat = "hl7" | "json" | "yaml" | "toml";

// ============================================================================
// Editor Bridge Event Payloads
// ============================================================================

/**
 * Payload for `extension-get-message-request` event.
 *
 * Emitted by the backend when an extension calls `editor/getMessage`.
 * The frontend must respond by calling `provideExtensionMessage()`.
 */
export interface GetMessageRequestPayload {
  /** Extension ID that made the request. */
  extensionId: string;

  /** JSON-RPC request ID (string or number). */
  requestId: string | number;

  /** Requested message format. */
  format: MessageFormat;
}

/**
 * Result provided to the backend for `editor/getMessage` requests.
 */
export interface GetMessageResult {
  /** The message content in the requested format. */
  message: string;

  /** Whether a file is currently open (has a path). */
  hasFile: boolean;

  /** The file path, if one is open. */
  filePath?: string;
}

/**
 * Payload for `extension-patch-message-request` event.
 *
 * Emitted by the backend when an extension calls `editor/patchMessage`.
 * The frontend must apply patches and respond via `provideExtensionPatchResult()`.
 */
export interface PatchMessageRequestPayload {
  /** Extension ID that made the request. */
  extensionId: string;

  /** JSON-RPC request ID (string or number). */
  requestId: string | number;

  /** Patch operations to apply. */
  patches: Patch[];
}

/**
 * A single patch operation.
 */
export interface Patch {
  /** HL7 path to the field or segment (e.g., "PID.5.1", "OBX[2].5", "NK1"). */
  path: string;

  /** New value for the field, or omit/null to remove. */
  value?: string;

  /** Explicitly remove this path. */
  remove?: boolean;

  /** Create a new segment (path must be segment name only, e.g., "NK1"). */
  create?: boolean;
}

/**
 * Error for a single patch operation.
 */
export interface PatchError {
  /** Index of the failed patch (0-based). */
  index: number;

  /** The path that failed. */
  path: string;

  /** Error description. */
  message: string;
}

/**
 * Result of a patch operation, returned to the extension.
 */
export interface PatchMessageResult {
  /** Whether all patches were applied successfully. */
  success: boolean;

  /** Number of patches applied. */
  patchesApplied: number;

  /** Errors for patches that failed (if any). */
  errors?: PatchError[];
}

/**
 * Payload for `extension-set-message` event.
 *
 * Emitted by the backend when an extension calls `editor/setMessage`.
 * Contains the HL7 message to set in the editor.
 */
export type SetMessagePayload = string;

