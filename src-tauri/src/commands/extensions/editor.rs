//! Editor operation handlers for extension requests.
//!
//! These functions handle `editor/*` JSON-RPC requests from extensions:
//!
//! - `editor/getMessage` - Get the current message in various formats
//! - `editor/setMessage` - Replace the entire message
//! - `editor/patchMessage` - Apply targeted patches to specific fields
//!
//! The handlers reuse existing export/import functionality where possible and
//! communicate with the frontend via Tauri events.

use crate::commands::editor::export::{export_to_json, export_to_toml, export_to_yaml};
use crate::commands::editor::import::{import_from_json, import_from_toml, import_from_yaml};
use crate::extensions::protocol::RpcError;
use crate::extensions::types::{
    GetMessageResult, MessageFormat, Patch, PatchError, PatchMessageResult, SetMessageParams,
    SetMessageResult,
};
use hl7_parser::builder::{
    ComponentBuilder, FieldBuilder, MessageBuilder, RepeatBuilder, SegmentBuilder,
};
use hl7_parser::query::LocationQuery;

/// Handle `editor/getMessage` request from an extension.
///
/// Converts the current HL7 message to the requested format (hl7, json, yaml, toml).
/// Now called synchronously from host.rs using the backend's copy of the message.
pub fn handle_get_message(
    raw_message: &str,
    format: MessageFormat,
) -> Result<GetMessageResult, RpcError> {
    let message = match format {
        MessageFormat::Hl7 => raw_message.to_string(),
        MessageFormat::Json => export_to_json(raw_message)
            .map_err(|e| RpcError::internal(format!("failed to export to JSON: {e}")))?,
        MessageFormat::Yaml => export_to_yaml(raw_message)
            .map_err(|e| RpcError::internal(format!("failed to export to YAML: {e}")))?,
        MessageFormat::Toml => export_to_toml(raw_message)
            .map_err(|e| RpcError::internal(format!("failed to export to TOML: {e}")))?,
    };

    Ok(GetMessageResult {
        message,
        has_file: false,
        file_path: None,
    })
}

/// Handle `editor/setMessage` request from an extension.
///
/// Converts the provided message to HL7 format if needed and validates it.
/// Returns the HL7 message to be set in the editor.
pub fn handle_set_message(
    params: SetMessageParams,
) -> Result<(String, SetMessageResult), RpcError> {
    // convert to HL7 if needed
    let hl7_message = match params.format {
        MessageFormat::Hl7 => params.message,
        MessageFormat::Json => import_from_json(&params.message)
            .map_err(|e| RpcError::invalid_message(format!("failed to import from JSON: {e}")))?,
        MessageFormat::Yaml => import_from_yaml(&params.message)
            .map_err(|e| RpcError::invalid_message(format!("failed to import from YAML: {e}")))?,
        MessageFormat::Toml => import_from_toml(&params.message)
            .map_err(|e| RpcError::invalid_message(format!("failed to import from TOML: {e}")))?,
    };

    // validate basic HL7 structure (must have MSH)
    if let Err(e) = validate_hl7_structure(&hl7_message) {
        return Ok((
            String::new(),
            SetMessageResult {
                success: false,
                error: Some(e),
            },
        ));
    }

    Ok((
        hl7_message,
        SetMessageResult {
            success: true,
            error: None,
        },
    ))
}

/// Handle `editor/patchMessage` request from an extension.
///
/// Applies a list of patches to the HL7 message. Each patch targets a specific
/// HL7 path (e.g., "PID.5.1", "OBX[2].5") and can set, clear, or remove fields.
///
/// Uses best-effort semantics: patches are applied in order, failures are recorded
/// but don't stop subsequent patches from being attempted.
///
/// Returns the new message and the result. Now called synchronously from host.rs.
pub fn handle_patch_message(
    raw_message: &str,
    patches: Vec<Patch>,
) -> (String, PatchMessageResult) {
    let mut message = raw_message.to_string();
    let mut errors: Vec<PatchError> = Vec::new();
    let mut patches_applied = 0;

    for (index, patch) in patches.iter().enumerate() {
        match apply_patch(&message, patch) {
            Ok(new_message) => {
                message = new_message;
                patches_applied += 1;
            }
            Err(e) => {
                errors.push(PatchError {
                    index,
                    path: patch.path.clone(),
                    message: e,
                });
            }
        }
    }

    let result = PatchMessageResult {
        success: errors.is_empty(),
        patches_applied,
        errors: if errors.is_empty() {
            None
        } else {
            Some(errors)
        },
    };

    (message, result)
}

/// Validates basic HL7 message structure.
fn validate_hl7_structure(message: &str) -> Result<(), String> {
    if message.is_empty() {
        return Err("Message is empty".to_string());
    }

    // check for MSH segment
    if !message.starts_with("MSH") {
        return Err("Message must start with MSH segment".to_string());
    }

    // try to parse the message
    hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("Invalid HL7 message: {e}"))?;

    Ok(())
}

/// Apply a single patch operation to an HL7 message.
fn apply_patch(message: &str, patch: &Patch) -> Result<String, String> {
    // parse the path using hl7_parser's LocationQuery
    let query = LocationQuery::parse(&patch.path)
        .map_err(|e| format!("Invalid path '{}': {}", patch.path, e))?;

    // handle segment creation
    if patch.create == Some(true) && query.field.is_none() {
        return create_segment(message, &query.segment, query.segment_index);
    }

    // handle segment removal
    if patch.remove == Some(true) && query.field.is_none() {
        return remove_segment(message, &query.segment, query.segment_index);
    }

    // for field operations, we need a field number
    let field_num = query.field.ok_or_else(|| {
        format!(
            "Path '{}' must specify a field number for set/remove operations",
            patch.path
        )
    })?;

    // get the value to set (empty string clears the field)
    let value = patch.value.clone().unwrap_or_default();

    set_field_value(
        message,
        &query.segment,
        query.segment_index,
        field_num,
        query.repeat,
        query.component,
        query.subcomponent,
        &value,
    )
}

/// Create a new segment in the message.
fn create_segment(
    message: &str,
    segment_name: &str,
    segment_index: Option<usize>,
) -> Result<String, String> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("failed to parse message: {e}"))?;

    let mut builder = MessageBuilder::from(&parsed);
    let segments = builder.segments_mut();

    // find insert position: after the last occurrence of this segment type, or at end
    let mut insert_position = segments.len();
    let mut count = 0;

    for (i, seg) in segments.iter().enumerate() {
        if seg.name() == segment_name {
            count += 1;
            insert_position = i + 1;
        }
    }

    // if a specific index was requested and we need placeholder segments
    if let Some(idx) = segment_index {
        if idx > count + 1 {
            // need to create placeholder segments
            for _ in (count + 1)..idx {
                segments.insert(insert_position, SegmentBuilder::new(segment_name));
                insert_position += 1;
            }
        }
    }

    // insert the new segment
    segments.insert(insert_position, SegmentBuilder::new(segment_name));

    Ok(builder.to_string())
}

/// Remove a segment from the message.
fn remove_segment(
    message: &str,
    segment_name: &str,
    segment_index: Option<usize>,
) -> Result<String, String> {
    // don't allow removing MSH
    if segment_name == "MSH" {
        return Err("Cannot remove MSH segment".to_string());
    }

    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("failed to parse message: {e}"))?;

    let mut builder = MessageBuilder::from(&parsed);

    // use the library's segment_n method to find and remove by occurrence (1-based)
    let target_idx = segment_index.unwrap_or(1);
    if builder.remove_segment_n(segment_name, target_idx).is_none() {
        // if segment doesn't exist, silently succeed
        return Ok(message.to_string());
    }

    Ok(builder.to_string())
}

/// Set a field value in the message using the hl7-parser builder API.
#[allow(clippy::too_many_arguments)]
fn set_field_value(
    message: &str,
    segment_name: &str,
    segment_index: Option<usize>,
    field_num: usize,
    field_repetition: Option<usize>,
    component: Option<usize>,
    subcomponent: Option<usize>,
    value: &str,
) -> Result<String, String> {
    let parsed = hl7_parser::parse_message_with_lenient_newlines(message)
        .map_err(|e| format!("failed to parse message: {e}"))?;

    let mut builder = MessageBuilder::from(&parsed);

    // find the target segment by name and occurrence (1-based)
    let target_idx = segment_index.unwrap_or(1);
    let segment = builder
        .segment_n_mut(segment_name, target_idx)
        .ok_or_else(|| format!("Segment {} not found", segment_name))?;

    // modify the field using the builder API
    modify_segment_field(
        segment,
        field_num,
        field_repetition,
        component,
        subcomponent,
        value,
    );

    Ok(builder.to_string())
}

/// Modify a field within a segment using the hl7-parser builder API.
///
/// Sets the value at the specified field/component/subcomponent location. The builder
/// handles proper HL7 encoding and delimiter handling automatically.
fn modify_segment_field(
    segment: &mut SegmentBuilder,
    field_num: usize,
    field_repetition: Option<usize>,
    component: Option<usize>,
    subcomponent: Option<usize>,
    value: &str,
) {
    // field numbering is 1-based in hl7-parser builder API
    match (component, subcomponent) {
        (None, None) => {
            // set the entire field value
            segment.set_field_value(field_num, value);
        }
        (Some(comp), None) => {
            // set a component value within the field
            let field = segment.fields_mut().entry(field_num).or_default();

            set_component_value(field, field_repetition, comp, value);
        }
        (Some(comp), Some(subcomp)) => {
            // set a subcomponent value within the field/component
            let field = segment.fields_mut().entry(field_num).or_default();

            set_subcomponent_value(field, field_repetition, comp, subcomp, value);
        }
        (None, Some(_)) => {
            // invalid: subcomponent without component - just set field value
            segment.set_field_value(field_num, value);
        }
    }
}

/// Set a component value within a field, handling repeats if necessary.
fn set_component_value(
    field: &mut FieldBuilder,
    field_repetition: Option<usize>,
    component: usize,
    value: &str,
) {
    match field {
        FieldBuilder::Value(_) => {
            // convert to repeats and set the component
            let mut repeat = RepeatBuilder::default();
            repeat.set_component_value(component, value);
            *field = FieldBuilder::Repeats(vec![repeat]);
        }
        FieldBuilder::Repeats(repeats) => {
            let repeat_idx = field_repetition.unwrap_or(1).saturating_sub(1);
            // ensure we have enough repeats
            while repeats.len() <= repeat_idx {
                repeats.push(RepeatBuilder::default());
            }
            if let Some(repeat) = repeats.get_mut(repeat_idx) {
                repeat.set_component_value(component, value);
            }
        }
    }
}

/// Set a subcomponent value within a field/component, handling repeats if necessary.
fn set_subcomponent_value(
    field: &mut FieldBuilder,
    field_repetition: Option<usize>,
    component: usize,
    subcomponent: usize,
    value: &str,
) {
    match field {
        FieldBuilder::Value(_) => {
            // convert to repeats with a component with subcomponents
            let mut comp = ComponentBuilder::default();
            comp.set_subcomponent(subcomponent, value);
            let mut repeat = RepeatBuilder::default();
            repeat.set_component(component, comp);
            *field = FieldBuilder::Repeats(vec![repeat]);
        }
        FieldBuilder::Repeats(repeats) => {
            let repeat_idx = field_repetition.unwrap_or(1).saturating_sub(1);
            // ensure we have enough repeats
            while repeats.len() <= repeat_idx {
                repeats.push(RepeatBuilder::default());
            }
            let Some(repeat) = repeats.get_mut(repeat_idx) else {
                return;
            };

            // get or create the component, then set the subcomponent
            match repeat {
                RepeatBuilder::Value(_) => {
                    let mut comp = ComponentBuilder::default();
                    comp.set_subcomponent(subcomponent, value);
                    repeat.set_component(component, comp);
                }
                RepeatBuilder::Components(components) => {
                    let comp = components
                        .entry(component)
                        .or_insert_with(ComponentBuilder::default);
                    comp.set_subcomponent(subcomponent, value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_path() {
        let query = LocationQuery::parse("PID.5").unwrap();
        assert_eq!(query.segment, "PID");
        assert_eq!(query.segment_index, None);
        assert_eq!(query.field, Some(5));
        assert_eq!(query.component, None);
    }

    #[test]
    fn test_parse_path_with_component() {
        let query = LocationQuery::parse("PID.5.1").unwrap();
        assert_eq!(query.segment, "PID");
        assert_eq!(query.field, Some(5));
        assert_eq!(query.component, Some(1));
        assert_eq!(query.subcomponent, None);
    }

    #[test]
    fn test_parse_path_with_subcomponent() {
        let query = LocationQuery::parse("PID.5.1.2").unwrap();
        assert_eq!(query.segment, "PID");
        assert_eq!(query.field, Some(5));
        assert_eq!(query.component, Some(1));
        assert_eq!(query.subcomponent, Some(2));
    }

    #[test]
    fn test_parse_path_with_segment_index() {
        let query = LocationQuery::parse("OBX[2].5").unwrap();
        assert_eq!(query.segment, "OBX");
        assert_eq!(query.segment_index, Some(2));
        assert_eq!(query.field, Some(5));
    }

    #[test]
    fn test_parse_path_with_field_repetition() {
        let query = LocationQuery::parse("PID.13[2]").unwrap();
        assert_eq!(query.segment, "PID");
        assert_eq!(query.field, Some(13));
        assert_eq!(query.repeat, Some(2));
    }

    #[test]
    fn test_parse_segment_only() {
        let query = LocationQuery::parse("NK1").unwrap();
        assert_eq!(query.segment, "NK1");
        assert_eq!(query.field, None);
    }

    #[test]
    fn test_set_simple_field() {
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let result =
            set_field_value(message, "PID", None, 5, None, Some(1), None, "SMITH").unwrap();
        assert!(result.contains("SMITH^JOHN"));
    }

    #[test]
    fn test_set_component_value() {
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let result = set_field_value(message, "PID", None, 5, None, Some(2), None, "JANE").unwrap();
        assert!(result.contains("DOE^JANE"));
    }

    #[test]
    fn test_create_new_segment() {
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let result = create_segment(message, "NK1", None).unwrap();
        assert!(result.contains("\rNK1"));
    }

    #[test]
    fn test_remove_segment() {
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN\rNK1|1|DOE^JANE|SPOUSE";
        let result = remove_segment(message, "NK1", None).unwrap();
        assert!(!result.contains("NK1"));
        assert!(result.contains("PID"));
    }

    #[test]
    fn test_cannot_remove_msh() {
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let result = remove_segment(message, "MSH", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_nonexistent_segment_succeeds() {
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let result = remove_segment(message, "NK1", None).unwrap();
        assert_eq!(result, message);
    }

    #[test]
    fn test_handle_get_message_hl7() {
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1";
        let result = handle_get_message(message, MessageFormat::Hl7).unwrap();
        assert_eq!(result.message, message);
        assert!(!result.has_file);
    }

    #[test]
    fn test_handle_get_message_json() {
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1";
        let result = handle_get_message(message, MessageFormat::Json).unwrap();
        assert!(result.message.contains("\"MSH\""));
        assert!(!result.has_file);
        assert_eq!(result.file_path, None);
    }

    #[test]
    fn test_handle_set_message_hl7() {
        let params = SetMessageParams {
            message: "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1".to_string(),
            format: MessageFormat::Hl7,
        };
        let (msg, result) = handle_set_message(params).unwrap();
        assert!(result.success);
        assert!(msg.starts_with("MSH"));
    }

    #[test]
    fn test_handle_set_message_invalid() {
        let params = SetMessageParams {
            message: "NOT AN HL7 MESSAGE".to_string(),
            format: MessageFormat::Hl7,
        };
        let (_, result) = handle_set_message(params).unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_handle_patch_message() {
        let patches = vec![Patch {
            path: "PID.5.1".to_string(),
            value: Some("SMITH".to_string()),
            remove: None,
            create: None,
        }];
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let (new_msg, result) = handle_patch_message(message, patches);
        assert!(result.success);
        assert_eq!(result.patches_applied, 1);
        assert!(new_msg.contains("SMITH^JOHN"));
    }

    #[test]
    fn test_handle_patch_message_partial_failure() {
        let patches = vec![
            Patch {
                path: "PID.5.1".to_string(),
                value: Some("SMITH".to_string()),
                remove: None,
                create: None,
            },
            Patch {
                path: "XYZ.1".to_string(), // invalid segment
                value: Some("VALUE".to_string()),
                remove: None,
                create: None,
            },
        ];
        let message = "MSH|^~\\&|APP|FAC|||20231215||ADT^A01|123|P|2.5.1\rPID|||12345||DOE^JOHN";
        let (new_msg, result) = handle_patch_message(message, patches);
        assert!(!result.success);
        assert_eq!(result.patches_applied, 1);
        assert!(result.errors.is_some());
        assert_eq!(result.errors.unwrap().len(), 1);
        assert!(new_msg.contains("SMITH")); // first patch still applied
    }

    #[test]
    fn test_validate_hl7_structure() {
        assert!(validate_hl7_structure("MSH|^~\\&|APP").is_ok());
        assert!(validate_hl7_structure("").is_err());
        assert!(validate_hl7_structure("PID|||12345").is_err()); // no MSH
    }
}
