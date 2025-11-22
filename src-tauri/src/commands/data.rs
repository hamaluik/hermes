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
use rand::distr::{Alphanumeric, SampleString};
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

/// Result of generating a new control ID.
///
/// Contains both the modified message and the range where the control ID was inserted,
/// allowing the frontend to select the new value in the editor.
#[derive(Deserialize, Serialize, Debug)]
pub struct GenerateControlIdResult {
    /// The modified message with the new control ID
    pub message: String,
    /// Character range of the control ID field (MSH.10) for editor selection
    pub range: CursorRange,
}

/// Generate a new control ID and insert it into MSH.10.
///
/// This command generates a random 20-character alphanumeric control ID and
/// replaces the value in MSH.10. The control ID uniquely identifies each message
/// and is used by receiving systems to detect duplicates.
///
/// # Use Case
/// When resending a message or creating a new message from a template, users need
/// to generate a fresh control ID. This tool automates that process without requiring
/// manual ID creation.
///
/// # Arguments
/// * `message` - The HL7 message as a string
///
/// # Returns
/// * `Ok(GenerateControlIdResult)` - Modified message and range of MSH.10
/// * `Err(String)` - If message parsing fails
#[tauri::command]
pub fn generate_control_id(message: &str) -> Result<GenerateControlIdResult, String> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("Failed to parse message: {e}"))?;

    // Generate a 20-character alphanumeric control ID
    let new_control_id = Alphanumeric.sample_string(&mut rand::rng(), 20);

    // Convert to builder and update MSH.10
    let mut builder: MessageBuilder = (&parsed).into();
    let msh = builder
        .segment_named_mut("MSH")
        .ok_or("Message has no MSH segment")?;
    msh.set_field_value(10, &new_control_id);

    // Render the updated message
    let rendered = builder.render_with_newlines().to_string();

    // Find the range of MSH.10 in the rendered message
    let new_parsed = hl7_parser::parse_message_with_lenient_newlines(&rendered)
        .map_err(|e| format!("Failed to parse updated message: {e}"))?;
    let range = new_parsed
        .query("MSH.10")
        .map(|r| r.range())
        .ok_or("Could not find MSH.10 in updated message")?;

    Ok(GenerateControlIdResult {
        message: rendered,
        range: CursorRange {
            start: range.start,
            end: range.end,
        },
    })
}

/// Get the character range of the current navigable cell (field/component) at the cursor.
///
/// This command finds the smallest navigable unit containing the cursor position.
/// A "cell" is the smallest unit that should be treated as a single navigation target:
/// - If a field has no repeats, the entire field is a cell
/// - If a field has repeats but a repeat has no components, each repeat is a cell
/// - If a repeat has components but no subcomponents, each component is a cell
/// - If a component has subcomponents, each subcomponent is a cell
///
/// # Use Case
/// Used by the "Insert Timestamp" feature to determine what text to replace when
/// inserting a timestamp at the current cursor position.
///
/// # Arguments
/// * `message` - The HL7 message as a string
/// * `cursor` - Current cursor position (character offset)
///
/// # Returns
/// * `Some(CursorRange)` - Range of the cell containing the cursor
/// * `None` - If cursor is not within a valid cell (e.g., on segment name, between segments)
#[tauri::command]
pub fn get_current_cell_range(message: &str, cursor: usize) -> Option<CursorRange> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;

    // Flatten message into navigable cells and find the one containing the cursor
    for segment in message.segments() {
        // Skip segment name - we don't want to replace segment identifiers
        let segment_name_end = segment.range.start + segment.name.len();
        if cursor >= segment.range.start && cursor <= segment_name_end {
            return None; // Cursor is on segment name, not a valid cell for replacement
        }

        for field in segment.fields() {
            if field.repeats.is_empty() {
                if cursor >= field.range.start && cursor <= field.range.end {
                    return Some(CursorRange {
                        start: field.range.start,
                        end: field.range.end,
                    });
                }
                continue;
            }

            for repeat in field.repeats.iter() {
                if repeat.components.is_empty() {
                    if cursor >= repeat.range.start && cursor <= repeat.range.end {
                        return Some(CursorRange {
                            start: repeat.range.start,
                            end: repeat.range.end,
                        });
                    }
                    continue;
                }

                for component in repeat.components.iter() {
                    if component.subcomponents.is_empty() {
                        if cursor >= component.range.start && cursor <= component.range.end {
                            return Some(CursorRange {
                                start: component.range.start,
                                end: component.range.end,
                            });
                        }
                        continue;
                    }

                    for subcomponent in component.subcomponents.iter() {
                        if cursor >= subcomponent.range.start && cursor <= subcomponent.range.end {
                            return Some(CursorRange {
                                start: subcomponent.range.start,
                                end: subcomponent.range.end,
                            });
                        }
                    }
                }
            }
        }
    }

    None
}

/// Generate a current timestamp in HL7 format.
///
/// Creates an HL7-formatted timestamp (DTM data type) for the current time
/// in the local timezone. The format is YYYYMMDDHHmmss with optional UTC offset.
///
/// # Arguments
/// * `include_offset` - Whether to include the UTC offset (e.g., -0500)
///
/// # Returns
/// The formatted timestamp string (e.g., "20250115143000" or "20250115143000-0500")
#[tauri::command]
pub fn get_current_hl7_timestamp(include_offset: bool) -> String {
    let now = jiff::Zoned::now();
    format_hl7_timestamp(&now, include_offset)
}

/// Format a datetime string into HL7 timestamp format.
///
/// Parses an ISO 8601 datetime string and converts it to HL7 DTM format.
/// Handles both offset-aware and naive datetime inputs.
///
/// # Why Manual Offset Parsing?
///
/// The frontend sends datetime strings like `2025-01-15T14:30:00-05:00`. However,
/// jiff's `Zoned` type requires a timezone annotation in brackets (e.g., `[America/New_York]`),
/// not just an offset. When only an offset is provided, `Zoned::parse()` fails.
///
/// To preserve the user's selected offset (rather than falling back to local timezone),
/// we manually split the datetime string at the offset position and parse each part
/// separately. This ensures that selecting "UTC-08:00 (Pacific)" in the UI actually
/// generates a timestamp with `-0800`, not the system's local offset.
///
/// # Parsing Strategy
///
/// 1. Try parsing as `jiff::Zoned` (handles strings with timezone annotation)
/// 2. Look for an offset at position 19+ (after `YYYY-MM-DDTHH:MM:SS`)
/// 3. If offset found, split and parse datetime + offset separately
/// 4. Otherwise, parse as naive datetime and use local timezone
///
/// # Arguments
/// * `datetime` - ISO 8601 formatted datetime string (e.g., "2025-01-15T14:30:00-05:00")
/// * `include_offset` - Whether to include the UTC offset in the output
///
/// # Returns
/// * `Ok(String)` - The formatted HL7 timestamp
/// * `Err(String)` - If the datetime string cannot be parsed
#[tauri::command]
pub fn format_datetime_to_hl7(datetime: &str, include_offset: bool) -> Result<String, String> {
    // Try to parse as a Zoned datetime (with timezone annotation like [America/New_York])
    if let Ok(zoned) = datetime.parse::<jiff::Zoned>() {
        return Ok(format_hl7_timestamp_from_zoned(&zoned, include_offset));
    }

    // Check if the string contains an offset by looking for + or - after the time portion
    // This handles strings like "2025-01-15T14:30:00-05:00"
    if let Some(offset_start) = datetime.rfind(|c| c == '+' || c == '-') {
        // Make sure it's after the time portion (not a negative year or part of date)
        // The 'T' separator is at position 10, so offset should be after position 18 (after seconds)
        if offset_start >= 19 {
            let datetime_part = &datetime[..offset_start];
            let offset_str = &datetime[offset_start..];

            if let Ok(civil_dt) = datetime_part.parse::<jiff::civil::DateTime>() {
                if let Ok(offset) = parse_offset(offset_str) {
                    return Ok(format_hl7_timestamp_from_parts(&civil_dt, offset, include_offset));
                }
            }
        }
    }

    // Try to parse as a civil datetime (no offset) and assume local timezone
    if let Ok(civil) = datetime.parse::<jiff::civil::DateTime>() {
        let zoned = civil
            .to_zoned(jiff::tz::TimeZone::system())
            .map_err(|e| format!("Failed to convert to local time: {e}"))?;
        return Ok(format_hl7_timestamp_from_zoned(&zoned, include_offset));
    }

    Err(format!("Could not parse datetime: {datetime}"))
}

/// Parse an offset string like "-05:00" or "+05:30" into a jiff::tz::Offset.
///
/// This is needed because jiff doesn't provide a direct way to parse ISO 8601
/// offset strings without a full datetime. The offset is extracted from the
/// user's timezone selection in the Insert Timestamp modal.
///
/// # Format
/// Expects `±HH:MM` format (e.g., "-05:00", "+05:30", "+00:00")
fn parse_offset(offset_str: &str) -> Result<jiff::tz::Offset, String> {
    let sign = if offset_str.starts_with('-') { -1 } else { 1 };
    let rest = offset_str.trim_start_matches(['+', '-']);

    let parts: Vec<&str> = rest.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid offset format: {offset_str}"));
    }

    let hours: i8 = parts[0].parse().map_err(|_| format!("Invalid hours in offset: {offset_str}"))?;
    let minutes: i8 = parts[1].parse().map_err(|_| format!("Invalid minutes in offset: {offset_str}"))?;

    let total_seconds = sign * (hours as i32 * 3600 + minutes as i32 * 60);
    jiff::tz::Offset::from_seconds(total_seconds)
        .map_err(|e| format!("Invalid offset value: {e}"))
}

/// Format a Zoned datetime into HL7 DTM format.
///
/// HL7 DTM format: YYYYMMDDHHmmss[±ZZZZ]
/// - YYYY: 4-digit year
/// - MM: 2-digit month (01-12)
/// - DD: 2-digit day (01-31)
/// - HH: 2-digit hour (00-23)
/// - mm: 2-digit minute (00-59)
/// - ss: 2-digit second (00-59)
/// - ±ZZZZ: UTC offset as ±HHmm (optional)
fn format_hl7_timestamp(dt: &jiff::Zoned, include_offset: bool) -> String {
    format_hl7_timestamp_from_zoned(dt, include_offset)
}

/// Format a Zoned datetime into HL7 DTM format.
fn format_hl7_timestamp_from_zoned(dt: &jiff::Zoned, include_offset: bool) -> String {
    let datetime = dt.datetime();
    let base = format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        datetime.year(),
        datetime.month(),
        datetime.day(),
        datetime.hour(),
        datetime.minute(),
        datetime.second()
    );

    if include_offset {
        let offset = dt.offset();
        format_hl7_offset(&base, offset)
    } else {
        base
    }
}

/// Format a civil DateTime with an explicit offset into HL7 DTM format.
fn format_hl7_timestamp_from_parts(
    dt: &jiff::civil::DateTime,
    offset: jiff::tz::Offset,
    include_offset: bool,
) -> String {
    let base = format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    );

    if include_offset {
        format_hl7_offset(&base, offset)
    } else {
        base
    }
}

/// Append an HL7-formatted offset to a base timestamp string.
fn format_hl7_offset(base: &str, offset: jiff::tz::Offset) -> String {
    let total_seconds = offset.seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds.abs() % 3600) / 60;
    let sign = if total_seconds >= 0 { '+' } else { '-' };
    format!("{}{}{:02}{:02}", base, sign, hours.abs(), minutes)
}

/// Generate an HL7 message from a template.
///
/// Creates an HL7 message for the specified message type by looking up
/// the required segments in the schema and populating fields with template values.
///
/// # Template Names
/// Template names correspond to message type keys in messages.toml:
/// - `adt_a01`, `adt_a02`, ..., `adt_a50` for ADT messages
/// - `orm_o01` for Order messages
/// - `oru_r01` for Observation Result messages
/// - `orr_o02` for Order Response messages
/// - `dft_p03` for Financial Transaction messages
///
/// # Message Structure
/// The generated message includes:
/// - MSH segment with message type/trigger event pre-filled
/// - All segments defined in the schema for that message type
/// - Fields populated with template values from segment schemas
///
/// # Template Values
/// Each field in the segment schema can have a `template` value. Special values:
/// - `{auto}` - Placeholder for dynamic values (timestamps, control IDs) expanded at send time
/// - Empty string - Field left blank
/// - Any other value - Used directly
///
/// # Arguments
/// * `template_name` - Template identifier (e.g., "adt_a01", "orm_o01")
/// * `state` - Application state containing the schema cache
///
/// # Returns
/// * `Ok(String)` - The generated HL7 message
/// * `Err(String)` - If template not found or schema loading fails
#[tauri::command]
pub fn generate_template_message(
    template_name: &str,
    state: State<'_, AppData>,
) -> Result<String, String> {
    let schema = state
        .schema
        .get_messages()
        .map_err(|e| format!("Failed to load messages schema: {e:#}"))?;

    let segments = schema
        .message
        .get(template_name)
        .ok_or_else(|| format!("Template '{template_name}' not found in schema"))?;

    // Parse template name to extract message type and trigger event
    // e.g., "adt_a01" -> ("ADT", "A01")
    let parts: Vec<&str> = template_name.split('_').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid template name format: {template_name}"));
    }
    let message_type = parts[0].to_uppercase();
    let trigger_event = parts[1].to_uppercase();

    // Build the message using hl7_parser's MessageBuilder
    let mut builder = MessageBuilder::default();

    // Add each segment from the schema
    for segment_meta in segments.iter() {
        let segment_name = &segment_meta.name;
        let mut seg = SegmentBuilder::new(segment_name);

        // Load the segment schema to get field definitions with template values
        let segment_fields = state.schema.get_segment(segment_name).ok();

        // Determine the max field number
        let max_field = segment_fields
            .as_ref()
            .and_then(|fields| fields.iter().map(|f| f.field).max())
            .unwrap_or(1) as usize;

        // Group fields by field number to handle multi-component fields
        let fields_by_number: HashMap<u8, Vec<&crate::schema::segment::Field>> =
            if let Some(ref fields) = segment_fields {
                let mut grouped: HashMap<u8, Vec<&crate::schema::segment::Field>> = HashMap::new();
                for field in fields.iter() {
                    grouped.entry(field.field).or_default().push(field);
                }
                grouped
            } else {
                HashMap::new()
            };

        // Special handling for MSH.9 (message type/trigger event)
        if segment_name == "MSH" {
            seg.set_field(9, {
                let mut field = FieldBuilder::default();
                field.set_component(1, &message_type);
                field.set_component(2, &trigger_event);
                field
            });
        }

        // Special handling for EVN.1 (event type code from trigger)
        if segment_name == "EVN" {
            seg.set_field_value(1, &trigger_event);
        }

        // Apply template values for all fields
        for field_num in 1..=max_field {
            let field_num_u8 = field_num as u8;

            // Skip MSH.9 (already set) and EVN.1 (already set)
            if (segment_name == "MSH" && field_num == 9)
                || (segment_name == "EVN" && field_num == 1)
            {
                continue;
            }

            if let Some(field_defs) = fields_by_number.get(&field_num_u8) {
                // Check if any field definitions have components
                let has_components = field_defs.iter().any(|f| f.component.is_some());

                if has_components {
                    // Multi-component field: use FieldBuilder
                    let mut field_builder = FieldBuilder::default();
                    let mut has_any_value = false;

                    for field_def in field_defs {
                        if let Some(component_num) = field_def.component {
                            if let Some(ref template) = field_def.template {
                                field_builder.set_component(component_num as usize, template);
                                has_any_value = true;
                            }
                        }
                    }

                    if has_any_value {
                        seg.set_field(field_num, field_builder);
                    } else if !seg.has_field(field_num) {
                        seg.set_field_value(field_num, "");
                    }
                } else {
                    // Single field (no components)
                    let field_def = field_defs.first();
                    if let Some(template) = field_def.and_then(|f| f.template.as_ref()) {
                        seg.set_field_value(field_num, template);
                    } else if !seg.has_field(field_num) {
                        seg.set_field_value(field_num, "");
                    }
                }
            } else {
                // No schema definition for this field, ensure it exists as empty
                if !seg.has_field(field_num) {
                    seg.set_field_value(field_num, "");
                }
            }
        }

        builder.push_segment(seg);
    }

    Ok(builder.render_with_newlines().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_datetime_with_explicit_negative_offset() {
        let result = format_datetime_to_hl7("2025-01-15T14:30:00-05:00", true).unwrap();
        assert_eq!(result, "20250115143000-0500");
    }

    #[test]
    fn format_datetime_with_explicit_positive_offset() {
        let result = format_datetime_to_hl7("2025-01-15T14:30:00+05:30", true).unwrap();
        assert_eq!(result, "20250115143000+0530");
    }

    #[test]
    fn format_datetime_with_explicit_offset_no_include() {
        let result = format_datetime_to_hl7("2025-01-15T14:30:00-05:00", false).unwrap();
        assert_eq!(result, "20250115143000");
    }

    #[test]
    fn format_datetime_without_offset_uses_local() {
        // This will use local timezone, so we can only check the base part
        let result = format_datetime_to_hl7("2025-01-15T14:30:00", false).unwrap();
        assert_eq!(result, "20250115143000");
    }

    #[test]
    fn parse_offset_negative() {
        let offset = parse_offset("-05:00").unwrap();
        assert_eq!(offset.seconds(), -5 * 3600);
    }

    #[test]
    fn parse_offset_positive() {
        let offset = parse_offset("+05:30").unwrap();
        assert_eq!(offset.seconds(), 5 * 3600 + 30 * 60);
    }

    #[test]
    fn parse_offset_utc() {
        let offset = parse_offset("+00:00").unwrap();
        assert_eq!(offset.seconds(), 0);
    }
}
