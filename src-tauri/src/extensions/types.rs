//! Shared type definitions for the extension system.
//!
//! These types are used for configuration, lifecycle management, and
//! JSON-RPC message parameters/results.

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

use jiff::Timestamp;

// ============================================================================
// Nullable type for schema overrides
// ============================================================================

/// A type that distinguishes between a value, explicit null, and absence.
///
/// This is used in `FieldOverride` to support three distinct states:
/// - `Option::None` - property was absent from JSON (inherit from base schema)
/// - `Option::Some(Nullable::Null)` - property was explicitly null (unset inherited value)
/// - `Option::Some(Nullable::Value(T))` - property has a value (override with this value)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nullable<T> {
    /// Property is set to a value.
    Value(T),
    /// Property is explicitly null (unset inherited value).
    Null,
}

impl<T: Serialize> Serialize for Nullable<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Nullable::Value(value) => value.serialize(serializer),
            Nullable::Null => serializer.serialize_none(),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Nullable<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(|opt| match opt {
            Some(value) => Nullable::Value(value),
            None => Nullable::Null,
        })
    }
}

/// Helper module for `Option<Nullable<T>>` serialization and deserialization.
///
/// This module provides custom serde functions that properly distinguish between:
/// - Absent field -> `None` (skipped in serialization)
/// - Explicit JSON `null` -> `Some(Nullable::Null)` (serialized as `null`)
/// - JSON value -> `Some(Nullable::Value(t))` (serialized as value)
mod option_nullable {
    use super::Nullable;
    use serde::de::{self, Deserialize, Deserializer};
    use serde::{Serialize, Serializer};

    /// Custom deserializer that handles the distinction between absent and null.
    ///
    /// This works by deserializing as `Option<T>` (not `Option<Option<T>>`), but
    /// using a visitor that can detect the difference between a missing field and
    /// an explicit null. When combined with `#[serde(default)]`, missing fields
    /// result in `None`, while explicit null or values go through the deserializer.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<Nullable<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        struct NullableVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T> de::Visitor<'de> for NullableVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Option<Nullable<T>>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a value or null")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // explicit null
                Ok(Some(Nullable::Null))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // explicit null (some formats use visit_none for null)
                Ok(Some(Nullable::Null))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                // explicit value
                T::deserialize(deserializer).map(|v| Some(Nullable::Value(v)))
            }
        }

        deserializer.deserialize_option(NullableVisitor(std::marker::PhantomData))
    }

    /// Custom serializer that matches the deserializer behavior.
    ///
    /// Note: The `None` branch is effectively dead code when used with
    /// `skip_serializing_if = "is_none"`, since those fields are skipped
    /// entirely. It's included for completeness if this serializer is used
    /// without the skip attribute.
    pub fn serialize<S, T>(value: &Option<Nullable<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        match value {
            None => serializer.serialize_none(),
            Some(nullable) => nullable.serialize(serializer),
        }
    }

    /// Helper to determine if we should skip serializing this value.
    /// Only skip if it's `None` (absent field).
    pub fn is_none<T>(value: &Option<Nullable<T>>) -> bool {
        value.is_none()
    }
}

/// Extension configuration stored in settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// Absolute path to the extension executable.
    pub path: String,

    /// Command-line arguments to pass to the extension.
    #[serde(default)]
    pub args: Vec<String>,

    /// Additional environment variables to set.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether the extension is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Extension lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionState {
    /// Process spawned, waiting for initialize.
    Starting,
    /// Initialize sent, waiting for response.
    Initializing,
    /// Ready to handle commands.
    Running,
    /// Shutdown sent, waiting for response.
    ShuttingDown,
    /// Clean shutdown completed.
    Stopped,
    /// Error occurred, with message.
    Failed(String),
}

impl ExtensionState {
    /// Check if the extension is in a state where it can handle commands.
    pub fn is_running(&self) -> bool {
        matches!(self, ExtensionState::Running)
    }

    /// Check if the extension has terminated (stopped or failed).
    pub fn is_terminated(&self) -> bool {
        matches!(self, ExtensionState::Stopped | ExtensionState::Failed(_))
    }
}

/// Log entry for an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionLog {
    /// Timestamp of the log entry.
    pub timestamp: Timestamp,

    /// Log level.
    pub level: LogLevel,

    /// Log message.
    pub message: String,
}

/// Log level for extension events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

/// Metadata returned by an extension during initialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMetadata {
    /// Human-readable extension name.
    pub name: String,

    /// Extension version string.
    pub version: String,

    /// Brief description of the extension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// List of authors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,

    /// Homepage or documentation URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Capabilities declared by the extension.
    #[serde(default)]
    pub capabilities: Capabilities,

    /// Toolbar buttons to add to the UI.
    #[serde(default, rename = "toolbarButtons")]
    pub toolbar_buttons: Vec<ToolbarButton>,

    /// Schema overrides provided by the extension.
    #[serde(default)]
    pub schema: Option<SchemaOverride>,
}

/// Extension capabilities declaration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capabilities {
    /// Commands the extension can handle.
    #[serde(default)]
    pub commands: Vec<String>,

    /// Whether the extension provides schema overrides.
    #[serde(default, rename = "schemaProvider")]
    pub schema_provider: bool,
}

/// Toolbar button definition from an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolbarButton {
    /// Unique identifier for the button.
    pub id: String,

    /// Tooltip/label text.
    pub label: String,

    /// SVG icon markup (should use currentColor for theme compatibility).
    pub icon: String,

    /// Command ID to execute when clicked.
    pub command: String,

    /// Optional visual grouping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

/// Schema overrides provided by an extension.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaOverride {
    /// Segment-level overrides keyed by segment name (e.g., "PID", "OBX").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<IndexMap<String, SegmentOverride>>,
}

/// Overrides for a single segment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SegmentOverride {
    /// Field-level overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldOverride>>,
}

/// Overrides for a single field within a segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOverride {
    /// 1-based field number.
    pub field: u32,

    /// 1-based component number (if overriding a component).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<u32>,

    /// Override the field name.
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(s))` = set name to `s`
    /// - `Some(Nullable::Null)` = unset inherited name
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub name: Option<Nullable<String>>,

    /// Override the field group.
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(s))` = set group to `s`
    /// - `Some(Nullable::Null)` = unset inherited group
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub group: Option<Nullable<String>>,

    /// Override the field note/description.
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(s))` = set note to `s`
    /// - `Some(Nullable::Null)` = unset inherited note
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub note: Option<Nullable<String>>,

    /// Override whether the field is required.
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(b))` = set required to `b`
    /// - `Some(Nullable::Null)` = unset inherited required
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub required: Option<Nullable<bool>>,

    /// Override minimum length.
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(n))` = set minlength to `n`
    /// - `Some(Nullable::Null)` = unset inherited minlength
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub minlength: Option<Nullable<u32>>,

    /// Override maximum length.
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(n))` = set maxlength to `n`
    /// - `Some(Nullable::Null)` = unset inherited maxlength
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub maxlength: Option<Nullable<u32>>,

    /// Override validation pattern (regex).
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(s))` = set pattern to `s`
    /// - `Some(Nullable::Null)` = unset inherited pattern
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub pattern: Option<Nullable<String>>,

    /// Override datatype ("date" | "datetime").
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(s))` = set datatype to `s`
    /// - `Some(Nullable::Null)` = unset inherited datatype
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub datatype: Option<Nullable<String>>,

    /// Override placeholder text.
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(s))` = set placeholder to `s`
    /// - `Some(Nullable::Null)` = unset inherited placeholder
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub placeholder: Option<Nullable<String>>,

    /// Override allowed values (code -> display name).
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(map))` = set values to `map`
    /// - `Some(Nullable::Null)` = unset inherited values
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub values: Option<Nullable<IndexMap<String, String>>>,

    /// Override template value.
    /// - `None` = inherit from base schema
    /// - `Some(Nullable::Value(s))` = set template to `s`
    /// - `Some(Nullable::Null)` = unset inherited template
    #[serde(
        default,
        serialize_with = "option_nullable::serialize",
        deserialize_with = "option_nullable::deserialize",
        skip_serializing_if = "option_nullable::is_none"
    )]
    pub template: Option<Nullable<String>>,
}

// ============================================================================
// Initialize handshake types
// ============================================================================

/// Parameters for the `initialize` request sent by Hermes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Hermes application version.
    #[serde(rename = "hermesVersion")]
    pub hermes_version: String,

    /// Extension API version.
    #[serde(rename = "apiVersion")]
    pub api_version: String,

    /// Path to the extension's data directory.
    #[serde(rename = "dataDirectory")]
    pub data_directory: String,
}

/// Result of a successful `initialize` response from an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Human-readable extension name.
    pub name: String,

    /// Extension version string.
    pub version: String,

    /// Brief description of the extension.
    #[serde(default)]
    pub description: Option<String>,

    /// List of authors.
    #[serde(default)]
    pub authors: Option<Vec<String>>,

    /// Homepage or documentation URL.
    #[serde(default)]
    pub homepage: Option<String>,

    /// Capabilities declared by the extension.
    #[serde(default)]
    pub capabilities: Capabilities,

    /// Toolbar buttons to add to the UI.
    #[serde(default, rename = "toolbarButtons")]
    pub toolbar_buttons: Vec<ToolbarButton>,

    /// Schema overrides provided by the extension.
    #[serde(default)]
    pub schema: Option<SchemaOverride>,
}

impl From<InitializeResult> for ExtensionMetadata {
    fn from(result: InitializeResult) -> Self {
        Self {
            name: result.name,
            version: result.version,
            description: result.description,
            authors: result.authors,
            homepage: result.homepage,
            capabilities: result.capabilities,
            toolbar_buttons: result.toolbar_buttons,
            schema: result.schema,
        }
    }
}

// ============================================================================
// Shutdown types
// ============================================================================

/// Parameters for the `shutdown` request sent by Hermes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownParams {
    /// Reason for shutdown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<ShutdownReason>,
}

/// Reason for extension shutdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShutdownReason {
    /// Hermes is closing.
    Closing,
    /// Extension was disabled by user.
    Disabled,
    /// Extensions are being reloaded.
    Reload,
    /// An error occurred.
    Error,
}

// ============================================================================
// Command execution types
// ============================================================================

/// Parameters for `command/execute` notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecuteParams {
    /// Command identifier to execute.
    pub command: String,
}

// ============================================================================
// Editor operation types
// ============================================================================

/// Message format for editor operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageFormat {
    Hl7,
    Json,
    Yaml,
    Toml,
}

/// Parameters for `editor/getMessage` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMessageParams {
    /// Desired output format.
    pub format: MessageFormat,
}

/// Result of `editor/getMessage` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMessageResult {
    /// Message content in the requested format.
    pub message: String,

    /// Whether the message has an associated file.
    #[serde(rename = "hasFile")]
    pub has_file: bool,

    /// File path if the message is saved.
    #[serde(rename = "filePath", skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

/// Parameters for `editor/setMessage` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetMessageParams {
    /// New message content.
    pub message: String,

    /// Format of the provided message.
    pub format: MessageFormat,
}

/// Result of `editor/setMessage` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetMessageResult {
    /// Whether the operation succeeded.
    pub success: bool,

    /// Error message if the operation failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Parameters for `editor/patchMessage` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchMessageParams {
    /// List of patches to apply.
    pub patches: Vec<Patch>,
}

/// A single patch operation for `editor/patchMessage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    /// HL7 path to the target (e.g., "PID.5.1", "OBX[2].5", "NK1").
    pub path: String,

    /// New value to set (if not removing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Remove an entire segment (path must be segment name only, e.g., "NK1" or "OBX[2]").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<bool>,

    /// Create a new segment (path must be segment name only, e.g., "NK1").
    /// Fields are auto-created when setting values; this is only for segments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create: Option<bool>,
}

/// Result of `editor/patchMessage` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchMessageResult {
    /// Whether all patches were applied successfully.
    pub success: bool,

    /// Number of patches that were applied.
    #[serde(rename = "patchesApplied")]
    pub patches_applied: usize,

    /// Errors for patches that failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<PatchError>>,
}

/// Error details for a failed patch operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchError {
    /// Index of the patch in the original array.
    pub index: usize,

    /// Path that failed.
    pub path: String,

    /// Error message.
    pub message: String,
}

// ============================================================================
// UI operation types
// ============================================================================

/// Parameters for `ui/openWindow` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWindowParams {
    /// URL to load in the window (must be http or https).
    pub url: String,

    /// Window title.
    pub title: String,

    /// Window width in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Window height in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Whether the window should be modal (blocks parent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modal: Option<bool>,

    /// Whether the window is resizable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resizable: Option<bool>,
}

/// Result of `ui/openWindow` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWindowResult {
    /// Unique identifier for the created window.
    #[serde(rename = "windowId")]
    pub window_id: String,
}

/// Parameters for `ui/closeWindow` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseWindowParams {
    /// Window identifier to close.
    #[serde(rename = "windowId")]
    pub window_id: String,
}

/// Result of `ui/closeWindow` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseWindowResult {
    /// Whether the window was closed successfully.
    pub success: bool,
}

/// Parameters for `window/closed` notification sent to extensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowClosedParams {
    /// Window identifier that was closed.
    #[serde(rename = "windowId")]
    pub window_id: String,

    /// Reason the window was closed.
    pub reason: WindowClosedReason,
}

/// Reason a window was closed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowClosedReason {
    /// User clicked the close button.
    User,
    /// Extension called closeWindow.
    Extension,
    /// Hermes is shutting down.
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_config_defaults() {
        let json = r#"{"path": "/usr/bin/ext"}"#;
        let config: ExtensionConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.path, "/usr/bin/ext");
        assert!(config.args.is_empty());
        assert!(config.env.is_empty());
        assert!(config.enabled);
    }

    #[test]
    fn test_extension_state_is_running() {
        assert!(ExtensionState::Running.is_running());
        assert!(!ExtensionState::Starting.is_running());
        assert!(!ExtensionState::Failed("error".to_string()).is_running());
    }

    #[test]
    fn test_extension_state_is_terminated() {
        assert!(ExtensionState::Stopped.is_terminated());
        assert!(ExtensionState::Failed("error".to_string()).is_terminated());
        assert!(!ExtensionState::Running.is_terminated());
        assert!(!ExtensionState::Starting.is_terminated());
    }

    #[test]
    fn test_initialize_result_to_metadata() {
        let result = InitializeResult {
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test extension".to_string()),
            authors: Some(vec!["Test Author".to_string()]),
            homepage: None,
            capabilities: Capabilities {
                commands: vec!["test/command".to_string()],
                schema_provider: false,
            },
            toolbar_buttons: vec![],
            schema: None,
        };

        let metadata: ExtensionMetadata = result.into();
        assert_eq!(metadata.name, "Test Extension");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, Some("A test extension".to_string()));
        assert_eq!(metadata.capabilities.commands.len(), 1);
    }

    #[test]
    fn test_message_format_serialization() {
        assert_eq!(
            serde_json::to_string(&MessageFormat::Hl7).unwrap(),
            "\"hl7\""
        );
        assert_eq!(
            serde_json::to_string(&MessageFormat::Json).unwrap(),
            "\"json\""
        );
    }

    #[test]
    fn test_shutdown_reason_serialization() {
        assert_eq!(
            serde_json::to_string(&ShutdownReason::Closing).unwrap(),
            "\"closing\""
        );
        assert_eq!(
            serde_json::to_string(&ShutdownReason::Reload).unwrap(),
            "\"reload\""
        );
    }

    #[test]
    fn test_toolbar_button_serialization() {
        let button = ToolbarButton {
            id: "btn-1".to_string(),
            label: "Test Button".to_string(),
            icon: "<svg></svg>".to_string(),
            command: "test/action".to_string(),
            group: Some("tools".to_string()),
        };

        let json = serde_json::to_string(&button).unwrap();
        assert!(json.contains("\"id\":\"btn-1\""));
        assert!(json.contains("\"command\":\"test/action\""));
        assert!(json.contains("\"group\":\"tools\""));
    }

    #[test]
    fn test_patch_serialization() {
        let patch = Patch {
            path: "PID.5.1".to_string(),
            value: Some("DOE".to_string()),
            remove: None,
            create: Some(true),
        };

        let json = serde_json::to_string(&patch).unwrap();
        assert!(json.contains("\"path\":\"PID.5.1\""));
        assert!(json.contains("\"value\":\"DOE\""));
        assert!(json.contains("\"create\":true"));
        assert!(!json.contains("\"remove\"")); // should be skipped when None
    }

    // ========================================================================
    // Nullable<T> tests
    // ========================================================================

    #[test]
    fn test_nullable_value_serialization() {
        let nullable = Nullable::Value("test".to_string());
        let json = serde_json::to_string(&nullable).unwrap();
        assert_eq!(json, "\"test\"");
    }

    #[test]
    fn test_nullable_null_serialization() {
        let nullable: Nullable<String> = Nullable::Null;
        let json = serde_json::to_string(&nullable).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn test_nullable_value_deserialization() {
        let json = "\"test\"";
        let nullable: Nullable<String> = serde_json::from_str(json).unwrap();
        assert_eq!(nullable, Nullable::Value("test".to_string()));
    }

    #[test]
    fn test_nullable_null_deserialization() {
        let json = "null";
        let nullable: Nullable<String> = serde_json::from_str(json).unwrap();
        assert_eq!(nullable, Nullable::Null);
    }

    #[test]
    fn test_option_nullable_none_skipped_in_serialization() {
        #[derive(Serialize)]
        struct TestStruct {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            field: Option<Nullable<String>>,
        }

        let test = TestStruct { field: None };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_option_nullable_value_serialization() {
        #[derive(Serialize)]
        struct TestStruct {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            field: Option<Nullable<String>>,
        }

        let test = TestStruct {
            field: Some(Nullable::Value("test".to_string())),
        };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, "{\"field\":\"test\"}");
    }

    #[test]
    fn test_option_nullable_null_serialization() {
        #[derive(Serialize)]
        struct TestStruct {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            field: Option<Nullable<String>>,
        }

        let test = TestStruct {
            field: Some(Nullable::Null),
        };
        let json = serde_json::to_string(&test).unwrap();
        assert_eq!(json, "{\"field\":null}");
    }

    #[test]
    fn test_field_override_with_null_values() {
        let json = r#"{
            "field": 5,
            "name": null,
            "group": "test_group",
            "required": null,
            "maxlength": 100
        }"#;

        let override_: FieldOverride = serde_json::from_str(json).unwrap();
        assert_eq!(override_.field, 5);
        assert_eq!(override_.component, None);
        assert_eq!(override_.name, Some(Nullable::Null));
        assert_eq!(
            override_.group,
            Some(Nullable::Value("test_group".to_string()))
        );
        assert_eq!(override_.note, None);
        assert_eq!(override_.required, Some(Nullable::Null));
        assert_eq!(override_.minlength, None);
        assert_eq!(override_.maxlength, Some(Nullable::Value(100)));
        assert_eq!(override_.pattern, None);
        assert_eq!(override_.datatype, None);
        assert_eq!(override_.placeholder, None);
        assert_eq!(override_.values, None);
        assert_eq!(override_.template, None);
    }

    #[test]
    fn test_field_override_round_trip() {
        let mut values_map = IndexMap::new();
        values_map.insert("M".to_string(), "Male".to_string());
        values_map.insert("F".to_string(), "Female".to_string());

        let override_ = FieldOverride {
            field: 5,
            component: None,
            name: Some(Nullable::Value("Patient Name".to_string())),
            group: Some(Nullable::Null),
            note: None,
            required: Some(Nullable::Value(true)),
            minlength: Some(Nullable::Null),
            maxlength: Some(Nullable::Value(50)),
            pattern: None,
            datatype: Some(Nullable::Value("string".to_string())),
            placeholder: Some(Nullable::Value("Enter name".to_string())),
            values: Some(Nullable::Value(values_map.clone())),
            template: Some(Nullable::Null),
        };

        let json = serde_json::to_string(&override_).unwrap();
        let deserialized: FieldOverride = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.field, 5);
        assert_eq!(
            deserialized.name,
            Some(Nullable::Value("Patient Name".to_string()))
        );
        assert_eq!(deserialized.group, Some(Nullable::Null));
        assert_eq!(deserialized.note, None);
        assert_eq!(deserialized.required, Some(Nullable::Value(true)));
        assert_eq!(deserialized.minlength, Some(Nullable::Null));
        assert_eq!(deserialized.maxlength, Some(Nullable::Value(50)));
        assert_eq!(
            deserialized.datatype,
            Some(Nullable::Value("string".to_string()))
        );
        assert_eq!(
            deserialized.placeholder,
            Some(Nullable::Value("Enter name".to_string()))
        );
        assert_eq!(deserialized.values, Some(Nullable::Value(values_map)));
        assert_eq!(deserialized.template, Some(Nullable::Null));
    }

    #[test]
    fn test_nullable_with_different_types() {
        // test with bool
        let nullable_bool = Nullable::Value(true);
        let json = serde_json::to_string(&nullable_bool).unwrap();
        assert_eq!(json, "true");
        let deserialized: Nullable<bool> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Nullable::Value(true));

        // test with u32
        let nullable_u32 = Nullable::Value(42u32);
        let json = serde_json::to_string(&nullable_u32).unwrap();
        assert_eq!(json, "42");
        let deserialized: Nullable<u32> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Nullable::Value(42));

        // test with IndexMap
        let mut map = IndexMap::new();
        map.insert("key".to_string(), "value".to_string());
        let nullable_map = Nullable::Value(map.clone());
        let json = serde_json::to_string(&nullable_map).unwrap();
        assert!(json.contains("\"key\":\"value\""));
        let deserialized: Nullable<IndexMap<String, String>> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Nullable::Value(map));
    }
}
