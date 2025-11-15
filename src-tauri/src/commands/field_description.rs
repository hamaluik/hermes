//! Field description lookup from HL7 standard specifications.
//!
//! This module provides a command for retrieving human-readable descriptions of
//! HL7 segments, fields, and components from the standard specification and
//! standard specifications.
//!
//! # Description Hierarchy
//!
//! Descriptions can be requested at three levels of granularity:
//! 1. **Segment**: What this segment represents (e.g., "Patient Identification")
//! 2. **Field**: What this field represents (e.g., "Patient ID")
//! 3. **Component**: What a specific component represents (e.g., "ID Number")
//!
//! # Version Handling
//!
//! Currently hard-coded to version "2.5.1" for consistency. This version is widely
//! used in healthcare systems and provides comprehensive field definitions.
//!
//! # Why Hard-Coded Version?
//!
//! While the message itself may specify a different HL7 version (in MSH.12), using
//! a consistent version for descriptions ensures predictable behavior. The field
//! definitions are largely compatible across HL7 v2.x versions, and 2.5.1 provides
//! a good balance of completeness and compatibility.

use crate::spec::std_spec::{describe_component, describe_field, segment_description};

/// Get a description for a segment, field, or component from the HL7 standard.
///
/// This command is invoked by the frontend when the cursor moves within the message
/// editor. The description is displayed in a tooltip or side panel to help users
/// understand what data should go in each field.
///
/// # Description Sources
///
/// Descriptions come from two sources, with priority given to the first:
/// 1. **HL7 system-specific descriptions** (from ``) for fields used
///    in blood banking and transfusion medicine
/// 2. **Standard HL7 descriptions** (from `std_spec.rs`) for general HL7 fields
///
/// # Granularity Selection
///
/// The level of description returned depends on which parameters are provided:
/// * **segment only**: Returns segment-level description (e.g., "PID - Patient Identification")
/// * **segment + field**: Returns field-level description (e.g., "PID.3 - Patient Identifier List")
/// * **segment + field + component**: Returns component-level description (e.g., "PID.3.1 - ID Number")
///
/// # Arguments
/// * `segment` - Segment identifier (e.g., "PID", "MSH", "OBX")
/// * `field` - Optional field number (1-based, matching HL7 notation)
/// * `component` - Optional component number (1-based, requires field to be set)
///
/// # Returns
/// Human-readable description string, or an empty string if no description is available
#[tauri::command]
pub fn get_std_description(
    segment: &str,
    field: Option<usize>,
    component: Option<usize>,
) -> String {
    let version = "2.5.1";
    match (field, component) {
        (Some(field), Some(component)) => describe_component(version, segment, field, component),
        (Some(field), None) => describe_field(version, segment, field),
        _ => segment_description(version, segment),
    }
}
