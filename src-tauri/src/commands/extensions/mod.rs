//! Tauri commands for the extension system.
//!
//! These commands provide the interface between the frontend and the extension
//! host, allowing the UI to:
//!
//! - Query extension status and toolbar buttons
//! - Execute extension commands (triggered by toolbar button clicks)
//! - Reload extensions after configuration changes
//! - Provide responses from the frontend for async editor operations
//!
//! Extension-to-Hermes requests (editor/*, ui/*) are handled internally by
//! the extension host and don't require separate Tauri commands.

pub mod editor;
pub mod ui;

use crate::extensions::host::{ExtensionStatus, ToolbarButtonInfo};
use crate::extensions::types::{
    CommandExecuteResult, ExtensionConfig, ExtensionLog, GetMessageResult, Patch, PatchError,
    PatchMessageResult,
};
use crate::AppData;
use tauri::State;

/// Get status information for all extensions.
///
/// Returns a list of extension statuses including ID, name, version, state,
/// and any error messages for failed extensions.
#[tauri::command]
pub async fn get_extensions(state: State<'_, AppData>) -> Result<Vec<ExtensionStatus>, String> {
    let host = state.extension_host.lock().await;
    Ok(host.get_extension_statuses().await)
}

/// Get all toolbar buttons from all running extensions.
///
/// Returns buttons with their associated extension IDs, allowing the frontend
/// to display them and route clicks appropriately.
#[tauri::command]
pub async fn get_extension_toolbar_buttons(
    state: State<'_, AppData>,
) -> Result<Vec<ToolbarButtonInfo>, String> {
    let host = state.extension_host.lock().await;
    Ok(host.get_toolbar_buttons().to_vec())
}

/// Get log entries for a specific extension.
///
/// Returns the recent log entries (up to 100) for the specified extension.
#[tauri::command]
pub async fn get_extension_logs(
    extension_id: String,
    state: State<'_, AppData>,
) -> Result<Vec<ExtensionLog>, String> {
    let host = state.extension_host.lock().await;
    host.get_extension_logs(&extension_id)
        .await
        .ok_or_else(|| format!("extension not found: {extension_id}"))
}

/// Reload all extensions.
///
/// Shuts down existing extensions and restarts them with the provided configuration.
/// After reloading, merges extension schema overrides and updates the SchemaCache.
/// This is typically called after the user modifies extension settings, or on app
/// startup when settings are loaded from disk.
///
/// # Arguments
///
/// * `configs` - Extension configurations from the frontend Settings class.
///   Each config specifies the path to an extension executable, optional arguments,
///   environment variables, and whether the extension is enabled.
#[tauri::command]
pub async fn reload_extensions(
    configs: Vec<ExtensionConfig>,
    state: State<'_, AppData>,
) -> Result<(), String> {
    let mut host = state.extension_host.lock().await;
    host.reload(configs, &state.window_manager, &state.schema)
        .await
        .map_err(|e| e.to_string())
}

/// Execute an extension command.
///
/// This is called when a user clicks an extension toolbar button. The command
/// string identifies which extension and action to invoke.
#[tauri::command]
pub async fn execute_extension_command(
    command: String,
    state: State<'_, AppData>,
) -> Result<CommandExecuteResult, String> {
    // start the command without holding the lock for the entire duration
    // this prevents deadlock when the frontend calls provide_extension_patch_result
    let mut host = state.extension_host.lock().await;
    let receiver = host
        .start_command_async(&command, &state.window_manager)
        .await
        .map_err(|e| e.to_string())?;

    // release the lock before waiting for the response
    drop(host);

    // wait for the response (lock is released, so frontend can call complete_pending_request)
    let result = receiver.await.map_err(|e| e.to_string())?;

    result.map_err(|e| e.to_string())
}

/// Provide the message content in response to an extension's `editor/getMessage` request.
///
/// Called by the frontend after receiving an `extension-get-message-request` event.
/// The frontend converts the message to the requested format and sends it back,
/// which is then routed to the waiting extension.
#[tauri::command]
pub async fn provide_extension_message(
    extension_id: String,
    request_id: String,
    message: String,
    has_file: bool,
    file_path: Option<String>,
    state: State<'_, AppData>,
) -> Result<(), String> {
    let result = GetMessageResult {
        message,
        has_file,
        file_path,
    };

    let result_value =
        serde_json::to_value(result).map_err(|e| format!("failed to serialize result: {e}"))?;

    let mut host = state.extension_host.lock().await;
    host.complete_pending_request(&extension_id, &request_id, result_value)
        .await
        .map_err(|e| e.to_string())
}

/// Provide the result of applying patches in response to an extension's
/// `editor/patchMessage` request.
///
/// Called by the frontend after receiving an `extension-patch-message-request` event,
/// applying the patches locally, and updating the editor.
#[tauri::command]
pub async fn provide_extension_patch_result(
    extension_id: String,
    request_id: String,
    success: bool,
    patches_applied: usize,
    errors: Option<Vec<PatchError>>,
    state: State<'_, AppData>,
) -> Result<(), String> {
    let result = PatchMessageResult {
        success,
        patches_applied,
        errors,
    };

    let result_value =
        serde_json::to_value(result).map_err(|e| format!("failed to serialize result: {e}"))?;

    let mut host = state.extension_host.lock().await;
    host.complete_pending_request(&extension_id, &request_id, result_value)
        .await
        .map_err(|e| e.to_string())
}

/// Apply patches to an HL7 message.
///
/// This Tauri command is called by the frontend when it needs to apply patches
/// from an extension's `editor/patchMessage` request. It uses the same patch
/// logic as the internal editor handlers.
#[tauri::command]
pub fn apply_extension_patches(
    message: String,
    patches: Vec<Patch>,
) -> Result<ApplyPatchesResult, String> {
    use crate::extensions::types::PatchMessageParams;

    let params = PatchMessageParams { patches };
    let (new_message, result) =
        editor::handle_patch_message(params, &message).map_err(|e| e.to_string())?;

    Ok(ApplyPatchesResult {
        message: new_message,
        success: result.success,
        patches_applied: result.patches_applied,
        errors: result.errors,
    })
}

/// Result type for apply_extension_patches command.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplyPatchesResult {
    /// The patched message content (HL7 format).
    pub message: String,
    /// Whether all patches were applied successfully.
    pub success: bool,
    /// Number of patches applied.
    #[serde(rename = "patchesApplied")]
    pub patches_applied: usize,
    /// Errors for patches that failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<PatchError>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_execute_result_serialisation() {
        let result = CommandExecuteResult {
            success: true,
            message: Some("Operation completed".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"message\":\"Operation completed\""));
    }

    #[test]
    fn test_command_execute_result_without_message() {
        let result = CommandExecuteResult {
            success: false,
            message: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":false"));
        // message should be omitted when None
        assert!(!json.contains("\"message\""));
    }

    /// Test that ExtensionConfig deserialises correctly from the format
    /// produced by the TypeScript frontend (matching settings.ts).
    #[test]
    fn test_extension_config_from_frontend_json() {
        // full config with all fields
        let json = r#"{
            "path": "/usr/local/bin/my-extension",
            "args": ["--config", "/path/to/config.json"],
            "env": {"MY_DEBUG": "1", "OTHER_VAR": "value"},
            "enabled": true
        }"#;

        let config: ExtensionConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.path, "/usr/local/bin/my-extension");
        assert_eq!(config.args, vec!["--config", "/path/to/config.json"]);
        assert_eq!(config.env.get("MY_DEBUG"), Some(&"1".to_string()));
        assert_eq!(config.env.get("OTHER_VAR"), Some(&"value".to_string()));
        assert!(config.enabled);
    }

    /// Test that optional fields default correctly when omitted.
    #[test]
    fn test_extension_config_minimal() {
        // minimal config with only required fields
        let json = r#"{
            "path": "/usr/bin/ext",
            "enabled": false
        }"#;

        let config: ExtensionConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.path, "/usr/bin/ext");
        assert!(config.args.is_empty());
        assert!(config.env.is_empty());
        assert!(!config.enabled);
    }

    /// Test deserialisation of an array of configs (as stored in settings.json).
    #[test]
    fn test_extension_config_array() {
        let json = r#"[
            {"path": "/bin/ext1", "enabled": true},
            {"path": "/bin/ext2", "args": ["--verbose"], "enabled": false}
        ]"#;

        let configs: Vec<ExtensionConfig> = serde_json::from_str(json).unwrap();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].path, "/bin/ext1");
        assert!(configs[0].enabled);
        assert_eq!(configs[1].path, "/bin/ext2");
        assert_eq!(configs[1].args, vec!["--verbose"]);
        assert!(!configs[1].enabled);
    }
}
