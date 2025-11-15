/**
 * Bridge module for syntax highlighting HL7 messages.
 *
 * Wraps HL7 message fields in HTML span tags with CSS classes to enable visual
 * distinction between different parts of the message structure. This makes it
 * easier to read and debug HL7 messages by color-coding segments, fields, and
 * delimiters.
 *
 * ## Syntax Highlighting Flow
 *
 * 1. User types in the message editor
 * 2. Frontend calls `syntaxHighlight()` with the current message text
 * 3. Rust backend:
 *    - Parses the message to identify structural elements
 *    - Wraps segments, fields, components in <span class="..."> tags
 *    - Returns HTML string with highlighted elements
 * 4. Frontend renders the HTML in an overlay div positioned behind the textarea
 * 5. CSS rules apply colors to different element classes
 *
 * ## Why Overlay Instead of Rich Text Editor?
 *
 * Using a plain textarea with a highlighted HTML overlay preserves native text
 * editing behavior (cursor, selection, undo/redo) while still providing visual
 * syntax highlighting. Rich text editors have accessibility and behavioral quirks.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Converts an HL7 message to HTML with syntax highlighting spans.
 *
 * The returned HTML contains the same text as the input, but wrapped in span
 * elements with CSS classes like "segment", "field", "delimiter", etc. The
 * calling component should render this HTML and apply appropriate CSS styling.
 *
 * @param message - Raw HL7 message string
 * @returns HTML string with syntax highlighting markup
 *
 * @example
 * ```ts
 * const html = await syntaxHighlight("MSH|^~\\&|...");
 * // Returns: '<span class="segment">MSH</span><span class="delimiter">|</span>...'
 * ```
 */
export async function syntaxHighlight(message: string): Promise<string> {
  return invoke("syntax_highlight", { message });
}

