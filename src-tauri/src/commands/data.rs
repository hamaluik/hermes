//! HL7 message data manipulation commands.
//!
//! This module provides commands for querying and modifying HL7 messages at the
//! segment level. It enables the frontend to extract structured data from messages
//! and apply changes back to the message.
//!
//! # Workflow for Segment Editing
//! 1. **Parse**: Extract segment data using `parse_message_segment` (returns field/value map)
//! 2. **Modify**: Frontend edits the field values
//! 3. **Render**: Apply changes using `render_message_segment` (returns updated message)
//!
//! # Field Identification Format
//! Fields are identified using dot notation: `SEGMENT.FIELD[.COMPONENT]`
//! Examples:
//! * `PID.3` - Patient ID field
//! * `PID.5.1` - Patient name, last name component
//! * `MSH.9.1` - Message type code

use std::collections::HashMap;

use color_eyre::eyre::Context;
use hl7_parser::builder::{FieldBuilder, MessageBuilder, SegmentBuilder};
use serde::{Deserialize, Serialize};
use tauri::State;

use super::locate_cursor::CursorRange;
use crate::AppData;

/// Segment data extracted from an HL7 message.
///
/// Maps field identifiers (e.g., "PID.3", "PID.5.1") to their values.
/// None values indicate the field exists in the schema but is not populated in the message.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SegmentData {
    /// Map of field IDs to field values
    fields: HashMap<String, Option<String>>,
}

/// Get the list of segment names in a message.
///
/// This command extracts the segment identifiers in order of appearance.
/// Used by the frontend to populate segment selection UI.
///
/// # Fallback Behavior
/// If message parsing fails, returns a single-element vector containing "MSH",
/// since all valid HL7 messages must have an MSH segment.
///
/// # Arguments
/// * `message` - The HL7 message as a string
///
/// # Returns
/// Vector of segment names (e.g., ["MSH", "EVN", "PID", "PV1"])
#[tauri::command]
pub fn get_message_segment_names(message: &str) -> Vec<String> {
    let Ok(message) = hl7_parser::parse_message_with_lenient_newlines(message) else {
        return vec!["MSH".to_string()];
    };
    message
        .segments()
        .map(|segment| segment.name.to_string())
        .collect()
}

/// Extract the trigger event code from a message.
///
/// The trigger event (MSH.9.2) specifies the event that triggered the message,
/// such as "A01" for patient admission or "A08" for patient update.
///
/// # Arguments
/// * `message` - The HL7 message as a string
///
/// # Returns
/// * `Some(String)` - The trigger event code if present
/// * `None` - If message parsing fails or MSH.9.2 is not populated
#[tauri::command]
pub fn get_message_trigger_event(message: &str) -> Option<String> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    message
        .query("MSH.9.2")
        .map(|value| message.separators.decode(value.raw_value()).to_string())
}

/// Extract the message type from a message.
///
/// The message type (MSH.9.1) specifies the general category of the message,
/// such as "ADT" (Admission/Discharge/Transfer) or "ORM" (Order).
///
/// # Arguments
/// * `message` - The HL7 message as a string
///
/// # Returns
/// * `Some(String)` - The message type code if present
/// * `None` - If message parsing fails or MSH.9.1 is not populated
#[tauri::command]
pub fn get_message_type(message: &str) -> Option<String> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    message
        .query("MSH.9.1")
        .map(|value| message.separators.decode(value.raw_value()).to_string())
}

/// Extract structured data from a specific segment in a message.
///
/// This command loads the segment schema, queries the message for each field
/// defined in the schema, and returns a map of field IDs to values.
///
/// # Schema-Driven Extraction
/// The function uses the cached segment schema to determine which fields to extract.
/// This ensures that all defined fields are included in the result, even if they're
/// not populated in the message (they'll have None values).
///
/// # Why Schema-Driven?
/// Using the schema as the template ensures the frontend can display all possible
/// fields for editing, not just the fields that happen to be populated. This is
/// important for message composition workflows.
///
/// # Arguments
/// * `message` - The HL7 message as a string
/// * `segment` - Segment identifier (e.g., "PID", "PV1")
/// * `segment_repeat` - Currently unused; reserved for future support of repeating segments
/// * `state` - Application state containing the schema cache
///
/// # Returns
/// * `Ok(SegmentData)` - Segment data with field/value mappings
/// * `Err(String)` - Failed to load schema or parse message
#[tauri::command]
pub fn parse_message_segment(
    message: &str,
    segment: &str,
    segment_repeat: usize,
    state: State<'_, AppData>,
) -> Result<SegmentData, String> {
    let schema = state
        .schema
        .get_segment(segment)
        .wrap_err_with(|| format!("Failed to load segment {segment} schema"))
        .map_err(|e| format!("{e:#}"))?;

    let message = hl7_parser::parse_message_with_lenient_newlines(message)
        .wrap_err_with(|| "Failed to parse message")
        .map_err(|e| format!("{e:#}"))?;

    Ok(SegmentData {
        fields: schema
            .into_iter()
            .map(|field| {
                let field_id = format!(
                    "{segment}.{field}{component}",
                    // segment_repeat = segment_repeat + 1,
                    field = field.field,
                    component = if let Some(comp) = field.component {
                        format!(".{comp}")
                    } else {
                        String::new()
                    }
                );
                let field_value = message
                    .query(&field_id)
                    .map(|value| message.separators.decode(value.raw_value()).to_string());

                (field_id, field_value)
            })
            .collect(),
    })
}

/// Apply modified segment data back to a message.
///
/// This command updates the specified segment in the message with the provided
/// field values. If the segment doesn't exist, it will be created.
///
/// # Field vs Component Handling
/// The function distinguishes between field-level and component-level updates:
/// * **Field-level** (e.g., "PID.3"): Replaces the entire field value
/// * **Component-level** (e.g., "PID.5.1"): Updates only the specific component,
///   preserving other components in the field
///
/// For component updates, if the field doesn't exist, it creates an empty field
/// first, then sets the component. This ensures components can be set independently.
///
/// # Segment Creation
/// If the specified segment doesn't exist in the message, a new segment is appended.
/// Currently, this doesn't handle segment ordering according to HL7 specifications
/// (see TODO comment in code).
///
/// # Arguments
/// * `message` - The HL7 message as a string
/// * `segment` - Segment identifier to update
/// * `segment_repeat` - Currently unused; reserved for repeating segment support
/// * `data` - Segment data with field values to apply
///
/// # Returns
/// The modified message as a string. If message parsing fails, returns the original message unchanged.
#[tauri::command]
pub fn render_message_segment(
    message: &str,
    segment: &str,
    segment_repeat: usize,
    data: SegmentData,
) -> String {
    let Ok(message) = hl7_parser::parse_message_with_lenient_newlines(message) else {
        return message.to_string();
    };

    let mut message: MessageBuilder = (&message).into();
    // ensure the message has at least `segment_repeat + 1` segments of this type
    // while message.segment_n(segment, segment_repeat + 1).is_none() {
    //     message.push_segment(SegmentBuilder::new(segment));
    // }
    // let seg = message
    //     .segment_n_mut(segment, segment_repeat + 1)
    //     .expect("message has segment");
    if !message.segment_named(segment).is_some() {
        message.push_segment(SegmentBuilder::new(segment));
    }
    let seg = message
        .segment_named_mut(segment)
        .expect("message has segment");

    for (field_id, field_value) in data.fields.into_iter() {
        let Some((field_id, component_id)) = parse_field_id(&field_id, segment) else {
            continue;
        };

        if let Some(component_id) = component_id {
            if !seg.has_field(field_id) {
                seg.set_field(field_id, FieldBuilder::default());
            }
            let field = seg.field_mut(field_id).expect("field exists");
            field.set_component(component_id, field_value.unwrap_or_default());
        } else {
            seg.set_field_value(field_id, field_value.unwrap_or_default());
        }
    }

    // TODO: rearrange the segments if needed

    message.render_with_newlines().to_string()
}

/// Get the character range of a field within an HL7 message by query path.
///
/// This command enables "Jump to Field" functionality in the editor. Given a field
/// path like "PID.5.1", it returns the character range where that field exists in
/// the message, allowing the frontend to position the cursor there.
///
/// # Query Syntax
/// Uses the hl7-parser library's query syntax:
/// * `PID.5` - Fifth field of first PID segment
/// * `PID.5.1` - First component of fifth field
/// * `PID[2].5` - Fifth field of second PID segment occurrence
/// * `PID.5[1].1` - First component of first repeat of fifth field
///
/// # Arguments
/// * `message` - The HL7 message as a string
/// * `field_path` - Query path (e.g., "PID.5.1", "MSH.9")
///
/// # Returns
/// * `Some(CursorRange)` - Character range of the field if found
/// * `None` - If message parsing fails or field path is invalid/not found
#[tauri::command]
pub fn get_field_range(message: &str, field_path: &str) -> Option<CursorRange> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    let result = message.query(field_path)?;
    let range = result.range();
    Some(CursorRange {
        start: range.start,
        end: range.end,
    })
}

/// Parse a field identifier string into segment, field, and component numbers.
///
/// This helper validates and decomposes field identifiers like "PID.3" or "PID.5.1"
/// into their numeric components for use with the HL7 parser.
///
/// # Format
/// Expected format: `SEGMENT.FIELD[.COMPONENT]`
/// * SEGMENT must match the provided segment parameter
/// * FIELD must be a positive integer (1-based)
/// * COMPONENT (optional) must be a positive integer (1-based)
///
/// # Returns
/// * `Some((field, component))` - Parsed field and optional component numbers
/// * `None` - Invalid format or validation failure
fn parse_field_id(field_id: &str, segment: &str) -> Option<(usize, Option<usize>)> {
    // Split the field_id into segment, field, and component parts
    let parts: Vec<&str> = field_id.split('.').collect();
    if parts.len() < 2 {
        log::warn!("Invalid field_id: {field_id}");
        return None;
    }

    let Ok(segment_name) = parts[0].parse::<String>(); // split always has at least 1 part
    if segment_name != segment {
        log::warn!("Segment name does not match: {segment_name} != {segment}");
        return None;
    }

    let Ok(field) = parts[1].parse::<usize>() else {
        log::warn!("Invalid field number in field_id: {field_id}");
        return None;
    };
    if field < 1 {
        log::warn!("Field number must be greater than 0: {field_id}");
        return None;
    }

    let component = if parts.len() > 2 {
        let Ok(component) = parts[2].parse::<usize>() else {
            log::warn!("Invalid component number in field_id: {field_id}");
            return None;
        };
        if component < 1 {
            log::warn!("Component number must be greater than 0: {field_id}");
            return None;
        }
        Some(component)
    } else {
        None
    };

    Some((field, component))
}
