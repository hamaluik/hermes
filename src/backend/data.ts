/**
 * Bridge module for parsing and manipulating HL7 message data.
 *
 * Provides functions to extract information from HL7 messages (segment names,
 * message type, trigger event) and to parse/render individual segments. This
 * module enables the "segment tab" UI pattern where each segment (MSH, PID, ORC,
 * etc.) can be edited in a structured form and then rendered back into the message.
 *
 * ## Parse/Render Cycle Flow
 *
 * 1. User opens a segment tab (e.g., "PID - Patient Identification")
 * 2. Frontend calls `parseMessageSegment()` to extract fields from the message
 * 3. Rust backend:
 *    - Parses the full HL7 message using hl7-parser
 *    - Finds the specified segment occurrence (handles repeating segments)
 *    - Extracts field values into a flat structure (PID.3, PID.5.1, etc.)
 * 4. Frontend displays field values in input fields
 * 5. User edits field values in the form
 * 6. When user applies changes, frontend calls `renderMessageSegment()`
 * 7. Rust backend:
 *    - Parses the original message
 *    - Updates the specified segment with new field values
 *    - Serializes the message back to HL7 format
 * 8. Frontend replaces the message text with the rendered result
 *
 * ## Why Parse Then Render?
 *
 * Direct text manipulation of HL7 messages is error-prone due to complex field
 * delimiters (|, ^, ~, &) and encoding rules. Parsing to a structured format,
 * editing, then rendering ensures delimiter handling is correct and prevents
 * breaking the message structure.
 */

import { invoke } from "@tauri-apps/api/core";
import type { SegmentSchema } from "./schema";

/**
 * Structured representation of a segment's field data.
 *
 * Fields are accessed by their hierarchical path string (e.g., "PID.3" for
 * patient ID, "PID.5.1" for patient last name). This flat structure simplifies
 * form binding in the UI compared to nested objects.
 */
export interface SegmentData {
  /** Map of field paths to values. Null means the field is empty. */
  fields: Record<string, string | null>;
}

/**
 * Extracts the list of segment names from an HL7 message.
 *
 * Used to populate the segment tabs in the UI, showing which segments are
 * present in the current message (MSH, PID, PV1, ORC, etc.).
 *
 * @param message - Raw HL7 message string
 * @returns Array of segment names in the order they appear
 */
export async function getMessageSegmentNames(
  message: string,
): Promise<string[]> {
  try {
    return await invoke("get_message_segment_names", { message });
  } catch (error) {
    console.error("Error getting message segment names:", error);
    throw error;
  }
}

/**
 * Extracts the trigger event from an HL7 message's MSH segment.
 *
 * The trigger event (found in MSH.9.2) identifies what action caused the message
 * to be sent (e.g., "A01" for patient admit, "O01" for order message). This is
 * used to determine which fields are relevant for the message type.
 *
 * @param message - Raw HL7 message string
 * @returns Trigger event code (e.g., "A01", "O01") or null if not found
 */
export async function getMessageTriggerEvent(
  message: string,
): Promise<string | null> {
  try {
    return await invoke("get_message_trigger_event", { message });
  } catch (error) {
    console.error("Error getting message trigger event:", error);
    throw error;
  }
}

/**
 * Extracts the message type from an HL7 message's MSH segment.
 *
 * The message type (found in MSH.9.1) identifies the broad category of the
 * message (e.g., "ADT" for admission/discharge/transfer, "ORM" for order).
 *
 * @param message - Raw HL7 message string
 * @returns Message type code (e.g., "ADT", "ORM") or null if not found
 */
export async function getMessageType(
  message: string,
): Promise<string | null> {
  try {
    return await invoke("get_message_type", { message });
  } catch (error) {
    console.error("Error getting message type:", error);
    throw error;
  }
}

/**
 * Parses a specific segment from an HL7 message into structured field data.
 *
 * Handles repeating segments (e.g., multiple OBX segments) by specifying which
 * occurrence to parse. The segmentRepeat parameter is 0-indexed.
 *
 * @param message - Raw HL7 message string
 * @param segment - Segment name (e.g., "PID", "ORC")
 * @param segmentRepeat - Which occurrence of the segment (0 = first, 1 = second, etc.)
 * @returns Structured field data with field paths as keys
 */
export async function parseMessageSegment(
  message: string,
  segment: string,
  segmentRepeat: number,
): Promise<SegmentData> {
  try {
    return await invoke("parse_message_segment", {
      message,
      segment,
      segmentRepeat,
    });
  } catch (error) {
    console.error("Error parsing message segment:", error);
    throw error;
  }
}

/**
 * Updates a segment in an HL7 message with new field values and returns the modified message.
 *
 * Preserves all other segments and fields unchanged. Only the specified segment
 * occurrence is updated with the provided data. Empty string values in the data
 * clear the corresponding fields in the message.
 *
 * @param message - Raw HL7 message string
 * @param segment - Segment name (e.g., "PID", "ORC")
 * @param segmentRepeat - Which occurrence to update (0 = first, 1 = second, etc.)
 * @param data - New field values to apply
 * @returns Modified HL7 message string
 */
export async function renderMessageSegment(
  message: string,
  segment: string,
  segmentRepeat: number,
  data: SegmentData,
): Promise<string> {
  return await invoke("render_message_segment", {
    message,
    segment,
    segmentRepeat,
    data,
  });
}

/**
 * Result of generating a new control ID.
 */
export interface GenerateControlIdResult {
  /** The modified message with the new control ID */
  message: string;
  /** Character range of the control ID field (MSH.10) for editor selection */
  range: { start: number; end: number };
}

/**
 * Generates a new control ID and inserts it into MSH.10.
 *
 * Creates a random 20-character alphanumeric control ID and replaces the
 * value in MSH.10. The control ID uniquely identifies each message and is
 * used by receiving systems to detect duplicates.
 *
 * Use this when resending a message or creating a new message from a template,
 * as the control ID should be unique for each message sent.
 *
 * @param message - Raw HL7 message string
 * @returns Modified message and the range of the new control ID
 */
export async function generateControlId(
  message: string,
): Promise<GenerateControlIdResult> {
  return await invoke("generate_control_id", { message });
}

/**
 * Creates a SegmentData structure with all fields set to null.
 *
 * Used to initialize the form when creating a new segment from scratch or when
 * the segment doesn't exist in the message yet. The schema determines which
 * fields should be present based on the segment type.
 *
 * @param segment - Segment name (e.g., "PID", "ORC")
 * @param schema - Schema defining the fields for this segment type
 * @returns SegmentData with all field values set to null
 */
export function generateDefaultData(
  segment: string,
  schema: SegmentSchema,
): SegmentData {
  const data: SegmentData = { fields: {} };
  for (const field of schema) {
    // Skip group headers - they're not actual fields
    if (field.group) continue;
    const fieldName = `${segment}.${field.field}`;
    if (field.component) {
      data.fields[`${fieldName}.${field.component}`] = null;
    } else {
      data.fields[fieldName] = null;
    }
  }
  return data;
}

/**
 * Character range within a message (start and end offsets).
 */
export interface CursorRange {
  start: number;
  end: number;
}

/**
 * Gets the character range of the current navigable cell at the cursor position.
 *
 * A "cell" is the smallest navigable unit (field, repeat, component, or subcomponent)
 * that the cursor is currently within. This is used to determine what text to replace
 * when inserting content at the cursor position.
 *
 * Returns null if the cursor is not within a valid cell (e.g., on a segment name,
 * between segments, or if the message can't be parsed).
 *
 * @param message - Raw HL7 message string
 * @param cursor - Cursor position (character offset)
 * @returns Range of the cell containing the cursor, or null if not in a valid cell
 */
export async function getCurrentCellRange(
  message: string,
  cursor: number,
): Promise<CursorRange | null> {
  return await invoke("get_current_cell_range", { message, cursor });
}

/**
 * Generates a current timestamp in HL7 DTM format.
 *
 * Creates an HL7-formatted timestamp for the current time in the local timezone.
 * Format: YYYYMMDDHHmmss (without offset) or YYYYMMDDHHmmss+/-ZZZZ (with offset)
 *
 * @param includeOffset - Whether to include the UTC offset (e.g., -0500)
 * @returns Formatted timestamp string
 */
export async function getCurrentHl7Timestamp(
  includeOffset: boolean,
): Promise<string> {
  return await invoke("get_current_hl7_timestamp", { includeOffset });
}

/**
 * Formats a datetime string into HL7 DTM format.
 *
 * Parses an ISO 8601 datetime string and converts it to HL7 timestamp format.
 * Handles three cases:
 *
 * 1. **With explicit offset** (e.g., "2025-01-15T14:30:00-05:00"):
 *    Uses the provided offset. This is the typical case when a user selects
 *    a specific timezone in the Insert Timestamp modal.
 *
 * 2. **Without offset** (e.g., "2025-01-15T14:30:00"):
 *    Uses the system's local timezone. This happens when "Local timezone"
 *    is selected in the modal.
 *
 * 3. **With timezone annotation** (e.g., "2025-01-15T14:30:00[America/New_York]"):
 *    Uses the named timezone. This format is less common but fully supported.
 *
 * @param datetime - ISO 8601 formatted datetime with optional offset
 * @param includeOffset - Whether to include the UTC offset in the output
 * @returns Formatted HL7 timestamp string (e.g., "20250115143000-0500")
 * @throws If the datetime string cannot be parsed
 *
 * @example
 * // With offset - uses the specified offset
 * await formatDatetimeToHl7("2025-01-15T14:30:00-08:00", true);
 * // Returns: "20250115143000-0800"
 *
 * @example
 * // Without offset - uses local timezone
 * await formatDatetimeToHl7("2025-01-15T14:30:00", true);
 * // Returns: "20250115143000-0500" (if local is EST)
 *
 * @example
 * // Exclude offset from output
 * await formatDatetimeToHl7("2025-01-15T14:30:00-08:00", false);
 * // Returns: "20250115143000"
 */
export async function formatDatetimeToHl7(
  datetime: string,
  includeOffset: boolean,
): Promise<string> {
  return await invoke("format_datetime_to_hl7", { datetime, includeOffset });
}
