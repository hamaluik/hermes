//! Cursor position tracking within HL7 messages.
//!
//! This module provides commands for determining where a cursor is positioned within
//! the hierarchical structure of an HL7 message, and for navigating between fields.
//!
//! # HL7 Message Structure
//! HL7 messages have a hierarchical structure:
//! ```text
//! Message
//!   └─ Segment (e.g., PID)
//!       └─ Field (e.g., PID.3)
//!           └─ Repeat (for repeating fields, separated by ~)
//!               └─ Component (separated by ^)
//!                   └─ Subcomponent (separated by &)
//! ```
//!
//! # Navigation Workflow
//! The frontend uses cursor location to:
//! 1. Display context-aware field descriptions (what field the cursor is in)
//! 2. Enable keyboard navigation (Tab/Shift-Tab to move between fields)
//! 3. Highlight the current field or component in the UI
//!
//! # Keyboard Navigation
//! The `get_range_of_next_field` and `get_range_of_previous_field` commands support
//! Tab/Shift-Tab navigation by finding the next/previous "cell" in the message.
//! A "cell" is the smallest navigable unit: a field, component, or subcomponent.

use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Structured representation of a cursor's position within an HL7 message.
///
/// Contains hierarchical location information from segment down to subcomponent level.
/// Fields are `Option` because not all cursor positions are within all levels
/// (e.g., cursor in MSH segment name has no field/component information).
#[derive(Default, Serialize)]
pub struct CursorLocation {
    /// Segment identifier (e.g., "MSH", "PID", "PV1")
    segment: Option<String>,
    /// Segment occurrence number (0-based) for repeating segments
    segment_number: Option<usize>,
    /// Field number within the segment (1-based, matching HL7 notation)
    field: Option<usize>,
    /// Repeat index within the field (0-based) for repeating fields
    repeat: Option<usize>,
    /// Component number within the repeat (1-based, matching HL7 notation)
    component: Option<usize>,
    /// Subcomponent number within the component (1-based, matching HL7 notation)
    subcomponent: Option<usize>,
}

/// Determine the HL7 structural location of a cursor position.
///
/// This command translates a character offset in the message string into a structured
/// location within the HL7 hierarchy. It uses the hl7-parser library's `locate_cursor`
/// method, which walks the parsed message tree to find the element containing the cursor.
///
/// # Why Structured Location?
/// The frontend needs to know what field the cursor is in to:
/// * Display the appropriate field description from the HL7 specification
/// * Highlight the current field in the editor
/// * Enable field-aware copy/paste operations
///
/// # Component/Subcomponent Flattening
/// The function only reports component/subcomponent numbers if the field actually
/// contains components/subcomponents. For simple fields with no delimiters, only
/// the field number is reported. This prevents confusion in the UI.
///
/// # Arguments
/// * `message` - The HL7 message as a string
/// * `cursor` - Character offset (0-based) within the message
///
/// # Returns
/// * `Some(CursorLocation)` - Structured location if cursor is within the message
/// * `None` - If message parsing fails or cursor is out of bounds
#[tauri::command]
pub fn locate_cursor(message: &str, cursor: usize) -> Option<CursorLocation> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;

    message.locate_cursor(cursor).map(|loc| {
        let mut location = CursorLocation::default();

        if let Some((segment, segment_n, _)) = loc.segment {
            location.segment = Some(segment.to_string());
            location.segment_number = Some(segment_n);
            if let Some((field_i, field)) = loc.field {
                location.field = Some(field_i);
                if let Some((repeat_i, repeat)) = loc.repeat {
                    if field.has_repeats() {
                        location.repeat = Some(repeat_i);
                    }
                    if let Some((component_i, component)) = loc.component {
                        if repeat.has_components() {
                            location.component = Some(component_i);
                        }
                        if let Some((subcomponent_i, _)) = loc.sub_component {
                            if component.has_subcomponents() {
                                location.subcomponent = Some(subcomponent_i);
                                location.component = Some(component_i);
                            }
                        }
                    }
                }
            }
        }

        location
    })
}

/// Character range within the message (start/end offsets).
///
/// Used to communicate field boundaries to the frontend for navigation and selection.
#[derive(Serialize, Deserialize, Debug)]
pub struct CursorRange {
    /// Starting character offset (inclusive)
    pub start: usize,
    /// Ending character offset (exclusive)
    pub end: usize,
}

/// Flatten an HL7 message into a linear sequence of navigable "cells".
///
/// This function traverses the hierarchical message structure and extracts the character
/// ranges of each navigable element in document order. A "cell" is the smallest unit
/// that should be treated as a single navigation target.
///
/// # Navigation Cell Granularity
/// The granularity adapts to the structure:
/// * **Segment name** is always a cell (e.g., "PID")
/// * If a field has no repeats, the entire field is a cell
/// * If a field has repeats but a repeat has no components, each repeat is a cell
/// * If a repeat has components but no subcomponents, each component is a cell
/// * If a component has subcomponents, each subcomponent is a cell
///
/// This ensures navigation stops at the most specific level that actually exists,
/// rather than jumping over fine-grained structure.
///
/// # Why Flattening?
/// The hierarchical structure is perfect for parsing and understanding, but navigation
/// requires a linear sequence. Tab moves forward in document order, Shift-Tab moves
/// backward. Flattening provides that linear sequence.
///
/// # Arguments
/// * `message` - Parsed HL7 message
///
/// # Returns
/// Vector of character ranges in document order, representing navigable cells.
fn flatten_message(message: &hl7_parser::Message) -> Vec<Range<usize>> {
    let mut cells = Vec::new();
    for segment in message.segments() {
        // include segment name as a navigable cell
        cells.push(Range {
            start: segment.range.start,
            end: segment.range.start + segment.name.len(),
        });

        if segment.fields.is_empty() {
            continue;
        }

        for field in segment.fields() {
            if field.repeats.is_empty() {
                cells.push(field.range.clone());
                continue;
            }

            for repeat in field.repeats.iter() {
                if repeat.components.is_empty() {
                    cells.push(repeat.range.clone());
                    continue;
                }

                for component in repeat.components.iter() {
                    if component.subcomponents.is_empty() {
                        cells.push(component.range.clone());
                        continue;
                    }

                    for subcomponent in component.subcomponents.iter() {
                        cells.push(subcomponent.range.clone());
                    }
                }
            }
        }
    }
    cells
}

/// Get the character range of the next navigable field after the cursor.
///
/// This command implements Tab-key navigation in the message editor. It finds the
/// current cell containing the cursor, then returns the range of the next cell.
///
/// # Navigation Behavior
/// * If the cursor is within a cell, returns the next cell after it
/// * If the cursor is at the end of the message, returns None
/// * If the cursor is between cells, returns the next cell
///
/// # Frontend Integration
/// The frontend uses the returned range to:
/// 1. Move the cursor to the start of the next field
/// 2. Select the entire field for easy editing
/// 3. Scroll the field into view if needed
///
/// # Arguments
/// * `message` - The HL7 message as a string
/// * `cursor` - Current cursor position (character offset)
///
/// # Returns
/// * `Some(CursorRange)` - Range of the next field
/// * `None` - If at end of message or parsing fails
#[tauri::command]
pub fn get_range_of_next_field(message: &str, cursor: usize) -> Option<CursorRange> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    let cells = flatten_message(&message);

    let mut cells_iter = cells.iter();
    while let Some(cell) = cells_iter.next() {
        if cursor >= cell.start && cursor <= cell.end {
            return cells_iter.next().map(|next_cell| CursorRange {
                start: next_cell.start,
                end: next_cell.end,
            });
        }
    }

    None
}

/// Get the character range of the previous navigable field before the cursor.
///
/// This command implements Shift-Tab navigation in the message editor. It's the
/// reverse of `get_range_of_next_field`, traversing the cell list backward.
///
/// # Navigation Behavior
/// * If the cursor is within a cell, returns the previous cell before it
/// * If the cursor is at the start of the message, returns None
/// * If the cursor is between cells, returns the previous cell
///
/// # Implementation Note
/// Uses a reverse iterator over the flattened cells to efficiently find the
/// previous cell without building a reversed copy of the entire vector.
///
/// # Arguments
/// * `message` - The HL7 message as a string
/// * `cursor` - Current cursor position (character offset)
///
/// # Returns
/// * `Some(CursorRange)` - Range of the previous field
/// * `None` - If at start of message or parsing fails
#[tauri::command]
pub fn get_range_of_previous_field(message: &str, cursor: usize) -> Option<CursorRange> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    let cells = flatten_message(&message);

    let mut cells_iter = cells.iter().rev();
    while let Some(cell) = cells_iter.next() {
        if cursor >= cell.start && cursor <= cell.end {
            return cells_iter.next().map(|prev_cell| CursorRange {
                start: prev_cell.start,
                end: prev_cell.end,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_range_of_next_field_in_component_next_component() {
        let message = r#"MSH|^~\&|a^b"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 12);
    }

    #[test]
    fn can_get_range_of_next_field_in_component_next_repeat_component() {
        let message = r#"MSH|^~\&|a~b^c"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 12);
    }

    #[test]
    fn can_get_range_of_next_field_in_component_next_repeat() {
        let message = r#"MSH|^~\&|a~bc"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 13);
    }

    #[test]
    fn can_get_range_of_next_field_in_field_next_component() {
        let message = r#"MSH|^~\&|a|b^c"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 12);
    }

    #[test]
    fn can_get_range_of_next_field_in_field_next_field() {
        let message = r#"MSH|^~\&|a|bc"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 13);
    }
}
