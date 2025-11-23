//! Message export commands for converting HL7 messages to various formats.
//!
//! Exports messages to JSON, YAML, and TOML formats as a natural tree structure
//! optimised for human readability and downstream processing. The output omits
//! internal parser metadata (byte ranges, source references) that would clutter
//! the exported data.
//!
//! # Output Structure
//!
//! The export produces a hierarchical representation where:
//!
//! - **Segments** become object keys. When a segment appears multiple times
//!   (e.g., multiple OBX segments), the value becomes an array of field objects.
//! - **Fields** use 1-based string indices as keys. Empty fields are omitted.
//! - **Components** use 1-based string indices. Simple fields (single component,
//!   no subcomponents) are represented as plain strings.
//! - **Subcomponents** follow the same pattern as components.
//! - **Field repetitions** (separated by `~`) become arrays.
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
//!   "MSH": {
//!     "1": "^~\\&",
//!     "2": "APP",
//!     "3": "FAC",
//!     "6": "20231215",
//!     "8": { "1": "ADT", "2": "A01" },
//!     "9": "123",
//!     "10": "P",
//!     "11": "2.5.1"
//!   },
//!   "PID": {
//!     "3": "12345",
//!     "5": { "1": "DOE", "2": "JOHN" }
//!   },
//!   "OBX": [
//!     { "1": "1", "2": "ST", "3": "CODE", "5": "Value1" },
//!     { "1": "2", "2": "NM", "3": "CODE2", "5": "42" }
//!   ]
//! }
//! ```

use hl7_parser::message::{Component, Field, Message, Repeat, Segment};
use indexmap::IndexMap;
use serde_json::Value;

/// Converts a parsed HL7 message to a natural tree structure.
///
/// The structure uses:
/// - Segment names as keys (repeated segments become arrays)
/// - Field indices as string keys (1-based, only non-empty fields)
/// - Component indices as string keys (only non-empty components)
/// - Field repetitions as arrays
fn message_to_tree(message: &Message) -> IndexMap<String, Value> {
    let mut result: IndexMap<String, Value> = IndexMap::new();

    for segment in message.segments() {
        let segment_name = segment.name.to_string();
        let fields_obj = fields_to_value(segment);

        // check if this segment name already exists
        match result.get_mut(&segment_name) {
            Some(Value::Array(arr)) => {
                arr.push(fields_obj);
            }
            Some(existing) => {
                // first duplicate - convert existing to array
                let prev = existing.clone();
                *existing = Value::Array(vec![prev, fields_obj]);
            }
            None => {
                result.insert(segment_name, fields_obj);
            }
        }
    }

    result
}

/// Converts segment fields to a JSON object with field indices as keys.
fn fields_to_value(segment: &Segment) -> Value {
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

    Value::Object(fields.into_iter().collect())
}

/// Converts a field to a JSON value, handling repetitions.
fn field_to_value(field: &Field) -> Value {
    let repeats: Vec<Value> = field.repeats.iter().map(repeat_to_value).collect();

    match repeats.len() {
        0 => Value::Null,
        1 => repeats.into_iter().next().unwrap(),
        _ => Value::Array(repeats),
    }
}

/// Converts a field repetition to a JSON value.
fn repeat_to_value(repeat: &Repeat) -> Value {
    let components: Vec<Value> = repeat.components.iter().map(component_to_value).collect();

    // if single component with no subcomponents, return as string
    if components.len() == 1 {
        return components.into_iter().next().unwrap();
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

    match subcomponents.len() {
        0 => Value::Null,
        1 => {
            let val = subcomponents[0];
            if val.is_empty() {
                Value::Null
            } else {
                Value::String(val.to_string())
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
        _ => false,
    }
}

/// Exports an HL7 message to JSON format.
#[tauri::command]
pub fn export_to_json(message: &str) -> Result<String, String> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("Failed to parse message: {e}"))?;
    let tree = message_to_tree(&parsed);
    serde_json::to_string_pretty(&tree).map_err(|e| format!("Failed to serialise to JSON: {e}"))
}

/// Exports an HL7 message to YAML format.
#[tauri::command]
pub fn export_to_yaml(message: &str) -> Result<String, String> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("Failed to parse message: {e}"))?;
    let tree = message_to_tree(&parsed);
    serde_yml::to_string(&tree).map_err(|e| format!("Failed to serialise to YAML: {e}"))
}

/// Exports an HL7 message to TOML format.
#[tauri::command]
pub fn export_to_toml(message: &str) -> Result<String, String> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("Failed to parse message: {e}"))?;
    let tree = message_to_tree(&parsed);
    toml::to_string_pretty(&tree).map_err(|e| format!("Failed to serialise to TOML: {e}"))
}
