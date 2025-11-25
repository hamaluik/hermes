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
use crate::extensions::protocol::{Notification, Request, RequestId, Response, RpcError};
use crate::extensions::types::{
    CloseWindowParams, CommandExecuteResult, ExtensionConfig, ExtensionState, GetMessageParams,
    OpenWindowParams, PatchMessageParams, SchemaOverride, SetMessageParams, ShutdownReason,
    ToolbarButton,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

/// A pending request from an extension awaiting a response from the frontend.
///
/// Used for editor operations that require frontend interaction, such as
/// `editor/getMessage` and `editor/patchMessage`.
#[derive(Debug)]
pub struct PendingRequest {
    /// The JSON-RPC request ID to include in the response.
    pub request_id: RequestId,
    /// The method that was called.
    pub method: String,
    /// Original request params (for patchMessage).
    ///
    /// TODO: Phase 6 - this field can be used for error recovery or logging.
    /// Currently unused but kept for potential future diagnostic use.
    #[allow(dead_code)]
    pub params: Option<serde_json::Value>,
}

/// Payload emitted for `extension-get-message-request` event.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GetMessageRequestPayload {
    /// Extension ID that made the request.
    #[serde(rename = "extensionId")]
    pub extension_id: String,
    /// JSON-RPC request ID.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// Requested message format.
    pub format: String,
}

/// Payload emitted for `extension-patch-message-request` event.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PatchMessageRequestPayload {
    /// Extension ID that made the request.
    #[serde(rename = "extensionId")]
    pub extension_id: String,
    /// JSON-RPC request ID.
    #[serde(rename = "requestId")]
    pub request_id: String,
    /// Patch operations to apply.
    pub patches: Vec<serde_json::Value>,
}

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

    /// Pending requests awaiting responses from the frontend.
    ///
    /// Keyed by a composite key: "{extension_id}:{request_id}".
    /// Used for editor operations that require frontend interaction.
    pending_requests: HashMap<String, PendingRequest>,
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
            pending_requests: HashMap::new(),
        }
    }

    /// Load and start all enabled extensions from configuration.
    ///
    /// After starting extensions, this method rebuilds the merged schema and updates
    /// the provided SchemaCache with the extension overrides.
    pub async fn start_extensions(
        &mut self,
        configs: Vec<ExtensionConfig>,
        schema_cache: &crate::schema::cache::SchemaCache,
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

        // update schema cache with merged overrides
        schema_cache.set_extension_overrides(self.merged_schema.clone());

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

        let process =
            ExtensionProcess::spawn(config, &ext_data_dir, &self.hermes_version, API_VERSION)
                .await?;

        let ext_id = process.id.clone();

        // insert early so we can emit status updates
        self.extensions.insert(ext_id.clone(), process);

        // emit starting status
        self.emit_extension_status(&ext_id).await;

        // perform initialization handshake
        if let Some(ext) = self.extensions.get_mut(&ext_id) {
            let init_result = ext
                .initialize(&self.hermes_version, API_VERSION, &ext_data_dir)
                .await;

            // emit final status (running or failed)
            self.emit_extension_status(&ext_id).await;

            init_result?;
        }

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
            if let Some(mut ext) = self.extensions.remove(&ext_id) {
                if let Err(e) = ext.shutdown(ShutdownReason::Reload).await {
                    log::warn!("error shutting down extension {ext_id} for reload: {e}");
                }
            }
        }

        // start with new configs
        self.start_extensions(configs, schema_cache).await
    }

    /// Start executing a command asynchronously.
    ///
    /// Returns a channel that will receive the result when the command completes.
    /// This allows the caller to release the host lock while waiting for the response,
    /// preventing deadlock when the frontend needs to call back into the host.
    pub async fn start_command_async(
        &mut self,
        command: &str,
        window_manager: &SharedWindowManager,
    ) -> Result<
        tokio::sync::oneshot::Receiver<Result<CommandExecuteResult, ExtensionError>>,
        ExtensionError,
    > {
        // find extension that registered this command
        let ext_id = self.find_extension_for_command(command).await;

        let ext_id = ext_id
            .ok_or_else(|| ExtensionError::Rpc(RpcError::command_not_found(command)))?
            .clone();

        // start the command execution
        let ext = self
            .extensions
            .get_mut(&ext_id)
            .ok_or_else(|| ExtensionError::InvalidState(format!("extension {ext_id} not found")))?;

        let mut response_rx = ext.start_command(command).await?;

        // create a channel for the final result
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();

        // clone what we need for the background task
        let window_manager = window_manager.clone();
        let app_handle = self.app_handle.clone();
        let command = command.to_string();

        // spawn a background task to process messages
        tokio::spawn(async move {
            use crate::extensions::process::InternalMessage;
            use tauri::Manager;
            use tokio::time::{sleep, timeout, Duration};

            let deadline = Duration::from_secs(30);
            let result = timeout(deadline, async {
                loop {
                    // try to receive the command response (non-blocking via try_recv)
                    match response_rx.try_recv() {
                        Ok(result) => {
                            // got the response!
                            return result;
                        }
                        Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                            // no response yet, continue
                        }
                        Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                            return Err(ExtensionError::Channel(
                                "response channel closed".to_string(),
                            ));
                        }
                    }

                    // process incoming messages from the extension
                    // note: we need to access the extension through AppData's host
                    // this requires acquiring the lock briefly
                    let app_data = app_handle.state::<crate::AppData>();
                    let mut host = app_data.extension_host.lock().await;

                    let ext = match host.extensions.get_mut(&ext_id) {
                        Some(ext) => ext,
                        None => {
                            return Err(ExtensionError::InvalidState(format!(
                                "extension {ext_id} not found"
                            )))
                        }
                    };

                    if let Some(msg) = ext.process_incoming().await {
                        let mut is_async_response = false;

                        match msg {
                            InternalMessage::Request(request) => {
                                // handle the request
                                let response = host
                                    .handle_extension_request(&ext_id, request, &window_manager)
                                    .await;

                                match response {
                                    Ok(Some(response)) => {
                                        // send response back to extension
                                        if let Some(ext) = host.extensions.get_mut(&ext_id) {
                                            let _ = ext.send_response(response).await;
                                        }
                                    }
                                    Ok(None) => {
                                        // async response - will be completed later via complete_pending_request
                                        is_async_response = true;
                                    }
                                    Err(e) => {
                                        // send error response
                                        if let Some(ext) = host.extensions.get_mut(&ext_id) {
                                            let _ = ext.send_error_response(None, e).await;
                                        }
                                    }
                                }
                            }
                            InternalMessage::Notification(notification) => {
                                host.handle_extension_notification(&ext_id, notification);
                            }
                            InternalMessage::ReaderError(_) => {
                                return Err(ExtensionError::Channel(
                                    "extension reader error".to_string(),
                                ));
                            }
                            InternalMessage::Send(_) | InternalMessage::Response(_, _) => {
                                log::warn!(
                                    "unexpected internal message type in start_command_async"
                                );
                            }
                        }

                        // drop the lock before continuing the loop
                        drop(host);

                        if is_async_response {
                            // HACK: sleep to give frontend time to call back with the response.
                            // Without this, the background task immediately re-acquires the lock,
                            // blocking provideExtensionPatchResult for ~30 seconds until timeout.
                            // See protocol_issue.md for full explanation and long-term solutions.
                            sleep(Duration::from_millis(50)).await;
                        } else {
                            // yield to give other tasks a chance to acquire the lock
                            tokio::task::yield_now().await;
                        }
                    } else {
                        // drop the lock before sleeping
                        drop(host);
                        // no incoming messages, yield to avoid busy loop
                        sleep(Duration::from_millis(10)).await;
                    }
                }
            })
            .await;

            // complete the command and send the result
            let final_result = match result {
                Ok(Ok(response)) => {
                    let app_data = app_handle.state::<crate::AppData>();
                    let mut host = app_data.extension_host.lock().await;

                    if let Some(ext) = host.extensions.get_mut(&ext_id) {
                        ext.complete_command(&command, Ok(response.clone())).await;
                    }

                    // parse the result
                    serde_json::from_value(response.result).map_err(|e| {
                        ExtensionError::Protocol(crate::extensions::protocol::ProtocolError::Json(
                            e,
                        ))
                    })
                }
                Ok(Err(e)) => {
                    let app_data = app_handle.state::<crate::AppData>();
                    let mut host = app_data.extension_host.lock().await;

                    if let Some(ext) = host.extensions.get_mut(&ext_id) {
                        let err_msg = e.to_string();
                        ext.complete_command(&command, Err(ExtensionError::InvalidState(err_msg)))
                            .await;
                    }
                    Err(e)
                }
                Err(_) => {
                    let app_data = app_handle.state::<crate::AppData>();
                    let mut host = app_data.extension_host.lock().await;

                    if let Some(ext) = host.extensions.get_mut(&ext_id) {
                        ext.complete_command(
                            &command,
                            Err(ExtensionError::Timeout(format!("command/{command}"))),
                        )
                        .await;
                    }
                    Err(ExtensionError::Timeout(format!("command/{command}")))
                }
            };

            let _ = result_tx.send(final_result);
        });

        Ok(result_rx)
    }

    /// Execute a command on the appropriate extension.
    ///
    /// This method processes incoming messages from the extension while waiting for
    /// the command response, preventing deadlocks when the extension makes requests
    /// back to Hermes (e.g., editor/patchMessage) during command execution.
    ///
    /// Note: This method holds the host lock for the entire duration. For Tauri commands,
    /// prefer using start_command_async which releases the lock while waiting.
    #[allow(dead_code)]
    pub async fn execute_command(
        &mut self,
        command: &str,
        window_manager: &SharedWindowManager,
    ) -> Result<CommandExecuteResult, ExtensionError> {
        use crate::extensions::process::InternalMessage;
        use tokio::time::{timeout, Duration};

        // find extension that registered this command
        let ext_id = self.find_extension_for_command(command).await;

        let ext_id =
            ext_id.ok_or_else(|| ExtensionError::Rpc(RpcError::command_not_found(command)))?;

        // start the command execution (sends request but doesn't wait)
        let ext = self
            .extensions
            .get_mut(&ext_id)
            .ok_or_else(|| ExtensionError::InvalidState(format!("extension {ext_id} not found")))?;

        let mut response_rx = ext.start_command(command).await?;

        // process incoming messages while waiting for response
        let deadline = Duration::from_secs(30);
        let result = timeout(deadline, async {
            loop {
                // try to receive the command response (non-blocking via try_recv)
                match response_rx.try_recv() {
                    Ok(result) => {
                        // got the response!
                        return result;
                    }
                    Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                        // no response yet, continue processing incoming messages
                    }
                    Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                        return Err(ExtensionError::Channel(
                            "response channel closed".to_string(),
                        ));
                    }
                }

                // process any incoming messages from the extension
                let ext = self.extensions.get_mut(&ext_id).ok_or_else(|| {
                    ExtensionError::InvalidState(format!("extension {ext_id} not found"))
                })?;

                if let Some(msg) = ext.process_incoming().await {
                    let mut is_async_response = false;

                    match msg {
                        InternalMessage::Request(request) => {
                            // handle the request
                            let response = self
                                .handle_extension_request(&ext_id, request, window_manager)
                                .await;

                            match response {
                                Ok(Some(response)) => {
                                    // send response back to extension
                                    if let Some(ext) = self.extensions.get_mut(&ext_id) {
                                        let _ = ext.send_response(response).await;
                                    }
                                }
                                Ok(None) => {
                                    // async response - will be completed later via complete_pending_request
                                    is_async_response = true;
                                }
                                Err(e) => {
                                    // send error response
                                    if let Some(ext) = self.extensions.get_mut(&ext_id) {
                                        let _ = ext.send_error_response(None, e).await;
                                    }
                                }
                            }
                        }
                        InternalMessage::Notification(notification) => {
                            self.handle_extension_notification(&ext_id, notification);
                        }
                        InternalMessage::ReaderError(_) => {
                            return Err(ExtensionError::Channel(
                                "extension reader error".to_string(),
                            ));
                        }
                        // these shouldn't appear in process_incoming since they're handled internally
                        InternalMessage::Send(_) | InternalMessage::Response(_, _) => {
                            log::warn!("unexpected internal message type in execute_command");
                        }
                    }

                    if is_async_response {
                        // HACK: sleep to give frontend time to call back with the response.
                        // Without this, we immediately re-acquire the lock, blocking
                        // provideExtensionPatchResult. See protocol_issue.md.
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    } else {
                        // yield to give other tasks a chance to acquire the lock
                        tokio::task::yield_now().await;
                    }
                } else {
                    // no incoming messages, yield to avoid busy loop
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        })
        .await;

        let ext = self
            .extensions
            .get_mut(&ext_id)
            .ok_or_else(|| ExtensionError::InvalidState(format!("extension {ext_id} not found")))?;

        match result {
            Ok(Ok(response)) => {
                ext.complete_command(command, Ok(response.clone())).await;

                // parse the result
                let result: CommandExecuteResult = serde_json::from_value(response.result)
                    .map_err(|e| {
                        ExtensionError::Protocol(crate::extensions::protocol::ProtocolError::Json(
                            e,
                        ))
                    })?;

                Ok(result)
            }
            Ok(Err(e)) => {
                let err_msg = e.to_string();
                ext.complete_command(command, Err(ExtensionError::InvalidState(err_msg)))
                    .await;
                Err(e)
            }
            Err(_) => {
                let err = ExtensionError::Timeout(format!("command/{command}"));
                ext.complete_command(
                    command,
                    Err(ExtensionError::Timeout(format!("command/{command}"))),
                )
                .await;
                Err(err)
            }
        }
    }

    /// Handle a request from an extension.
    ///
    /// This routes editor/* and ui/* requests from extensions to the appropriate handlers.
    ///
    /// For editor operations, the handler emits events to the frontend for processing.
    /// For UI operations, the handler uses the window manager to create/close windows.
    ///
    /// Note: `editor/getMessage` and `editor/patchMessage` require async frontend interaction.
    /// They return `None` to indicate the response will be provided later via
    /// `complete_pending_request()`. The caller should not send a response for these.
    pub async fn handle_extension_request(
        &mut self,
        ext_id: &str,
        request: Request,
        window_manager: &SharedWindowManager,
    ) -> Result<Option<Response>, RpcError> {
        log::debug!("handling request from {ext_id}: {}", request.method);

        match request.method.as_str() {
            "editor/getMessage" => {
                let params_value = request
                    .params
                    .clone()
                    .ok_or_else(|| RpcError::invalid_params("missing params"))?;
                let params: GetMessageParams = serde_json::from_value(params_value)
                    .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

                // store pending request
                let request_id_str = request.id.to_string();
                let key = format!("{ext_id}:{request_id_str}");

                self.pending_requests.insert(
                    key,
                    PendingRequest {
                        request_id: request.id,
                        method: "editor/getMessage".to_string(),
                        params: request.params,
                    },
                );

                // emit event to frontend
                let payload = GetMessageRequestPayload {
                    extension_id: ext_id.to_string(),
                    request_id: request_id_str,
                    format: format!("{:?}", params.format).to_lowercase(),
                };

                self.app_handle
                    .emit("extension-get-message-request", &payload)
                    .map_err(|e| RpcError::internal(format!("failed to emit event: {e}")))?;

                // return None to indicate async response
                Ok(None)
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

                Ok(Some(Response::new(
                    request.id,
                    serde_json::to_value(result).unwrap(),
                )))
            }
            "editor/patchMessage" => {
                let params_value = request
                    .params
                    .clone()
                    .ok_or_else(|| RpcError::invalid_params("missing params"))?;
                let params: PatchMessageParams = serde_json::from_value(params_value.clone())
                    .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

                // store pending request
                let request_id_str = request.id.to_string();
                let key = format!("{ext_id}:{request_id_str}");

                self.pending_requests.insert(
                    key,
                    PendingRequest {
                        request_id: request.id,
                        method: "editor/patchMessage".to_string(),
                        params: request.params,
                    },
                );

                // emit event to frontend with patches
                let patches: Vec<serde_json::Value> = params
                    .patches
                    .iter()
                    .map(|p| serde_json::to_value(p).unwrap())
                    .collect();

                let payload = PatchMessageRequestPayload {
                    extension_id: ext_id.to_string(),
                    request_id: request_id_str.clone(),
                    patches,
                };

                self.app_handle
                    .emit("extension-patch-message-request", &payload)
                    .map_err(|e| RpcError::internal(format!("failed to emit event: {e}")))?;

                // return None to indicate async response
                Ok(None)
            }
            "ui/openWindow" => {
                let params_value = request
                    .params
                    .ok_or_else(|| RpcError::invalid_params("missing params"))?;
                let params: OpenWindowParams = serde_json::from_value(params_value)
                    .map_err(|e| RpcError::invalid_params(format!("invalid params: {e}")))?;

                let result =
                    handle_open_window(&self.app_handle, ext_id, params, window_manager).await?;
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

                let result =
                    handle_close_window(&self.app_handle, ext_id, params, window_manager).await?;
                Ok(Some(Response::new(
                    request.id,
                    serde_json::to_value(result).unwrap(),
                )))
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

    /// Complete a pending request by sending the response to the extension.
    ///
    /// Called by the frontend when it has processed an async request like
    /// `editor/getMessage` or `editor/patchMessage`.
    ///
    /// # Arguments
    ///
    /// * `ext_id` - Extension ID that made the original request
    /// * `request_id` - JSON-RPC request ID from the original request
    /// * `result` - The result value to send as the response
    pub async fn complete_pending_request(
        &mut self,
        ext_id: &str,
        request_id: &str,
        result: serde_json::Value,
    ) -> Result<(), ExtensionError> {
        let key = format!("{ext_id}:{request_id}");

        let pending = self
            .pending_requests
            .remove(&key)
            .ok_or_else(|| ExtensionError::InvalidState(format!("no pending request for {key}")))?;

        log::debug!(
            "completing pending request {key} for method {}",
            pending.method
        );

        // get the extension process
        let ext = self
            .extensions
            .get_mut(ext_id)
            .ok_or_else(|| ExtensionError::InvalidState(format!("extension {ext_id} not found")))?;

        // send the response
        let response = Response::new(pending.request_id, result);
        ext.send_response(response).await
    }

    /// Complete a pending request with an error.
    ///
    /// Called when the frontend cannot process a request (e.g., conversion error).
    pub async fn complete_pending_request_with_error(
        &mut self,
        ext_id: &str,
        request_id: &str,
        error: RpcError,
    ) -> Result<(), ExtensionError> {
        let key = format!("{ext_id}:{request_id}");

        let pending = self
            .pending_requests
            .remove(&key)
            .ok_or_else(|| ExtensionError::InvalidState(format!("no pending request for {key}")))?;

        log::debug!(
            "completing pending request {key} with error for method {}",
            pending.method
        );

        // get the extension process
        let ext = self
            .extensions
            .get_mut(ext_id)
            .ok_or_else(|| ExtensionError::InvalidState(format!("extension {ext_id} not found")))?;

        // send the error response
        ext.send_error_response(Some(pending.request_id), error)
            .await
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
