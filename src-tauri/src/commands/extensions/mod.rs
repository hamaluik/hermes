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
use crate::extensions::types::CommandExecuteResult;
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
/// Shuts down existing extensions and restarts them with current configuration.
/// This is typically called after the user modifies extension settings.
#[tauri::command]
pub async fn reload_extensions(state: State<'_, AppData>) -> Result<(), String> {
    let mut host = state.extension_host.lock().await;
    // TODO: Phase 3 - load configs from settings when settings integration is complete.
    // This should read from the Settings class's `extensions` property, which needs to be
    // implemented in `src/settings.ts`. The settings will persist ExtensionConfig[] to
    // `settings.json`. See EXTENSIONPLAN.md Phase 3 for details.
    let configs = Vec::new();
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
    use crate::extensions::types::CommandExecuteResult;

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
}
