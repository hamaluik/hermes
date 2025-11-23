/**
 * Bridge module for comparing HL7 messages.
 *
 * Provides functions to compare two HL7 messages and identify differences
 * at the segment, field, component, and subcomponent levels.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Type of difference detected between two message elements.
 */
export type DiffType = "removed" | "added" | "modified" | "unchanged";

/**
 * A single difference at the field/component level.
 */
export interface FieldDiff {
  /** Field path (e.g., "PID.5.1", "MSH.9") */
  path: string;
  /** Human-readable description of the field if known */
  description: string | null;
  /** Type of difference */
  diff_type: DiffType;
  /** Value in the left message (null if added) */
  left_value: string | null;
  /** Value in the right message (null if removed) */
  right_value: string | null;
  /** Character range in left message for highlighting [start, end] */
  left_range: [number, number] | null;
  /** Character range in right message for highlighting [start, end] */
  right_range: [number, number] | null;
}

/**
 * Differences for a single segment.
 */
export interface SegmentDiff {
  /** Segment name (e.g., "PID", "MSH") */
  name: string;
  /** Segment occurrence index (0-based, for repeating segments) */
  occurrence: number;
  /** Type of difference for the segment as a whole */
  diff_type: DiffType;
  /** Field-level differences within this segment */
  fields: FieldDiff[];
  /** Character range in left message for the entire segment [start, end] */
  left_range: [number, number] | null;
  /** Character range in right message for the entire segment [start, end] */
  right_range: [number, number] | null;
}

/**
 * Summary statistics for the diff.
 */
export interface DiffSummary {
  /** Total number of segments added */
  segments_added: number;
  /** Total number of segments removed */
  segments_removed: number;
  /** Total number of segments modified */
  segments_modified: number;
  /** Total number of field-level differences */
  total_field_changes: number;
}

/**
 * Complete diff result for two messages.
 */
export interface MessageDiff {
  /** Segment-level differences */
  segments: SegmentDiff[];
  /** Summary statistics */
  summary: DiffSummary;
}

/**
 * Compares two HL7 messages and returns structured differences.
 *
 * Performs a semantic comparison at multiple levels:
 * - Segment level: identifies added, removed, or modified segments
 * - Field level: within each segment, identifies field changes
 * - Component/subcomponent level: tracks changes at the finest granularity
 *
 * Segments are matched by name and occurrence index. For example, if both
 * messages have two PID segments, PID[0] is compared to PID[0] and PID[1]
 * to PID[1].
 *
 * @param left - The "original" or "before" message
 * @param right - The "new" or "after" message
 * @returns Structured diff result with all differences
 * @throws If either message cannot be parsed
 *
 * @example
 * const diff = await compareMessages(originalMessage, modifiedMessage);
 * console.log(`${diff.summary.total_field_changes} differences found`);
 *
 * for (const segment of diff.segments) {
 *   if (segment.diff_type !== 'unchanged') {
 *     console.log(`${segment.name}: ${segment.diff_type}`);
 *   }
 * }
 */
export async function compareMessages(
  left: string,
  right: string,
): Promise<MessageDiff> {
  return await invoke("compare_messages", { left, right });
}
