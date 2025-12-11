//! Message import commands for converting various formats to HL7 messages.
//!
//! Imports messages from JSON, YAML, and TOML formats, reversing the array
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
//! - **Root object** contains a `segments` array.
//! - **Segments** are objects with `segment` name and `fields` object.
//!   Each segment occurrence is a separate array entry.
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
//!     }
//!   ]
//! }
//! ```
//!
//! The import produces:
//! ```text
//! MSH|^~\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1
//! PID|||12345||DOE^JOHN
//! ```

use hl7_parser::{
    builder::{ComponentBuilder, FieldBuilder, MessageBuilder, RepeatBuilder, SegmentBuilder},
    message::Separators,
};
use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// A single segment in the import format.
#[derive(Deserialize)]
struct SegmentImport {
    segment: String,
    #[serde(default)]
    fields: IndexMap<String, Value>,
}

/// Root structure for imported messages.
#[derive(Deserialize)]
struct MessageImport {
    segments: Vec<SegmentImport>,
}

/// Converts an import structure back to an HL7 message using MessageBuilder.
fn tree_to_message(import: &MessageImport) -> Result<String, String> {
    let separators = extract_separators(import);
    let mut builder = MessageBuilder::new(separators);

    for seg in &import.segments {
        let segment = build_segment(&seg.segment, &seg.fields)?;
        builder.push_segment(segment);
    }

    Ok(builder.to_string())
}

/// Extracts separator configuration from the first MSH segment if present.
fn extract_separators(import: &MessageImport) -> Separators {
    // find first MSH segment
    let msh = import.segments.iter().find(|s| s.segment == "MSH");

    let Some(msh) = msh else {
        return Separators::default();
    };

    // MSH.2 contains encoding characters (^~\&)
    match msh.fields.get("2") {
        Some(Value::String(encoding_chars)) => separators_from_encoding_chars(encoding_chars),
        _ => Separators::default(),
    }
}

/// Parses separators from MSH.2 encoding characters string.
fn separators_from_encoding_chars(encoding_chars: &str) -> Separators {
    let chars: Vec<char> = encoding_chars.chars().collect();
    Separators {
        field: '|',
        component: chars.first().copied().unwrap_or('^'),
        repetition: chars.get(1).copied().unwrap_or('~'),
        escape: chars.get(2).copied().unwrap_or('\\'),
        subcomponent: chars.get(3).copied().unwrap_or('&'),
        lenient_newlines: false,
    }
}

/// Builds a SegmentBuilder from segment name and fields map.
fn build_segment(
    segment_name: &str,
    fields: &IndexMap<String, Value>,
) -> Result<SegmentBuilder, String> {
    let max_field_idx = fields
        .keys()
        .filter_map(|k| k.parse::<usize>().ok())
        .max()
        .unwrap_or(0);

    let mut segment = SegmentBuilder::new(segment_name);

    // MSH is special: field "1" in export is the field separator, skip it
    // since SegmentBuilder handles MSH.1/MSH.2 automatically
    let start_idx = if segment_name == "MSH" { 2 } else { 1 };

    for idx in start_idx..=max_field_idx {
        if let Some(field_value) = fields.get(&idx.to_string()) {
            let field = value_to_field(field_value)?;
            segment.set_field(idx, field);
        }
    }

    Ok(segment)
}

/// Converts a JSON value to a FieldBuilder, handling repetitions.
fn value_to_field(value: &Value) -> Result<FieldBuilder, String> {
    match value {
        Value::Null => Ok(FieldBuilder::default()),
        Value::String(s) => Ok(FieldBuilder::with_value(s.clone())),
        Value::Number(n) => Ok(FieldBuilder::with_value(n.to_string())),
        Value::Bool(b) => Ok(FieldBuilder::with_value(b.to_string())),
        Value::Array(arr) => {
            // field repetitions
            let repeats: Result<Vec<RepeatBuilder>, String> =
                arr.iter().map(value_to_repeat).collect();
            Ok(FieldBuilder::with_repeats(repeats?))
        }
        Value::Object(_) => {
            // single repeat with components
            let repeat = value_to_repeat(value)?;
            Ok(FieldBuilder::with_repeats(vec![repeat]))
        }
    }
}

/// Converts a JSON value to a RepeatBuilder (components).
fn value_to_repeat(value: &Value) -> Result<RepeatBuilder, String> {
    match value {
        Value::Null => Ok(RepeatBuilder::default()),
        Value::String(s) => Ok(RepeatBuilder::with_value(s.clone())),
        Value::Number(n) => Ok(RepeatBuilder::with_value(n.to_string())),
        Value::Bool(b) => Ok(RepeatBuilder::with_value(b.to_string())),
        Value::Object(obj) => {
            let mut components: HashMap<usize, ComponentBuilder> = HashMap::new();
            for (key, comp_value) in obj {
                let idx: usize = key
                    .parse()
                    .map_err(|_| format!("Invalid component index: {key}"))?;
                let component = value_to_component(comp_value)?;
                components.insert(idx, component);
            }
            Ok(RepeatBuilder::with_components(components))
        }
        Value::Array(_) => Err("Unexpected array in repeat position".to_string()),
    }
}

/// Converts a JSON value to a ComponentBuilder, handling subcomponents.
fn value_to_component(value: &Value) -> Result<ComponentBuilder, String> {
    match value {
        Value::Null => Ok(ComponentBuilder::default()),
        Value::String(s) => Ok(ComponentBuilder::with_value(s.clone())),
        Value::Number(n) => Ok(ComponentBuilder::with_value(n.to_string())),
        Value::Bool(b) => Ok(ComponentBuilder::with_value(b.to_string())),
        Value::Object(obj) => {
            let mut subcomponents: HashMap<usize, String> = HashMap::new();
            for (key, sub_value) in obj {
                let idx: usize = key
                    .parse()
                    .map_err(|_| format!("Invalid subcomponent index: {key}"))?;
                let sub_str = match sub_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => String::new(),
                    other @ (Value::Array(_) | Value::Object(_)) => {
                        return Err(format!(
                            "Unexpected value type in subcomponent position: {other:?}"
                        ));
                    }
                };
                subcomponents.insert(idx, sub_str);
            }
            Ok(ComponentBuilder::with_subcomponents(subcomponents))
        }
        Value::Array(_) => Err("Unexpected array in component position".to_string()),
    }
}

/// Imports an HL7 message from JSON format.
#[tauri::command]
pub fn import_from_json(content: &str) -> Result<String, String> {
    let import: MessageImport =
        serde_json::from_str(content).map_err(|e| format!("Failed to parse JSON: {e}"))?;
    tree_to_message(&import)
}

/// Imports an HL7 message from YAML format.
#[tauri::command]
pub fn import_from_yaml(content: &str) -> Result<String, String> {
    let import: MessageImport =
        serde_yml::from_str(content).map_err(|e| format!("Failed to parse YAML: {e}"))?;
    tree_to_message(&import)
}

/// Imports an HL7 message from TOML format.
#[tauri::command]
pub fn import_from_toml(content: &str) -> Result<String, String> {
    let import: MessageImport =
        toml::from_str(content).map_err(|e| format!("Failed to parse TOML: {e}"))?;
    tree_to_message(&import)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
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
            "segments": [
                {
                    "segment": "MSH",
                    "fields": {
                        "1": "^~\\&",
                        "2": "APP",
                        "9": 123,
                        "11": 2.5
                    }
                }
            ]
        }"#;
        let result = import_from_json(json).expect("can import JSON with numbers");
        assert!(result.contains("|123|"));
        assert!(result.contains("|2.5"));
    }

    #[test]
    fn import_handles_missing_msh() {
        // edge case: no MSH segment (uses default delimiters)
        let json = r#"{
            "segments": [
                {
                    "segment": "PID",
                    "fields": {
                        "3": "12345",
                        "5": { "1": "DOE", "2": "JOHN" }
                    }
                }
            ]
        }"#;
        let result = import_from_json(json).expect("can import JSON without MSH");
        assert_eq!(result, "PID|||12345||DOE^JOHN");
    }

    #[test]
    fn export_produces_segments_array_structure() {
        // verify the JSON export structure has segments array with segment/fields objects
        let original =
            "MSH|^~\\&|APP|||20231215||ADT^A01\rPID|||12345||DOE^JOHN\rOBX|1||CODE||Value";
        let json = export_to_json(original).expect("can export to JSON");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("can parse JSON");

        // root has segments array
        assert!(parsed.get("segments").is_some(), "should have segments key");
        let segments = parsed["segments"]
            .as_array()
            .expect("segments should be array");
        assert_eq!(segments.len(), 3, "should have 3 segments");

        // each segment has segment name and fields object
        for seg in segments {
            assert!(
                seg.get("segment").is_some(),
                "segment should have segment key"
            );
            assert!(seg["segment"].is_string(), "segment name should be string");
            assert!(
                seg.get("fields").is_some(),
                "segment should have fields key"
            );
            assert!(seg["fields"].is_object(), "fields should be object");
        }

        // verify segment names in order
        assert_eq!(segments[0]["segment"], "MSH");
        assert_eq!(segments[1]["segment"], "PID");
        assert_eq!(segments[2]["segment"], "OBX");
    }
}
