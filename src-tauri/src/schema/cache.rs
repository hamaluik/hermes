//! Schema caching with compile-time embedded data.
//!
//! This module provides a thread-safe cache for HL7 schema data embedded at compile time.
//! Schema TOML files are processed by build.rs and included via `include_str!` macros.
//!
//! # Why Embedded Schemas?
//! Embedding schemas at compile time allows distributing a single binary without external
//! data files. The trade-off is that schema changes require recompilation.
//!
//! # Extension Overrides
//! Extensions can still provide runtime schema overrides that merge with the base schemas.
//! These are applied via `set_extension_overrides()` and affect all subsequent
//! `get_segment()` calls.

use color_eyre::{eyre::Context, Result};
use std::{collections::HashMap, sync::RwLock};

use super::{message::MessagesSchema, segment::Field};
use crate::extensions::types::SchemaOverride;

// include the generated embedded schemas module
include!(concat!(env!("OUT_DIR"), "/embedded_schemas.rs"));

/// Thread-safe cache for HL7 schema data with extension override support.
///
/// Base schemas are parsed once from compile-time embedded TOML content.
/// Extension overrides can be applied at runtime to customise field definitions.
pub struct SchemaCache {
    /// Parsed messages schema (message types and segment mappings)
    messages: MessagesSchema,

    /// Parsed segment schemas keyed by segment name (e.g., "PID", "MSH")
    segments: HashMap<String, Vec<Field>>,

    /// Extension schema overrides to apply on top of base schemas.
    extension_overrides: RwLock<Option<SchemaOverride>>,
}

impl SchemaCache {
    /// Create a new schema cache by parsing embedded schema data.
    ///
    /// All schemas are parsed once at initialisation. Extension overrides
    /// are applied later via `set_extension_overrides()`.
    ///
    /// # Returns
    /// * `Ok(SchemaCache)` - Initialized cache with all schemas loaded
    /// * `Err` - Failed to parse embedded schema content
    pub fn new() -> Result<Self> {
        let messages = MessagesSchema::parse(MESSAGES_TOML)
            .wrap_err("failed to parse embedded messages.toml")?;

        let mut segments = HashMap::new();
        for (segment_name, toml_content) in SEGMENT_SCHEMAS {
            let fields = Field::parse(toml_content)
                .wrap_err_with(|| format!("failed to parse embedded schema for {segment_name}"))?;
            segments.insert((*segment_name).to_string(), fields);
        }

        Ok(Self {
            messages,
            segments,
            extension_overrides: RwLock::new(None),
        })
    }

    /// Get a segment schema with extension overrides applied.
    ///
    /// Retrieves the base segment schema and applies any extension overrides
    /// that have been set. If no overrides are present or the segment has no
    /// overrides, the base schema is returned unchanged.
    ///
    /// # Arguments
    /// * `segment` - Segment name to retrieve (e.g., "PID", "MSH")
    ///
    /// # Returns
    /// * `Ok(Vec<Field>)` - Field definitions for the segment with overrides applied
    /// * `Err` - Segment not found in schema
    pub fn get_segment(&self, segment: &str) -> Result<Vec<Field>> {
        let base_fields = self
            .segments
            .get(segment)
            .cloned()
            .ok_or_else(|| color_eyre::eyre::eyre!("segment {segment} not found in schema"))?;

        let overrides = self
            .extension_overrides
            .read()
            .expect("can read extension overrides");

        if let Some(ref schema_override) = *overrides {
            if let Some(ref segments) = schema_override.segments {
                if let Some(segment_override) = segments.get(segment) {
                    if let Some(ref field_overrides) = segment_override.fields {
                        return Ok(crate::schema::merge::merge_segment_fields(
                            &base_fields,
                            field_overrides,
                        ));
                    }
                }
            }
        }

        Ok(base_fields)
    }

    /// Set the extension schema overrides.
    ///
    /// Called by ExtensionHost after merging all extension schemas.
    /// The overrides will be applied to all subsequent calls to `get_segment()`.
    ///
    /// # Arguments
    /// * `overrides` - The merged schema override to apply, or None to clear overrides
    pub fn set_extension_overrides(&self, overrides: Option<SchemaOverride>) {
        let mut ext_overrides = self
            .extension_overrides
            .write()
            .expect("can write extension overrides");
        *ext_overrides = overrides;
    }

    /// Get the messages schema.
    ///
    /// Returns the parsed messages schema containing message type definitions
    /// and segment path mappings.
    ///
    /// # Returns
    /// The messages schema (cloned for thread safety)
    pub fn get_messages(&self) -> MessagesSchema {
        self.messages.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::types::{FieldOverride, Nullable, SchemaOverride, SegmentOverride};
    use indexmap::IndexMap;

    #[test]
    fn test_schema_cache_loads_embedded_schemas() {
        let cache = SchemaCache::new().expect("can create cache");

        // verify messages schema loaded
        let messages = cache.get_messages();
        assert!(
            messages.segments.contains_key("PID"),
            "messages should contain PID segment mapping"
        );
        assert!(
            messages.segments.contains_key("MSH"),
            "messages should contain MSH segment mapping"
        );

        // verify segment schemas loaded
        let pid_fields = cache.get_segment("PID").expect("can get PID segment");
        assert!(!pid_fields.is_empty(), "PID should have fields");

        let msh_fields = cache.get_segment("MSH").expect("can get MSH segment");
        assert!(!msh_fields.is_empty(), "MSH should have fields");
    }

    #[test]
    fn test_schema_cache_with_overrides() {
        let cache = SchemaCache::new().expect("can create cache");

        // get base PID segment fields
        let base_fields = cache.get_segment("PID").expect("can get PID segment");
        let field_3_base = base_fields
            .iter()
            .find(|f| f.field == 3 && f.component.is_none())
            .expect("can find field 3");
        let original_name = field_3_base.name.clone();
        let original_note = field_3_base.note.clone();

        // create an override that changes field 3 name and adds a note
        let mut segments = IndexMap::new();
        segments.insert(
            "PID".to_string(),
            SegmentOverride {
                fields: Some(vec![FieldOverride {
                    field: 3,
                    component: None,
                    name: Some(Nullable::Value("Overridden MRN".to_string())),
                    group: None,
                    note: Some(Nullable::Value("This is an override note".to_string())),
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

        let override_schema = SchemaOverride {
            segments: Some(segments),
        };

        // set the override
        cache.set_extension_overrides(Some(override_schema));

        // get fields again and check that override was applied
        let fields_with_override = cache
            .get_segment("PID")
            .expect("can get PID segment with override");
        let field_3_override = fields_with_override
            .iter()
            .find(|f| f.field == 3 && f.component.is_none())
            .expect("can find overridden field 3");

        assert_eq!(field_3_override.name, "Overridden MRN");
        assert_eq!(
            field_3_override.note,
            Some("This is an override note".to_string())
        );

        // clear overrides
        cache.set_extension_overrides(None);

        // get fields again and check that base is restored
        let fields_restored = cache
            .get_segment("PID")
            .expect("can get PID segment after clear");
        let field_3_restored = fields_restored
            .iter()
            .find(|f| f.field == 3 && f.component.is_none())
            .expect("can find restored field 3");

        assert_eq!(field_3_restored.name, original_name);
        assert_eq!(field_3_restored.note, original_note);
    }
}
