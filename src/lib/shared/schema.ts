/**
 * Bridge module for querying HL7 message and segment schemas.
 *
 * Provides access to the schema information loaded from messages.toml in the Rust
 * backend. This schema defines which segments belong to which message types, what
 * fields exist in each segment, and metadata like field names, validation rules,
 * and whether fields are required.
 *
 * ## Schema Loading Flow
 *
 * 1. At app startup, Rust backend loads messages.toml into a SchemaCache
 * 2. Frontend calls `getMessagesSchema()` to get the list of all message types
 * 3. For each segment the user wants to edit, frontend calls `getSegmentSchema()`
 * 4. Schema includes:
 *    - Field names and numbers (e.g., field 3 = "Patient ID")
 *    - Component information (e.g., PID.5.1 = Last Name, PID.5.2 = First Name)
 *    - Validation rules (min/max length, patterns, required fields)
 *    - UI hints (placeholders, data types, allowed values)
 * 5. Frontend uses schema to:
 *    - Show field labels in forms
 *    - Validate user input
 *    - Filter fields by trigger event (some fields only apply to certain events)
 *    - Display dropdowns for fields with predefined values
 *
 * ## Why Load Schema From Backend?
 *
 * The schema is defined in TOML and parsed by Rust at startup. Keeping it in
 * the backend avoids duplicating the schema definition and ensures the same
 * validation rules are used for both parsing and rendering.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Maps segment names to their full descriptive names.
 * Example: { "MSH": "Message Header", "PID": "Patient Identification" }
 */
export type SegmentPaths = Record<string, string>;

/**
 * Metadata about a segment within a message type definition.
 */
export interface SegmentMetadata {
  /** Segment code (e.g., "MSH", "PID") */
  name: string;
  /** Whether this segment is required for the message type */
  required?: boolean;
}

/**
 * Top-level schema structure defining all message types and their segments.
 */
export interface MessagesSchema {
  /** Map of segment codes to full names */
  segments: SegmentPaths;
  /** Map of trigger events (e.g., "A01") to their segment structures */
  message: Record<string, SegmentMetadata[]>;
}

/**
 * Special data types that require specific UI handling.
 */
export enum DataType {
  /** Date field - UI should show date picker */
  Date = "date",
  /** DateTime field - UI should show date+time picker */
  DateTime = "datetime",
}

/**
 * Schema definition for a single field or component within a segment.
 *
 * Each Field represents either a top-level field (e.g., PID.3) or a component
 * within a field (e.g., PID.5.1). The schema drives form generation and validation.
 */
export interface Field {
  /** Field number (e.g., 3 for PID.3) */
  field: number;
  /** Human-readable field name (e.g., "Patient Identifier List") */
  name: string;
  /** Component number if this is a component (e.g., 1 for PID.5.1) */
  component?: number;
  /** Group name for organising related fields in the UI */
  group?: string;
  /** Only show this field for specific trigger events (e.g., "A01,A04") */
  trigger_filter?: string;
  /** Minimum allowed length for validation */
  minlength?: number;
  /** Maximum allowed length for validation */
  maxlength?: number;
  /** Placeholder text to show in empty input fields */
  placeholder?: string;
  /** Whether this field is required (validation and UI indication) */
  required?: boolean;
  /** Special data type requiring custom UI handling */
  datatype?: DataType;
  /** Regex pattern for validation */
  pattern?: string;
  /** Explanatory note to show in UI (help text) */
  note?: string;
  /** Predefined values for dropdown/select UI (value -> display label) */
  values?: Record<string, string>;
}

/**
 * Generates the full field path string for a field.
 *
 * Used throughout the UI to identify fields consistently. The path format matches
 * the keys used in SegmentData.fields for easy cross-referencing.
 *
 * @param segment - Segment name (e.g., "PID")
 * @param field - Field schema
 * @returns Field path string (e.g., "PID.3" or "PID.5.1")
 */
export const fieldId = (segment: string, field: Field): string => {
  return (
    `${segment}.${field.field}` +
    (Number.isFinite(field.component) ? `.${field.component}` : "")
  );
};

/**
 * Array of field schemas defining all fields in a segment.
 */
export type SegmentSchema = Field[];

/**
 * Map of segment names to their schemas.
 */
export type SegmentSchemas = Record<string, SegmentSchema>;

/**
 * Retrieves the top-level messages schema from the backend.
 *
 * This is typically called once at app startup to populate the list of available
 * message types and their segment structures. The result is cached in the UI.
 *
 * @returns Schema containing all segment names and message type definitions
 */
export async function getMessagesSchema(): Promise<MessagesSchema> {
  try {
    return await invoke("get_messages_schema");
  } catch (error) {
    console.error("Error getting messages schema:", error);
    throw error;
  }
}

/**
 * Retrieves the field schema for a specific segment.
 *
 * Called on-demand when the user opens a segment tab, rather than loading all
 * schemas at startup. This lazy loading reduces initial load time and memory usage.
 *
 * @param segment - Segment name (e.g., "PID", "ORC")
 * @returns Array of field definitions for the segment
 */
export async function getSegmentSchema(
  segment: string,
): Promise<SegmentSchema> {
  try {
    return await invoke<SegmentSchema>("get_segment_schema", { segment });
  } catch (error) {
    console.error(`Error getting segment {segment} schema:`, error);
    throw error;
  }
}

/**
 * Retrieves schemas for all known segments.
 *
 * Used when the UI needs to pre-load all schemas at once (e.g., for search/filter
 * functionality). The reduce pattern transforms the array of schemas into a lookup
 * map for O(1) access by segment name.
 *
 * @returns Map of segment names to their field schemas
 */
export async function getAllSegmentSchemas(): Promise<SegmentSchemas> {
  return getMessagesSchema().then(async (schema) => {
    const segments = Object.keys(schema.segments);
    console.debug("Segments to fetch:", segments);
    const schemas = await Promise.all(segments.map(getSegmentSchema));
    console.debug("All segment schemas:", schemas);
    // Transform array of schemas into lookup map
    return schemas.reduce((acc, schema, index) => {
      const segment = segments[index];
      acc[segment] = schema;
      return acc;
    }, {} as SegmentSchemas);
  });
}
