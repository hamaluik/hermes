//! Message schema definitions loaded from TOML.
//!
//! This module defines the structure of the messages.toml schema file, which maps:
//! 1. Segment names to their schema file paths
//! 2. Message types to their expected segment structures
//!
//! # Schema File Format
//! The messages.toml file has two main sections:
//!
//! ```toml
//! [segments]
//! PID = "segments/pid.toml"
//! PV1 = "segments/pv1.toml"
//!
//! [[message.ADT_A01]]
//! name = "MSH"
//! required = true
//!
//! [[message.ADT_A01]]
//! name = "PID"
//! required = true
//! ```

use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Map of segment names to their schema file paths (relative to data directory).
pub type SegmentPaths = HashMap<String, String>;

/// Metadata about a segment's role in a message type.
///
/// Used to define the expected structure of specific message types
/// (e.g., ADT^A01 requires MSH, EVN, PID, PV1 segments).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SegmentMetadata {
    /// Segment identifier (e.g., "MSH", "PID", "PV1")
    pub name: String,
    /// Whether this segment is required in the message type
    pub required: Option<bool>,
}

/// Top-level messages schema loaded from messages.toml.
///
/// Contains two main mappings:
/// 1. Segment names → schema file paths
/// 2. Message types → expected segment structures
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MessagesSchema {
    /// Map of segment names to their TOML schema file paths
    pub segments: SegmentPaths,
    /// Map of message types to their expected segment structures
    ///
    /// Keys are message type identifiers like "ADT_A01", "ORM_O01"
    /// Values are ordered lists of segments expected in that message type
    pub message: HashMap<String, Vec<SegmentMetadata>>,
}

impl MessagesSchema {
    /// Parse a messages schema from TOML content.
    ///
    /// # Arguments
    /// * `contents` - TOML string content
    ///
    /// # Returns
    /// * `Ok(MessagesSchema)` - Parsed schema
    /// * `Err` - Failed to parse the TOML content
    pub fn parse(contents: &str) -> Result<Self> {
        let schema: MessagesSchema =
            toml::from_str(contents).wrap_err("failed to parse messages schema")?;
        Ok(schema)
    }
}
