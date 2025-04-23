use hl7_parser::Message;

/// This file is lifted from https://github.com/hamaluik/hl7-ls/blob/main/src/spec.rs

pub fn is_valid_version(version: &str) -> bool {
    hl7_definitions::VERSIONS.contains(&version)
}

pub fn get_version_with_fallback<'m>(message: &'m Message) -> &'m str {
    message
        .query("MSH.12")
        .map(|v| {
            let v = v.raw_value();
            if !is_valid_version(v) {
                "2.7.1"
            } else {
                v
            }
        })
        .unwrap_or("2.7.1")
}

pub fn segment_description(version: &str, segment: &str) -> String {
    hl7_definitions::get_segment(version, segment)
        .map(|s| s.description.to_string())
        .unwrap_or_else(|| "Unknown segment".to_string())
}

/// Check if a field is a timestamp
///
/// # Arguments
///
/// * `version` - The HL7 version
/// * `segment` - The segment name
/// * `field` - The field number (1-indexed)
pub fn is_field_a_timestamp(version: &str, segment: &str, field: usize) -> bool {
    hl7_definitions::get_segment(version, segment)
        .and_then(|s| s.fields.get(field - 1))
        .map(|f| f.datatype == "TS" || f.datatype == "DTM")
        .unwrap_or(false)
}

/// Check if a component of a field is a timestamp
///
/// # Arguments
///
/// * `version` - The HL7 version
/// * `segment` - The segment name
/// * `field` - The field number (1-indexed)
/// * `component` - The component number (1-indexed)
pub fn is_component_a_timestamp(
    version: &str,
    segment: &str,
    field: usize,
    component: usize,
) -> bool {
    hl7_definitions::get_segment(version, segment)
        .and_then(|s| s.fields.get(field - 1))
        .and_then(|f| hl7_definitions::get_field(version, f.datatype))
        .and_then(|f| f.subfields.get(component - 1))
        .map(|c| c.datatype == "TS" || c.datatype == "DTM")
        .unwrap_or(false)
}

/// Describe a field in a segment, including its datatype, length,
/// repeatability, and optionality.
///
/// # Arguments
///
/// * `version` - The HL7 version
/// * `segment` - The segment name
/// * `field` - The field number (1-indexed)
pub fn describe_field(version: &str, segment: &str, field: usize) -> String {
    hl7_definitions::get_segment(version, segment)
        .map(|s| {
            s.fields
                .get(field - 1)
                .map(|f| {
                    let datatype = hl7_definitions::get_field(version, f.datatype)
                        .map(|d| d.description)
                        .unwrap_or_else(|| "Unknown datatype");

                    let repeat = match f.repeatability {
                        hl7_definitions::FieldRepeatability::Unbounded => "∞",
                        hl7_definitions::FieldRepeatability::Single => "1",
                        hl7_definitions::FieldRepeatability::Bounded(n) => &n.to_string(),
                    };

                    let optional = match f.optionality {
                        hl7_definitions::FieldOptionality::Required => "*required*",
                        hl7_definitions::FieldOptionality::Optional => "*optional*",
                        hl7_definitions::FieldOptionality::Conditional => "*conditional*",
                        hl7_definitions::FieldOptionality::BackwardCompatibility => {
                            "*backwards compatibility*"
                        }
                    };

                    format!(
                        "{description}, len: {len} ({datatype}) [{optional}/{repeat}]",
                        description = f.description,
                        len = f
                            .max_length
                            .map(|l| l.to_string())
                            .unwrap_or_else(|| "∞".to_string()),
                    )
                })
                .unwrap_or_else(|| "Unknown field".to_string())
        })
        .unwrap_or_else(|| "Unknown segment".to_string())
}

/// Describe a component in a field, including its datatype, length,
/// repeatability, and optionality.
///
/// # Arguments
///
/// * `version` - The HL7 version
/// * `segment` - The segment name
/// * `field` - The field number (1-indexed)
/// * `component` - The component number (1-indexed)
pub fn describe_component(version: &str, segment: &str, field: usize, component: usize) -> String {
    hl7_definitions::get_segment(version, segment)
        .map(|s| {
            s.fields
                .get(field - 1)
                .map(|f| {
                    hl7_definitions::get_field(version, f.datatype)
                        .and_then(|f| f.subfields.get(component - 1))
                        .map(|c| {
                            let datatype = hl7_definitions::get_field(version, c.datatype)
                                .map(|d| d.description)
                                .unwrap_or_else(|| "Unknown datatype");

                            let repeat = match c.repeatability {
                                hl7_definitions::FieldRepeatability::Unbounded => "∞",
                                hl7_definitions::FieldRepeatability::Single => "1",
                                hl7_definitions::FieldRepeatability::Bounded(n) => &n.to_string(),
                            };

                            let optional = match c.optionality {
                                hl7_definitions::FieldOptionality::Required => "*required*",
                                hl7_definitions::FieldOptionality::Optional => "*optional*",
                                hl7_definitions::FieldOptionality::Conditional => "*conditional*",
                                hl7_definitions::FieldOptionality::BackwardCompatibility => {
                                    "*backwards compatibility*"
                                }
                            };

                            format!(
                                "{field_description} / {component_description}, len: {len} ({datatype}) [{optional}/{repeat}]",
                                field_description = f.description,
                                component_description = c.description,
                                len = c
                                    .max_length
                                    .map(|l| l.to_string())
                                    .unwrap_or_else(|| "∞".to_string()),
                            )
                        })
                        .unwrap_or_else(|| "Unknown component".to_string())
                })
                .unwrap_or_else(|| "Unknown field".to_string())
        })
        .unwrap_or_else(|| "Unknown segment".to_string())
}
