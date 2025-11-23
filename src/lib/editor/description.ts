/**
 * Bridge module for loading HL7 field descriptions from the standard specification.
 *
 * Provides human-readable explanations of what each HL7 field means according to
 * the HL7 v2.x standard. These descriptions are shown in the UI to help users
 * understand what data should go in each field.
 *
 * ## Description Loading Flow
 *
 * 1. User positions cursor in a field in the message editor
 * 2. Cursor location is determined via cursor.ts
 * 3. Frontend calls `loadSpec()` with segment, field, and component numbers
 * 4. Rust backend:
 *    - Looks up the field in the hl7-definitions library
 *    - Retrieves the standard description text
 *    - Returns the description or null if not found
 * 5. Frontend displays the description below the editor
 *
 * ## Why "Spec" and Not "Description"?
 *
 * The name comes from the HL7 specification documents that define what each
 * field means. "loadSpec" loads the specification text for a field.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Loads the standard HL7 specification description for a field or component.
 *
 * Pass null for field to get the segment description. Pass null for component
 * to get the field description without drilling down to a specific component.
 *
 * @param segment - Segment name (e.g., "MSH", "PID")
 * @param field - Field number (null for segment-level description)
 * @param component - Component number (null for field-level description)
 * @returns Description text from HL7 standard, or null if not found
 *
 * @example
 * ```ts
 * // Get description of PID segment
 * const segDesc = await loadSpec("PID", null, null);
 *
 * // Get description of PID.5 (patient name)
 * const fieldDesc = await loadSpec("PID", 5, null);
 *
 * // Get description of PID.5.1 (patient last name)
 * const compDesc = await loadSpec("PID", 5, 1);
 * ```
 */
export async function loadSpec(
  segment: string,
  field: number | null,
  component: number | null,
): Promise<string | null> {
  return invoke("get_std_description", { segment, field, component });
}
