/**
 * Bridge module for exporting HL7 messages to various formats.
 *
 * Provides functions to convert HL7 messages to JSON, YAML, and TOML formats.
 * The output is a clean hierarchical structure optimised for readability,
 * without internal parser metadata like byte ranges.
 *
 * Use cases:
 * - Version control: structured diffs are more readable than pipe-delimited
 * - External tool integration: other systems can consume JSON/YAML more easily
 * - Programmatic editing: modify specific fields without parsing HL7 syntax
 *
 * Output structure:
 * - Segments → object keys (repeated segments become arrays)
 * - Fields → 1-based string indices, empty fields omitted
 * - Components → 1-based indices; simple fields become plain strings
 * - Field repetitions → arrays
 *
 * Note: For MSH, field "1" contains the field separator (|) and field "2"
 * contains the encoding characters (^~\&). See import.ts for re-import details.
 *
 * @example
 * // For message: MSH|^~\&|APP|||20231215||ADT^A01
 * // JSON output:
 * // {
 * //   "MSH": {
 * //     "1": "|",
 * //     "2": "^~\\&",
 * //     "3": "APP",
 * //     "7": "20231215",
 * //     "9": { "1": "ADT", "2": "A01" }
 * //   }
 * // }
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Exports an HL7 message to JSON format.
 *
 * @param message - The raw HL7 message text
 * @returns The message serialized as pretty-printed JSON
 */
export async function exportToJson(message: string): Promise<string> {
  return invoke<string>("export_to_json", { message });
}

/**
 * Exports an HL7 message to YAML format.
 *
 * @param message - The raw HL7 message text
 * @returns The message serialized as YAML
 */
export async function exportToYaml(message: string): Promise<string> {
  return invoke<string>("export_to_yaml", { message });
}

/**
 * Exports an HL7 message to TOML format.
 *
 * @param message - The raw HL7 message text
 * @returns The message serialized as TOML
 */
export async function exportToToml(message: string): Promise<string> {
  return invoke<string>("export_to_toml", { message });
}
