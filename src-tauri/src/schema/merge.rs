//! Schema merging for extension overrides.

use crate::extensions::types::{FieldOverride, Nullable, SchemaOverride, SegmentOverride};
use crate::schema::segment::{DataType, Field};
use indexmap::IndexMap;
use std::collections::HashMap;

/// Merge a single FieldOverride into a Field, returning a new Field.
///
/// # Merge Logic
/// - Field and component are preserved from base (override provides match info only)
/// - If override has `Nullable::Value(v)`, use v
/// - If override has `Nullable::Null`, set to None (unset)
/// - If override has `None` (absent), keep base value
///
/// # Type Conversions
/// - `u32` length constraints are cast to `u16`
/// - `String` datatype ("date"/"datetime") is parsed to `DataType` enum
/// - `IndexMap<String, String>` values are converted to `HashMap<String, String>`
pub fn merge_field(base: &Field, override_field: &FieldOverride) -> Field {
    Field {
        field: base.field,
        component: base.component,
        name: merge_option_nullable(&Some(base.name.clone()), &override_field.name)
            .unwrap_or_else(|| base.name.clone()),
        group: merge_option_nullable(&base.group, &override_field.group),
        trigger_filter: base.trigger_filter.clone(),
        minlength: merge_minmax_length(&base.minlength, &override_field.minlength),
        maxlength: merge_minmax_length(&base.maxlength, &override_field.maxlength),
        placeholder: merge_option_nullable(&base.placeholder, &override_field.placeholder),
        required: merge_option_nullable(&base.required, &override_field.required),
        datatype: merge_datatype(&base.datatype, &override_field.datatype),
        pattern: merge_option_nullable(&base.pattern, &override_field.pattern),
        note: merge_option_nullable(&base.note, &override_field.note),
        values: merge_values(&base.values, &override_field.values),
        template: merge_option_nullable(&base.template, &override_field.template),
    }
}

/// Merge an `Option<Nullable<T>>` with a base `Option<T>`.
///
/// Returns the merged value, or None if the override explicitly unsets it.
fn merge_option_nullable<T: Clone>(
    base: &Option<T>,
    override_value: &Option<Nullable<T>>,
) -> Option<T> {
    match override_value {
        None => base.clone(),
        Some(Nullable::Value(v)) => Some(v.clone()),
        Some(Nullable::Null) => None,
    }
}

/// Merge an `Option<Nullable<u32>>` with a base `Option<u16>`, casting the value.
///
/// Used for length constraints that need u32 -> u16 conversion. Values larger
/// than u16::MAX are saturated to prevent silent truncation.
fn merge_minmax_length(base: &Option<u16>, override_value: &Option<Nullable<u32>>) -> Option<u16> {
    match override_value {
        None => *base,
        Some(Nullable::Value(v)) => Some((*v).min(u16::MAX as u32) as u16),
        Some(Nullable::Null) => None,
    }
}

/// Merge datatype field, converting from string to DataType enum.
fn merge_datatype(
    base: &Option<DataType>,
    override_value: &Option<Nullable<String>>,
) -> Option<DataType> {
    match override_value {
        None => *base,
        Some(Nullable::Value(s)) => match s.as_str() {
            "date" => Some(DataType::Date),
            "datetime" => Some(DataType::DateTime),
            _ => {
                log::warn!(
                    "invalid datatype '{}' in schema override, keeping base value",
                    s
                );
                *base
            }
        },
        Some(Nullable::Null) => None,
    }
}

/// Merge values field, converting from IndexMap to HashMap.
///
/// Note: This performs a **full replacement**, not a key-by-key merge. When an
/// override provides values, the entire base values map is replaced. To add new
/// allowed values while keeping existing ones, extensions must include all
/// desired values in their override.
fn merge_values(
    base: &Option<HashMap<String, String>>,
    override_value: &Option<Nullable<IndexMap<String, String>>>,
) -> Option<HashMap<String, String>> {
    match override_value {
        None => base.clone(),
        Some(Nullable::Value(index_map)) => Some(
            index_map
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        ),
        Some(Nullable::Null) => None,
    }
}

/// Merge field overrides with base fields for a segment.
///
/// # Matching Logic
/// Uses two-pass matching with priority:
/// 1. Exact match: field number AND component must both match
/// 2. Flexible match: if base has no component, override with component can match
///
/// Exact matches across ALL base fields are processed first before any flexible
/// matches. This ensures component-level overrides (PID.3.1) match component-level
/// base entries before field-level entries can claim them via flexible matching.
///
/// # Merge Strategy
/// - Matched fields are merged using `merge_field()`
/// - New fields from overrides are added
/// - Unmatched base fields are preserved
pub fn merge_segment_fields(base: &[Field], overrides: &[FieldOverride]) -> Vec<Field> {
    let mut used_overrides = vec![false; overrides.len()];
    let mut matched_bases: Vec<Option<usize>> = vec![None; base.len()];

    // first pass: exact matches for ALL base fields
    for (base_idx, base_field) in base.iter().enumerate() {
        for (override_idx, override_field) in overrides.iter().enumerate() {
            let Some(used) = used_overrides.get_mut(override_idx) else {
                continue;
            };
            if !*used && fields_match_exact(base_field, override_field) {
                if let Some(matched) = matched_bases.get_mut(base_idx) {
                    *matched = Some(override_idx);
                }
                *used = true;
                break;
            }
        }
    }

    // second pass: flexible matches for unmatched field-level base fields
    for (base_idx, base_field) in base.iter().enumerate() {
        let is_unmatched = matched_bases.get(base_idx).is_some_and(|m| m.is_none());
        if is_unmatched && base_field.component.is_none() {
            for (override_idx, override_field) in overrides.iter().enumerate() {
                let Some(used) = used_overrides.get_mut(override_idx) else {
                    continue;
                };
                if !*used && fields_match_flexible(base_field, override_field) {
                    if let Some(matched) = matched_bases.get_mut(base_idx) {
                        *matched = Some(override_idx);
                    }
                    *used = true;
                    break;
                }
            }
        }
    }

    // build result
    let mut result = Vec::new();
    for (base_idx, base_field) in base.iter().enumerate() {
        let matched_override = matched_bases.get(base_idx).and_then(|m| *m);
        if let Some(override_idx) = matched_override {
            if let Some(override_field) = overrides.get(override_idx) {
                result.push(merge_field(base_field, override_field));
            } else {
                result.push(base_field.clone());
            }
        } else {
            result.push(base_field.clone());
        }
    }

    // add new fields from unused overrides
    for (override_idx, override_field) in overrides.iter().enumerate() {
        let is_unused = used_overrides.get(override_idx).is_some_and(|u| !*u);
        if is_unused {
            result.push(field_from_override(override_field));
        }
    }

    result
}

/// Exact match: both field number AND component must match.
fn fields_match_exact(base: &Field, override_field: &FieldOverride) -> bool {
    base.field == override_field.field as u8
        && base.component == override_field.component.map(|c| c as u8)
}

/// Flexible match: field number matches and base has no component.
/// Allows override with component to match field-level base entry.
fn fields_match_flexible(base: &Field, override_field: &FieldOverride) -> bool {
    base.field == override_field.field as u8
        && base.component.is_none()
        && override_field.component.is_some()
}

/// Create a new Field from a FieldOverride with no base.
///
/// All `Option<Nullable<T>>` fields are unwrapped:
/// - `None` or `Nullable::Null` become `None`
/// - `Nullable::Value(v)` becomes `Some(v)`
fn field_from_override(override_field: &FieldOverride) -> Field {
    Field {
        field: override_field.field as u8,
        component: override_field.component.map(|c| c as u8),
        name: unwrap_nullable(&override_field.name).unwrap_or_default(),
        group: unwrap_nullable(&override_field.group),
        trigger_filter: None,
        minlength: unwrap_nullable_u16(&override_field.minlength),
        maxlength: unwrap_nullable_u16(&override_field.maxlength),
        placeholder: unwrap_nullable(&override_field.placeholder),
        required: unwrap_nullable(&override_field.required),
        datatype: parse_datatype(&override_field.datatype),
        pattern: unwrap_nullable(&override_field.pattern),
        note: unwrap_nullable(&override_field.note),
        values: unwrap_values(&override_field.values),
        template: unwrap_nullable(&override_field.template),
    }
}

/// Unwrap an `Option<Nullable<T>>` to `Option<T>`.
fn unwrap_nullable<T: Clone>(value: &Option<Nullable<T>>) -> Option<T> {
    match value {
        Some(Nullable::Value(v)) => Some(v.clone()),
        _ => None,
    }
}

/// Unwrap an `Option<Nullable<u32>>` to `Option<u16>` with saturating cast.
fn unwrap_nullable_u16(value: &Option<Nullable<u32>>) -> Option<u16> {
    match value {
        Some(Nullable::Value(v)) => Some((*v).min(u16::MAX as u32) as u16),
        _ => None,
    }
}

/// Parse datatype string to DataType enum.
fn parse_datatype(value: &Option<Nullable<String>>) -> Option<DataType> {
    match value {
        Some(Nullable::Value(s)) => match s.as_str() {
            "date" => Some(DataType::Date),
            "datetime" => Some(DataType::DateTime),
            _ => None,
        },
        _ => None,
    }
}

/// Unwrap values field from IndexMap to HashMap.
fn unwrap_values(
    value: &Option<Nullable<IndexMap<String, String>>>,
) -> Option<HashMap<String, String>> {
    match value {
        Some(Nullable::Value(index_map)) => Some(
            index_map
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        ),
        _ => None,
    }
}

/// Convert a Field back to a FieldOverride for storage in merged schema.
///
/// This is used when merging multiple schema overrides together.
fn field_to_override(field: &Field) -> FieldOverride {
    FieldOverride {
        field: field.field as u32,
        component: field.component.map(|c| c as u32),
        name: Some(Nullable::Value(field.name.clone())),
        group: option_to_nullable(&field.group),
        note: option_to_nullable(&field.note),
        required: option_to_nullable(&field.required),
        minlength: field.minlength.map(|v| Nullable::Value(v as u32)),
        maxlength: field.maxlength.map(|v| Nullable::Value(v as u32)),
        pattern: option_to_nullable(&field.pattern),
        datatype: field.datatype.map(|dt| {
            Nullable::Value(match dt {
                DataType::Date => "date".to_string(),
                DataType::DateTime => "datetime".to_string(),
            })
        }),
        placeholder: option_to_nullable(&field.placeholder),
        values: field.values.as_ref().map(|hm| {
            let mut index_map = IndexMap::new();
            for (k, v) in hm {
                index_map.insert(k.clone(), v.clone());
            }
            Nullable::Value(index_map)
        }),
        template: option_to_nullable(&field.template),
    }
}

/// Convert an Option<T> to Option<Nullable<T>>.
fn option_to_nullable<T: Clone>(value: &Option<T>) -> Option<Nullable<T>> {
    value.as_ref().map(|v| Nullable::Value(v.clone()))
}

/// Merge multiple SchemaOverrides into one (for multiple extensions).
///
/// Later overrides take precedence. Segments are merged by name,
/// and fields within segments are merged using `merge_segment_fields()`.
pub fn merge_schema_overrides(overrides: &[SchemaOverride]) -> SchemaOverride {
    if overrides.is_empty() {
        return SchemaOverride::default();
    }

    let mut merged_segments: IndexMap<String, SegmentOverride> = IndexMap::new();

    for schema in overrides {
        if let Some(segments) = &schema.segments {
            for (segment_name, segment_override) in segments {
                if let Some(new_override_fields) = &segment_override.fields {
                    merged_segments
                        .entry(segment_name.clone())
                        .and_modify(|existing| {
                            // convert existing FieldOverrides to Fields if needed,
                            // then merge with new overrides
                            if let Some(existing_overrides) = &existing.fields {
                                // first convert existing overrides to base fields
                                let base_fields: Vec<Field> =
                                    existing_overrides.iter().map(field_from_override).collect();
                                // then merge with new overrides
                                let merged_fields =
                                    merge_segment_fields(&base_fields, new_override_fields);
                                // convert back to overrides (store as base Field data in FieldOverride format)
                                existing.fields =
                                    Some(merged_fields.iter().map(field_to_override).collect());
                            } else {
                                existing.fields = Some(new_override_fields.clone());
                            }
                        })
                        .or_insert_with(|| SegmentOverride {
                            fields: Some(new_override_fields.clone()),
                        });
                } else if !merged_segments.contains_key(segment_name) {
                    merged_segments.insert(segment_name.clone(), segment_override.clone());
                }
            }
        }
    }

    SchemaOverride {
        segments: if merged_segments.is_empty() {
            None
        } else {
            Some(merged_segments)
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn make_base_field(field: u8, component: Option<u8>, name: &str) -> Field {
        Field {
            field,
            component,
            name: name.to_string(),
            group: None,
            trigger_filter: None,
            minlength: None,
            maxlength: None,
            placeholder: None,
            required: None,
            datatype: None,
            pattern: None,
            note: None,
            values: None,
            template: None,
        }
    }

    #[test]
    fn test_merge_field_adds_new_properties() {
        let base = make_base_field(5, None, "Patient Name");

        let override_ = FieldOverride {
            field: 5,
            component: None,
            name: None,
            group: None,
            note: Some(Nullable::Value("Family name component".to_string())),
            required: None,
            minlength: None,
            maxlength: None,
            pattern: None,
            datatype: None,
            placeholder: None,
            values: None,
            template: None,
        };

        let merged = merge_field(&base, &override_);
        assert_eq!(merged.name, "Patient Name");
        assert_eq!(merged.note, Some("Family name component".to_string()));
    }

    #[test]
    fn test_merge_field_overrides_existing_properties() {
        let base = Field {
            field: 5,
            component: None,
            name: "Old Name".to_string(),
            group: None,
            trigger_filter: None,
            minlength: Some(10),
            maxlength: Some(100),
            placeholder: None,
            required: Some(true),
            datatype: None,
            pattern: None,
            note: None,
            values: None,
            template: None,
        };

        let override_ = FieldOverride {
            field: 5,
            component: None,
            name: Some(Nullable::Value("New Name".to_string())),
            group: None,
            note: None,
            required: Some(Nullable::Value(false)),
            minlength: Some(Nullable::Value(5)),
            maxlength: None,
            pattern: None,
            datatype: None,
            placeholder: None,
            values: None,
            template: None,
        };

        let merged = merge_field(&base, &override_);
        assert_eq!(merged.name, "New Name");
        assert_eq!(merged.required, Some(false));
        assert_eq!(merged.minlength, Some(5));
        assert_eq!(merged.maxlength, Some(100)); // preserved from base
    }

    #[test]
    fn test_merge_field_preserves_unspecified_properties() {
        let base = Field {
            field: 5,
            component: None,
            name: "Patient Name".to_string(),
            group: Some("Demographics".to_string()),
            trigger_filter: None,
            minlength: Some(10),
            maxlength: Some(100),
            placeholder: Some("Enter name".to_string()),
            required: Some(true),
            datatype: Some(DataType::Date),
            pattern: Some("[A-Z]+".to_string()),
            note: Some("Important field".to_string()),
            values: None,
            template: Some("DOE".to_string()),
        };

        let override_ = FieldOverride {
            field: 5,
            component: None,
            name: Some(Nullable::Value("Updated Name".to_string())),
            group: None,       // absent, should preserve base
            note: None,        // absent, should preserve base
            required: None,    // absent, should preserve base
            minlength: None,   // absent, should preserve base
            maxlength: None,   // absent, should preserve base
            pattern: None,     // absent, should preserve base
            datatype: None,    // absent, should preserve base
            placeholder: None, // absent, should preserve base
            values: None,      // absent, should preserve base
            template: None,    // absent, should preserve base
        };

        let merged = merge_field(&base, &override_);
        assert_eq!(merged.name, "Updated Name");
        assert_eq!(merged.group, Some("Demographics".to_string()));
        assert_eq!(merged.minlength, Some(10));
        assert_eq!(merged.maxlength, Some(100));
        assert_eq!(merged.placeholder, Some("Enter name".to_string()));
        assert_eq!(merged.required, Some(true));
        assert_eq!(merged.datatype, Some(DataType::Date));
        assert_eq!(merged.pattern, Some("[A-Z]+".to_string()));
        assert_eq!(merged.note, Some("Important field".to_string()));
        assert_eq!(merged.template, Some("DOE".to_string()));
    }

    #[test]
    fn test_merge_field_unsets_with_null() {
        let mut base_values = HashMap::new();
        base_values.insert("M".to_string(), "Male".to_string());
        base_values.insert("F".to_string(), "Female".to_string());

        let base = Field {
            field: 8,
            component: None,
            name: "Gender".to_string(),
            group: Some("Demographics".to_string()),
            trigger_filter: None,
            minlength: Some(1),
            maxlength: Some(1),
            placeholder: Some("M/F".to_string()),
            required: Some(true),
            datatype: None,
            pattern: Some("[MF]".to_string()),
            note: Some("Patient gender".to_string()),
            values: Some(base_values),
            template: Some("M".to_string()),
        };

        let override_ = FieldOverride {
            field: 8,
            component: None,
            name: None,
            group: Some(Nullable::Null),     // unset group
            note: Some(Nullable::Null),      // unset note
            required: Some(Nullable::Null),  // unset required
            minlength: Some(Nullable::Null), // unset minlength
            maxlength: Some(Nullable::Null), // unset maxlength
            pattern: Some(Nullable::Null),   // unset pattern
            datatype: None,
            placeholder: Some(Nullable::Null), // unset placeholder
            values: Some(Nullable::Null),      // unset values
            template: Some(Nullable::Null),    // unset template
        };

        let merged = merge_field(&base, &override_);
        assert_eq!(merged.name, "Gender"); // preserved
        assert_eq!(merged.group, None);
        assert_eq!(merged.note, None);
        assert_eq!(merged.required, None);
        assert_eq!(merged.minlength, None);
        assert_eq!(merged.maxlength, None);
        assert_eq!(merged.pattern, None);
        assert_eq!(merged.placeholder, None);
        assert_eq!(merged.values, None);
        assert_eq!(merged.template, None);
    }

    #[test]
    fn test_merge_segment_adds_new_fields() {
        let base = vec![
            make_base_field(1, None, "Field 1"),
            make_base_field(2, None, "Field 2"),
        ];

        let overrides = vec![FieldOverride {
            field: 3,
            component: None,
            name: Some(Nullable::Value("Field 3".to_string())),
            group: None,
            note: None,
            required: None,
            minlength: None,
            maxlength: None,
            pattern: None,
            datatype: None,
            placeholder: None,
            values: None,
            template: None,
        }];

        let merged = merge_segment_fields(&base, &overrides);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].name, "Field 1");
        assert_eq!(merged[1].name, "Field 2");
        assert_eq!(merged[2].name, "Field 3");
        assert_eq!(merged[2].field, 3);
    }

    #[test]
    fn test_merge_segment_updates_existing_fields() {
        let base = vec![
            make_base_field(1, None, "Field 1"),
            make_base_field(2, None, "Field 2"),
            make_base_field(3, None, "Field 3"),
        ];

        let overrides = vec![FieldOverride {
            field: 2,
            component: None,
            name: Some(Nullable::Value("Updated Field 2".to_string())),
            group: None,
            note: Some(Nullable::Value("New note".to_string())),
            required: None,
            minlength: None,
            maxlength: None,
            pattern: None,
            datatype: None,
            placeholder: None,
            values: None,
            template: None,
        }];

        let merged = merge_segment_fields(&base, &overrides);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].name, "Field 1");
        assert_eq!(merged[1].name, "Updated Field 2");
        assert_eq!(merged[1].note, Some("New note".to_string()));
        assert_eq!(merged[2].name, "Field 3");
    }

    #[test]
    fn test_merge_multiple_overrides_later_wins() {
        let override1 = SchemaOverride {
            segments: Some({
                let mut map = IndexMap::new();
                map.insert(
                    "PID".to_string(),
                    SegmentOverride {
                        fields: Some(vec![FieldOverride {
                            field: 5,
                            component: None,
                            name: Some(Nullable::Value("First Name".to_string())),
                            group: None,
                            note: None,
                            required: None,
                            minlength: None,
                            maxlength: None,
                            pattern: None,
                            datatype: None,
                            placeholder: None,
                            values: None,
                            template: None,
                        }]),
                    },
                );
                map
            }),
        };

        let override2 = SchemaOverride {
            segments: Some({
                let mut map = IndexMap::new();
                map.insert(
                    "PID".to_string(),
                    SegmentOverride {
                        fields: Some(vec![FieldOverride {
                            field: 5,
                            component: None,
                            name: Some(Nullable::Value("Second Name".to_string())),
                            group: None,
                            note: None,
                            required: None,
                            minlength: None,
                            maxlength: None,
                            pattern: None,
                            datatype: None,
                            placeholder: None,
                            values: None,
                            template: None,
                        }]),
                    },
                );
                map
            }),
        };

        let merged = merge_schema_overrides(&[override1, override2]);
        let segments = merged.segments.unwrap();
        let pid_segment = segments.get("PID").unwrap();
        let fields = pid_segment.fields.as_ref().unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(
            fields[0].name,
            Some(Nullable::Value("Second Name".to_string()))
        );
    }

    #[test]
    fn test_merge_empty_overrides() {
        let merged = merge_schema_overrides(&[]);
        assert!(merged.segments.is_none());
    }

    #[test]
    fn test_datatype_string_to_enum_conversion() {
        let base = make_base_field(7, None, "Date Field");

        let override_ = FieldOverride {
            field: 7,
            component: None,
            name: None,
            group: None,
            note: None,
            required: None,
            minlength: None,
            maxlength: None,
            pattern: None,
            datatype: Some(Nullable::Value("date".to_string())),
            placeholder: None,
            values: None,
            template: None,
        };

        let merged = merge_field(&base, &override_);
        assert_eq!(merged.datatype, Some(DataType::Date));

        let override_datetime = FieldOverride {
            field: 7,
            component: None,
            name: None,
            group: None,
            note: None,
            required: None,
            minlength: None,
            maxlength: None,
            pattern: None,
            datatype: Some(Nullable::Value("datetime".to_string())),
            placeholder: None,
            values: None,
            template: None,
        };

        let merged_datetime = merge_field(&base, &override_datetime);
        assert_eq!(merged_datetime.datatype, Some(DataType::DateTime));
    }

    #[test]
    fn test_values_indexmap_to_hashmap_conversion() {
        let base = make_base_field(8, None, "Gender");

        let mut override_values = IndexMap::new();
        override_values.insert("M".to_string(), "Male".to_string());
        override_values.insert("F".to_string(), "Female".to_string());
        override_values.insert("O".to_string(), "Other".to_string());

        let override_ = FieldOverride {
            field: 8,
            component: None,
            name: None,
            group: None,
            note: None,
            required: None,
            minlength: None,
            maxlength: None,
            pattern: None,
            datatype: None,
            placeholder: None,
            values: Some(Nullable::Value(override_values.clone())),
            template: None,
        };

        let merged = merge_field(&base, &override_);
        let values = merged.values.unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values.get("M"), Some(&"Male".to_string()));
        assert_eq!(values.get("F"), Some(&"Female".to_string()));
        assert_eq!(values.get("O"), Some(&"Other".to_string()));
    }

    #[test]
    fn test_flexible_matching_component_override_matches_field_level_base() {
        // base has field-level entry (no component)
        let base = vec![make_base_field(3, None, "Patient ID")];

        // override specifies component
        let overrides = vec![FieldOverride {
            field: 3,
            component: Some(1),
            name: None,
            group: None,
            note: Some(Nullable::Value("8-digit MRN".to_string())),
            required: None,
            minlength: None,
            maxlength: None,
            pattern: None,
            datatype: None,
            placeholder: None,
            values: None,
            template: None,
        }];

        let merged = merge_segment_fields(&base, &overrides);

        // should match flexibly and apply the override
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].name, "Patient ID");
        assert_eq!(merged[0].note, Some("8-digit MRN".to_string()));
        // component should remain None (from base)
        assert_eq!(merged[0].component, None);
    }

    #[test]
    fn test_exact_match_takes_priority_over_flexible() {
        // base has both field-level and component-level entries
        let base = vec![
            make_base_field(3, None, "Patient ID Field"),
            make_base_field(3, Some(1), "Patient ID Component 1"),
        ];

        // override specifies component 1
        let overrides = vec![FieldOverride {
            field: 3,
            component: Some(1),
            name: None,
            group: None,
            note: Some(Nullable::Value("Component note".to_string())),
            required: None,
            minlength: None,
            maxlength: None,
            pattern: None,
            datatype: None,
            placeholder: None,
            values: None,
            template: None,
        }];

        let merged = merge_segment_fields(&base, &overrides);

        // should have two fields, exact match should be used
        assert_eq!(merged.len(), 2);
        // field-level should be unchanged
        assert_eq!(merged[0].name, "Patient ID Field");
        assert_eq!(merged[0].note, None);
        // component-level should have the override
        assert_eq!(merged[1].name, "Patient ID Component 1");
        assert_eq!(merged[1].note, Some("Component note".to_string()));
    }
}
