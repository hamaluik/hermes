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
 * Represents a search match range for highlighting in the editor.
 */
export interface SearchMatch {
  /** Start position of the match (byte offset) */
  start: number;
  /** End position of the match (byte offset, exclusive) */
  end: number;
}

/**
 * Type of difference for diff highlighting.
 */
export type DiffHighlightType = "added" | "removed" | "modified";

/**
 * Represents a diff highlight range with its type.
 */
export interface DiffMatch {
  /** Start position of the diff (byte offset) */
  start: number;
  /** End position of the diff (byte offset, exclusive) */
  end: number;
  /** Type of difference */
  diff_type: DiffHighlightType;
}

/**
 * Converts an HL7 message to HTML with syntax highlighting spans.
 *
 * The returned HTML contains the same text as the input, but wrapped in span
 * elements with CSS classes like "segment", "field", "delimiter", etc. The
 * calling component should render this HTML and apply appropriate CSS styling.
 *
 * @param message - Raw HL7 message string
 * @param searchMatches - Optional array of search match ranges to highlight
 * @param currentMatchIndex - Optional index of the currently selected match (0-based)
 * @param diffMatches - Optional array of diff highlight ranges with their types
 * @returns HTML string with syntax highlighting markup
 *
 * @example
 * ```ts
 * const html = await syntaxHighlight("MSH|^~\\&|...");
 * // Returns: '<span class="segment">MSH</span><span class="delimiter">|</span>...'
 *
 * // With search matches:
 * const htmlWithMatches = await syntaxHighlight("MSH|^~\\&|...", [{ start: 0, end: 3 }], 0);
 * // Returns: '<span class="search-match-current"><span class="msh">MSH</span></span>...'
 *
 * // With diff highlights:
 * const htmlWithDiff = await syntaxHighlight("MSH|^~\\&|...", undefined, undefined, [{ start: 0, end: 3, diff_type: "modified" }]);
 * // Returns: '<span class="diff-highlight-modified"><span class="msh">MSH</span></span>...'
 * ```
 */
export async function syntaxHighlight(
  message: string,
  searchMatches?: SearchMatch[],
  currentMatchIndex?: number,
  diffMatches?: DiffMatch[],
): Promise<string> {
  return invoke("syntax_highlight", {
    message,
    searchMatches: searchMatches ?? null,
    currentMatchIndex: currentMatchIndex ?? null,
    diffMatches: diffMatches ?? null,
  });
}

