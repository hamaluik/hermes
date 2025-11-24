//! Shared type definitions for the extension system.
//!
//! These types are used for configuration, lifecycle management, and
//! JSON-RPC message parameters/results.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<u32>,

    /// Override the field name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Override the field group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    /// Override the field note/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Override whether the field is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// Override minimum length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minlength: Option<u32>,

    /// Override maximum length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxlength: Option<u32>,

    /// Override validation pattern (regex).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Override datatype ("date" | "datetime").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datatype: Option<String>,

    /// Override placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    /// Override allowed values (code -> display name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<IndexMap<String, String>>,

    /// Override template value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
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

/// Parameters for `command/execute` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecuteParams {
    /// Command identifier to execute.
    pub command: String,
}

/// Result of `command/execute` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecuteResult {
    /// Whether the command succeeded.
    pub success: bool,

    /// Optional message (error or informational).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
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
}
