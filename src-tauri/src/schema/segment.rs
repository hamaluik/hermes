//! Segment field definitions loaded from TOML.
//!
//! This module defines the structure of segment schema files (e.g., pid.toml, pv1.toml).
//! Each segment schema describes the fields that can appear in that segment, along with
//! metadata about validation, display, and behavior.
//!
//! # Schema File Format
//! Segment TOML files contain a `[[fields]]` array:
//!
//! ```toml
//! [[fields]]
//! field = 3
//! name = "Patient ID"
//! required = true
//! maxlength = 20
//!
//! [[fields]]
//! field = 5
//! component = 1
//! name = "Family Name"
//! ```

use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

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
    /// Optional grouping identifier for UI organization
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
}

/// Wrapper for deserializing TOML segment files.
///
/// TOML segment files have a `[[fields]]` array at the top level.
#[derive(Debug, Deserialize, Serialize)]
struct Fields {
    fields: Vec<Field>,
}

impl Field {
    /// Load field definitions from a TOML segment schema file.
    ///
    /// This method is intentionally synchronous (not async) to avoid requiring a Mutex
    /// around the entire SchemaCache, which would defeat the purpose of using RwLock
    /// for concurrent access. File I/O is relatively fast, so blocking is acceptable here.
    ///
    /// # Arguments
    /// * `path` - Path to the segment schema TOML file
    ///
    /// # Returns
    /// * `Ok(Vec<Field>)` - Parsed field definitions
    /// * `Err` - Failed to read or parse the TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Self>> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .wrap_err_with(|| format!("Failed to read file {:?}", path.display()))?;
        let fields: Fields = toml::from_str(&contents)
            .wrap_err_with(|| format!("Failed to parse file {:?}", path.display()))?;
        Ok(fields.fields)
    }
}
