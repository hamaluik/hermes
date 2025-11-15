/**
 * Bridge module for tracking cursor position within HL7 messages.
 *
 * Enables context-aware features in the message editor by determining which
 * HL7 field/component the cursor is currently positioned in. This drives the
 * field description panel and Tab key navigation between fields.
 *
 * ## Cursor Location Flow
 *
 * 1. User types or moves cursor in the message editor
 * 2. Editor component calls `locateCursor()` with message text and cursor offset
 * 3. Rust backend:
 *    - Parses the message to understand its structure
 *    - Walks through segments and fields to find cursor position
 *    - Determines which segment, field, component, etc. contains the cursor
 * 4. Frontend receives location and uses it to:
 *    - Display field description below the editor (via description.ts)
 *    - Show the field path (e.g., "PID.5.1 - Patient Last Name")
 *    - Enable Tab key to jump to next field (via getRangeOfNextField)
 *
 * ## Why Cursor Tracking?
 *
 * HL7 messages are difficult to read due to delimiter-based structure. Showing
 * context about the current field helps users understand what they're editing
 * without having to count field separators or reference documentation.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Hierarchical position of the cursor within an HL7 message structure.
 *
 * All fields are optional because the cursor might be in an invalid location
 * (e.g., between segments, in whitespace, or in an unparseable message).
 */
export interface LocatedCursor {
  /** Segment name (e.g., "MSH", "PID") */
  segment?: string;
  /** Which occurrence of the segment (0-indexed, for repeating segments) */
  segment_n?: number;
  /** Field number within the segment */
  field?: number;
  /** Repeat number if the field is repeating */
  repeat?: number;
  /** Component number within the field */
  component?: number;
  /** Subcomponent number within the component */
  subcomponent?: number;
}

/**
 * Determines the HL7 structural location of the cursor.
 *
 * Returns null if the cursor is not within a valid field (e.g., in whitespace
 * between segments or in an unparseable message).
 *
 * @param message - Raw HL7 message string
 * @param cursor - Character offset of the cursor (0 = start of message)
 * @returns Location information or null if cursor is not in a valid field
 */
export async function locateCursor(
  message: string,
  cursor: number,
): Promise<LocatedCursor | null> {
  return invoke("locate_cursor", {
    message,
    cursor,
  });
}

/**
 * Finds the character range of the field immediately before the cursor.
 *
 * Used for Shift+Tab navigation to move cursor to the previous field. The
 * returned range can be used to set the editor selection.
 *
 * @param message - Raw HL7 message string
 * @param cursor - Current cursor position
 * @returns Start and end offsets of the previous field, or null if no previous field
 */
export async function getRangeOfPreviousField(
  message: string,
  cursor: number,
): Promise<{ start: number; end: number } | null> {
  return invoke("get_range_of_previous_field", {
    message,
    cursor,
  });
}

/**
 * Finds the character range of the field immediately after the cursor.
 *
 * Used for Tab key navigation to move cursor to the next field. The returned
 * range can be used to set the editor selection, allowing the user to quickly
 * navigate through fields without clicking.
 *
 * @param message - Raw HL7 message string
 * @param cursor - Current cursor position
 * @returns Start and end offsets of the next field, or null if no next field
 */
export async function getRangeOfNextField(
  message: string,
  cursor: number,
): Promise<{ start: number; end: number } | null> {
  return invoke("get_range_of_next_field", {
    message,
    cursor,
  });
}
