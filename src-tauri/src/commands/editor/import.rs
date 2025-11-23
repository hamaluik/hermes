//! Message import commands for converting various formats to HL7 messages.
//!
//! Imports messages from JSON, YAML, and TOML formats, reversing the tree
//! structure produced by the export module back into pipe-delimited HL7 format.
//!
//! # Why This Module Exists
//!
//! While `.hl7` files are the native format, JSON/YAML/TOML exports are useful
//! for version control diffing, external tool integration, and programmatic
//! message generation. This module enables round-tripping: messages can be
//! exported, modified externally, and re-imported without data loss.
//!
//! # Expected Input Structure
//!
//! The input should match the structure produced by the export commands:
//!
//! - **Segments** are object keys. Arrays represent repeated segments.
//! - **Fields** use 1-based string indices as keys.
//! - **Components** use 1-based string indices for multi-component fields.
//! - **Subcomponents** follow the same pattern as components.
//! - **Field repetitions** are arrays within field values.
//!
//! # MSH Field Numbering Caveat
//!
//! The export module stores MSH.1 (the field separator `|`) at index "1" and
//! MSH.2 (encoding characters `^~\&`) at index "2". This differs from how
//! HL7 traditionally describes MSH where MSH.1 is implicit. During import,
//! field "1" is skipped since the separator is reconstructed by joining fields.
//!
//! # Example
//!
//! Given JSON input:
//! ```json
//! {
//!   "MSH": {
//!     "1": "|",
//!     "2": "^~\\&",
//!     "3": "APP",
//!     "4": "FAC",
//!     "7": "20231215",
//!     "9": { "1": "ADT", "2": "A01" },
//!     "10": "123",
//!     "11": "P",
//!     "12": "2.5.1"
//!   },
//!   "PID": {
//!     "3": "12345",
//!     "5": { "1": "DOE", "2": "JOHN" }
//!   }
//! }
//! ```
//!
//! The import produces:
//! ```text
//! MSH|^~\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1
//! PID|||12345||DOE^JOHN
//! ```

use indexmap::IndexMap;
use serde_json::Value;

/// Default HL7 delimiters.
const DEFAULT_FIELD_SEP: char = '|';
const DEFAULT_COMPONENT_SEP: char = '^';
const DEFAULT_REPETITION_SEP: char = '~';
const DEFAULT_SUBCOMPONENT_SEP: char = '&';

/// HL7 delimiter configuration extracted from MSH segment.
#[derive(Debug, Clone)]
struct Delimiters {
    field: char,
    component: char,
    repetition: char,
    subcomponent: char,
}

impl Default for Delimiters {
    fn default() -> Self {
        Self {
            field: DEFAULT_FIELD_SEP,
            component: DEFAULT_COMPONENT_SEP,
            repetition: DEFAULT_REPETITION_SEP,
            subcomponent: DEFAULT_SUBCOMPONENT_SEP,
        }
    }
}

impl Delimiters {
    /// Parses delimiters from MSH.2 encoding characters string.
    fn from_encoding_chars(encoding_chars: &str) -> Self {
        let chars: Vec<char> = encoding_chars.chars().collect();
        Self {
            field: DEFAULT_FIELD_SEP,
            component: chars.first().copied().unwrap_or(DEFAULT_COMPONENT_SEP),
            repetition: chars.get(1).copied().unwrap_or(DEFAULT_REPETITION_SEP),
            // chars[2] is escape char, chars[3] is subcomponent
            subcomponent: chars.get(3).copied().unwrap_or(DEFAULT_SUBCOMPONENT_SEP),
        }
    }
}

/// Converts a tree structure back to an HL7 message.
fn tree_to_message(tree: &IndexMap<String, Value>) -> Result<String, String> {
    // extract delimiters from MSH if present
    let delimiters = extract_delimiters(tree);

    let mut segments: Vec<String> = Vec::new();

    for (segment_name, value) in tree {
        match value {
            Value::Array(arr) => {
                // repeated segments
                for segment_value in arr {
                    let segment_line =
                        build_segment_line(segment_name, segment_value, &delimiters)?;
                    segments.push(segment_line);
                }
            }
            Value::Object(_) => {
                let segment_line = build_segment_line(segment_name, value, &delimiters)?;
                segments.push(segment_line);
            }
            _ => {
                return Err(format!(
                    "Invalid segment value for '{segment_name}': expected object or array"
                ));
            }
        }
    }

    Ok(segments.join("\r"))
}

/// Extracts delimiter configuration from MSH segment if present.
fn extract_delimiters(tree: &IndexMap<String, Value>) -> Delimiters {
    let Some(msh) = tree.get("MSH") else {
        return Delimiters::default();
    };

    // handle both single MSH and array of MSH (though array is unusual)
    let msh_obj = match msh {
        Value::Object(obj) => obj,
        Value::Array(arr) => match arr.first() {
            Some(Value::Object(obj)) => obj,
            _ => return Delimiters::default(),
        },
        _ => return Delimiters::default(),
    };

    // MSH.1 is the field separator (|), MSH.2 contains encoding characters (^~\&)
    // The export stores MSH.1 at "1" and MSH.2 at "2"
    match msh_obj.get("2") {
        Some(Value::String(encoding_chars)) => Delimiters::from_encoding_chars(encoding_chars),
        _ => Delimiters::default(),
    }
}

/// Builds a single segment line from segment name and fields object.
fn build_segment_line(
    segment_name: &str,
    fields_value: &Value,
    delimiters: &Delimiters,
) -> Result<String, String> {
    let fields_obj = match fields_value {
        Value::Object(obj) => obj,
        _ => {
            return Err(format!(
                "Invalid fields value for segment '{segment_name}': expected object"
            ));
        }
    };

    // find the maximum field index
    let max_field_idx = fields_obj
        .keys()
        .filter_map(|k| k.parse::<usize>().ok())
        .max()
        .unwrap_or(0);

    if segment_name == "MSH" {
        // MSH is special: MSH.1 is the field separator (|), MSH.2 is encoding chars
        build_msh_segment(fields_obj, max_field_idx, delimiters)
    } else {
        build_regular_segment(segment_name, fields_obj, max_field_idx, delimiters)
    }
}

/// Builds an MSH segment line.
///
/// MSH is unique because MSH.1 (the field separator) is implicit in the output.
/// The export stores MSH.1 as "|" at index "1" and MSH.2 (encoding chars) at "2".
/// We skip field "1" since the field separator is implicitly added by joining.
fn build_msh_segment(
    fields_obj: &serde_json::Map<String, Value>,
    max_field_idx: usize,
    delimiters: &Delimiters,
) -> Result<String, String> {
    let mut parts: Vec<String> = vec!["MSH".to_string()];

    // skip field "1" (field separator "|"), start from field "2" (encoding chars)
    for idx in 2..=max_field_idx {
        let field_value = fields_obj.get(&idx.to_string());
        let field_str = match field_value {
            Some(v) => value_to_field(v, delimiters)?,
            None => String::new(),
        };
        parts.push(field_str);
    }

    Ok(parts.join(&delimiters.field.to_string()))
}

/// Builds a regular (non-MSH) segment line.
fn build_regular_segment(
    segment_name: &str,
    fields_obj: &serde_json::Map<String, Value>,
    max_field_idx: usize,
    delimiters: &Delimiters,
) -> Result<String, String> {
    let mut parts: Vec<String> = vec![segment_name.to_string()];

    for idx in 1..=max_field_idx {
        let field_value = fields_obj.get(&idx.to_string());
        let field_str = match field_value {
            Some(v) => value_to_field(v, delimiters)?,
            None => String::new(),
        };
        parts.push(field_str);
    }

    Ok(parts.join(&delimiters.field.to_string()))
}

/// Converts a JSON value to a field string, handling repetitions.
fn value_to_field(value: &Value, delimiters: &Delimiters) -> Result<String, String> {
    match value {
        Value::Null => Ok(String::new()),
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Array(arr) => {
            // field repetitions
            let parts: Result<Vec<String>, String> =
                arr.iter().map(|v| value_to_repeat(v, delimiters)).collect();
            Ok(parts?.join(&delimiters.repetition.to_string()))
        }
        Value::Object(_) => {
            // single repeat with components
            value_to_repeat(value, delimiters)
        }
    }
}

/// Converts a JSON value to a single field repetition (components).
fn value_to_repeat(value: &Value, delimiters: &Delimiters) -> Result<String, String> {
    match value {
        Value::Null => Ok(String::new()),
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Object(obj) => {
            // multiple components
            let max_comp_idx = obj
                .keys()
                .filter_map(|k| k.parse::<usize>().ok())
                .max()
                .unwrap_or(0);

            let mut parts: Vec<String> = Vec::new();
            for idx in 1..=max_comp_idx {
                let comp_value = obj.get(&idx.to_string());
                let comp_str = match comp_value {
                    Some(v) => value_to_component(v, delimiters)?,
                    None => String::new(),
                };
                parts.push(comp_str);
            }
            Ok(parts.join(&delimiters.component.to_string()))
        }
        Value::Array(_) => Err("Unexpected array in repeat position".to_string()),
    }
}

/// Converts a JSON value to a component string, handling subcomponents.
fn value_to_component(value: &Value, delimiters: &Delimiters) -> Result<String, String> {
    match value {
        Value::Null => Ok(String::new()),
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Object(obj) => {
            // multiple subcomponents
            let max_sub_idx = obj
                .keys()
                .filter_map(|k| k.parse::<usize>().ok())
                .max()
                .unwrap_or(0);

            let mut parts: Vec<String> = Vec::new();
            for idx in 1..=max_sub_idx {
                let sub_value = obj.get(&idx.to_string());
                let sub_str = match sub_value {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Bool(b)) => b.to_string(),
                    Some(Value::Null) | None => String::new(),
                    Some(other) => {
                        return Err(format!(
                            "Unexpected value type in subcomponent position: {other:?}"
                        ));
                    }
                };
                parts.push(sub_str);
            }
            Ok(parts.join(&delimiters.subcomponent.to_string()))
        }
        Value::Array(_) => Err("Unexpected array in component position".to_string()),
    }
}

/// Imports an HL7 message from JSON format.
#[tauri::command]
pub fn import_from_json(content: &str) -> Result<String, String> {
    let tree: IndexMap<String, Value> =
        serde_json::from_str(content).map_err(|e| format!("Failed to parse JSON: {e}"))?;
    tree_to_message(&tree)
}

/// Imports an HL7 message from YAML format.
#[tauri::command]
pub fn import_from_yaml(content: &str) -> Result<String, String> {
    let tree: IndexMap<String, Value> =
        serde_yml::from_str(content).map_err(|e| format!("Failed to parse YAML: {e}"))?;
    tree_to_message(&tree)
}

/// Imports an HL7 message from TOML format.
#[tauri::command]
pub fn import_from_toml(content: &str) -> Result<String, String> {
    let tree: IndexMap<String, Value> =
        toml::from_str(content).map_err(|e| format!("Failed to parse TOML: {e}"))?;
    tree_to_message(&tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::editor::export::{export_to_json, export_to_toml, export_to_yaml};

    /// Helper to normalise messages for comparison.
    /// Trims trailing empty fields from each segment.
    fn normalise_message(msg: &str) -> String {
        msg.lines()
            .map(|line| {
                let trimmed = line.trim_end_matches('|');
                trimmed.to_string()
            })
            .collect::<Vec<_>>()
            .join("\r")
    }

    #[test]
    fn roundtrip_simple_message_json() {
        let original = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let json = export_to_json(original).expect("can export to JSON");
        let imported = import_from_json(&json).expect("can import from JSON");
        assert_eq!(normalise_message(original), normalise_message(&imported));
    }

    #[test]
    fn roundtrip_simple_message_yaml() {
        let original = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let yaml = export_to_yaml(original).expect("can export to YAML");
        let imported = import_from_yaml(&yaml).expect("can import from YAML");
        assert_eq!(normalise_message(original), normalise_message(&imported));
    }

    #[test]
    fn roundtrip_simple_message_toml() {
        let original = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let toml = export_to_toml(original).expect("can export to TOML");
        let imported = import_from_toml(&toml).expect("can import from TOML");
        assert_eq!(normalise_message(original), normalise_message(&imported));
    }

    #[test]
    fn roundtrip_repeated_segments() {
        let original = "MSH|^~\\&|APP|FAC|||20231215||ORU^R01|123|P|2.5.1\r\
                        OBX|1|ST|CODE1||Value1\r\
                        OBX|2|NM|CODE2||42\r\
                        OBX|3|TX|CODE3||Some text";
        let json = export_to_json(original).expect("can export to JSON");
        let imported = import_from_json(&json).expect("can import from JSON");
        assert_eq!(normalise_message(original), normalise_message(&imported));
    }

    #[test]
    fn roundtrip_field_repetitions() {
        let original = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\r\
                        PID|||ID1~ID2~ID3||DOE^JOHN";
        let json = export_to_json(original).expect("can export to JSON");
        let imported = import_from_json(&json).expect("can import from JSON");
        assert_eq!(normalise_message(original), normalise_message(&imported));
    }

    #[test]
    fn roundtrip_complex_components() {
        let original = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\r\
                        PID|||12345^^^HOSP^MR||DOE^JOHN^MIDDLE^SR^DR";
        let json = export_to_json(original).expect("can export to JSON");
        let imported = import_from_json(&json).expect("can import from JSON");
        assert_eq!(normalise_message(original), normalise_message(&imported));
    }

    #[test]
    fn roundtrip_subcomponents() {
        let original = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\r\
                        PID|||12345^^^HOSP&1.2.3&ISO^MR";
        let json = export_to_json(original).expect("can export to JSON");
        let imported = import_from_json(&json).expect("can import from JSON");
        assert_eq!(normalise_message(original), normalise_message(&imported));
    }

    #[test]
    fn roundtrip_empty_fields() {
        let original = "MSH|^~\\&|APP||||20231215||ADT^A01|123|P|2.5.1\r\
                        PID|||||DOE^JOHN";
        let json = export_to_json(original).expect("can export to JSON");
        let imported = import_from_json(&json).expect("can import from JSON");
        assert_eq!(normalise_message(original), normalise_message(&imported));
    }

    #[test]
    fn roundtrip_full_oru_message() {
        let original = "MSH|^~\\&|LAB|FACILITY|APP|DEST|20231215120000||ORU^R01|MSG001|P|2.5.1\r\
                        PID|||12345^^^HOSP^MR||DOE^JOHN^A||19800101|M\r\
                        OBR|1||ORD001|PANEL^Complete Blood Count|||20231215100000\r\
                        OBX|1|NM|WBC^White Blood Count||7.5|10*3/uL|4.5-11.0|N|||F\r\
                        OBX|2|NM|RBC^Red Blood Count||4.8|10*6/uL|4.5-5.5|N|||F\r\
                        OBX|3|NM|HGB^Haemoglobin||14.2|g/dL|13.5-17.5|N|||F";
        let json = export_to_json(original).expect("can export to JSON");
        let imported = import_from_json(&json).expect("can import from JSON");
        assert_eq!(normalise_message(original), normalise_message(&imported));
    }

    #[test]
    fn import_handles_numeric_values() {
        // JSON might parse numbers as numbers, not strings
        let json = r#"{
            "MSH": {
                "1": "^~\\&",
                "2": "APP",
                "9": 123,
                "11": 2.5
            }
        }"#;
        let result = import_from_json(json).expect("can import JSON with numbers");
        assert!(result.contains("|123|"));
        assert!(result.contains("|2.5"));
    }

    #[test]
    fn import_handles_missing_msh() {
        // edge case: no MSH segment (uses default delimiters)
        let json = r#"{
            "PID": {
                "3": "12345",
                "5": { "1": "DOE", "2": "JOHN" }
            }
        }"#;
        let result = import_from_json(json).expect("can import JSON without MSH");
        assert_eq!(result, "PID|||12345||DOE^JOHN");
    }
}
