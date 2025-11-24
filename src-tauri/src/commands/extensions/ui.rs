//! UI operation handlers for extension requests.
//!
//! These functions handle `ui/*` JSON-RPC requests from extensions:
//!
//! - `ui/openWindow` - Open a browser window with extension-served URL
//! - `ui/closeWindow` - Close a previously opened window
//!
//! Also provides the `window/closed` notification for informing extensions
//! when their windows are closed.

use crate::extensions::protocol::RpcError;
use crate::extensions::types::{
    CloseWindowParams, CloseWindowResult, OpenWindowParams, OpenWindowResult, WindowClosedParams,
    WindowClosedReason,
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::Mutex;
use url::Url;

/// Tracks extension windows for cleanup and notification routing.
#[derive(Debug, Default)]
pub struct WindowManager {
    /// Maps window ID to (extension ID, is_tracked) for cleanup and notifications.
    windows: HashMap<String, WindowInfo>,
}

#[derive(Debug, Clone)]
struct WindowInfo {
    extension_id: String,
}

impl WindowManager {
    /// Create a new window manager.
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    /// Track a new window.
    pub fn track_window(&mut self, window_id: &str, extension_id: &str) {
        self.windows.insert(
            window_id.to_string(),
            WindowInfo {
                extension_id: extension_id.to_string(),
            },
        );
    }

    /// Remove a window from tracking.
    pub fn untrack_window(&mut self, window_id: &str) -> Option<String> {
        self.windows.remove(window_id).map(|info| info.extension_id)
    }

    /// Get the extension ID for a window.
    pub fn get_extension_id(&self, window_id: &str) -> Option<&str> {
        self.windows
            .get(window_id)
            .map(|info| info.extension_id.as_str())
    }

    /// Get all window IDs for an extension.
    pub fn get_windows_for_extension(&self, extension_id: &str) -> Vec<String> {
        self.windows
            .iter()
            .filter(|(_, info)| info.extension_id == extension_id)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Clear all windows.
    pub fn clear(&mut self) {
        self.windows.clear();
    }
}

/// Global window manager instance.
///
/// This is wrapped in Arc<Mutex> to allow safe concurrent access from
/// multiple async tasks (window event handlers, extension requests).
pub type SharedWindowManager = Arc<Mutex<WindowManager>>;

/// Create a new shared window manager.
pub fn create_window_manager() -> SharedWindowManager {
    Arc::new(Mutex::new(WindowManager::new()))
}

/// Handle `ui/openWindow` request from an extension.
///
/// Opens a new browser window loading the specified URL. The URL must use
/// http or https scheme (typically localhost for extension-served content).
pub async fn handle_open_window(
    app: &AppHandle,
    ext_id: &str,
    params: OpenWindowParams,
    window_manager: &SharedWindowManager,
) -> Result<OpenWindowResult, RpcError> {
    // validate URL
    let url =
        Url::parse(&params.url).map_err(|e| RpcError::invalid_url(format!("Invalid URL: {e}")))?;

    // check scheme
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(RpcError::invalid_url(
            "URL scheme must be http or https".to_string(),
        ));
    }

    // generate unique window ID
    let window_id = format!("ext-window-{}", uuid::Uuid::new_v4());

    // build window configuration
    let width = params.width.unwrap_or(800) as f64;
    let height = params.height.unwrap_or(600) as f64;
    let resizable = params.resizable.unwrap_or(true);

    // create the window
    let mut builder = WebviewWindowBuilder::new(app, &window_id, WebviewUrl::External(url))
        .title(&params.title)
        .inner_size(width, height)
        .resizable(resizable);

    // handle modal windows
    if params.modal.unwrap_or(false) {
        if let Some(main_window) = app.get_webview_window("main") {
            // set parent for modal behaviour
            #[cfg(target_os = "macos")]
            {
                builder = builder.parent(&main_window).map_err(|e| {
                    RpcError::window_error(format!("Failed to set parent window: {e}"))
                })?;
            }
            #[cfg(not(target_os = "macos"))]
            {
                // on other platforms, just use owner relationship
                builder = builder.owner(&main_window).map_err(|e| {
                    RpcError::window_error(format!("Failed to set owner window: {e}"))
                })?;
            }
        }
    }

    let window = builder
        .build()
        .map_err(|e| RpcError::window_error(format!("Failed to create window: {e}")))?;

    // track the window
    {
        let mut manager = window_manager.lock().await;
        manager.track_window(&window_id, ext_id);
    }

    // set up close event listener
    let window_id_clone = window_id.clone();
    let app_clone = app.clone();
    let window_manager_clone = window_manager.clone();

    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            let window_id = window_id_clone.clone();
            let app = app_clone.clone();
            let manager = window_manager_clone.clone();

            // spawn async task to handle cleanup
            tauri::async_runtime::spawn(async move {
                handle_window_closed(&app, &window_id, WindowClosedReason::User, &manager).await;
            });
        }
    });

    Ok(OpenWindowResult { window_id })
}

/// Handle `ui/closeWindow` request from an extension.
///
/// Closes a window that was previously opened by the extension.
pub async fn handle_close_window(
    app: &AppHandle,
    ext_id: &str,
    params: CloseWindowParams,
    window_manager: &SharedWindowManager,
) -> Result<CloseWindowResult, RpcError> {
    // verify the window belongs to this extension and untrack it before closing
    // we untrack first to prevent the window event handler from also emitting
    let ext_id_from_tracking = {
        let mut manager = window_manager.lock().await;
        if let Some(owner_id) = manager.get_extension_id(&params.window_id) {
            if owner_id != ext_id {
                return Err(RpcError::window_error(format!(
                    "Window {} does not belong to extension {}",
                    params.window_id, ext_id
                )));
            }
        }
        // untrack now so the window event handler won't emit a duplicate notification
        manager.untrack_window(&params.window_id)
    };

    // try to close the window
    if let Some(window) = app.get_webview_window(&params.window_id) {
        window
            .close()
            .map_err(|e| RpcError::window_error(format!("Failed to close window: {e}")))?;
    }

    // emit notification with the correct reason (Extension, not User)
    if let Some(ext_id) = ext_id_from_tracking {
        let params = WindowClosedParams {
            window_id: params.window_id,
            reason: WindowClosedReason::Extension,
        };
        if let Err(e) = app.emit("extension-window-closed", (&ext_id, &params)) {
            log::warn!("Failed to emit window-closed event: {e}");
        }
    }

    // always return success (window might already be closed)
    Ok(CloseWindowResult { success: true })
}

/// Handle window closed event.
///
/// Removes the window from tracking and sends notification to the extension.
pub async fn handle_window_closed(
    app: &AppHandle,
    window_id: &str,
    reason: WindowClosedReason,
    window_manager: &SharedWindowManager,
) {
    // remove from tracking and get extension ID
    let ext_id = {
        let mut manager = window_manager.lock().await;
        manager.untrack_window(window_id)
    };

    // emit event for extension host to forward to extension
    if let Some(ext_id) = ext_id {
        let params = WindowClosedParams {
            window_id: window_id.to_string(),
            reason,
        };

        // emit event that the host can listen for
        if let Err(e) = app.emit("extension-window-closed", (&ext_id, &params)) {
            log::warn!("Failed to emit window-closed event: {e}");
        }
    }
}

/// Close all windows belonging to an extension.
///
/// Called during extension shutdown to clean up any open windows.
pub async fn close_extension_windows(
    app: &AppHandle,
    ext_id: &str,
    window_manager: &SharedWindowManager,
) {
    // get all windows for this extension
    let window_ids = {
        let manager = window_manager.lock().await;
        manager.get_windows_for_extension(ext_id)
    };

    // close each window
    for window_id in window_ids {
        if let Some(window) = app.get_webview_window(&window_id) {
            if let Err(e) = window.close() {
                log::warn!("Failed to close extension window {window_id}: {e}");
            }
        }

        // clean up tracking
        handle_window_closed(
            app,
            &window_id,
            WindowClosedReason::Shutdown,
            window_manager,
        )
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_track_untrack() {
        let mut manager = WindowManager::new();

        manager.track_window("win-1", "ext-1");
        manager.track_window("win-2", "ext-1");
        manager.track_window("win-3", "ext-2");

        assert_eq!(manager.get_extension_id("win-1"), Some("ext-1"));
        assert_eq!(manager.get_extension_id("win-3"), Some("ext-2"));
        assert_eq!(manager.get_extension_id("win-999"), None);

        let ext_id = manager.untrack_window("win-1");
        assert_eq!(ext_id, Some("ext-1".to_string()));
        assert_eq!(manager.get_extension_id("win-1"), None);
    }

    #[test]
    fn test_window_manager_get_windows_for_extension() {
        let mut manager = WindowManager::new();

        manager.track_window("win-1", "ext-1");
        manager.track_window("win-2", "ext-1");
        manager.track_window("win-3", "ext-2");

        let ext1_windows = manager.get_windows_for_extension("ext-1");
        assert_eq!(ext1_windows.len(), 2);
        assert!(ext1_windows.contains(&"win-1".to_string()));
        assert!(ext1_windows.contains(&"win-2".to_string()));

        let ext2_windows = manager.get_windows_for_extension("ext-2");
        assert_eq!(ext2_windows.len(), 1);
        assert!(ext2_windows.contains(&"win-3".to_string()));

        let ext3_windows = manager.get_windows_for_extension("ext-3");
        assert!(ext3_windows.is_empty());
    }

    #[test]
    fn test_window_manager_clear() {
        let mut manager = WindowManager::new();

        manager.track_window("win-1", "ext-1");
        manager.track_window("win-2", "ext-2");

        manager.clear();

        assert_eq!(manager.get_extension_id("win-1"), None);
        assert_eq!(manager.get_extension_id("win-2"), None);
    }

    #[test]
    fn test_open_window_params_serialisation() {
        let params = OpenWindowParams {
            url: "http://localhost:9876/wizard".to_string(),
            title: "Test Window".to_string(),
            width: Some(600),
            height: Some(400),
            modal: Some(true),
            resizable: Some(false),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"url\":\"http://localhost:9876/wizard\""));
        assert!(json.contains("\"modal\":true"));
    }

    #[test]
    fn test_close_window_params_serialisation() {
        let params = CloseWindowParams {
            window_id: "ext-window-abc123".to_string(),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"windowId\":\"ext-window-abc123\""));
    }

    #[test]
    fn test_window_closed_params_serialisation() {
        let params = WindowClosedParams {
            window_id: "ext-window-abc123".to_string(),
            reason: WindowClosedReason::User,
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"windowId\":\"ext-window-abc123\""));
        assert!(json.contains("\"reason\":\"user\""));
    }

    #[test]
    fn test_window_closed_reason_serialisation() {
        assert_eq!(
            serde_json::to_string(&WindowClosedReason::User).unwrap(),
            "\"user\""
        );
        assert_eq!(
            serde_json::to_string(&WindowClosedReason::Extension).unwrap(),
            "\"extension\""
        );
        assert_eq!(
            serde_json::to_string(&WindowClosedReason::Shutdown).unwrap(),
            "\"shutdown\""
        );
    }

    #[test]
    fn test_url_validation_logic() {
        // valid URLs
        assert!(Url::parse("http://localhost:9876/wizard").is_ok());
        assert!(Url::parse("https://example.com/app").is_ok());

        // invalid URLs
        assert!(Url::parse("file:///etc/passwd").is_ok()); // parses but scheme check should fail
        assert!(Url::parse("not a url").is_err());

        // scheme checks
        let http_url = Url::parse("http://localhost:9876").unwrap();
        assert!(http_url.scheme() == "http" || http_url.scheme() == "https");

        let file_url = Url::parse("file:///etc/passwd").unwrap();
        assert!(file_url.scheme() != "http" && file_url.scheme() != "https");
    }
}
