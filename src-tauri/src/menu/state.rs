//! Menu state control commands.
//!
//! This module provides Tauri commands for dynamically controlling menu item state
//! from the frontend, and for opening auxiliary windows.

use std::path::Path;
use tauri::menu::{MenuItemBuilder, PredefinedMenuItem};
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Manager, State, WebviewUrl};

use crate::AppData;

/// Set the enabled state of the Save menu item.
#[tauri::command]
pub fn set_save_enabled(enabled: bool, state: State<'_, AppData>) -> Result<(), String> {
    state
        .save_menu_item
        .set_enabled(enabled)
        .map_err(|e| format!("Failed to set save menu enabled state: {e}"))
}

/// Set the enabled state of the Undo menu item.
#[tauri::command]
pub fn set_undo_enabled(enabled: bool, state: State<'_, AppData>) -> Result<(), String> {
    state
        .undo_menu_item
        .set_enabled(enabled)
        .map_err(|e| format!("Failed to set undo menu enabled state: {e}"))
}

/// Set the enabled state of the Redo menu item.
#[tauri::command]
pub fn set_redo_enabled(enabled: bool, state: State<'_, AppData>) -> Result<(), String> {
    state
        .redo_menu_item
        .set_enabled(enabled)
        .map_err(|e| format!("Failed to set redo menu enabled state: {e}"))
}

/// Set the checked state of the Auto-Save menu item.
#[tauri::command]
pub fn set_auto_save_checked(checked: bool, state: State<'_, AppData>) -> Result<(), String> {
    state
        .auto_save_menu_item
        .set_checked(checked)
        .map_err(|e| format!("Failed to set auto-save menu checked state: {e}"))
}

/// Update the "Open Recent" submenu with the given list of file paths.
#[tauri::command]
pub async fn update_recent_files_menu(
    files: Vec<String>,
    app: AppHandle,
    state: State<'_, AppData>,
) -> Result<(), String> {
    // store the files list for lookup when menu events fire
    {
        let mut recent_files = state.recent_files.lock().await;
        *recent_files = files.clone();
    }

    let submenu = &state.recent_files_submenu;

    // remove all existing items
    while let Ok(Some(item)) = submenu.remove_at(0) {
        drop(item);
    }

    // if no files, disable the submenu
    if files.is_empty() {
        submenu
            .set_enabled(false)
            .map_err(|e| format!("Failed to disable recent files menu: {e}"))?;
        return Ok(());
    }

    // enable the submenu and add items
    submenu
        .set_enabled(true)
        .map_err(|e| format!("Failed to enable recent files menu: {e}"))?;

    // add a menu item for each recent file
    for (index, file_path) in files.iter().enumerate() {
        let display_name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path);

        let menu_item = MenuItemBuilder::new(display_name)
            .id(format!("recent-file-{}", index))
            .build(&app)
            .map_err(|e| format!("Failed to build recent file menu item: {e}"))?;

        submenu
            .append(&menu_item)
            .map_err(|e| format!("Failed to append recent file to menu: {e}"))?;
    }

    // add separator and "Clear Recent" item
    let separator = PredefinedMenuItem::separator(&app)
        .map_err(|e| format!("Failed to create separator: {e}"))?;
    submenu
        .append(&separator)
        .map_err(|e| format!("Failed to append separator: {e}"))?;

    let clear_item = MenuItemBuilder::new("Clear Recent")
        .id("recent-clear")
        .build(&app)
        .map_err(|e| format!("Failed to build clear recent menu item: {e}"))?;
    submenu
        .append(&clear_item)
        .map_err(|e| format!("Failed to append clear recent item: {e}"))?;

    Ok(())
}

/// Set the enabled state of the timestamp insertion menu items.
#[tauri::command]
pub fn set_insert_timestamp_enabled(
    enabled: bool,
    state: State<'_, AppData>,
) -> Result<(), String> {
    state
        .insert_timestamp_now_menu_item
        .set_enabled(enabled)
        .map_err(|e| format!("Failed to set insert timestamp now menu enabled state: {e}"))?;
    state
        .insert_timestamp_menu_item
        .set_enabled(enabled)
        .map_err(|e| format!("Failed to set insert timestamp menu enabled state: {e}"))
}

/// Open the help window.
#[tauri::command]
pub async fn open_help_window(app: AppHandle) -> Result<(), String> {
    // check if help window already exists
    if let Some(window) = app.get_webview_window("help") {
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus help window: {e}"))?;
        return Ok(());
    }

    // create new help window
    WebviewWindowBuilder::new(&app, "help", WebviewUrl::App("/help.html".into()))
        .title("Hermes Help")
        .inner_size(900.0, 700.0)
        .build()
        .map_err(|e| format!("Failed to create help window: {e}"))?;

    Ok(())
}
