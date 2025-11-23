/**
 * Bridge module for validating HL7 messages against schema definitions.
 *
 * Provides functions to validate HL7 messages for structural correctness,
 * required fields, length limits, patterns, and allowed values.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Severity level for validation issues.
 */
export type Severity = "error" | "warning" | "info";

/**
 * Type of validation rule that was violated.
 */
export type ValidationRule =
  | "parse_error"
  | "required_field"
  | "min_length"
  | "max_length"
  | "pattern"
  | "allowed_values"
  | "required_segment"
  | "invalid_date";

/**
 * A single validation issue found in the message.
 */
export interface ValidationIssue {
  /** HL7 path to the field (e.g., "PID.3", "MSH.9.1") */
  path: string;
  /** Character range in the message for highlighting [start, end] */
  range: [number, number] | null;
  /** Severity of the issue */
  severity: Severity;
  /** Human-readable description of the issue */
  message: string;
  /** Which validation rule was violated */
  rule: ValidationRule;
  /** The actual value that caused the issue (if applicable) */
  actual_value: string | null;
}

/**
 * Summary of validation results.
 */
export interface ValidationSummary {
  /** Number of errors found */
  errors: number;
  /** Number of warnings found */
  warnings: number;
  /** Number of info messages */
  info: number;
}

/**
 * Complete validation result for a message.
 */
export interface ValidationResult {
  /** List of all validation issues */
  issues: ValidationIssue[];
  /** Summary counts */
  summary: ValidationSummary;
}

/**
 * Perform light validation (fast, for passive background checking).
 *
 * Checks:
 * - Parse errors
 * - Required fields
 *
 * This is designed to run frequently without noticeable performance impact.
 *
 * @param message - The HL7 message to validate
 * @returns Validation result with issues and summary
 *
 * @example
 * const result = await validateLight(message);
 * if (result.summary.errors > 0) {
 *   console.log(`${result.summary.errors} errors found`);
 * }
 */
export async function validateLight(message: string): Promise<ValidationResult> {
  return await invoke("validate_light", { message });
}

/**
 * Perform full validation (comprehensive, for on-demand checking).
 *
 * Checks everything from light validation plus:
 * - Length limits (minlength, maxlength)
 * - Pattern matching
 * - Allowed values
 * - Message structure (required segments)
 * - Date/datetime format validation
 *
 * @param message - The HL7 message to validate
 * @returns Validation result with issues and summary
 *
 * @example
 * const result = await validateFull(message);
 * for (const issue of result.issues) {
 *   console.log(`${issue.severity}: ${issue.path} - ${issue.message}`);
 * }
 */
export async function validateFull(message: string): Promise<ValidationResult> {
  return await invoke("validate_full", { message });
}

/**
 * Get validation highlights for syntax highlighting.
 *
 * Converts validation issues to highlight ranges compatible with the
 * syntax highlighting system.
 *
 * @param issues - Validation issues to convert
 * @returns Array of highlight ranges with severity-based types
 */
export function getValidationHighlights(
  issues: ValidationIssue[],
): { start: number; end: number; validation_type: Severity }[] {
  return issues
    .filter((issue) => issue.range !== null)
    .map((issue) => ({
      start: issue.range![0],
      end: issue.range![1],
      validation_type: issue.severity,
    }));
}
