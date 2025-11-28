//! Segment field definitions loaded from TOML.
//!
//! This module defines the structure of segment schema files (e.g., pid.toml, pv1.toml).
//! Each segment schema describes the fields that can appear in that segment, along with
//! metadata about validation, display, and behaviour.
//!
//! # Schema File Format
//!
//! Segment TOML files contain a `[[fields]]` array. Each field definition specifies
//! the field number, human-readable name, and optional metadata for validation,
//! display, and template generation.
//!
//! ```toml
//! [[fields]]
//! field = 3
//! name = "Patient ID"
//! required = true
//! maxlength = 20
//! template = "MRN123456"
//!
//! [[fields]]
//! field = 5
//! component = 1
//! name = "Family Name"
//! template = "Doe"
//! ```
//!
//! ## Template Values
//!
//! The `template` field provides default values used when generating messages via
//! **File > New from Template**. Special values:
//!
//! * `{auto}` - Placeholder for values generated at send time (timestamps, control IDs)
//! * Empty string - Field left blank intentionally
//! * Regular value - Used directly in the generated message

use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Field data type for special handling.
///
/// Certain field types require custom rendering or validation logic
/// in the frontend (e.g., date pickers, timestamp formatting).
#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// Date-only field (YYYYMMDD)
    Date,
    /// Date and time field (YYYYMMDDHHMMSS)
    DateTime,
}

/// Definition of a field or component within an HL7 segment.
///
/// Fields can be either:
/// * **Field-level** (component = None): Represents the entire field
/// * **Component-level** (component = Some(n)): Represents a specific component within a field
///
/// The schema uses this same structure for both, distinguishing them by the presence
/// of the `component` value.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Field {
    /// Field number (1-based, as used in HL7 notation)
    pub field: u8,
    /// Human-readable field name
    pub name: String,
    /// Component number if this represents a specific component (1-based)
    pub component: Option<u8>,
    /// Optional grouping identifier for UI organisation
    pub group: Option<String>,
    /// Optional filter expression for conditional display based on trigger event
    pub trigger_filter: Option<String>,
    /// Minimum length for validation
    pub minlength: Option<u16>,
    /// Maximum length for validation
    pub maxlength: Option<u16>,
    /// Placeholder text for empty fields in the UI
    pub placeholder: Option<String>,
    /// Whether this field is required (for validation)
    pub required: Option<bool>,
    /// Special data type for custom rendering/validation
    pub datatype: Option<DataType>,
    /// Regex pattern for validation
    pub pattern: Option<String>,
    /// Explanatory note displayed in the UI
    pub note: Option<String>,
    /// Map of valid values (code → description) for enumerated fields
    pub values: Option<HashMap<String, String>>,
    /// Template/example value used when generating message templates
    pub template: Option<String>,
}

/// Wrapper for deserializing TOML segment files.
///
/// TOML segment files have a `[[fields]]` array at the top level.
#[derive(Debug, Deserialize, Serialize)]
struct Fields {
    fields: Vec<Field>,
}

impl Field {
    /// Parse field definitions from TOML content.
    ///
    /// # Arguments
    /// * `contents` - TOML string content
    ///
    /// # Returns
    /// * `Ok(Vec<Field>)` - Parsed field definitions
    /// * `Err` - Failed to parse the TOML content
    pub fn parse(contents: &str) -> Result<Vec<Self>> {
        let fields: Fields = toml::from_str(contents).wrap_err("failed to parse segment schema")?;
        Ok(fields.fields)
    }
}
