//! Menu control commands.
//!
//! This module provides commands for dynamically controlling menu item state
//! from the frontend. This enables syncing menu item enabled/disabled states
//! with the corresponding toolbar buttons, and managing dynamic menu content
//! like the "Open Recent" submenu.
//!
//! # Dynamic Menu Updates
//!
//! Native menus in Tauri are typically static, but some features require dynamic updates:
//!
//! - **Enabled/Disabled State**: Save, Undo, and Redo menu items are enabled/disabled
//!   based on application state (unsaved changes, history availability).
//!
//! - **Open Recent Submenu**: The recent files list changes as users open/save files.
//!   The submenu is completely rebuilt each time via `update_recent_files_menu`.
//!
//! # Event Handling for Recent Files
//!
//! When a recent file menu item is clicked, the `on_menu_event` handler in `lib.rs`
//! matches the menu ID (e.g., `recent-file-0`) and looks up the corresponding file path
//! from `AppData.recent_files`. This lookup uses a blocking lock because `on_menu_event`
//! is synchronous. The file path is then emitted as the payload of `menu-open-recent`.

use std::path::Path;
use tauri::menu::{MenuItemBuilder, PredefinedMenuItem};
use tauri::{AppHandle, State};

use crate::AppData;

/// Set the enabled state of the Save menu item.
///
/// This command allows the frontend to sync the Save menu item's enabled state
/// with the toolbar save button. The save action should be disabled when there
/// are no unsaved changes or no file is currently open.
///
/// # Arguments
/// * `enabled` - Whether the Save menu item should be enabled
/// * `state` - Application state containing the menu item reference
///
/// # Returns
/// * `Ok(())` - State was updated successfully
/// * `Err(String)` - Failed to update the menu item state
#[tauri::command]
pub fn set_save_enabled(enabled: bool, state: State<'_, AppData>) -> Result<(), String> {
    state
        .save_menu_item
        .set_enabled(enabled)
        .map_err(|e| format!("Failed to set save menu enabled state: {e}"))
}

/// Set the enabled state of the Undo menu item.
///
/// This command allows the frontend to sync the Undo menu item's enabled state
/// with the undo history state. The undo action should be disabled when there
/// are no changes to undo.
///
/// # Arguments
/// * `enabled` - Whether the Undo menu item should be enabled
/// * `state` - Application state containing the menu item reference
///
/// # Returns
/// * `Ok(())` - State was updated successfully
/// * `Err(String)` - Failed to update the menu item state
#[tauri::command]
pub fn set_undo_enabled(enabled: bool, state: State<'_, AppData>) -> Result<(), String> {
    state
        .undo_menu_item
        .set_enabled(enabled)
        .map_err(|e| format!("Failed to set undo menu enabled state: {e}"))
}

/// Set the enabled state of the Redo menu item.
///
/// This command allows the frontend to sync the Redo menu item's enabled state
/// with the redo history state. The redo action should be disabled when there
/// are no changes to redo.
///
/// # Arguments
/// * `enabled` - Whether the Redo menu item should be enabled
/// * `state` - Application state containing the menu item reference
///
/// # Returns
/// * `Ok(())` - State was updated successfully
/// * `Err(String)` - Failed to update the menu item state
#[tauri::command]
pub fn set_redo_enabled(enabled: bool, state: State<'_, AppData>) -> Result<(), String> {
    state
        .redo_menu_item
        .set_enabled(enabled)
        .map_err(|e| format!("Failed to set redo menu enabled state: {e}"))
}

/// Update the "Open Recent" submenu with the given list of file paths.
///
/// This command clears the existing submenu items and repopulates it with
/// menu items for each recent file. Each item displays the filename and
/// emits a `menu-open-recent` event with the full path when clicked.
///
/// # Arguments
/// * `files` - List of file paths (most recent first, max 10)
/// * `app` - App handle for building menu items
/// * `state` - Application state containing the submenu reference
///
/// # Returns
/// * `Ok(())` - Menu was updated successfully
/// * `Err(String)` - Failed to update the menu
#[tauri::command]
pub async fn update_recent_files_menu(
    files: Vec<String>,
    app: AppHandle,
    state: State<'_, AppData>,
) -> Result<(), String> {
    // Store the files list for lookup when menu events fire
    {
        let mut recent_files = state.recent_files.lock().await;
        *recent_files = files.clone();
    }

    let submenu = &state.recent_files_submenu;

    // Remove all existing items
    while let Ok(Some(item)) = submenu.remove_at(0) {
        // Items are automatically cleaned up when removed
        drop(item);
    }

    // If no files, disable the submenu
    if files.is_empty() {
        submenu
            .set_enabled(false)
            .map_err(|e| format!("Failed to disable recent files menu: {e}"))?;
        return Ok(());
    }

    // Enable the submenu and add items
    submenu
        .set_enabled(true)
        .map_err(|e| format!("Failed to enable recent files menu: {e}"))?;

    // Add a menu item for each recent file
    for (index, file_path) in files.iter().enumerate() {
        // Extract just the filename for display
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

    // Add separator and "Clear Recent" item
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
