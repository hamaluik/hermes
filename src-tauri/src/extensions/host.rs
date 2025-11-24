//! Extension host for managing multiple extensions.
//!
//! The host is responsible for:
//! - Starting and stopping extensions based on configuration
//! - Routing commands to the appropriate extension
//! - Aggregating toolbar buttons from all extensions
//! - Handling requests from extensions (editor/*, ui/*)

use crate::commands::extensions::editor::handle_set_message;
use crate::commands::extensions::ui::{
    close_extension_windows, handle_close_window, handle_open_window, SharedWindowManager,
};
use crate::extensions::process::{ExtensionError, ExtensionProcess};
use crate::extensions::protocol::{Message, Notification, Request, Response, RpcError};
use crate::extensions::types::{
    CloseWindowParams, CommandExecuteResult, ExtensionConfig, ExtensionState, OpenWindowParams,
    SchemaOverride, SetMessageParams, ShutdownReason, ToolbarButton,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

/// Current extension API version.
pub const API_VERSION: &str = "1.0";

/// Toolbar button with extension ownership information.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolbarButtonInfo {
    /// ID of the extension that registered this button.
    #[serde(rename = "extensionId")]
    pub extension_id: String,
    /// The button configuration.
    pub button: ToolbarButton,
}

/// Status information for an extension.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtensionStatus {
    /// Extension ID.
    pub id: String,
    /// Human-readable extension name.
    pub name: String,
    /// Extension version.
    pub version: String,
    /// Current lifecycle state.
    pub state: ExtensionState,
    /// Error message if in failed state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Manages multiple extension processes.
pub struct ExtensionHost {
    /// Running extension processes keyed by ID.
    extensions: HashMap<String, ExtensionProcess>,

    /// Tauri app handle for emitting events.
    app_handle: AppHandle,

    /// Base data directory for extensions.
    data_dir: PathBuf,

    /// Hermes application version.
    hermes_version: String,

    /// Aggregated toolbar buttons from all extensions.
    toolbar_buttons: Vec<ToolbarButtonInfo>,

    /// Merged schema overrides from all extensions.
    merged_schema: Option<SchemaOverride>,
}

impl ExtensionHost {
    /// Create a new extension host.
    pub fn new(app_handle: AppHandle, data_dir: PathBuf, hermes_version: String) -> Self {
        Self {
            extensions: HashMap::new(),
            app_handle,
            data_dir,
            hermes_version,
            toolbar_buttons: Vec::new(),
            merged_schema: None,
        }
    }

    /// Load and start all enabled extensions from configuration.
    pub async fn start_extensions(
        &mut self,
        configs: Vec<ExtensionConfig>,
    ) -> Result<(), ExtensionError> {
        for config in configs {
            if !config.enabled {
                log::info!("skipping disabled extension: {}", config.path);
                continue;
            }

            if let Err(e) = self.start_extension(config).await {
                // log error but continue with other extensions
                log::error!("failed to start extension: {e}");
            }
        }

        // rebuild aggregated data
        self.rebuild_toolbar_buttons().await;
        self.rebuild_merged_schema().await;

        // notify frontend that extensions changed
        self.emit_extensions_changed();

        Ok(())
    }

    /// Start a single extension.
    async fn start_extension(&mut self, config: ExtensionConfig) -> Result<(), ExtensionError> {
        let ext_data_dir = self.data_dir.join("extensions");
        std::fs::create_dir_all(&ext_data_dir).map_err(|e| {
            ExtensionError::SpawnFailed(std::io::Error::other(format!(
                "failed to create extension data directory: {e}"
            )))
        })?;

        let mut process =
            ExtensionProcess::spawn(config, &ext_data_dir, &self.hermes_version, API_VERSION)
                .await?;

        // perform initialization handshake
        process
            .initialize(&self.hermes_version, API_VERSION, &ext_data_dir)
            .await?;

        let ext_id = process.id.clone();
        self.extensions.insert(ext_id, process);

        Ok(())
    }

    /// Gracefully shutdown all extensions.
    ///
    /// Closes any windows opened by extensions and then shuts down the extension
    /// processes gracefully.
    pub async fn shutdown_all(&mut self, window_manager: &SharedWindowManager) {
        log::info!("shutting down all extensions");

        let ext_ids: Vec<String> = self.extensions.keys().cloned().collect();

        for ext_id in &ext_ids {
            // close any windows opened by this extension
            close_extension_windows(&self.app_handle, ext_id, window_manager).await;
        }

        for ext_id in ext_ids {
            if let Some(mut ext) = self.extensions.remove(&ext_id) {
                if let Err(e) = ext.shutdown(ShutdownReason::Closing).await {
                    log::warn!("error shutting down extension {ext_id}: {e}");
                }
            }
        }

        self.toolbar_buttons.clear();
        self.merged_schema = None;
    }

    /// Reload all extensions (stop, then start with new configs).
    ///
    /// Closes windows, shuts down existing extensions, and starts with new configs.
    pub async fn reload(
        &mut self,
        configs: Vec<ExtensionConfig>,
        window_manager: &SharedWindowManager,
    ) -> Result<(), ExtensionError> {
        log::info!("reloading extensions");

        // shutdown existing extensions (this also closes their windows)
        let ext_ids: Vec<String> = self.extensions.keys().cloned().collect();
        for ext_id in &ext_ids {
            close_extension_windows(&self.app_handle, ext_id, window_manager).await;
        }

        for ext_id in ext_ids {
            if let Some(mut ext) = self.extensions.remove(&ext_id) {
                if let Err(e) = ext.shutdown(ShutdownReason::Reload).await {
                    log::warn!("error shutting down extension {ext_id} for reload: {e}");
                }
            }
        }

        // start with new configs
        self.start_extensions(configs).await
    }

    /// Execute a command on the appropriate extension.
    pub async fn execute_command(
        &mut self,
        command: &str,
    ) -> Result<CommandExecuteResult, ExtensionError> {
        // find extension that registered this command
        let ext_id = self.find_extension_for_command(command).await;

        let ext_id =
            ext_id.ok_or_else(|| ExtensionError::Rpc(RpcError::command_not_found(command)))?;

        let ext = self
            .extensions
            .get_mut(&ext_id)
            .ok_or_else(|| ExtensionError::InvalidState(format!("extension {ext_id} not found")))?;

        let response = ext.execute_command(command).await?;

        // parse the result
        let result: CommandExecuteResult =
            serde_json::from_value(response.result).map_err(|e| {
                ExtensionError::Protocol(crate::extensions::protocol::ProtocolError::Json(e))
            })?;

        Ok(result)
    }

    /// Handle a request from an extension.
    ///
    /// This routes editor/* and ui/* requests from extensions to the appropriate handlers.
    ///
    /// For editor operations, the handler emits events to the frontend for processing.
    /// For UI operations, the handler uses the window manager to create/close windows.
    pub async fn handle_extension_request(
        &mut self,
        ext_id: &str,
        request: Request,
        window_manager: &SharedWindowManager,
    ) -> Result<Response, RpcError> {
        log::debug!("handling request from {ext_id}: {}", request.method);

        match request.method.as_str() {
            "editor/getMessage" => {
                // TODO: Phase 4 - implement full frontend integration.
                // Currently emits event but response handling is incomplete.
                // The frontend needs to:
                // 1. Listen for "extension-get-message-request" event
                // 2. Call handle_get_message() with current editor content
                // 3. Route response back to extension via provide_extension_message command
                // See EXTENSIONPLAN.md Phase 4 section 4.3 "Editor Bridge" for details.
                self.app_handle
                    .emit("extension-get-message-request", (&ext_id, &request.id))
                    .map_err(|e| RpcError::internal(format!("failed to emit event: {e}")))?;

                Err(RpcError::internal(
                    "editor/getMessage requires frontend interaction - not yet fully implemented",
                ))
            }
            "editor/setMessage" => {
                let params_value = request
                    .params
                    .ok_or_else(|| RpcError::invalid_params("missing params"))?;
                let params: SetMessageParams = serde_json::from_value(params_value)
                    .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

                let (hl7_message, result) = handle_set_message(params)?;

                if result.success {
                    // emit event to frontend with the converted HL7 message
                    self.app_handle
                        .emit("extension-set-message", &hl7_message)
                        .map_err(|e| RpcError::internal(format!("failed to emit event: {e}")))?;
                }

                Ok(Response::new(
                    request.id,
                    serde_json::to_value(result).unwrap(),
                ))
            }
            "editor/patchMessage" => {
                // TODO: Phase 4 - implement full frontend integration.
                // Currently emits event but response handling is incomplete.
                // The frontend needs to:
                // 1. Listen for "extension-patch-message-request" event
                // 2. Get current message, call handle_patch_message() with it
                // 3. Update editor with patched message
                // 4. Route response back to extension
                // See EXTENSIONPLAN.md Phase 4 section 4.3 "Editor Bridge" for details.
                self.app_handle
                    .emit(
                        "extension-patch-message-request",
                        (&ext_id, &request.id, &request.params),
                    )
                    .map_err(|e| RpcError::internal(format!("failed to emit event: {e}")))?;

                Err(RpcError::internal(
                    "editor/patchMessage requires frontend interaction - not yet fully implemented",
                ))
            }
            "ui/openWindow" => {
                let params_value = request
                    .params
                    .ok_or_else(|| RpcError::invalid_params("missing params"))?;
                let params: OpenWindowParams = serde_json::from_value(params_value)
                    .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

                let result =
                    handle_open_window(&self.app_handle, ext_id, params, window_manager).await?;
                Ok(Response::new(
                    request.id,
                    serde_json::to_value(result).unwrap(),
                ))
            }
            "ui/closeWindow" => {
                let params_value = request
                    .params
                    .ok_or_else(|| RpcError::invalid_params("missing params"))?;
                let params: CloseWindowParams = serde_json::from_value(params_value)
                    .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

                let result =
                    handle_close_window(&self.app_handle, ext_id, params, window_manager).await?;
                Ok(Response::new(
                    request.id,
                    serde_json::to_value(result).unwrap(),
                ))
            }
            _ => Err(RpcError::method_not_found(&request.method)),
        }
    }

    /// Handle a notification from an extension.
    pub fn handle_extension_notification(&mut self, ext_id: &str, notification: Notification) {
        log::debug!(
            "received notification from {ext_id}: {}",
            notification.method
        );
        // notifications are informational; log and continue
    }

    /// Get all toolbar buttons from all extensions.
    pub fn get_toolbar_buttons(&self) -> &[ToolbarButtonInfo] {
        &self.toolbar_buttons
    }

    /// Get the merged schema from all extensions.
    pub fn get_merged_schema(&self) -> Option<&SchemaOverride> {
        self.merged_schema.as_ref()
    }

    /// Get status information for all extensions.
    pub async fn get_extension_statuses(&self) -> Vec<ExtensionStatus> {
        let mut statuses = Vec::new();

        for (id, ext) in &self.extensions {
            let state = ext.state().await;
            let metadata = ext.metadata().await;

            let (name, version) = if let Some(meta) = &metadata {
                (meta.name.clone(), meta.version.clone())
            } else {
                ("Unknown".to_string(), "0.0.0".to_string())
            };

            let error = if let ExtensionState::Failed(msg) = &state {
                Some(msg.clone())
            } else {
                None
            };

            statuses.push(ExtensionStatus {
                id: id.clone(),
                name,
                version,
                state,
                error,
            });
        }

        statuses
    }

    /// Send a notification to an extension.
    pub async fn send_notification(
        &mut self,
        ext_id: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<(), ExtensionError> {
        let ext = self
            .extensions
            .get_mut(ext_id)
            .ok_or_else(|| ExtensionError::InvalidState(format!("extension {ext_id} not found")))?;

        ext.send_notification(method, params).await
    }

    /// Send a response to an extension's pending request.
    ///
    /// TODO: Phase 4 - implement proper response routing.
    /// Currently responses are handled inline in handle_extension_request.
    /// For getMessage/patchMessage which require async frontend interaction,
    /// we need to track pending requests and route responses when the frontend
    /// calls back. This method will be used by provide_extension_message command.
    pub async fn send_response(
        &mut self,
        ext_id: &str,
        response: Message,
    ) -> Result<(), ExtensionError> {
        log::debug!("send_response to {ext_id}: {:?}", response);
        Ok(())
    }

    /// Find the extension that registered a given command.
    async fn find_extension_for_command(&self, command: &str) -> Option<String> {
        for (ext_id, ext) in &self.extensions {
            if let Some(metadata) = ext.metadata().await {
                // check if command matches any registered command
                if metadata
                    .capabilities
                    .commands
                    .contains(&command.to_string())
                {
                    return Some(ext_id.clone());
                }

                // also check toolbar button commands
                for button in &metadata.toolbar_buttons {
                    if button.command == command {
                        return Some(ext_id.clone());
                    }
                }
            }
        }
        None
    }

    /// Rebuild the aggregated toolbar buttons from all extensions.
    async fn rebuild_toolbar_buttons(&mut self) {
        self.toolbar_buttons.clear();

        for (ext_id, ext) in &self.extensions {
            if let Some(metadata) = ext.metadata().await {
                for button in &metadata.toolbar_buttons {
                    self.toolbar_buttons.push(ToolbarButtonInfo {
                        extension_id: ext_id.clone(),
                        button: button.clone(),
                    });
                }
            }
        }
    }

    /// Rebuild the merged schema from all extensions.
    ///
    /// TODO: Phase 5 - implement proper schema merging with field-level merge semantics.
    /// Currently uses naive "last extension wins" approach. Proper implementation should:
    /// 1. Create `src-tauri/src/schema/merge.rs` module
    /// 2. Merge built-in schema with all extension overrides
    /// 3. Handle field-level conflicts (later overrides win)
    /// 4. Support null values to unset properties
    ///
    /// See EXTENSIONPLAN.md Phase 5 and `extensions/api-docs/schema.md` for merging behaviour.
    async fn rebuild_merged_schema(&mut self) {
        let mut all_overrides: Vec<SchemaOverride> = Vec::new();

        for ext in self.extensions.values() {
            if let Some(metadata) = ext.metadata().await {
                if let Some(schema) = metadata.schema {
                    all_overrides.push(schema);
                }
            }
        }

        if all_overrides.is_empty() {
            self.merged_schema = None;
        } else {
            // naive last-one-wins approach; see TODO above for proper implementation
            self.merged_schema = all_overrides.pop();
        }
    }

    /// Emit an event to notify the frontend that extensions changed.
    fn emit_extensions_changed(&self) {
        if let Err(e) = self.app_handle.emit("extensions-changed", ()) {
            log::warn!("failed to emit extensions-changed event: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_button_info_serialization() {
        let info = ToolbarButtonInfo {
            extension_id: "ext-123".to_string(),
            button: ToolbarButton {
                id: "btn-1".to_string(),
                label: "Test".to_string(),
                icon: "<svg></svg>".to_string(),
                command: "test/action".to_string(),
                group: None,
            },
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"extensionId\":\"ext-123\""));
        assert!(json.contains("\"command\":\"test/action\""));
    }

    #[test]
    fn test_extension_status_serialization() {
        let status = ExtensionStatus {
            id: "ext-123".to_string(),
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            state: ExtensionState::Running,
            error: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"state\":\"running\""));
        assert!(!json.contains("\"error\"")); // None should be skipped
    }

    #[test]
    fn test_extension_status_with_error() {
        let status = ExtensionStatus {
            id: "ext-123".to_string(),
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            state: ExtensionState::Failed("connection lost".to_string()),
            error: Some("connection lost".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"error\":\"connection lost\""));
    }
}
