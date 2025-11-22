//! HL7 message comparison commands.
//!
//! This module provides commands for comparing two HL7 messages and identifying
//! differences at the segment, field, component, and subcomponent levels.

use std::collections::{BTreeMap, HashSet};

use hl7_parser::message::{Component, Field, Repeat, Segment};
use hl7_parser::Message;
use serde::{Deserialize, Serialize};

/// Type of difference detected between two message elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffType {
    /// Element exists only in the left (original) message
    Removed,
    /// Element exists only in the right (new) message
    Added,
    /// Element exists in both but has different values
    Modified,
    /// Element is the same in both messages
    Unchanged,
}

/// A single difference at the field/component level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDiff {
    /// Field path (e.g., "PID.5.1", "MSH.9")
    pub path: String,
    /// Human-readable description of the field if known
    pub description: Option<String>,
    /// Type of difference
    pub diff_type: DiffType,
    /// Value in the left message (None if added)
    pub left_value: Option<String>,
    /// Value in the right message (None if removed)
    pub right_value: Option<String>,
    /// Character range in left message for highlighting
    pub left_range: Option<(usize, usize)>,
    /// Character range in right message for highlighting
    pub right_range: Option<(usize, usize)>,
}

/// Differences for a single segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentDiff {
    /// Segment name (e.g., "PID", "MSH")
    pub name: String,
    /// Segment occurrence index (0-based, for repeating segments)
    pub occurrence: usize,
    /// Type of difference for the segment as a whole
    pub diff_type: DiffType,
    /// Field-level differences within this segment
    pub fields: Vec<FieldDiff>,
    /// Character range in left message for the entire segment
    pub left_range: Option<(usize, usize)>,
    /// Character range in right message for the entire segment
    pub right_range: Option<(usize, usize)>,
}

/// Complete diff result for two messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDiff {
    /// Segment-level differences
    pub segments: Vec<SegmentDiff>,
    /// Summary statistics
    pub summary: DiffSummary,
}

/// Summary statistics for the diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    /// Total number of segments added
    pub segments_added: usize,
    /// Total number of segments removed
    pub segments_removed: usize,
    /// Total number of segments modified
    pub segments_modified: usize,
    /// Total number of field-level differences
    pub total_field_changes: usize,
}

/// Compare two HL7 messages and return structured differences.
///
/// This command performs a semantic comparison of two HL7 messages at multiple levels:
/// - Segment level: identifies added, removed, or modified segments
/// - Field level: within each segment, identifies field changes
/// - Component/subcomponent level: tracks changes at the finest granularity
///
/// # Comparison Strategy
///
/// Segments are matched by name and occurrence index. For example, if both messages
/// have two PID segments, PID[0] is compared to PID[0] and PID[1] to PID[1].
///
/// # Arguments
/// * `left` - The "original" or "before" message
/// * `right` - The "new" or "after" message
///
/// # Returns
/// * `Ok(MessageDiff)` - Structured diff result with all differences
/// * `Err(String)` - If either message cannot be parsed
#[tauri::command]
pub fn compare_messages(left: &str, right: &str) -> Result<MessageDiff, String> {
    let left_msg = hl7_parser::parse_message_with_lenient_newlines(left)
        .map_err(|e| format!("Failed to parse left message: {e}"))?;
    let right_msg = hl7_parser::parse_message_with_lenient_newlines(right)
        .map_err(|e| format!("Failed to parse right message: {e}"))?;

    // Build maps of segments by (name, occurrence)
    let left_segments = build_segment_map(&left_msg);
    let right_segments = build_segment_map(&right_msg);

    // Collect all unique segment keys
    let all_keys: HashSet<_> = left_segments
        .keys()
        .chain(right_segments.keys())
        .cloned()
        .collect();

    let mut segment_diffs = Vec::new();
    let mut summary = DiffSummary {
        segments_added: 0,
        segments_removed: 0,
        segments_modified: 0,
        total_field_changes: 0,
    };

    // Sort keys for consistent output (by name then occurrence)
    let mut sorted_keys: Vec<_> = all_keys.into_iter().collect();
    sorted_keys.sort();

    for (name, occurrence) in sorted_keys {
        let left_seg = left_segments.get(&(name.clone(), occurrence));
        let right_seg = right_segments.get(&(name.clone(), occurrence));

        match (left_seg, right_seg) {
            (Some(ls), Some(rs)) => {
                // Segment exists in both - compare fields
                let (fields, has_changes) =
                    compare_segment_fields(ls, rs, &name, &left_msg, &right_msg);

                let diff_type = if has_changes {
                    summary.segments_modified += 1;
                    DiffType::Modified
                } else {
                    DiffType::Unchanged
                };

                summary.total_field_changes += fields.iter().filter(|f| f.diff_type != DiffType::Unchanged).count();

                segment_diffs.push(SegmentDiff {
                    name: name.clone(),
                    occurrence,
                    diff_type,
                    fields,
                    left_range: Some((ls.range.start, ls.range.end)),
                    right_range: Some((rs.range.start, rs.range.end)),
                });
            }
            (Some(ls), None) => {
                // Segment removed
                summary.segments_removed += 1;
                let fields = extract_segment_fields(ls, &name, &left_msg, DiffType::Removed, true);
                summary.total_field_changes += fields.len();

                segment_diffs.push(SegmentDiff {
                    name: name.clone(),
                    occurrence,
                    diff_type: DiffType::Removed,
                    fields,
                    left_range: Some((ls.range.start, ls.range.end)),
                    right_range: None,
                });
            }
            (None, Some(rs)) => {
                // Segment added
                summary.segments_added += 1;
                let fields = extract_segment_fields(rs, &name, &right_msg, DiffType::Added, false);
                summary.total_field_changes += fields.len();

                segment_diffs.push(SegmentDiff {
                    name: name.clone(),
                    occurrence,
                    diff_type: DiffType::Added,
                    fields,
                    left_range: None,
                    right_range: Some((rs.range.start, rs.range.end)),
                });
            }
            (None, None) => unreachable!(),
        }
    }

    Ok(MessageDiff {
        segments: segment_diffs,
        summary,
    })
}

/// Build a map of segments keyed by (name, occurrence_index).
fn build_segment_map<'a>(
    message: &'a Message<'a>,
) -> BTreeMap<(String, usize), &'a Segment<'a>> {
    let mut map = BTreeMap::new();
    let mut occurrence_counts: BTreeMap<String, usize> = BTreeMap::new();

    for segment in message.segments() {
        let name = segment.name.to_string();
        let occurrence = *occurrence_counts.get(&name).unwrap_or(&0);
        occurrence_counts
            .entry(name.clone())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        map.insert((name, occurrence), segment);
    }

    map
}

/// Compare fields between two segments.
///
/// Returns a tuple of (field_diffs, has_any_changes).
fn compare_segment_fields<'a>(
    left: &'a Segment<'a>,
    right: &'a Segment<'a>,
    segment_name: &str,
    left_msg: &'a Message<'a>,
    right_msg: &'a Message<'a>,
) -> (Vec<FieldDiff>, bool) {
    let mut diffs = Vec::new();
    let mut has_changes = false;

    // Determine max field index
    let max_field = left.fields.len().max(right.fields.len());

    for field_idx in 0..max_field {
        let left_field = left.fields.get(field_idx);
        let right_field = right.fields.get(field_idx);

        // Field number is 1-indexed (field_idx 0 is field 1, but MSH is special)
        // For MSH, field 1 is the separator character (index 0)
        let field_num = if segment_name == "MSH" {
            field_idx + 1
        } else {
            field_idx + 1
        };

        match (left_field, right_field) {
            (Some(lf), Some(rf)) => {
                // Both fields exist - compare at component level
                compare_field_contents(
                    lf,
                    rf,
                    segment_name,
                    field_num,
                    left_msg,
                    right_msg,
                    &mut diffs,
                    &mut has_changes,
                );
            }
            (Some(lf), None) => {
                // Field removed
                let value = left_msg.separators.decode(lf.raw_value()).to_string();
                if !value.is_empty() {
                    has_changes = true;
                    diffs.push(FieldDiff {
                        path: format!("{}.{}", segment_name, field_num),
                        description: None,
                        diff_type: DiffType::Removed,
                        left_value: Some(value),
                        right_value: None,
                        left_range: Some((lf.range.start, lf.range.end)),
                        right_range: None,
                    });
                }
            }
            (None, Some(rf)) => {
                // Field added
                let value = right_msg.separators.decode(rf.raw_value()).to_string();
                if !value.is_empty() {
                    has_changes = true;
                    diffs.push(FieldDiff {
                        path: format!("{}.{}", segment_name, field_num),
                        description: None,
                        diff_type: DiffType::Added,
                        left_value: None,
                        right_value: Some(value),
                        left_range: None,
                        right_range: Some((rf.range.start, rf.range.end)),
                    });
                }
            }
            (None, None) => {
                // Both empty - no diff
            }
        }
    }

    (diffs, has_changes)
}

/// Compare field contents at the component/subcomponent level.
fn compare_field_contents<'a>(
    left: &'a Field<'a>,
    right: &'a Field<'a>,
    segment_name: &str,
    field_num: usize,
    left_msg: &'a Message<'a>,
    right_msg: &'a Message<'a>,
    diffs: &mut Vec<FieldDiff>,
    has_changes: &mut bool,
) {
    // Handle repeating fields
    let max_repeats = left.repeats.len().max(right.repeats.len());

    if max_repeats == 0 {
        // No repeats - compare raw field values
        let left_val = left_msg.separators.decode(left.raw_value()).to_string();
        let right_val = right_msg.separators.decode(right.raw_value()).to_string();

        if left_val != right_val {
            *has_changes = true;
            diffs.push(FieldDiff {
                path: format!("{}.{}", segment_name, field_num),
                description: None,
                diff_type: DiffType::Modified,
                left_value: Some(left_val),
                right_value: Some(right_val),
                left_range: Some((left.range.start, left.range.end)),
                right_range: Some((right.range.start, right.range.end)),
            });
        }
        return;
    }

    for repeat_idx in 0..max_repeats {
        let left_repeat = left.repeats.get(repeat_idx);
        let right_repeat = right.repeats.get(repeat_idx);

        let repeat_suffix = if max_repeats > 1 {
            format!("[{}]", repeat_idx + 1)
        } else {
            String::new()
        };

        match (left_repeat, right_repeat) {
            (Some(lr), Some(rr)) => {
                // Compare at component level
                compare_repeat_contents(
                    lr,
                    rr,
                    segment_name,
                    field_num,
                    &repeat_suffix,
                    left_msg,
                    right_msg,
                    diffs,
                    has_changes,
                );
            }
            (Some(lr), None) => {
                let value = left_msg.separators.decode(lr.raw_value()).to_string();
                if !value.is_empty() {
                    *has_changes = true;
                    diffs.push(FieldDiff {
                        path: format!("{}.{}{}", segment_name, field_num, repeat_suffix),
                        description: None,
                        diff_type: DiffType::Removed,
                        left_value: Some(value),
                        right_value: None,
                        left_range: Some((lr.range.start, lr.range.end)),
                        right_range: None,
                    });
                }
            }
            (None, Some(rr)) => {
                let value = right_msg.separators.decode(rr.raw_value()).to_string();
                if !value.is_empty() {
                    *has_changes = true;
                    diffs.push(FieldDiff {
                        path: format!("{}.{}{}", segment_name, field_num, repeat_suffix),
                        description: None,
                        diff_type: DiffType::Added,
                        left_value: None,
                        right_value: Some(value),
                        left_range: None,
                        right_range: Some((rr.range.start, rr.range.end)),
                    });
                }
            }
            (None, None) => {}
        }
    }
}

/// Compare repeat contents at the component level.
fn compare_repeat_contents<'a>(
    left: &'a Repeat<'a>,
    right: &'a Repeat<'a>,
    segment_name: &str,
    field_num: usize,
    repeat_suffix: &str,
    left_msg: &'a Message<'a>,
    right_msg: &'a Message<'a>,
    diffs: &mut Vec<FieldDiff>,
    has_changes: &mut bool,
) {
    let max_components = left.components.len().max(right.components.len());

    if max_components == 0 {
        // No components - compare raw repeat values
        let left_val = left_msg.separators.decode(left.raw_value()).to_string();
        let right_val = right_msg.separators.decode(right.raw_value()).to_string();

        if left_val != right_val {
            *has_changes = true;
            diffs.push(FieldDiff {
                path: format!("{}.{}{}", segment_name, field_num, repeat_suffix),
                description: None,
                diff_type: DiffType::Modified,
                left_value: Some(left_val),
                right_value: Some(right_val),
                left_range: Some((left.range.start, left.range.end)),
                right_range: Some((right.range.start, right.range.end)),
            });
        }
        return;
    }

    for comp_idx in 0..max_components {
        let left_comp = left.components.get(comp_idx);
        let right_comp = right.components.get(comp_idx);
        let comp_num = comp_idx + 1;

        match (left_comp, right_comp) {
            (Some(lc), Some(rc)) => {
                // Compare at subcomponent level
                compare_component_contents(
                    lc,
                    rc,
                    segment_name,
                    field_num,
                    repeat_suffix,
                    comp_num,
                    left_msg,
                    right_msg,
                    diffs,
                    has_changes,
                );
            }
            (Some(lc), None) => {
                let value = left_msg.separators.decode(lc.raw_value()).to_string();
                if !value.is_empty() {
                    *has_changes = true;
                    diffs.push(FieldDiff {
                        path: format!("{}.{}{}.{}", segment_name, field_num, repeat_suffix, comp_num),
                        description: None,
                        diff_type: DiffType::Removed,
                        left_value: Some(value),
                        right_value: None,
                        left_range: Some((lc.range.start, lc.range.end)),
                        right_range: None,
                    });
                }
            }
            (None, Some(rc)) => {
                let value = right_msg.separators.decode(rc.raw_value()).to_string();
                if !value.is_empty() {
                    *has_changes = true;
                    diffs.push(FieldDiff {
                        path: format!("{}.{}{}.{}", segment_name, field_num, repeat_suffix, comp_num),
                        description: None,
                        diff_type: DiffType::Added,
                        left_value: None,
                        right_value: Some(value),
                        left_range: None,
                        right_range: Some((rc.range.start, rc.range.end)),
                    });
                }
            }
            (None, None) => {}
        }
    }
}

/// Compare component contents at the subcomponent level.
fn compare_component_contents<'a>(
    left: &'a Component<'a>,
    right: &'a Component<'a>,
    segment_name: &str,
    field_num: usize,
    repeat_suffix: &str,
    comp_num: usize,
    left_msg: &'a Message<'a>,
    right_msg: &'a Message<'a>,
    diffs: &mut Vec<FieldDiff>,
    has_changes: &mut bool,
) {
    let max_subcomponents = left.subcomponents.len().max(right.subcomponents.len());

    if max_subcomponents == 0 {
        // No subcomponents - compare raw component values
        let left_val = left_msg.separators.decode(left.raw_value()).to_string();
        let right_val = right_msg.separators.decode(right.raw_value()).to_string();

        if left_val != right_val {
            *has_changes = true;
            diffs.push(FieldDiff {
                path: format!("{}.{}{}.{}", segment_name, field_num, repeat_suffix, comp_num),
                description: None,
                diff_type: DiffType::Modified,
                left_value: Some(left_val),
                right_value: Some(right_val),
                left_range: Some((left.range.start, left.range.end)),
                right_range: Some((right.range.start, right.range.end)),
            });
        }
        return;
    }

    for sub_idx in 0..max_subcomponents {
        let left_sub = left.subcomponents.get(sub_idx);
        let right_sub = right.subcomponents.get(sub_idx);
        let sub_num = sub_idx + 1;

        let path = format!(
            "{}.{}{}.{}.{}",
            segment_name, field_num, repeat_suffix, comp_num, sub_num
        );

        match (left_sub, right_sub) {
            (Some(ls), Some(rs)) => {
                let left_val = left_msg.separators.decode(ls.raw_value()).to_string();
                let right_val = right_msg.separators.decode(rs.raw_value()).to_string();

                if left_val != right_val {
                    *has_changes = true;
                    diffs.push(FieldDiff {
                        path,
                        description: None,
                        diff_type: DiffType::Modified,
                        left_value: Some(left_val),
                        right_value: Some(right_val),
                        left_range: Some((ls.range.start, ls.range.end)),
                        right_range: Some((rs.range.start, rs.range.end)),
                    });
                }
            }
            (Some(ls), None) => {
                let value = left_msg.separators.decode(ls.raw_value()).to_string();
                if !value.is_empty() {
                    *has_changes = true;
                    diffs.push(FieldDiff {
                        path,
                        description: None,
                        diff_type: DiffType::Removed,
                        left_value: Some(value),
                        right_value: None,
                        left_range: Some((ls.range.start, ls.range.end)),
                        right_range: None,
                    });
                }
            }
            (None, Some(rs)) => {
                let value = right_msg.separators.decode(rs.raw_value()).to_string();
                if !value.is_empty() {
                    *has_changes = true;
                    diffs.push(FieldDiff {
                        path,
                        description: None,
                        diff_type: DiffType::Added,
                        left_value: None,
                        right_value: Some(value),
                        left_range: None,
                        right_range: Some((rs.range.start, rs.range.end)),
                    });
                }
            }
            (None, None) => {}
        }
    }
}

/// Extract all fields from a segment as diffs (for added/removed segments).
fn extract_segment_fields<'a>(
    segment: &'a Segment<'a>,
    segment_name: &str,
    message: &'a Message<'a>,
    diff_type: DiffType,
    is_left: bool,
) -> Vec<FieldDiff> {
    let mut diffs = Vec::new();

    for (field_idx, field) in segment.fields.iter().enumerate() {
        let field_num = if segment_name == "MSH" {
            field_idx + 1
        } else {
            field_idx + 1
        };

        let value = message.separators.decode(field.raw_value()).to_string();
        if value.is_empty() {
            continue;
        }

        let (left_value, right_value) = if is_left {
            (Some(value), None)
        } else {
            (None, Some(value))
        };

        let (left_range, right_range) = if is_left {
            (Some((field.range.start, field.range.end)), None)
        } else {
            (None, Some((field.range.start, field.range.end)))
        };

        diffs.push(FieldDiff {
            path: format!("{}.{}", segment_name, field_num),
            description: None,
            diff_type,
            left_value,
            right_value,
            left_range,
            right_range,
        });
    }

    diffs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_messages() {
        let msg = "MSH|^~\\&|SEND|FAC|RCV|FAC|20250101120000||ADT^A01|12345|P|2.3\rPID|1||12345^^^MRN||Doe^John|||M";
        let result = compare_messages(msg, msg).unwrap();

        assert_eq!(result.summary.segments_added, 0);
        assert_eq!(result.summary.segments_removed, 0);
        assert_eq!(result.summary.segments_modified, 0);
        assert_eq!(result.summary.total_field_changes, 0);
    }

    #[test]
    fn test_field_modification() {
        let left = "MSH|^~\\&|SEND|FAC|RCV|FAC|20250101120000||ADT^A01|12345|P|2.3\rPID|1||12345^^^MRN||Doe^John|||M";
        let right = "MSH|^~\\&|SEND|FAC|RCV|FAC|20250101120000||ADT^A01|12345|P|2.3\rPID|1||67890^^^MRN||Doe^John|||M";
        let result = compare_messages(left, right).unwrap();

        assert_eq!(result.summary.segments_modified, 1);
        assert!(result.summary.total_field_changes > 0);

        // Find the PID.3 change
        let pid_segment = result.segments.iter().find(|s| s.name == "PID").unwrap();
        let pid3_change = pid_segment.fields.iter().find(|f| f.path == "PID.3.1").unwrap();
        assert_eq!(pid3_change.diff_type, DiffType::Modified);
        assert_eq!(pid3_change.left_value.as_deref(), Some("12345"));
        assert_eq!(pid3_change.right_value.as_deref(), Some("67890"));
    }

    #[test]
    fn test_segment_added() {
        let left = "MSH|^~\\&|SEND|FAC|RCV|FAC|20250101120000||ADT^A01|12345|P|2.3";
        let right = "MSH|^~\\&|SEND|FAC|RCV|FAC|20250101120000||ADT^A01|12345|P|2.3\rPID|1||12345^^^MRN||Doe^John|||M";
        let result = compare_messages(left, right).unwrap();

        assert_eq!(result.summary.segments_added, 1);

        let pid_segment = result.segments.iter().find(|s| s.name == "PID").unwrap();
        assert_eq!(pid_segment.diff_type, DiffType::Added);
    }

    #[test]
    fn test_segment_removed() {
        let left = "MSH|^~\\&|SEND|FAC|RCV|FAC|20250101120000||ADT^A01|12345|P|2.3\rPID|1||12345^^^MRN||Doe^John|||M";
        let right = "MSH|^~\\&|SEND|FAC|RCV|FAC|20250101120000||ADT^A01|12345|P|2.3";
        let result = compare_messages(left, right).unwrap();

        assert_eq!(result.summary.segments_removed, 1);

        let pid_segment = result.segments.iter().find(|s| s.name == "PID").unwrap();
        assert_eq!(pid_segment.diff_type, DiffType::Removed);
    }
}
