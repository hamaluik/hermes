//! Menu control commands.
//!
//! This module provides commands for dynamically controlling menu item state
//! from the frontend. This enables syncing menu item enabled/disabled states
//! with the corresponding toolbar buttons.

use tauri::State;

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
