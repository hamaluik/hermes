//! Schema query commands for HL7 message and segment definitions.
//!
//! This module provides commands for the frontend to query the cached schema data.
//! The schema defines the structure of HL7 messages (what segments they contain)
//! and segments (what fields they contain).
//!
//! # Schema Usage in Frontend
//!
//! The frontend uses these commands to:
//! 1. **Populate segment editors**: `get_segment_schema` provides the list of fields
//!    to display in a segment editing form
//! 2. **Validate message structure**: `get_messages_schema` provides the expected
//!    segment structure for a given message type
//! 3. **Enable autocomplete**: Field names and valid values can be suggested based
//!    on schema metadata
//!
//! # Caching Behavior
//!
//! Both commands access the SchemaCache in AppData, which:
//! * Loads schemas from TOML files on first access
//! * Caches parsed schemas in memory
//! * Automatically reloads when files are modified (hot-reload during development)

use crate::{
    schema::{message::MessagesSchema, segment::Field},
    AppData,
};
use color_eyre::eyre::Context;
use tauri::State;

/// Retrieve the schema definition for a specific segment.
///
/// This command loads the segment's field definitions from the schema cache.
/// The field definitions include metadata like field names, types, validation rules,
/// and valid value sets.
///
/// # Use Case
///
/// When the user selects a segment to edit, the frontend calls this command to get
/// the list of fields. This enables the frontend to render a form with appropriate
/// input controls for each field (text inputs, date pickers, dropdowns for enumerated values).
///
/// # Arguments
/// * `segment` - Segment identifier (e.g., "PID", "PV1", "OBX")
/// * `state` - Application state containing the schema cache
///
/// # Returns
/// * `Ok(Vec<Field>)` - Field definitions for the segment
/// * `Err(String)` - Segment not found or failed to load schema file
#[tauri::command]
pub fn get_segment_schema(segment: &str, state: State<'_, AppData>) -> Result<Vec<Field>, String> {
    state
        .schema
        .get_segment(segment)
        .wrap_err_with(|| format!("Failed to load segment {segment} data"))
        .map_err(|e| format!("{e:#}"))
}

/// Retrieve the complete messages schema.
///
/// This command loads the messages.toml schema, which contains:
/// 1. A mapping of segment names to their schema file paths
/// 2. Message type definitions (e.g., ADT_A01) with expected segment structures
///
/// # Use Case
///
/// The frontend uses this schema to:
/// * Determine which segments are expected in a given message type
/// * Mark required vs optional segments in the UI
/// * Validate message structure before sending
/// * Provide autocomplete suggestions for new segments
///
/// # Hot-Reload Support
///
/// If messages.toml is modified while the application is running, this command
/// will automatically return the updated schema on the next call (no restart needed).
///
/// # Arguments
/// * `state` - Application state containing the schema cache
///
/// # Returns
/// * `Ok(MessagesSchema)` - The complete messages schema with segment paths and message definitions
/// * `Err(String)` - Failed to load or parse the messages.toml file
#[tauri::command]
pub fn get_messages_schema(state: State<'_, AppData>) -> Result<MessagesSchema, String> {
    state
        .schema
        .get_messages()
        .wrap_err_with(|| "Failed to load messages schema")
        .map_err(|e| format!("{e:#}"))
}
