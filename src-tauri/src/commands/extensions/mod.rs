//! Tauri commands for the extension system.
//!
//! These commands provide the interface between the frontend and the extension
//! host, allowing the UI to:
//!
//! - Query extension status and toolbar buttons
//! - Execute extension commands (triggered by toolbar button clicks)
//! - Reload extensions after configuration changes
//!
//! Extension-to-Hermes requests (editor/*, ui/*) are handled internally by
//! the extension host and don't require separate Tauri commands.

pub mod editor;
pub mod ui;

use crate::extensions::host::{ExtensionStatus, ToolbarButtonInfo};
use crate::extensions::types::{CommandExecuteResult, ExtensionConfig};
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

/// Reload all extensions.
///
/// Shuts down existing extensions and restarts them with the provided configuration.
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
    host.reload(configs, &state.window_manager)
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
    let mut host = state.extension_host.lock().await;
    host.execute_command(&command)
        .await
        .map_err(|e| e.to_string())
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
