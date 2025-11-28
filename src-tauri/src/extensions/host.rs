//! Extension host for managing multiple extensions.
//!
//! The host is responsible for:
//! - Starting and stopping extensions based on configuration
//! - Routing commands to the appropriate extension
//! - Aggregating toolbar buttons from all extensions
//! - Handling requests from extensions (editor/*, ui/*)
//! - Sending event notifications to subscribed extensions

use crate::commands::extensions::editor::{
    handle_get_message, handle_patch_message, handle_set_message,
};
use crate::commands::extensions::ui::{
    close_extension_windows, handle_close_window, handle_open_file, handle_open_files,
    handle_open_window, handle_save_file, handle_select_directory, handle_show_confirm,
    handle_show_message, SharedWindowManager,
};
use crate::extensions::process::{
    ExtensionError, ExtensionProcess, InternalMessage, ResponseSender,
};
use crate::extensions::protocol::{ErrorResponse, Request, Response, RpcError};
use crate::extensions::types::{
    CloseWindowParams, CommandExecuteParams, EventName, ExtensionConfig, ExtensionState,
    GetMessageParams, MessageChangedOptions, MessageChangedParams, MessageFormat,
    MessageOpenedParams, MessageSavedParams, OpenFileParams, OpenFilesParams, OpenWindowParams,
    PatchMessageParams, SaveFileParams, SchemaOverride, SelectDirectoryParams, SetMessageParams,
    ShowConfirmParams, ShowMessageParams, ShutdownReason, ToolbarButton,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

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
    /// Original path from configuration.
    pub path: String,
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
    ///
    /// This field stores the result of merging all extension schema overrides together.
    /// The merged schema is computed by `rebuild_merged_schema()` and applied to the
    /// SchemaCache via `set_extension_overrides()`.
    merged_schema: Option<SchemaOverride>,

    /// Background tasks that handle incoming requests from extensions.
    ///
    /// One task per extension, consuming from the extension's incoming_rx channel.
    request_handler_tasks: HashMap<String, JoinHandle<()>>,

    /// Handle for the debounced message/changed notification timer.
    message_changed_timer: Option<JoinHandle<()>>,
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
            request_handler_tasks: HashMap::new(),
            message_changed_timer: None,
        }
    }

    /// Load and start all enabled extensions from configuration.
    ///
    /// After starting extensions, this method rebuilds the merged schema and updates
    /// the provided SchemaCache with the extension overrides.
    pub async fn start_extensions(
        &mut self,
        configs: Vec<ExtensionConfig>,
        window_manager: &SharedWindowManager,
        schema_cache: &crate::schema::cache::SchemaCache,
    ) -> Result<(), ExtensionError> {
        for config in configs {
            if !config.enabled {
                log::info!("skipping disabled extension: {}", config.path);
                continue;
            }

            if let Err(e) = self.start_extension(config, window_manager).await {
                // log error but continue with other extensions
                log::error!("failed to start extension: {e}");
            }
        }

        // rebuild aggregated data
        self.rebuild_toolbar_buttons().await;
        self.rebuild_merged_schema().await;

        // update schema cache with merged overrides
        schema_cache.set_extension_overrides(self.merged_schema.clone());

        // notify frontend that extensions changed
        self.emit_extensions_changed();

        Ok(())
    }

    /// Start a single extension.
    async fn start_extension(
        &mut self,
        config: ExtensionConfig,
        window_manager: &SharedWindowManager,
    ) -> Result<(), ExtensionError> {
        let ext_data_dir = self.data_dir.join("extensions");
        std::fs::create_dir_all(&ext_data_dir).map_err(|e| {
            ExtensionError::SpawnFailed(std::io::Error::other(format!(
                "failed to create extension data directory: {e}"
            )))
        })?;

        let process =
            ExtensionProcess::spawn(config, &ext_data_dir, &self.hermes_version, API_VERSION)
                .await?;

        let ext_id = process.id.clone();

        // insert early so we can emit status updates
        self.extensions.insert(ext_id.clone(), process);

        // emit starting status
        self.emit_extension_status(&ext_id).await;

        // perform initialization handshake
        let init_result = {
            let ext = self
                .extensions
                .get_mut(&ext_id)
                .expect("extension was just inserted");
            ext.initialize(&self.hermes_version, API_VERSION, &ext_data_dir)
                .await
        };

        // emit final status (running or failed)
        self.emit_extension_status(&ext_id).await;

        if init_result.is_ok() {
            // spawn request handler task for this extension
            if let Some(ext) = self.extensions.get_mut(&ext_id) {
                if let (Some(incoming_rx), Some(response_sender)) =
                    (ext.take_incoming_rx(), ext.response_sender())
                {
                    // get editor_message from app data
                    let state = self.app_handle.state::<crate::AppData>();
                    let editor_message = state.editor_message.clone();

                    let task = Self::spawn_request_handler_task(
                        ext_id.clone(),
                        incoming_rx,
                        response_sender,
                        self.app_handle.clone(),
                        window_manager.clone(),
                        editor_message,
                    );
                    self.request_handler_tasks.insert(ext_id.clone(), task);
                    log::debug!("spawned request handler task for {ext_id}");
                }
            }
        }

        init_result?;

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
            // abort the request handler task
            if let Some(task) = self.request_handler_tasks.remove(&ext_id) {
                task.abort();
            }

            if let Some(mut ext) = self.extensions.remove(&ext_id) {
                if let Err(e) = ext.shutdown(ShutdownReason::Closing).await {
                    log::warn!("error shutting down extension {ext_id}: {e}");
                }
            }
        }

        self.toolbar_buttons.clear();
        self.merged_schema = None;
        self.request_handler_tasks.clear();
    }

    /// Reload all extensions (stop, then start with new configs).
    ///
    /// Closes windows, shuts down existing extensions, and starts with new configs.
    /// Updates the SchemaCache with the newly merged schema overrides.
    pub async fn reload(
        &mut self,
        configs: Vec<ExtensionConfig>,
        window_manager: &SharedWindowManager,
        schema_cache: &crate::schema::cache::SchemaCache,
    ) -> Result<(), ExtensionError> {
        log::info!("reloading extensions");

        // shutdown existing extensions (this also closes their windows)
        let ext_ids: Vec<String> = self.extensions.keys().cloned().collect();
        for ext_id in &ext_ids {
            close_extension_windows(&self.app_handle, ext_id, window_manager).await;
        }

        for ext_id in ext_ids {
            // abort the request handler task
            if let Some(task) = self.request_handler_tasks.remove(&ext_id) {
                task.abort();
            }

            if let Some(mut ext) = self.extensions.remove(&ext_id) {
                if let Err(e) = ext.shutdown(ShutdownReason::Reload).await {
                    log::warn!("error shutting down extension {ext_id} for reload: {e}");
                }
            }
        }

        // start with new configs
        self.start_extensions(configs, window_manager, schema_cache)
            .await
    }

    /// Send a command notification to the appropriate extension.
    ///
    /// This is a fire-and-forget operation - we don't wait for acknowledgement or results.
    pub async fn send_command_notification(&mut self, command: &str) -> Result<(), ExtensionError> {
        // find extension that owns this command
        let ext_id = self
            .find_extension_for_command(command)
            .await
            .ok_or_else(|| ExtensionError::CommandNotFound(command.to_string()))?;

        if let Some(ext) = self.extensions.get_mut(&ext_id) {
            let params = CommandExecuteParams {
                command: command.to_string(),
            };
            let params_value = serde_json::to_value(params)
                .map_err(|e| ExtensionError::InvalidState(format!("failed to serialize: {e}")))?;

            // send notification (no response expected)
            ext.send_notification("command/execute", params_value).await
        } else {
            Err(ExtensionError::InvalidState(format!(
                "extension {ext_id} not found"
            )))
        }
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
                path: ext.config.path.clone(),
                name,
                version,
                state,
                error,
            });
        }

        statuses
    }

    /// Get log entries for a specific extension.
    pub async fn get_extension_logs(
        &self,
        ext_id: &str,
    ) -> Option<Vec<crate::extensions::types::ExtensionLog>> {
        if let Some(ext) = self.extensions.get(ext_id) {
            Some(ext.get_logs().await)
        } else {
            None
        }
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
    /// Collects schema overrides from all running extensions and merges them using
    /// field-level merge semantics (later extensions win for conflicting fields).
    /// The merged result is stored in `self.merged_schema` for later application
    /// to the SchemaCache.
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
            self.merged_schema = Some(crate::schema::merge::merge_schema_overrides(&all_overrides));
        }
    }

    /// Emit an event to notify the frontend that extensions changed.
    fn emit_extensions_changed(&self) {
        if let Err(e) = self.app_handle.emit("extensions-changed", ()) {
            log::warn!("failed to emit extensions-changed event: {e}");
        }
    }

    /// Emit an event for a single extension status change.
    pub async fn emit_extension_status(&self, ext_id: &str) {
        if let Some(ext) = self.extensions.get(ext_id) {
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

            let status = ExtensionStatus {
                id: ext_id.to_string(),
                path: ext.config.path.clone(),
                name,
                version,
                state,
                error,
            };

            if let Err(e) = self.app_handle.emit("extension-status-changed", status) {
                log::warn!("failed to emit extension-status-changed event: {e}");
            }
        }
    }

    // ========================================================================
    // Message event notifications
    // ========================================================================

    /// Schedule a debounced `message/changed` notification.
    ///
    /// Cancels any pending timer and starts a new 500ms timer. When the timer
    /// fires, `send_message_changed_notifications` is called to notify all
    /// subscribed extensions.
    pub fn schedule_message_changed_notification(&mut self) {
        // cancel existing timer
        if let Some(handle) = self.message_changed_timer.take() {
            handle.abort();
        }

        let app_handle = self.app_handle.clone();

        self.message_changed_timer = Some(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;

            // re-acquire the lock and send notifications
            let state = app_handle.state::<crate::AppData>();
            let mut host = state.extension_host.lock().await;
            host.send_message_changed_notifications().await;
        }));
    }

    /// Send `message/changed` notifications to all subscribed extensions.
    ///
    /// Called by the debounce timer after 500ms of no changes.
    async fn send_message_changed_notifications(&mut self) {
        // get current editor state
        let state = self.app_handle.state::<crate::AppData>();
        let message = state.editor_message.lock().await.clone();
        let file_path = state.editor_file_path.lock().await.clone();

        for (ext_id, ext) in self.extensions.iter_mut() {
            if !ext.state().await.is_running() {
                continue;
            }

            if let Some(subscription) = ext.get_event_subscription(EventName::MessageChanged).await
            {
                let params = build_message_changed_params(
                    &message,
                    file_path.as_deref(),
                    subscription.options.as_ref(),
                );

                if let Ok(params_value) = serde_json::to_value(&params) {
                    if let Err(e) = ext.send_notification("message/changed", params_value).await {
                        log::debug!("failed to send message/changed to {ext_id}: {e}");
                    }
                }
            }
        }

        // clear the timer handle since we've fired
        self.message_changed_timer = None;
    }

    /// Send `message/opened` notification to all subscribed extensions.
    pub async fn notify_message_opened(&mut self, file_path: Option<&str>, is_new: bool) {
        for (ext_id, ext) in self.extensions.iter_mut() {
            if !ext.state().await.is_running() {
                continue;
            }

            if ext
                .get_event_subscription(EventName::MessageOpened)
                .await
                .is_some()
            {
                let params = MessageOpenedParams {
                    file_path: file_path.map(String::from),
                    is_new,
                };

                if let Ok(params_value) = serde_json::to_value(&params) {
                    if let Err(e) = ext.send_notification("message/opened", params_value).await {
                        log::debug!("failed to send message/opened to {ext_id}: {e}");
                    }
                }
            }
        }
    }

    /// Send `message/saved` notification to all subscribed extensions.
    pub async fn notify_message_saved(&mut self, file_path: &str, save_as: bool) {
        for (ext_id, ext) in self.extensions.iter_mut() {
            if !ext.state().await.is_running() {
                continue;
            }

            if ext
                .get_event_subscription(EventName::MessageSaved)
                .await
                .is_some()
            {
                let params = MessageSavedParams {
                    file_path: file_path.to_string(),
                    save_as,
                };

                if let Ok(params_value) = serde_json::to_value(&params) {
                    if let Err(e) = ext.send_notification("message/saved", params_value).await {
                        log::debug!("failed to send message/saved to {ext_id}: {e}");
                    }
                }
            }
        }
    }

    /// Spawn a background task that handles incoming requests from an extension.
    ///
    /// Consumes from the extension's `incoming_rx` channel and routes requests
    /// to the appropriate handlers. Responses are sent back via `ResponseSender`.
    fn spawn_request_handler_task(
        ext_id: String,
        mut incoming_rx: mpsc::Receiver<InternalMessage>,
        response_sender: ResponseSender,
        app_handle: AppHandle,
        window_manager: SharedWindowManager,
        editor_message: Arc<Mutex<String>>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(msg) = incoming_rx.recv().await {
                match msg {
                    InternalMessage::Request(request) => {
                        let request_id = request.id.clone();
                        let method = request.method.clone();

                        let result = handle_extension_request_standalone(
                            &ext_id,
                            request,
                            &app_handle,
                            &window_manager,
                            &editor_message,
                        )
                        .await;

                        match result {
                            Ok(Some(response)) => {
                                // send response back to extension
                                if let Err(e) = response_sender.send(response).await {
                                    log::error!(
                                        "failed to send response for {method} to {ext_id}: {e}"
                                    );
                                }
                            }
                            Ok(None) => {
                                // async request - response will be sent later via complete_pending_request
                                log::debug!("request {method} from {ext_id} deferred (async)");
                            }
                            Err(e) => {
                                // send error response
                                let error_response = ErrorResponse::new(Some(request_id), e);
                                if let Err(send_err) =
                                    response_sender.send_error(error_response).await
                                {
                                    log::error!(
                                        "failed to send error for {method} to {ext_id}: {send_err}"
                                    );
                                }
                            }
                        }
                    }
                    InternalMessage::Notification(notification) => {
                        // notifications don't need responses - just log for now
                        log::debug!(
                            "received notification from {ext_id}: {}",
                            notification.method
                        );
                    }
                    InternalMessage::ReaderError(e) => {
                        log::warn!("reader error for extension {ext_id}: {e}");
                        break;
                    }
                    _ => {}
                }
            }
            log::debug!("request handler task for {ext_id} ended");
        })
    }
}

impl Drop for ExtensionHost {
    fn drop(&mut self) {
        // cancel pending debounce timer
        if let Some(handle) = self.message_changed_timer.take() {
            handle.abort();
        }
    }
}

/// Handle a request from an extension (standalone version).
///
/// This is a standalone function that can be called from spawned tasks without
/// needing mutable access to `ExtensionHost`.
async fn handle_extension_request_standalone(
    ext_id: &str,
    request: Request,
    app_handle: &AppHandle,
    window_manager: &SharedWindowManager,
    editor_message: &Arc<Mutex<String>>,
) -> Result<Option<Response>, RpcError> {
    log::debug!("handling request from {ext_id}: {}", request.method);

    match request.method.as_str() {
        "editor/getMessage" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: GetMessageParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let editor_msg = editor_message.lock().await;
            let result = handle_get_message(&editor_msg, params.format)
                .map_err(|e| RpcError::internal(e.to_string()))?;

            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "editor/setMessage" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: SetMessageParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let (hl7_message, result) = handle_set_message(params)?;

            if result.success {
                let mut editor_msg = editor_message.lock().await;
                *editor_msg = hl7_message.clone();
                // emit event to frontend with the converted HL7 message
                app_handle
                    .emit("extension-set-message", &hl7_message)
                    .map_err(|e| RpcError::internal(format!("failed to emit event: {e}")))?;
            }

            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "editor/patchMessage" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: PatchMessageParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let mut editor_msg = editor_message.lock().await;
            let (new_message, result) = handle_patch_message(&editor_msg, params.patches);

            // update if any patches were applied (even partial success)
            if result.patches_applied > 0 {
                *editor_msg = new_message.clone();
                // notify frontend of the new message
                let _ = app_handle.emit("extension-set-message", &new_message);
            }

            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "ui/openWindow" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: OpenWindowParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let result = handle_open_window(app_handle, ext_id, params, window_manager).await?;
            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "ui/closeWindow" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: CloseWindowParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let result = handle_close_window(app_handle, ext_id, params, window_manager).await?;
            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "ui/showMessage" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: ShowMessageParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let result = handle_show_message(app_handle, params).await?;
            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "ui/showConfirm" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: ShowConfirmParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let result = handle_show_confirm(app_handle, params).await?;
            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "ui/openFile" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: OpenFileParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let result = handle_open_file(app_handle, params).await?;
            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "ui/openFiles" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: OpenFilesParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let result = handle_open_files(app_handle, params).await?;
            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "ui/saveFile" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: SaveFileParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let result = handle_save_file(app_handle, params).await?;
            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        "ui/selectDirectory" => {
            let params_value = request
                .params
                .ok_or_else(|| RpcError::invalid_params("missing params"))?;
            let params: SelectDirectoryParams = serde_json::from_value(params_value)
                .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

            let result = handle_select_directory(app_handle, params).await?;
            Ok(Some(Response::new(
                request.id,
                serde_json::to_value(result).unwrap(),
            )))
        }
        _ => Err(RpcError::method_not_found(&request.method)),
    }
}

/// Build the params for a `message/changed` notification based on subscription options.
fn build_message_changed_params(
    message: &str,
    file_path: Option<&str>,
    options: Option<&MessageChangedOptions>,
) -> MessageChangedParams {
    let (content, format) =
        if options.is_some_and(|o| o.include_content) {
            let format = options.and_then(|o| o.format).unwrap_or(MessageFormat::Hl7);

            // convert message to requested format if needed
            let content =
                match format {
                    MessageFormat::Hl7 => message.to_string(),
                    MessageFormat::Json => crate::commands::export_to_json(message)
                        .unwrap_or_else(|_| message.to_string()),
                    MessageFormat::Yaml => crate::commands::export_to_yaml(message)
                        .unwrap_or_else(|_| message.to_string()),
                    MessageFormat::Toml => crate::commands::export_to_toml(message)
                        .unwrap_or_else(|_| message.to_string()),
                };

            (Some(content), Some(format))
        } else {
            (None, None)
        };

    MessageChangedParams {
        message: content,
        format,
        has_file: file_path.is_some(),
        file_path: file_path.map(String::from),
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
            path: "/path/to/extension".to_string(),
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
            path: "/path/to/extension".to_string(),
            name: "Test Extension".to_string(),
            version: "1.0.0".to_string(),
            state: ExtensionState::Failed("connection lost".to_string()),
            error: Some("connection lost".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"error\":\"connection lost\""));
    }
}
