//! Syntax highlighting for HL7 messages.
//!
//! This module generates HTML-formatted syntax highlighting for HL7 messages by
//! analyzing the parsed message structure and applying CSS class names to different
//! element types. The output is displayed in the frontend message editor.
//!
//! # Highlighting Strategy
//!
//! The highlighting algorithm works in three phases:
//!
//! 1. **Collect Ranges**: Walk the parsed message tree and identify character ranges
//!    for each element type (segment names, separators, fields, timestamps, etc.)
//!
//! 2. **Create Position Mapping**: Build a character-by-character map of the entire
//!    message, assigning each position to a RangeType. This flattens overlapping
//!    hierarchies into a linear sequence.
//!
//! 3. **Generate HTML**: Traverse the message character-by-character, emitting
//!    `<span class="...">` tags when the RangeType changes. HTML-escape special
//!    characters and convert newlines to `<br/>`.
//!
//! # CSS Classes
//!
//! The generated HTML uses these CSS classes (styled in the frontend):
//! * `msh` - MSH segment name (special highlighting)
//! * `seg` - Other segment names
//! * `seps` - MSH.1 and MSH.2 (field/encoding separators)
//! * `sep` - Separator characters (|, ^, ~, &)
//! * `cell` - Regular field/component/subcomponent values
//! * `temp` - Templated placeholders (e.g., "{now}", "{random}")
//! * `ts` - Timestamp fields (detected via HL7 spec)
//! * `err` - Parse errors or unparsed content
//! * `search-match` - Search result matches (find/replace feature)
//! * `search-match-current` - Currently selected search match
//!
//! # Special Field Detection
//!
//! Timestamp fields are detected using the HL7 standard specification, which knows
//! which fields are expected to contain timestamps (e.g., MSH.7, EVN.2). This allows
//! the UI to render these fields with date/time-specific formatting or validation.
//!
//! Templated values (placeholders like "{now}" or "{random}") are detected by checking
//! if the value is wrapped in curly braces. These placeholders are automatically
//! transformed during message sending (see send_receive.rs).
//!
//! # Search Match Highlighting
//!
//! The syntax highlighting command accepts optional match ranges to highlight search
//! results. Matches are highlighted with `search-match` class, and the current match
//! (if specified) is highlighted with `search-match-current` class for distinction.

use hl7_parser::{parser::ParseError, Message};
use std::{borrow::Cow, ops::Range};

use crate::spec::std_spec::{
    get_version_with_fallback, is_component_a_timestamp, is_field_a_timestamp,
};

/// A range representing a search match for highlighting.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SearchMatch {
    /// Start position of the match (byte offset)
    pub start: usize,
    /// End position of the match (byte offset, exclusive)
    pub end: usize,
}

/// Generate HTML syntax highlighting for an HL7 message.
///
/// This command parses the message and produces HTML with CSS class annotations
/// for styling in the frontend. If parsing fails, it highlights the error position
/// by wrapping unparsed content in an error span.
///
/// # Error Handling
///
/// * **Successful parse, extra content**: If the message parses but has trailing
///   content beyond the last segment, the extra content is wrapped in `<span class="err">`
///
/// * **Parse failure at position**: Content before the error position is rendered
///   normally, content from the error onward is wrapped in `<span class="err">`
///
/// * **Incomplete input**: Similar to parse failure - the incomplete portion is
///   marked as an error
///
/// This approach ensures users always get visual feedback about what parsed correctly
/// and what didn't, rather than failing silently.
///
/// # Arguments
/// * `message` - The HL7 message as a string (with newlines or \r separators)
/// * `search_matches` - Optional list of search match ranges to highlight
/// * `current_match_index` - Optional index of the currently selected match (0-based)
///
/// # Returns
/// HTML string with syntax highlighting, safe for insertion into the DOM
/// (all special characters are HTML-escaped)
#[tauri::command]
pub fn syntax_highlight(
    message: &str,
    search_matches: Option<Vec<SearchMatch>>,
    current_match_index: Option<usize>,
) -> String {
    match hl7_parser::parse_message_with_lenient_newlines(message) {
        Ok(msg) => {
            let mut highlighted =
                do_syntax_highlight(&msg, search_matches.as_deref(), current_match_index);
            if msg.raw_value().len() != message.len() {
                // the delivered message extends beyond the parsed message
                // so just append the rest of the message wrapped in an error span
                // to indicate that it is not part of the parsed message
                let extra = html_escape(&message[msg.raw_value().len()..])
                    .replace('\n', "<br/>")
                    .replace(' ', "&nbsp;");
                highlighted.push_str(&format!(r#"<span class="err">{extra}</span>"#));
            }
            highlighted
        }
        Err(ParseError::FailedToParse { position, .. }) => {
            let before = html_escape(&message[..position]).replace('\n', "<br/>");
            let after = html_escape(&message[position..]).replace('\n', "<br/>");
            format!(r#"{before}<span class="err">{after}</span>"#)
        }
        Err(ParseError::IncompleteInput(position)) => {
            let position = position.unwrap_or(0);
            let before = html_escape(&message[..position]).replace('\n', "<br/>");
            let after = html_escape(&message[position..]).replace('\n', "<br/>");
            format!(r#"{before}<span class="err">{after}</span>"#)
        }
    }
}

/// Type classification for character ranges in an HL7 message.
///
/// Each character in the message is assigned to one of these types to determine
/// its CSS class in the highlighted HTML output.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RangeType {
    /// MSH segment identifier (gets special styling)
    #[allow(clippy::upper_case_acronyms)]
    MSH,
    /// MSH.1 and MSH.2 fields (field separator and encoding characters)
    Separators,
    /// Segment identifiers other than MSH (e.g., PID, PV1, OBX)
    SegmentName,
    /// Delimiter characters (|, ^, ~, &) between fields/components
    Separator,
    /// Regular data cells (field/component/subcomponent values)
    Cell,
    /// Templated placeholder values like "{now}" or "{random}"
    TemplatedValue,
    /// Timestamp fields (detected via HL7 spec)
    Timestamp,
}

impl RangeType {
    /// Get the CSS class name for this range type.
    ///
    /// These classes are defined in the frontend CSS and control the visual
    /// appearance of each element type.
    fn class(&self) -> &'static str {
        match self {
            RangeType::MSH => "msh",
            RangeType::Separators => "seps",
            RangeType::SegmentName => "seg",
            RangeType::Separator => "sep",
            RangeType::Cell => "cell",
            RangeType::TemplatedValue => "temp",
            RangeType::Timestamp => "ts",
        }
    }
}

/// Execute the three-phase highlighting algorithm on a parsed message.
///
/// This is the internal entry point for highlighting. It orchestrates the three phases:
/// collect ranges, create position mapping, and generate HTML.
///
/// # Arguments
/// * `message` - Parsed HL7 message
/// * `search_matches` - Optional slice of search match ranges to highlight
/// * `current_match_index` - Optional index of the currently selected match
///
/// # Returns
/// HTML-formatted string with syntax highlighting
fn do_syntax_highlight(
    message: &Message,
    search_matches: Option<&[SearchMatch]>,
    current_match_index: Option<usize>,
) -> String {
    let ranges = collect_ranges(message);
    // ranges will already be sorted by their start position because of the
    // structure of the message
    let position_types = create_position_mapping(ranges, message.raw_value().len());
    generate_html(message, &position_types, search_matches, current_match_index)
}

/// Collect all character ranges and their types from a parsed message.
///
/// This function walks the message structure (segments → fields → repeats → components
/// → subcomponents) and builds a list of (Range, RangeType) tuples representing each
/// element's position and classification.
///
/// # Special Cases
///
/// * **MSH segment**: The segment name "MSH" gets a special RangeType
/// * **MSH.1 and MSH.2**: These fields contain separator characters and are classified
///   as RangeType::Separators rather than Cell
/// * **Timestamp detection**: Uses the HL7 spec to identify fields that should contain
///   timestamps, enabling special formatting
/// * **Templated values**: Detects placeholders like "{now}" by checking for curly braces
///
/// # Return Value Order
///
/// The returned vector is already sorted by start position because we traverse the
/// message in document order. This is important for the position mapping phase.
///
/// # Arguments
/// * `message` - Parsed HL7 message
///
/// # Returns
/// Vector of (character range, type) tuples, sorted by start position
fn collect_ranges(message: &Message) -> Vec<(Range<usize>, RangeType)> {
    let mut ranges = Vec::new();
    let version = get_version_with_fallback(message);

    for segment in message.segments() {
        let name_start = segment.range.start;
        let name_end = segment.range.start + segment.name.len();

        if segment.name == "MSH" {
            ranges.push((name_start..name_end, RangeType::MSH));
        } else {
            ranges.push((name_start..name_end, RangeType::SegmentName));
        }

        for (field_i, field) in segment.fields().enumerate() {
            if segment.name == "MSH" && field_i < 2 {
                ranges.push((field.range.clone(), RangeType::Separators));
                continue;
            }

            for repeat in field.repeats() {
                for (component_i, component) in repeat.components().enumerate() {
                    for subcomponent in component.subcomponents() {
                        let is_templated_value = subcomponent.raw_value().starts_with('{')
                            && subcomponent.raw_value().ends_with('}');
                        ranges.push((
                            subcomponent.range.clone(),
                            if is_templated_value {
                                RangeType::TemplatedValue
                            } else if is_component_a_timestamp(
                                version,
                                segment.name,
                                field_i + 1,
                                component_i + 1,
                            ) || is_field_a_timestamp(version, segment.name, field_i + 1)
                            {
                                RangeType::Timestamp
                            } else {
                                RangeType::Cell
                            },
                        ));
                    }
                }
            }
        }
    }

    ranges
}

/// Create a character-by-character map of RangeTypes for the entire message.
///
/// This function flattens the list of ranges into a vector where each index represents
/// a character position in the message, and the value is the RangeType for that position.
///
/// # Why a Position Map?
///
/// The position map simplifies HTML generation by allowing us to just scan through
/// the message character-by-character and check `position_types[i]` to determine
/// what class to use. Without this map, we'd need to binary-search the ranges list
/// for every character.
///
/// # Separator Handling
///
/// Positions not covered by any range (i.e., the delimiters between fields) are
/// assigned `None`, which gets defaulted to `RangeType::Separator` during HTML generation.
///
/// # Arguments
/// * `ranges` - Sorted list of (range, type) tuples
/// * `message_len` - Total length of the message in bytes
///
/// # Returns
/// Vector where index = character position, value = RangeType (None for separators)
fn create_position_mapping(
    ranges: Vec<(Range<usize>, RangeType)>,
    message_len: usize,
) -> Vec<Option<RangeType>> {
    let mut position_types = vec![None; message_len];

    for (range, range_type) in ranges.into_iter() {
        for i in range {
            position_types[i] = Some(range_type);
        }
    }

    position_types
}

/// Determines the search match state for a given position.
///
/// Returns a tuple of (is_in_match, is_current_match) indicating whether the position
/// is within a search match and whether it's the currently selected match.
fn get_match_state(
    pos: usize,
    search_matches: Option<&[SearchMatch]>,
    current_match_index: Option<usize>,
) -> (bool, bool) {
    let Some(matches) = search_matches else {
        return (false, false);
    };

    for (i, m) in matches.iter().enumerate() {
        if pos >= m.start && pos < m.end {
            let is_current = current_match_index == Some(i);
            return (true, is_current);
        }
    }
    (false, false)
}

/// Generate HTML with CSS class spans from a message and position type mapping.
///
/// This function walks through the message character-by-character, emitting `<span>`
/// tags when the RangeType changes and closing spans when transitioning to a different type.
///
/// # HTML Escaping
///
/// Special HTML characters are escaped to prevent XSS and ensure correct rendering:
/// * `<` → `&lt;`
/// * `>` → `&gt;`
/// * `&` → `&amp;`
/// * `'` → `&apos;`
/// * `"` → `&quot;`
/// * `\n` → `<br/>`
///
/// # Span Optimization
///
/// The algorithm only opens/closes spans when the type changes. This minimizes the
/// number of span tags in the output, making the HTML more readable and slightly
/// more performant for the browser to render.
///
/// # Memory Allocation
///
/// Pre-allocates the output string with 3x the message length as a heuristic, since
/// HTML markup approximately triples the size due to tags and escaping.
///
/// # Search Match Highlighting
///
/// When search matches are provided, characters within match ranges are wrapped in
/// additional `<span class="search-match">` or `<span class="search-match-current">`
/// tags. The current match (if specified) uses the latter for visual distinction.
///
/// # Arguments
/// * `message` - Parsed HL7 message
/// * `position_types` - Position-to-type mapping from `create_position_mapping`
/// * `search_matches` - Optional slice of search match ranges
/// * `current_match_index` - Optional index of the currently selected match
///
/// # Returns
/// HTML string with syntax highlighting spans
fn generate_html(
    message: &Message,
    position_types: &[Option<RangeType>],
    search_matches: Option<&[SearchMatch]>,
    current_match_index: Option<usize>,
) -> String {
    let raw_message = message.raw_value();
    let mut highlighted = String::with_capacity(raw_message.len() * 3);
    let mut current_type = None;
    let mut current_match_state: (bool, bool) = (false, false);

    for (i, c) in raw_message.char_indices() {
        let range_type = position_types[i].unwrap_or(RangeType::Separator);
        let match_state = get_match_state(i, search_matches, current_match_index);

        // Handle search match span transitions
        if match_state != current_match_state {
            // Close previous match span if we were in one
            if current_match_state.0 {
                highlighted.push_str("</span>");
            }

            // If we're transitioning match state, we need to close and reopen the syntax span
            if current_type.is_some() {
                highlighted.push_str("</span>");
                current_type = None;
            }
        }

        // Handle syntax type transitions
        if current_type != Some(range_type) {
            if current_type.is_some() {
                highlighted.push_str("</span>");
            }

            highlighted.push_str(&format!(
                r#"<span class="{class}">"#,
                class = range_type.class(),
            ));

            current_type = Some(range_type);
        }

        // Open new match span if entering a match
        if match_state != current_match_state && match_state.0 {
            let match_class = if match_state.1 {
                "search-match-current"
            } else {
                "search-match"
            };
            highlighted.push_str(&format!(r#"<span class="{match_class}">"#));
        }

        current_match_state = match_state;

        match c {
            '<' => highlighted.push_str("&lt;"),
            '>' => highlighted.push_str("&gt;"),
            '&' => highlighted.push_str("&amp;"),
            '\'' => highlighted.push_str("&apos;"),
            '\"' => highlighted.push_str("&quot;"),
            '\n' => highlighted.push_str("<br/>"),
            _ => highlighted.push(c),
        }
    }

    // Close any open match span
    if current_match_state.0 {
        highlighted.push_str("</span>");
    }

    if current_type.is_some() {
        highlighted.push_str("</span>");
    }
    highlighted
}

/// HTML-escape special characters in a string.
///
/// This function escapes characters that have special meaning in HTML/XML to ensure
/// they render correctly and don't break the DOM structure.
///
/// # Escaped Characters
///
/// * `<`, `>`, `&`, `'`, `"` - Standard HTML entities
/// * `\t`, `\n`, `\r`, ` ` (space) - Numeric entities to preserve whitespace exactly
///
/// # Why Escape Whitespace?
///
/// The tab/newline/space escaping uses numeric character references because these
/// characters can act as delimiters in XML/HTML attribute values (specifically xs:list
/// types). While this function is used for message content (not attributes), the
/// escaping is comprehensive to handle all edge cases.
///
/// # Optimization
///
/// Uses copy-on-write (Cow) to avoid allocation if no escaping is needed. Only allocates
/// a new string if special characters are found.
///
/// # Arguments
/// * `raw` - Input string or string slice
///
/// # Returns
/// * `Cow::Borrowed` - If no escaping was needed
/// * `Cow::Owned` - If escaping was performed
fn html_escape<'a>(raw: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    let raw = raw.into();
    let bytes = raw.as_bytes();
    let mut escaped = None;
    let mut iter = bytes.iter();
    let mut pos = 0;
    while let Some(i) = iter.position(|&b| {
        matches!(
            b,
            b'<' | b'>' | b'&' | b'\'' | b'\"' | b'\t' | b'\r' | b'\n' | b' '
        )
    }) {
        if escaped.is_none() {
            escaped = Some(Vec::with_capacity(raw.len()));
        }
        let escaped = escaped.as_mut().expect("initialized");
        let new_pos = pos + i;
        escaped.extend_from_slice(&bytes[pos..new_pos]);
        match bytes[new_pos] {
            b'<' => escaped.extend_from_slice(b"&lt;"),
            b'>' => escaped.extend_from_slice(b"&gt;"),
            b'\'' => escaped.extend_from_slice(b"&apos;"),
            b'&' => escaped.extend_from_slice(b"&amp;"),
            b'"' => escaped.extend_from_slice(b"&quot;"),

            // This set of escapes handles characters that should be escaped
            // in elements of xs:lists, because those characters works as
            // delimiters of list elements
            b'\t' => escaped.extend_from_slice(b"&#9;"),
            b'\n' => escaped.extend_from_slice(b"&#10;"),
            b'\r' => escaped.extend_from_slice(b"&#13;"),
            b' ' => escaped.extend_from_slice(b"&#32;"),
            _ => unreachable!(
                "Only '<', '>','\', '&', '\"', '\\t', '\\r', '\\n', and ' ' are escaped"
            ),
        }
        pos = new_pos + 1;
    }

    if let Some(mut escaped) = escaped {
        if let Some(raw) = bytes.get(pos..) {
            escaped.extend_from_slice(raw);
        }
        // SAFETY: we operate on UTF-8 input and search for an one byte chars only,
        // so all slices that was put to the `escaped` is a valid UTF-8 encoded strings
        // TODO: Can be replaced with `unsafe { String::from_utf8_unchecked() }`
        // if unsafe code will be allowed
        Cow::Owned(String::from_utf8(escaped).unwrap())
    } else {
        raw
    }
}
