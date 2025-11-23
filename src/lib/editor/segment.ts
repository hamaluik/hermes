/**
 * Bridge module for segment-level operations in the message editor.
 *
 * Provides functions for manipulating entire segments: deleting, reordering,
 * and duplicating. These operations work at the segment level (MSH, PID, PV1, etc.)
 * rather than at the field level.
 *
 * ## Why Segment Operations?
 *
 * HL7 messages use delimiter-based encoding that makes manual segment manipulation
 * error-prone. Selecting exact segment boundaries, handling newline styles (CR vs
 * CRLF), and avoiding delimiter corruption during cut/paste is tedious. These
 * operations handle all boundary logic automatically.
 *
 * Common use cases:
 * - Duplicating OBX segments when adding multiple observations with similar structure
 * - Reordering segments to match expected message structure
 * - Quickly removing unwanted segments during message construction
 *
 * ## Keyboard Shortcuts
 *
 * - Cmd+Shift+K: Delete segment under cursor
 * - Cmd+Shift+↑: Move segment up
 * - Cmd+Shift+↓: Move segment down
 * - Cmd+Shift+D: Duplicate segment
 *
 * ## Constraints
 *
 * The MSH segment is protected:
 * - Cannot be deleted (every HL7 message must have MSH)
 * - Cannot be moved (must always be first)
 * - Cannot be duplicated (would create invalid message)
 *
 * ## Usage
 *
 * ```typescript
 * // Get segment at cursor, then duplicate it
 * const index = await getSegmentIndexAtCursor(message, cursorPos);
 * if (index !== null && index !== 0) {
 *   const result = await duplicateSegment(message, index);
 *   if (result) {
 *     updateMessage(result.message);
 *     editor.setSelectionRange(result.cursor, result.cursor);
 *   }
 * }
 * ```
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Result of a segment operation.
 *
 * Contains the modified message and where to position the cursor after
 * the operation completes.
 */
export interface SegmentOperationResult {
  /** The modified message content */
  message: string;
  /** Character offset where the cursor should be positioned */
  cursor: number;
}

/**
 * Direction for moving a segment.
 */
export type MoveDirection = "up" | "down";

/**
 * Gets the 0-based index of the segment containing the cursor.
 *
 * @param message - Raw HL7 message string
 * @param cursor - Cursor position (character offset)
 * @returns Segment index (0 = MSH, 1 = first segment after MSH, etc.) or null
 */
export async function getSegmentIndexAtCursor(
  message: string,
  cursor: number,
): Promise<number | null> {
  return invoke("get_segment_index_at_cursor", { message, cursor });
}

/**
 * Deletes the segment at the given index.
 *
 * The cursor is positioned at the start of the next segment, or the previous
 * segment if deleting the last one.
 *
 * @param message - Raw HL7 message string
 * @param segmentIndex - Index of segment to delete (cannot be 0/MSH)
 * @returns Modified message and cursor position, or null if operation invalid
 */
export async function deleteSegment(
  message: string,
  segmentIndex: number,
): Promise<SegmentOperationResult | null> {
  return invoke("delete_segment", { message, segmentIndex });
}

/**
 * Moves the segment at the given index up or down.
 *
 * The cursor follows the moved segment.
 *
 * @param message - Raw HL7 message string
 * @param segmentIndex - Index of segment to move
 * @param direction - "up" or "down"
 * @returns Modified message and cursor position, or null if operation invalid
 */
export async function moveSegment(
  message: string,
  segmentIndex: number,
  direction: MoveDirection,
): Promise<SegmentOperationResult | null> {
  return invoke("move_segment", { message, segmentIndex, direction });
}

/**
 * Duplicates the segment at the given index.
 *
 * Creates a copy immediately after the original. The cursor is positioned
 * at the start of the new duplicate segment.
 *
 * @param message - Raw HL7 message string
 * @param segmentIndex - Index of segment to duplicate (cannot be 0/MSH)
 * @returns Modified message and cursor position, or null if operation invalid
 */
export async function duplicateSegment(
  message: string,
  segmentIndex: number,
): Promise<SegmentOperationResult | null> {
  return invoke("duplicate_segment", { message, segmentIndex });
}
