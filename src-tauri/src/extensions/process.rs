//! Extension process management.
//!
//! Manages a single extension subprocess, including spawning, lifecycle,
//! message routing, and graceful shutdown.

use crate::extensions::protocol::{
    read_message, write_message, ErrorResponse, Message, Notification, ProtocolError, Request,
    RequestId, Response, RpcError,
};
use crate::extensions::types::{
    EventName, EventSubscription, ExtensionConfig, ExtensionLog, ExtensionMetadata, ExtensionState,
    InitializeParams, InitializeResult, LogLevel, ShutdownParams, ShutdownReason,
};
use jiff::Timestamp;
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufRead, AsyncWrite, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

/// Timeouts for extension operations.
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(10);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum number of log entries to keep per extension.
const MAX_LOG_ENTRIES: usize = 100;

/// Type alias for pending request tracking.
type PendingRequests =
    Arc<Mutex<HashMap<RequestId, oneshot::Sender<Result<Response, ErrorResponse>>>>>;

/// Cloneable handle for sending responses back to an extension.
///
/// This allows spawned tasks to send responses without needing mutable access
/// to the `ExtensionProcess`.
#[derive(Clone)]
pub struct ResponseSender {
    tx: mpsc::Sender<Message>,
}

impl ResponseSender {
    /// Send a successful response to the extension.
    pub async fn send(&self, response: Response) -> Result<(), ExtensionError> {
        self.tx
            .send(Message::Response(response))
            .await
            .map_err(|_| ExtensionError::Channel("failed to send response".to_string()))
    }

    /// Send an error response to the extension.
    pub async fn send_error(&self, error: ErrorResponse) -> Result<(), ExtensionError> {
        self.tx
            .send(Message::Error(error))
            .await
            .map_err(|_| ExtensionError::Channel("failed to send error response".to_string()))
    }
}

/// Errors that can occur during extension process operations.
#[derive(Debug)]
pub enum ExtensionError {
    /// Failed to spawn the extension process.
    SpawnFailed(std::io::Error),
    /// Protocol-level error.
    Protocol(ProtocolError),
    /// Extension returned an error response.
    Rpc(RpcError),
    /// Operation timed out.
    Timeout(String),
    /// Extension is not in a valid state for the operation.
    InvalidState(String),
    /// Channel communication error.
    Channel(String),
    /// Extension process exited unexpectedly.
    ProcessExited,
    /// Command not found in any extension.
    CommandNotFound(String),
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionError::SpawnFailed(e) => write!(f, "failed to spawn extension: {e}"),
            ExtensionError::Protocol(e) => write!(f, "protocol error: {e}"),
            ExtensionError::Rpc(e) => write!(f, "RPC error: {e}"),
            ExtensionError::Timeout(op) => write!(f, "operation timed out: {op}"),
            ExtensionError::InvalidState(msg) => write!(f, "invalid state: {msg}"),
            ExtensionError::Channel(msg) => write!(f, "channel error: {msg}"),
            ExtensionError::ProcessExited => write!(f, "extension process exited unexpectedly"),
            ExtensionError::CommandNotFound(cmd) => write!(f, "command not found: {cmd}"),
        }
    }
}

impl std::error::Error for ExtensionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExtensionError::SpawnFailed(e) => Some(e),
            ExtensionError::Protocol(e) => Some(e),
            ExtensionError::Rpc(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ProtocolError> for ExtensionError {
    fn from(e: ProtocolError) -> Self {
        ExtensionError::Protocol(e)
    }
}

impl From<RpcError> for ExtensionError {
    fn from(e: RpcError) -> Self {
        ExtensionError::Rpc(e)
    }
}

/// Internal message type for communication between tasks.
#[allow(dead_code)]
pub enum InternalMessage {
    /// Outgoing message to send to extension.
    Send(Message),
    /// Incoming response to route to waiting request.
    Response(RequestId, Result<Response, ErrorResponse>),
    /// Incoming request from extension (to be handled by host).
    Request(Request),
    /// Incoming notification from extension.
    Notification(Notification),
    /// Reader task encountered an error or EOF.
    ReaderError(ProtocolError),
}

/// Manages a single extension subprocess.
#[allow(dead_code)]
pub struct ExtensionProcess {
    /// Unique identifier for this extension instance.
    pub id: String,

    /// Configuration used to start this extension.
    pub config: ExtensionConfig,

    /// Current lifecycle state.
    state: Arc<Mutex<ExtensionState>>,

    /// Metadata from initialize response.
    metadata: Arc<Mutex<Option<ExtensionMetadata>>>,

    /// Channel to send messages to the writer task.
    outgoing_tx: Option<mpsc::Sender<Message>>,

    /// Pending requests waiting for responses.
    pending_requests: PendingRequests,

    /// Next request ID counter.
    next_request_id: Arc<Mutex<i64>>,

    /// Handle to the reader task.
    reader_task: Option<JoinHandle<()>>,

    /// Handle to the writer task.
    writer_task: Option<JoinHandle<()>>,

    /// Handle to the child process.
    child: Option<Child>,

    /// Channel for receiving requests/notifications from the extension.
    incoming_rx: Option<mpsc::Receiver<InternalMessage>>,

    /// Ring buffer of recent log entries.
    logs: Arc<Mutex<VecDeque<ExtensionLog>>>,
}

impl ExtensionProcess {
    /// Spawn a new extension process.
    ///
    /// # Arguments
    ///
    /// * `config` - Extension configuration (path, args, env).
    /// * `data_dir` - Path to the extension's data directory.
    /// * `hermes_version` - Hermes application version.
    /// * `api_version` - Extension API version.
    pub async fn spawn(
        config: ExtensionConfig,
        data_dir: &Path,
        hermes_version: &str,
        api_version: &str,
    ) -> Result<Self, ExtensionError> {
        // generate unique ID from path
        let id = generate_extension_id(&config.path);

        log::info!("spawning extension {id} from {}", config.path);

        // parse the command string if args are empty
        // this allows the UI to accept full command strings like "uv run script.py"
        let (executable, args) = if config.args.is_empty() {
            match shell_words::split(&config.path) {
                Ok(parts) if !parts.is_empty() => {
                    let executable = parts[0].clone();
                    let args = parts[1..].to_vec();
                    (executable, args)
                }
                Ok(_) => {
                    return Err(ExtensionError::SpawnFailed(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "empty command string",
                    )));
                }
                Err(e) => {
                    return Err(ExtensionError::SpawnFailed(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("failed to parse command: {e}"),
                    )));
                }
            }
        } else {
            // args explicitly provided, use path as-is
            (config.path.clone(), config.args.clone())
        };

        // spawn the process
        let mut command = Command::new(&executable);
        command
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // let extension stderr pass through for debugging
            .env("HERMES_VERSION", hermes_version)
            .env("HERMES_API_VERSION", api_version)
            .env("HERMES_DATA_DIR", data_dir);

        // add custom environment variables
        for (key, value) in &config.env {
            command.env(key, value);
        }

        let mut child = command.spawn().map_err(ExtensionError::SpawnFailed)?;

        let stdin = child.stdin.take().expect("can take stdin");
        let stdout = child.stdout.take().expect("can take stdout");

        // set up communication channels
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<Message>(32);
        let (incoming_tx, incoming_rx) = mpsc::channel::<InternalMessage>(32);
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));

        // spawn reader task
        let reader_task = spawn_reader_task(
            BufReader::new(stdout),
            incoming_tx.clone(),
            pending_requests.clone(),
        );

        // spawn writer task
        let writer_task = spawn_writer_task(stdin, outgoing_rx);

        let state = Arc::new(Mutex::new(ExtensionState::Starting));
        let metadata = Arc::new(Mutex::new(None));
        let next_request_id = Arc::new(Mutex::new(1i64));
        let logs = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES)));

        let process = Self {
            id: id.clone(),
            config,
            state,
            metadata,
            outgoing_tx: Some(outgoing_tx),
            pending_requests,
            next_request_id,
            reader_task: Some(reader_task),
            writer_task: Some(writer_task),
            child: Some(child),
            incoming_rx: Some(incoming_rx),
            logs,
        };

        // add initial log entry
        process
            .add_log(LogLevel::Info, format!("extension process spawned: {id}"))
            .await;

        Ok(process)
    }

    /// Get the current state of the extension.
    pub async fn state(&self) -> ExtensionState {
        self.state.lock().await.clone()
    }

    /// Get the extension metadata (available after initialization).
    pub async fn metadata(&self) -> Option<ExtensionMetadata> {
        self.metadata.lock().await.clone()
    }

    /// Get the event subscription for a specific event, if subscribed.
    pub async fn get_event_subscription(&self, event: EventName) -> Option<EventSubscription> {
        let metadata = self.metadata.lock().await;
        metadata
            .as_ref()?
            .capabilities
            .events
            .iter()
            .find(|sub| sub.name == event)
            .cloned()
    }

    /// Add a log entry.
    async fn add_log(&self, level: LogLevel, message: String) {
        let mut logs = self.logs.lock().await;
        let entry = ExtensionLog {
            timestamp: Timestamp::now(),
            level,
            message,
        };

        // add to end of deque
        logs.push_back(entry);

        // if we exceed capacity, remove oldest entry
        if logs.len() > MAX_LOG_ENTRIES {
            logs.pop_front();
        }
    }

    /// Get all log entries for this extension.
    pub async fn get_logs(&self) -> Vec<ExtensionLog> {
        self.logs.lock().await.iter().cloned().collect()
    }

    /// Send the initialize request and await response.
    pub async fn initialize(
        &mut self,
        hermes_version: &str,
        api_version: &str,
        data_dir: &Path,
    ) -> Result<(), ExtensionError> {
        {
            let mut state = self.state.lock().await;
            if *state != ExtensionState::Starting {
                return Err(ExtensionError::InvalidState(format!(
                    "cannot initialize from state {:?}",
                    *state
                )));
            }
            *state = ExtensionState::Initializing;
        }

        self.add_log(LogLevel::Info, "sending initialize request".to_string())
            .await;

        let params = InitializeParams {
            hermes_version: hermes_version.to_string(),
            api_version: api_version.to_string(),
            data_directory: data_dir.to_string_lossy().to_string(),
        };

        let result = timeout(
            INITIALIZE_TIMEOUT,
            self.send_request("initialize", serde_json::to_value(&params).unwrap()),
        )
        .await;

        match result {
            Ok(Ok(response)) => {
                // parse initialize result
                let init_result: InitializeResult = serde_json::from_value(response.result)
                    .map_err(|e| ExtensionError::Protocol(ProtocolError::Json(e)))?;

                log::info!(
                    "extension {} initialized: {} v{}",
                    self.id,
                    init_result.name,
                    init_result.version
                );

                let msg = format!(
                    "initialized successfully: {} v{}",
                    init_result.name, init_result.version
                );
                self.add_log(LogLevel::Info, msg).await;

                *self.metadata.lock().await = Some(init_result.into());
                *self.state.lock().await = ExtensionState::Running;
                Ok(())
            }
            Ok(Err(e)) => {
                let msg = format!("initialize failed: {e}");
                self.add_log(LogLevel::Error, msg.clone()).await;
                *self.state.lock().await = ExtensionState::Failed(msg.clone());
                Err(e)
            }
            Err(_) => {
                let msg = "initialize timed out".to_string();
                self.add_log(LogLevel::Error, msg.clone()).await;
                *self.state.lock().await = ExtensionState::Failed(msg.clone());
                self.kill().await;
                Err(ExtensionError::Timeout("initialize".to_string()))
            }
        }
    }

    /// Send a request to the extension and await the response.
    pub async fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Response, ExtensionError> {
        let id = {
            let mut next_id = self.next_request_id.lock().await;
            let id = *next_id;
            *next_id += 1;
            RequestId::Number(id)
        };

        let (response_tx, response_rx) = oneshot::channel();

        // register pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id.clone(), response_tx);
        }

        let request = Request::new(
            id.clone(),
            method,
            if params.is_null() { None } else { Some(params) },
        );

        // send the request
        if let Some(tx) = &self.outgoing_tx {
            tx.send(Message::Request(request))
                .await
                .map_err(|_| ExtensionError::Channel("failed to send request".to_string()))?;
        } else {
            return Err(ExtensionError::InvalidState(
                "extension not connected".to_string(),
            ));
        }

        // wait for response
        match response_rx.await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(error_response)) => Err(ExtensionError::Rpc(error_response.error)),
            Err(_) => Err(ExtensionError::Channel(
                "response channel closed".to_string(),
            )),
        }
    }

    /// Send a notification to the extension (no response expected).
    pub async fn send_notification(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<(), ExtensionError> {
        let notification =
            Notification::new(method, if params.is_null() { None } else { Some(params) });

        if let Some(tx) = &self.outgoing_tx {
            tx.send(Message::Notification(notification))
                .await
                .map_err(|_| ExtensionError::Channel("failed to send notification".to_string()))?;
            Ok(())
        } else {
            Err(ExtensionError::InvalidState(
                "extension not connected".to_string(),
            ))
        }
    }

    /// Initiate graceful shutdown of the extension.
    pub async fn shutdown(&mut self, reason: ShutdownReason) -> Result<(), ExtensionError> {
        {
            let mut state = self.state.lock().await;
            if state.is_terminated() {
                return Ok(()); // already stopped
            }
            *state = ExtensionState::ShuttingDown;
        }

        log::info!("shutting down extension {}", self.id);
        self.add_log(LogLevel::Info, "shutting down".to_string())
            .await;

        let params = ShutdownParams {
            reason: Some(reason),
        };
        let result = timeout(
            SHUTDOWN_TIMEOUT,
            self.send_request("shutdown", serde_json::to_value(&params).unwrap()),
        )
        .await;

        match result {
            Ok(Ok(_)) => {
                log::info!("extension {} shutdown gracefully", self.id);
                self.add_log(LogLevel::Info, "shutdown completed".to_string())
                    .await;
                *self.state.lock().await = ExtensionState::Stopped;
                self.cleanup().await;
                Ok(())
            }
            Ok(Err(e)) => {
                log::warn!("extension {} shutdown error: {e}", self.id);
                self.add_log(LogLevel::Warn, format!("shutdown error: {e}"))
                    .await;
                self.kill().await;
                Err(e)
            }
            Err(_) => {
                log::warn!("extension {} shutdown timed out, killing", self.id);
                self.add_log(
                    LogLevel::Warn,
                    "shutdown timed out, force killing".to_string(),
                )
                .await;
                self.kill().await;
                Ok(()) // timeout is acceptable for shutdown
            }
        }
    }

    /// Force kill the extension process.
    pub async fn kill(&mut self) {
        log::info!("killing extension {}", self.id);

        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
        }

        self.cleanup().await;

        let mut state = self.state.lock().await;
        if !state.is_terminated() {
            *state = ExtensionState::Stopped;
        }
    }

    /// Take the incoming message receiver for the host to process.
    ///
    /// The host spawns a task per extension to continuously process this channel,
    /// handling extension-initiated requests like `ui/openWindow` and `editor/getMessage`.
    pub fn take_incoming_rx(&mut self) -> Option<mpsc::Receiver<InternalMessage>> {
        self.incoming_rx.take()
    }

    /// Get a cloneable handle for sending responses back to this extension.
    ///
    /// Used by spawned tasks that need to send responses without holding a
    /// mutable reference to the `ExtensionProcess`.
    pub fn response_sender(&self) -> Option<ResponseSender> {
        self.outgoing_tx
            .as_ref()
            .map(|tx| ResponseSender { tx: tx.clone() })
    }

    /// Clean up resources after process termination.
    async fn cleanup(&mut self) {
        // drop outgoing channel to signal writer task to stop
        self.outgoing_tx = None;

        // cancel tasks
        if let Some(task) = self.reader_task.take() {
            task.abort();
        }
        if let Some(task) = self.writer_task.take() {
            task.abort();
        }

        // clear pending requests
        let mut pending = self.pending_requests.lock().await;
        pending.clear();
    }

    /// Mark the extension as failed with the given error message.
    #[allow(dead_code)]
    pub async fn mark_failed(&mut self, message: String) {
        *self.state.lock().await = ExtensionState::Failed(message);
        self.cleanup().await;
    }
}

/// Spawn the reader task that reads messages from the extension's stdout.
fn spawn_reader_task<R: AsyncBufRead + Unpin + Send + 'static>(
    mut reader: R,
    incoming_tx: mpsc::Sender<InternalMessage>,
    pending_requests: PendingRequests,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match read_message(&mut reader).await {
                Ok(msg) => {
                    match msg {
                        Message::Response(response) => {
                            // route response to waiting request
                            let id = response.id.clone();
                            let mut pending = pending_requests.lock().await;
                            if let Some(tx) = pending.remove(&id) {
                                let _ = tx.send(Ok(response));
                            } else {
                                log::warn!("received response for unknown request: {id}");
                            }
                        }
                        Message::Error(error) => {
                            // route error response to waiting request
                            if let Some(id) = &error.id {
                                let mut pending = pending_requests.lock().await;
                                if let Some(tx) = pending.remove(id) {
                                    let _ = tx.send(Err(error));
                                } else {
                                    log::warn!("received error for unknown request: {id}");
                                }
                            } else {
                                log::warn!("received error without request id: {:?}", error.error);
                            }
                        }
                        Message::Request(request) => {
                            // forward to host for handling
                            if incoming_tx
                                .send(InternalMessage::Request(request))
                                .await
                                .is_err()
                            {
                                break; // channel closed
                            }
                        }
                        Message::Notification(notification) => {
                            // forward to host for handling
                            if incoming_tx
                                .send(InternalMessage::Notification(notification))
                                .await
                                .is_err()
                            {
                                break; // channel closed
                            }
                        }
                    }
                }
                Err(ProtocolError::Eof) => {
                    log::info!("extension stdout closed (EOF)");
                    let _ = incoming_tx
                        .send(InternalMessage::ReaderError(ProtocolError::Eof))
                        .await;
                    break;
                }
                Err(e) => {
                    log::error!("protocol error reading from extension: {e}");
                    let _ = incoming_tx.send(InternalMessage::ReaderError(e)).await;
                    break;
                }
            }
        }
    })
}

/// Spawn the writer task that writes messages to the extension's stdin.
fn spawn_writer_task<W: AsyncWrite + Unpin + Send + 'static>(
    mut writer: W,
    mut outgoing_rx: mpsc::Receiver<Message>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            if let Err(e) = write_message(&mut writer, &msg).await {
                log::error!("failed to write message to extension: {e}");
                break;
            }
        }
    })
}

/// Generate a unique ID for an extension based on its path.
fn generate_extension_id(path: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    let hash = hasher.finish();
    format!("ext-{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_extension_id() {
        let id1 = generate_extension_id("/usr/bin/ext1");
        let id2 = generate_extension_id("/usr/bin/ext2");
        let id1_again = generate_extension_id("/usr/bin/ext1");

        assert!(id1.starts_with("ext-"));
        assert!(id2.starts_with("ext-"));
        assert_eq!(id1, id1_again);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_extension_error_display() {
        let error = ExtensionError::Timeout("initialize".to_string());
        assert!(format!("{error}").contains("timed out"));
        assert!(format!("{error}").contains("initialize"));

        let error = ExtensionError::ProcessExited;
        assert!(format!("{error}").contains("exited"));
    }

    #[test]
    fn test_extension_state_methods() {
        assert!(ExtensionState::Running.is_running());
        assert!(!ExtensionState::Starting.is_running());
        assert!(!ExtensionState::Stopped.is_running());

        assert!(ExtensionState::Stopped.is_terminated());
        assert!(ExtensionState::Failed("error".to_string()).is_terminated());
        assert!(!ExtensionState::Running.is_terminated());
    }
}
