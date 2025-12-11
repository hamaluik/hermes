//! Message export commands for converting HL7 messages to various formats.
//!
//! Exports messages to JSON, YAML, and TOML formats as an ordered array of
//! segments, preserving segment order explicitly. The output omits internal
//! parser metadata (byte ranges, source references) that would clutter the
//! exported data.
//!
//! # Why Export to These Formats?
//!
//! - **Version control**: JSON/YAML diffs are more readable than pipe-delimited HL7
//! - **External tools**: Other systems can consume structured data more easily
//! - **Programmatic editing**: Modify specific fields without parsing HL7 syntax
//!
//! # Output Structure
//!
//! The export produces an array of segment objects wrapped in a root object:
//!
//! - **Segments** are array elements with `segment` name and `fields` object.
//!   Each segment occurrence is a separate entry (no grouping of repeated segments).
//! - **Fields** use 1-based string indices as keys. Empty fields are omitted.
//! - **Components** use 1-based string indices. Simple fields (single component,
//!   no subcomponents) are represented as plain strings.
//! - **Subcomponents** follow the same pattern as components.
//! - **Field repetitions** (separated by `~`) become arrays.
//!
//! # MSH Field Numbering
//!
//! For MSH, the hl7-parser library includes the field separator as the first field:
//! - Index "1" = field separator (`|`)
//! - Index "2" = encoding characters (`^~\&`)
//! - Index "3" = sending application
//! - etc.
//!
//! This differs from how HL7 documentation numbers MSH fields (where MSH.1 is
//! traditionally considered implicit). The import module handles this by skipping
//! field "1" during reconstruction.
//!
//! # Example
//!
//! Given an HL7 message like:
//! ```text
//! MSH|^~\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1
//! PID|||12345||DOE^JOHN
//! OBX|1|ST|CODE||Value1
//! OBX|2|NM|CODE2||42
//! ```
//!
//! The JSON export produces:
//! ```json
//! {
//!   "segments": [
//!     {
//!       "segment": "MSH",
//!       "fields": {
//!         "1": "|",
//!         "2": "^~\\&",
//!         "3": "APP",
//!         "4": "FAC",
//!         "7": "20231215",
//!         "9": { "1": "ADT", "2": "A01" },
//!         "10": "123",
//!         "11": "P",
//!         "12": "2.5.1"
//!       }
//!     },
//!     {
//!       "segment": "PID",
//!       "fields": {
//!         "3": "12345",
//!         "5": { "1": "DOE", "2": "JOHN" }
//!       }
//!     },
//!     {
//!       "segment": "OBX",
//!       "fields": { "1": "1", "2": "ST", "3": "CODE", "5": "Value1" }
//!     },
//!     {
//!       "segment": "OBX",
//!       "fields": { "1": "2", "2": "NM", "3": "CODE2", "5": "42" }
//!     }
//!   ]
//! }
//! ```

use hl7_parser::message::{Component, Field, Message, Repeat, Segment};
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;

/// A single segment in the export format.
#[derive(Serialize)]
struct SegmentExport {
    segment: String,
    fields: IndexMap<String, Value>,
}

/// Root structure for exported messages.
#[derive(Serialize)]
struct MessageExport {
    segments: Vec<SegmentExport>,
}

/// Converts a parsed HL7 message to the export structure.
///
/// Each segment becomes a separate entry in the segments array,
/// preserving order explicitly.
fn message_to_export(message: &Message) -> MessageExport {
    let segments = message
        .segments()
        .map(|segment| SegmentExport {
            segment: segment.name.to_string(),
            fields: fields_to_map(segment),
        })
        .collect();

    MessageExport { segments }
}

/// Converts segment fields to an IndexMap with field indices as keys.
fn fields_to_map(segment: &Segment) -> IndexMap<String, Value> {
    let mut fields: IndexMap<String, Value> = IndexMap::new();

    for (idx, field) in segment.fields.iter().enumerate() {
        // field index is 1-based in HL7, but MSH.1 is the field separator
        // hl7_parser returns fields starting from MSH.2 (encoding chars)
        let field_idx = idx + 1;

        let value = field_to_value(field);
        if !is_empty_value(&value) {
            fields.insert(field_idx.to_string(), value);
        }
    }

    fields
}

/// Converts a field to a JSON value, handling repetitions.
fn field_to_value(field: &Field) -> Value {
    let repeats: Vec<Value> = field.repeats.iter().map(repeat_to_value).collect();

    match repeats.len() {
        0 => Value::Null,
        1 => repeats.into_iter().next().expect("can get single repeat"),
        _ => Value::Array(repeats),
    }
}

/// Converts a field repetition to a JSON value.
fn repeat_to_value(repeat: &Repeat) -> Value {
    let components: Vec<Value> = repeat.components.iter().map(component_to_value).collect();

    // if single component with no subcomponents, return as string
    if components.len() == 1 {
        return components
            .into_iter()
            .next()
            .expect("can get single component");
    }

    // build object with only non-empty components
    let mut obj: IndexMap<String, Value> = IndexMap::new();
    for (idx, comp) in components.into_iter().enumerate() {
        if !is_empty_value(&comp) {
            obj.insert((idx + 1).to_string(), comp);
        }
    }

    if obj.is_empty() {
        Value::Null
    } else {
        Value::Object(obj.into_iter().collect())
    }
}

/// Converts a component to a JSON value, handling subcomponents.
fn component_to_value(component: &Component) -> Value {
    let subcomponents: Vec<&str> = component.subcomponents.iter().map(|s| s.value).collect();

    match subcomponents.as_slice() {
        [] => Value::Null,
        [val] => {
            if val.is_empty() {
                Value::Null
            } else {
                Value::String((*val).to_string())
            }
        }
        _ => {
            // multiple subcomponents - build object with indices
            let mut obj: IndexMap<String, Value> = IndexMap::new();
            for (idx, sub) in subcomponents.iter().enumerate() {
                if !sub.is_empty() {
                    obj.insert((idx + 1).to_string(), Value::String((*sub).to_string()));
                }
            }
            if obj.is_empty() {
                Value::Null
            } else {
                Value::Object(obj.into_iter().collect())
            }
        }
    }
}

/// Checks if a JSON value is considered empty (null or empty object/array/string).
fn is_empty_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Array(arr) => arr.is_empty(),
        Value::Object(obj) => obj.is_empty(),
        Value::Bool(_) | Value::Number(_) => false,
    }
}

/// Exports an HL7 message to JSON format.
#[tauri::command]
pub fn export_to_json(message: &str) -> Result<String, String> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("Failed to parse message: {e}"))?;
    let export = message_to_export(&parsed);
    serde_json::to_string_pretty(&export).map_err(|e| format!("Failed to serialise to JSON: {e}"))
}

/// Exports an HL7 message to YAML format.
#[tauri::command]
pub fn export_to_yaml(message: &str) -> Result<String, String> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("Failed to parse message: {e}"))?;
    let export = message_to_export(&parsed);
    serde_yml::to_string(&export).map_err(|e| format!("Failed to serialise to YAML: {e}"))
}

/// Exports an HL7 message to TOML format.
#[tauri::command]
pub fn export_to_toml(message: &str) -> Result<String, String> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("Failed to parse message: {e}"))?;
    let export = message_to_export(&parsed);
    toml::to_string_pretty(&export).map_err(|e| format!("Failed to serialise to TOML: {e}"))
}
