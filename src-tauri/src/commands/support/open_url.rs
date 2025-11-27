//! Open URLs in the OS default browser.

/// Opens a URL in the user's default browser.
///
/// Used by help.html to ensure external links open in the browser rather than
/// navigating within the Tauri webview.
#[tauri::command]
pub fn open_url(url: &str) -> Result<(), String> {
    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())
}
