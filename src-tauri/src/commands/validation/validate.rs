//! HL7 message validation commands.
//!
//! This module provides validation for HL7 messages against schema definitions,
//! checking required fields, length limits, patterns, allowed values, and message structure.
//!
//! Two validation modes are provided:
//! * **Light validation** - Fast checks for passive background validation (required fields, parse errors)
//! * **Full validation** - Comprehensive checks for on-demand validation (all rules)

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

use crate::schema::segment::{DataType, Field};
use crate::AppData;

/// Severity level for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Critical issue that likely prevents message processing
    Error,
    /// Potential problem that may cause issues
    Warning,
    /// Informational note about the message
    Info,
}

/// Type of validation rule that was violated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationRule {
    /// Message could not be parsed
    ParseError,
    /// Required field is missing or empty
    RequiredField,
    /// Value is shorter than minimum length
    MinLength,
    /// Value exceeds maximum length
    MaxLength,
    /// Value does not match expected pattern
    Pattern,
    /// Value is not in the list of allowed values
    AllowedValues,
    /// Required segment is missing from message
    RequiredSegment,
    /// Date/datetime format is invalid
    InvalidDate,
}

/// A single validation issue found in the message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// HL7 path to the field (e.g., "PID.3", "MSH.9.1")
    pub path: String,
    /// Character range in the message for highlighting (start, end)
    pub range: Option<(usize, usize)>,
    /// Severity of the issue
    pub severity: Severity,
    /// Human-readable description of the issue
    pub message: String,
    /// Which validation rule was violated
    pub rule: ValidationRule,
    /// The actual value that caused the issue (if applicable)
    pub actual_value: Option<String>,
}

/// Summary of validation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Number of errors found
    pub errors: usize,
    /// Number of warnings found
    pub warnings: usize,
    /// Number of info messages
    pub info: usize,
}

/// Complete validation result for a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// List of all validation issues
    pub issues: Vec<ValidationIssue>,
    /// Summary counts
    pub summary: ValidationSummary,
}

impl ValidationResult {
    fn new(issues: Vec<ValidationIssue>) -> Self {
        let summary = ValidationSummary {
            errors: issues
                .iter()
                .filter(|i| i.severity == Severity::Error)
                .count(),
            warnings: issues
                .iter()
                .filter(|i| i.severity == Severity::Warning)
                .count(),
            info: issues
                .iter()
                .filter(|i| i.severity == Severity::Info)
                .count(),
        };
        Self { issues, summary }
    }
}

/// Perform light validation (fast, for passive background checking).
///
/// Checks:
/// * Parse errors
/// * Required fields
///
/// This is designed to run frequently without noticeable performance impact.
#[tauri::command]
pub fn validate_light(message: &str, state: State<AppData>) -> ValidationResult {
    let mut issues = Vec::new();

    // try to parse the message
    let parsed = match hl7_parser::parse_message_with_lenient_newlines(message) {
        Ok(msg) => {
            // check for trailing unparsed content
            if msg.raw_value().len() != message.len() {
                issues.push(ValidationIssue {
                    path: String::new(),
                    range: Some((msg.raw_value().len(), message.len())),
                    severity: Severity::Error,
                    message: "Message contains unparsed content after last segment".to_string(),
                    rule: ValidationRule::ParseError,
                    actual_value: Some(message[msg.raw_value().len()..].to_string()),
                });
            }
            Some(msg)
        }
        Err(hl7_parser::parser::ParseError::FailedToParse { position, .. }) => {
            issues.push(ValidationIssue {
                path: String::new(),
                range: Some((position, message.len())),
                severity: Severity::Error,
                message: "Failed to parse message".to_string(),
                rule: ValidationRule::ParseError,
                actual_value: None,
            });
            None
        }
        Err(hl7_parser::parser::ParseError::IncompleteInput(position)) => {
            let pos = position.unwrap_or(0);
            issues.push(ValidationIssue {
                path: String::new(),
                range: Some((pos, message.len())),
                severity: Severity::Error,
                message: "Incomplete message input".to_string(),
                rule: ValidationRule::ParseError,
                actual_value: None,
            });
            None
        }
    };

    // if parsing succeeded, check required fields
    if let Some(ref msg) = parsed {
        validate_required_fields(msg, &state, &mut issues);
    }

    ValidationResult::new(issues)
}

/// Perform full validation (comprehensive, for on-demand checking).
///
/// Checks everything from light validation plus:
/// * Length limits (minlength, maxlength)
/// * Pattern matching
/// * Allowed values
/// * Message structure (required segments)
/// * Date/datetime format validation
#[tauri::command]
pub fn validate_full(message: &str, state: State<AppData>) -> ValidationResult {
    let mut issues = Vec::new();

    // try to parse the message
    let parsed = match hl7_parser::parse_message_with_lenient_newlines(message) {
        Ok(msg) => {
            // check for trailing unparsed content
            if msg.raw_value().len() != message.len() {
                issues.push(ValidationIssue {
                    path: String::new(),
                    range: Some((msg.raw_value().len(), message.len())),
                    severity: Severity::Error,
                    message: "Message contains unparsed content after last segment".to_string(),
                    rule: ValidationRule::ParseError,
                    actual_value: Some(message[msg.raw_value().len()..].to_string()),
                });
            }
            Some(msg)
        }
        Err(hl7_parser::parser::ParseError::FailedToParse { position, .. }) => {
            issues.push(ValidationIssue {
                path: String::new(),
                range: Some((position, message.len())),
                severity: Severity::Error,
                message: "Failed to parse message".to_string(),
                rule: ValidationRule::ParseError,
                actual_value: None,
            });
            None
        }
        Err(hl7_parser::parser::ParseError::IncompleteInput(position)) => {
            let pos = position.unwrap_or(0);
            issues.push(ValidationIssue {
                path: String::new(),
                range: Some((pos, message.len())),
                severity: Severity::Error,
                message: "Incomplete message input".to_string(),
                rule: ValidationRule::ParseError,
                actual_value: None,
            });
            None
        }
    };

    if let Some(ref msg) = parsed {
        // validate message structure (required segments)
        validate_message_structure(msg, &state, &mut issues);

        // validate all fields against schema
        validate_required_fields(msg, &state, &mut issues);
        validate_field_constraints(msg, &state, &mut issues);
    }

    ValidationResult::new(issues)
}

/// Extract message type and trigger event from MSH.9.
fn get_message_type(msg: &hl7_parser::Message) -> (String, String) {
    let msh = match msg.segments().find(|s| s.name == "MSH") {
        Some(s) => s,
        None => return (String::new(), String::new()),
    };

    let msg_type_field = msh.fields.get(8);
    match msg_type_field {
        Some(field) => {
            let first_repeat = field.repeats.first();
            let msg_type = first_repeat
                .and_then(|r| r.components.first())
                .map(|c| msg.separators.decode(c.raw_value()).to_string())
                .unwrap_or_default();
            let trigger = first_repeat
                .and_then(|r| r.components.get(1))
                .map(|c| msg.separators.decode(c.raw_value()).to_string())
                .unwrap_or_default();
            (msg_type, trigger)
        }
        None => (String::new(), String::new()),
    }
}

/// Check if a field's trigger filter matches the current message.
fn matches_trigger_filter(field_def: &Field, trigger_event: &str) -> bool {
    match &field_def.trigger_filter {
        Some(filter) => filter.eq_ignore_ascii_case(trigger_event),
        None => true, // no filter means applies to all messages
    }
}

/// Check that required fields have values.
fn validate_required_fields(
    msg: &hl7_parser::Message,
    state: &State<AppData>,
    issues: &mut Vec<ValidationIssue>,
) {
    let (_msg_type, trigger_event) = get_message_type(msg);

    for segment in msg.segments() {
        let schema = match state.schema.get_segment(segment.name) {
            Ok(s) => s,
            Err(_) => continue, // no schema for this segment
        };

        let required_fields: HashMap<(u8, Option<u8>), &Field> = schema
            .iter()
            .filter(|f| f.required == Some(true))
            .filter(|f| matches_trigger_filter(f, &trigger_event))
            .map(|f| ((f.field, f.component), f))
            .collect();

        for ((field_num, component_num), field_def) in required_fields {
            let value = get_field_value(segment, field_num, component_num, msg);
            let is_empty = value.as_ref().map(|(v, _)| v.is_empty()).unwrap_or(true);

            if is_empty {
                let path = match component_num {
                    Some(c) => format!("{}.{}.{}", segment.name, field_num, c),
                    None => format!("{}.{}", segment.name, field_num),
                };

                // find range for highlighting (use segment range if field not present)
                let range = value
                    .as_ref()
                    .and_then(|(_, r)| *r)
                    .or(Some((segment.range.start, segment.range.end)));

                issues.push(ValidationIssue {
                    path: path.clone(),
                    range,
                    severity: Severity::Error,
                    message: format!("{} ({}) is required", path, field_def.name),
                    rule: ValidationRule::RequiredField,
                    actual_value: None,
                });
            }
        }
    }
}

/// Validate field constraints (length, pattern, allowed values, datatypes).
fn validate_field_constraints(
    msg: &hl7_parser::Message,
    state: &State<AppData>,
    issues: &mut Vec<ValidationIssue>,
) {
    let (_msg_type, trigger_event) = get_message_type(msg);

    for segment in msg.segments() {
        let schema = match state.schema.get_segment(segment.name) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for field_def in schema
            .iter()
            .filter(|f| matches_trigger_filter(f, &trigger_event))
        {
            let value = get_field_value(segment, field_def.field, field_def.component, msg);

            if let Some((value, range)) = value {
                // skip empty values and template placeholders
                if value.is_empty() || value.starts_with('{') && value.ends_with('}') {
                    continue;
                }

                let path = match field_def.component {
                    Some(c) => format!("{}.{}.{}", segment.name, field_def.field, c),
                    None => format!("{}.{}", segment.name, field_def.field),
                };

                // check minlength
                if let Some(minlen) = field_def.minlength {
                    if value.len() < minlen as usize {
                        issues.push(ValidationIssue {
                            path: path.clone(),
                            range,
                            severity: Severity::Warning,
                            message: format!(
                                "{} ({}) is too short: {} chars, minimum is {}",
                                path,
                                field_def.name,
                                value.len(),
                                minlen
                            ),
                            rule: ValidationRule::MinLength,
                            actual_value: Some(value.clone()),
                        });
                    }
                }

                // check maxlength
                if let Some(maxlen) = field_def.maxlength {
                    if value.len() > maxlen as usize {
                        issues.push(ValidationIssue {
                            path: path.clone(),
                            range,
                            severity: Severity::Warning,
                            message: format!(
                                "{} ({}) is too long: {} chars, maximum is {}",
                                path,
                                field_def.name,
                                value.len(),
                                maxlen
                            ),
                            rule: ValidationRule::MaxLength,
                            actual_value: Some(value.clone()),
                        });
                    }
                }

                // check pattern
                if let Some(ref pattern) = field_def.pattern {
                    // anchor the pattern to match the entire value
                    let anchored = format!("^({})$", pattern);
                    if let Ok(re) = Regex::new(&anchored) {
                        if !re.is_match(&value) {
                            issues.push(ValidationIssue {
                                path: path.clone(),
                                range,
                                severity: Severity::Warning,
                                message: format!(
                                    "{} ({}) does not match expected format",
                                    path, field_def.name
                                ),
                                rule: ValidationRule::Pattern,
                                actual_value: Some(value.clone()),
                            });
                        }
                    }
                }

                // check allowed values (if no pattern specified)
                if field_def.pattern.is_none() {
                    if let Some(ref allowed) = field_def.values {
                        // filter out template placeholders like {auto}, {now} from allowed values
                        let real_values: Vec<&String> = allowed
                            .keys()
                            .filter(|k| !(k.starts_with('{') && k.ends_with('}')))
                            .collect();

                        // only validate if there are non-template allowed values
                        if !real_values.is_empty() && !real_values.contains(&&value) {
                            issues.push(ValidationIssue {
                                path: path.clone(),
                                range,
                                severity: Severity::Warning,
                                message: format!(
                                    "{} ({}) has unexpected value '{}'. Expected one of: {}",
                                    path,
                                    field_def.name,
                                    value,
                                    real_values
                                        .iter()
                                        .take(5)
                                        .map(|s| format!("'{}'", s))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ),
                                rule: ValidationRule::AllowedValues,
                                actual_value: Some(value.clone()),
                            });
                        }
                    }
                }

                // check date/datetime format
                if let Some(datatype) = field_def.datatype {
                    validate_datetime(&value, datatype, &path, &field_def.name, range, issues);
                }
            }
        }
    }
}

/// Validate date/datetime format.
fn validate_datetime(
    value: &str,
    datatype: DataType,
    path: &str,
    field_name: &str,
    range: Option<(usize, usize)>,
    issues: &mut Vec<ValidationIssue>,
) {
    // skip template placeholders
    if value.starts_with('{') && value.ends_with('}') {
        return;
    }

    let is_valid = match datatype {
        DataType::Date => {
            // YYYYMMDD format
            value.len() >= 8 && value[..8].chars().all(|c| c.is_ascii_digit())
        }
        DataType::DateTime => {
            // YYYYMMDDHHMMSS[.SSS][+/-ZZZZ] format
            value.len() >= 8 && value[..8].chars().all(|c| c.is_ascii_digit())
        }
    };

    if !is_valid {
        let expected_format = match datatype {
            DataType::Date => "YYYYMMDD",
            DataType::DateTime => "YYYYMMDDHHMMSS",
        };
        issues.push(ValidationIssue {
            path: path.to_string(),
            range,
            severity: Severity::Warning,
            message: format!(
                "{} ({}) has invalid date format. Expected: {}",
                path, field_name, expected_format
            ),
            rule: ValidationRule::InvalidDate,
            actual_value: Some(value.to_string()),
        });
    }
}

/// Validate message structure (required segments).
fn validate_message_structure(
    msg: &hl7_parser::Message,
    state: &State<AppData>,
    issues: &mut Vec<ValidationIssue>,
) {
    // check for MSH segment
    if !msg.segments().any(|s| s.name == "MSH") {
        issues.push(ValidationIssue {
            path: "MSH".to_string(),
            range: None,
            severity: Severity::Error,
            message: "MSH segment is required".to_string(),
            rule: ValidationRule::RequiredSegment,
            actual_value: None,
        });
        return;
    }

    let (msg_type, trigger_event) = get_message_type(msg);
    if msg_type.is_empty() || trigger_event.is_empty() {
        return;
    }

    // look up message structure
    let message_key = format!(
        "{}_{}",
        msg_type.to_lowercase(),
        trigger_event.to_lowercase()
    );
    let messages_schema = match state.schema.get_messages() {
        Ok(s) => s,
        Err(_) => return,
    };

    let message_def = match messages_schema.message.get(&message_key) {
        Some(def) => def,
        None => return, // unknown message type, skip structure validation
    };

    // collect segments present in the message
    let present_segments: Vec<&str> = msg.segments().map(|s| s.name).collect();

    // check required segments
    for segment_meta in message_def {
        if segment_meta.required == Some(true)
            && !present_segments.contains(&segment_meta.name.as_str())
        {
            issues.push(ValidationIssue {
                path: segment_meta.name.clone(),
                range: None,
                severity: Severity::Error,
                message: format!(
                    "{} segment is required for {}^{} messages",
                    segment_meta.name, msg_type, trigger_event
                ),
                rule: ValidationRule::RequiredSegment,
                actual_value: None,
            });
        }
    }
}

/// Get the value and range of a field or component from a segment.
fn get_field_value(
    segment: &hl7_parser::message::Segment,
    field_num: u8,
    component_num: Option<u8>,
    msg: &hl7_parser::Message,
) -> Option<(String, Option<(usize, usize)>)> {
    let field_idx = field_num as usize - 1;

    let field = segment.fields.get(field_idx)?;
    let repeat = field.repeats.first()?;

    match component_num {
        Some(comp_num) => {
            let component = repeat.components.get(comp_num as usize - 1)?;
            let value = msg.separators.decode(component.raw_value()).to_string();
            Some((value, Some((component.range.start, component.range.end))))
        }
        None => {
            // return entire field value (first component or raw value)
            if repeat.components.is_empty() {
                let value = msg.separators.decode(repeat.raw_value()).to_string();
                Some((value, Some((repeat.range.start, repeat.range.end))))
            } else {
                let component = repeat.components.first()?;
                let value = msg.separators.decode(component.raw_value()).to_string();
                Some((value, Some((component.range.start, component.range.end))))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_validation_date() {
        let mut issues = Vec::new();
        validate_datetime(
            "20250101",
            DataType::Date,
            "PID.7",
            "DOB",
            None,
            &mut issues,
        );
        assert!(issues.is_empty());

        validate_datetime("invalid", DataType::Date, "PID.7", "DOB", None, &mut issues);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, ValidationRule::InvalidDate);
    }

    #[test]
    fn test_datetime_validation_datetime() {
        let mut issues = Vec::new();
        validate_datetime(
            "20250101120000",
            DataType::DateTime,
            "MSH.7",
            "DateTime",
            None,
            &mut issues,
        );
        assert!(issues.is_empty());

        validate_datetime(
            "{now}",
            DataType::DateTime,
            "MSH.7",
            "DateTime",
            None,
            &mut issues,
        );
        // template placeholders should be skipped
        assert!(issues.is_empty());
    }
}
