//! Application menu configuration and event handling.
//!
//! This module handles the native menu system for the Hermes application, including:
//! - Building the complete menu structure (File, Edit, View, Tools, Window, Help)
//! - Managing dynamic menu items (Save, Undo, Redo, Auto-Save, Recent Files)
//! - Routing menu events to the frontend via Tauri events
//!
//! # Why a Separate Module?
//!
//! Menu logic was extracted from `lib.rs` to keep the entry point focused on app
//! initialization. The menu system is complex enough (template submenus, dynamic
//! recent files, multiple state-synced items) to warrant its own module.
//!
//! # Event Flow
//!
//! 1. User clicks a menu item
//! 2. `setup_menu_event_handler` receives the event
//! 3. Event is mapped to a frontend event name (e.g., `file-save` → `menu-file-save`)
//! 4. Frontend event listeners handle the action
//!
//! # Dynamic State
//!
//! Some menu items need runtime state updates:
//! - **Save/Undo/Redo** - Enabled based on document state
//! - **Auto-Save** - Checked state synced with settings
//! - **Recent Files** - Rebuilt when files are opened/saved
//! - **Timestamp items** - Enabled only when cursor is in a valid field
//!
//! The [`state`] submodule provides Tauri commands for these updates.

mod state;

pub use state::*;

use tauri::menu::{
    AboutMetadata, CheckMenuItem, CheckMenuItemBuilder, MenuBuilder, MenuItem, MenuItemBuilder,
    PredefinedMenuItem, Submenu, SubmenuBuilder,
};
use tauri::{App, Emitter, Manager, Wry};

use crate::AppData;

/// Menu item references for dynamic state management.
///
/// These references are stored in AppData to allow runtime updates to menu item
/// enabled/checked states from frontend commands.
pub struct MenuItems {
    pub save_menu_item: MenuItem<Wry>,
    pub auto_save_menu_item: CheckMenuItem<Wry>,
    pub undo_menu_item: MenuItem<Wry>,
    pub redo_menu_item: MenuItem<Wry>,
    pub recent_files_submenu: Submenu<Wry>,
    pub insert_timestamp_now_menu_item: MenuItem<Wry>,
    pub insert_timestamp_menu_item: MenuItem<Wry>,
}

/// Build the complete application menu and return references to dynamic items.
///
/// This function constructs all menu items and submenus, sets the application menu,
/// and returns references to items that need runtime state updates.
pub fn build_menu(app: &App) -> color_eyre::Result<MenuItems> {
    // Build the Save menu item separately so we can store a reference for dynamic enable/disable
    let save_menu_item = MenuItemBuilder::new("&Save")
        .id("file-save")
        .accelerator("CmdOrCtrl+S")
        .enabled(false)
        .build(app)?;

    // Build the Auto-Save checkable menu item (initial state synced from settings by frontend)
    let auto_save_menu_item = CheckMenuItemBuilder::new("Auto-Save")
        .id("file-auto-save")
        .checked(false)
        .build(app)?;

    // Build the "Open Recent" submenu (starts empty, populated by frontend)
    let recent_files_submenu = SubmenuBuilder::new(app, "Open &Recent")
        .id("file-open-recent")
        .enabled(false)
        .build()?;

    // Build the "New from Template" submenu with pre-populated message types
    let template_submenu = build_template_submenu(app)?;

    // Build the "Export As" submenu for exporting to different formats
    let export_submenu = SubmenuBuilder::new(app, "&Export As")
        .id("file-export")
        .item(
            &MenuItemBuilder::new("&JSON...")
                .id("file-export-json")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("&YAML...")
                .id("file-export-yaml")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("&TOML...")
                .id("file-export-toml")
                .build(app)?,
        )
        .build()?;

    // Build the "Import From" submenu for importing from different formats
    let import_submenu = SubmenuBuilder::new(app, "&Import From")
        .id("file-import")
        .item(
            &MenuItemBuilder::new("&JSON...")
                .id("file-import-json")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("&YAML...")
                .id("file-import-yaml")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("&TOML...")
                .id("file-import-toml")
                .build(app)?,
        )
        .build()?;

    // Build the File menu with standard file operations
    let file_menu = SubmenuBuilder::new(app, "&File")
        .item(
            &MenuItemBuilder::new("&New")
                .id("file-new")
                .accelerator("CmdOrCtrl+N")
                .build(app)?,
        )
        .item(&template_submenu)
        .item(
            &MenuItemBuilder::new("&Open...")
                .id("file-open")
                .accelerator("CmdOrCtrl+O")
                .build(app)?,
        )
        .item(&recent_files_submenu)
        .separator()
        .item(&save_menu_item)
        .item(
            &MenuItemBuilder::new("Save &As...")
                .id("file-save-as")
                .accelerator("CmdOrCtrl+Shift+S")
                .build(app)?,
        )
        .item(&export_submenu)
        .item(&import_submenu)
        .separator()
        .item(&auto_save_menu_item)
        .separator()
        .item(&PredefinedMenuItem::quit(app, Some("&Quit"))?)
        .build()?;

    // Build Edit menu
    let undo_menu_item = MenuItemBuilder::new("&Undo")
        .id("edit-undo")
        .accelerator("CmdOrCtrl+Z")
        .enabled(false)
        .build(app)?;

    let redo_menu_item = MenuItemBuilder::new("&Redo")
        .id("edit-redo")
        .accelerator("CmdOrCtrl+Shift+Z")
        .enabled(false)
        .build(app)?;

    let edit_menu = build_edit_menu(app, &undo_menu_item, &redo_menu_item)?;

    // Build Help menu
    let help_menu = build_help_menu(app)?;

    // Build View menu
    let view_menu = build_view_menu(app)?;

    // Build Tools menu with timestamp items
    let insert_timestamp_now_menu_item = MenuItemBuilder::new("Insert &Current Timestamp")
        .id("tools-insert-timestamp-now")
        .accelerator("CmdOrCtrl+Shift+T")
        .enabled(false)
        .build(app)?;

    let insert_timestamp_menu_item = MenuItemBuilder::new("Insert &Timestamp...")
        .id("tools-insert-timestamp")
        .enabled(false)
        .build(app)?;

    let tools_menu = build_tools_menu(
        app,
        &insert_timestamp_now_menu_item,
        &insert_timestamp_menu_item,
    )?;

    // Build Window menu
    let window_menu = build_window_menu(app)?;

    let menu = MenuBuilder::new(app)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&tools_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()?;

    app.set_menu(menu)?;

    Ok(MenuItems {
        save_menu_item,
        auto_save_menu_item,
        undo_menu_item,
        redo_menu_item,
        recent_files_submenu,
        insert_timestamp_now_menu_item,
        insert_timestamp_menu_item,
    })
}

/// Register the menu event handler that routes events to the frontend.
pub fn setup_menu_event_handler(app: &App) {
    app.on_menu_event(move |app_handle, event| {
        let event_id = event.id().as_ref();

        // handle standard menu events (emit empty payload)
        let event_name = match event_id {
            "file-new" => Some("menu-file-new"),
            "file-open" => Some("menu-file-open"),
            "file-save" => Some("menu-file-save"),
            "file-save-as" => Some("menu-file-save-as"),
            "file-export-json" => Some("menu-file-export-json"),
            "file-export-yaml" => Some("menu-file-export-yaml"),
            "file-export-toml" => Some("menu-file-export-toml"),
            "file-import-json" => Some("menu-file-import-json"),
            "file-import-yaml" => Some("menu-file-import-yaml"),
            "file-import-toml" => Some("menu-file-import-toml"),
            "file-auto-save" => Some("menu-file-auto-save"),
            "edit-undo" => Some("menu-edit-undo"),
            "edit-redo" => Some("menu-edit-redo"),
            "edit-find" => Some("menu-edit-find"),
            "edit-find-replace" => Some("menu-edit-find-replace"),
            "edit-jump-to-field" => Some("menu-edit-jump-to-field"),
            "edit-delete-segment" => Some("menu-edit-delete-segment"),
            "edit-move-segment-up" => Some("menu-edit-move-segment-up"),
            "edit-move-segment-down" => Some("menu-edit-move-segment-down"),
            "edit-duplicate-segment" => Some("menu-edit-duplicate-segment"),
            "view-zoom-in" => Some("menu-view-zoom-in"),
            "view-zoom-out" => Some("menu-view-zoom-out"),
            "view-reset-zoom" => Some("menu-view-reset-zoom"),
            "view-keyboard-shortcuts" => Some("menu-view-keyboard-shortcuts"),
            "tools-send" => Some("menu-tools-send"),
            "tools-listen" => Some("menu-tools-listen"),
            "tools-validate" => Some("menu-tools-validate"),
            "tools-compare" => Some("menu-tools-compare"),
            "tools-generate-control-id" => Some("menu-tools-generate-control-id"),
            "tools-insert-timestamp-now" => Some("menu-tools-insert-timestamp-now"),
            "tools-insert-timestamp" => Some("menu-tools-insert-timestamp"),
            "recent-clear" => Some("menu-clear-recent"),
            "help" => Some("menu-help"),
            "help-check-updates" => {
                crate::updater::handle_check_updates(app_handle);
                return;
            }
            _ => None,
        };

        if let Some(name) = event_name {
            // emit to main window only so other windows can handle shortcuts independently
            let _ = app_handle.emit_to("main", name, ());
            return;
        }

        // handle recent file menu items (emit file path as payload)
        if let Some(index_str) = event_id.strip_prefix("recent-file-") {
            if let Ok(index) = index_str.parse::<usize>() {
                if let Some(state) = app_handle.try_state::<AppData>() {
                    let recent_files = state.recent_files.blocking_lock();
                    if let Some(file_path) = recent_files.get(index) {
                        let _ = app_handle.emit("menu-open-recent", file_path.clone());
                    }
                }
            }
            return;
        }

        // handle template menu items (emit template name as payload)
        if let Some(template_name) = event_id.strip_prefix("template-") {
            let _ = app_handle.emit("menu-new-from-template", template_name);
        }
    });
}

fn build_template_submenu(app: &App) -> color_eyre::Result<Submenu<Wry>> {
    let submenu = SubmenuBuilder::new(app, "New from &Template")
        .id("file-new-from-template")
        // ADT messages
        .item(
            &MenuItemBuilder::new("ADT^A01 (Admit)")
                .id("template-adt_a01")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A02 (Transfer)")
                .id("template-adt_a02")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A03 (Discharge)")
                .id("template-adt_a03")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A04 (Register)")
                .id("template-adt_a04")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A05 (Pre-admit)")
                .id("template-adt_a05")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A08 (Update)")
                .id("template-adt_a08")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A11 (Cancel Admit)")
                .id("template-adt_a11")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A12 (Cancel Transfer)")
                .id("template-adt_a12")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A13 (Cancel Discharge)")
                .id("template-adt_a13")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A23 (Delete)")
                .id("template-adt_a23")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A34 (Merge Patient - ID Only)")
                .id("template-adt_a34")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A40 (Merge Patient)")
                .id("template-adt_a40")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A49 (Change Patient ID)")
                .id("template-adt_a49")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ADT^A50 (Change Visit ID)")
                .id("template-adt_a50")
                .build(app)?,
        )
        .separator()
        // Order messages
        .item(
            &MenuItemBuilder::new("ORM^O01 (Order)")
                .id("template-orm_o01")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ORU^R01 (Results)")
                .id("template-oru_r01")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("ORR^O02 (Order Response)")
                .id("template-orr_o02")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("DFT^P03 (Financial)")
                .id("template-dft_p03")
                .build(app)?,
        )
        .build()?;

    Ok(submenu)
}

fn build_edit_menu(
    app: &App,
    undo_menu_item: &MenuItem<Wry>,
    redo_menu_item: &MenuItem<Wry>,
) -> color_eyre::Result<Submenu<Wry>> {
    let find_menu_item = MenuItemBuilder::new("&Find...")
        .id("edit-find")
        .accelerator("CmdOrCtrl+F")
        .build(app)?;

    let find_replace_menu_item = MenuItemBuilder::new("Find and &Replace...")
        .id("edit-find-replace")
        .accelerator("CmdOrCtrl+H")
        .build(app)?;

    let jump_to_field_menu_item = MenuItemBuilder::new("&Jump to Field...")
        .id("edit-jump-to-field")
        .accelerator("CmdOrCtrl+J")
        .build(app)?;

    let delete_segment_menu_item = MenuItemBuilder::new("D&elete Segment")
        .id("edit-delete-segment")
        .accelerator("CmdOrCtrl+Shift+K")
        .build(app)?;

    let move_segment_up_menu_item = MenuItemBuilder::new("Move Segment &Up")
        .id("edit-move-segment-up")
        .accelerator("CmdOrCtrl+Shift+Up")
        .build(app)?;

    let move_segment_down_menu_item = MenuItemBuilder::new("Move Segment Do&wn")
        .id("edit-move-segment-down")
        .accelerator("CmdOrCtrl+Shift+Down")
        .build(app)?;

    let duplicate_segment_menu_item = MenuItemBuilder::new("Duplicate Se&gment")
        .id("edit-duplicate-segment")
        .accelerator("CmdOrCtrl+Shift+D")
        .build(app)?;

    let menu = SubmenuBuilder::new(app, "&Edit")
        .item(undo_menu_item)
        .item(redo_menu_item)
        .separator()
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&PredefinedMenuItem::paste(app, None)?)
        .separator()
        .item(&find_menu_item)
        .item(&find_replace_menu_item)
        .item(&jump_to_field_menu_item)
        .separator()
        .item(&delete_segment_menu_item)
        .item(&move_segment_up_menu_item)
        .item(&move_segment_down_menu_item)
        .item(&duplicate_segment_menu_item)
        .separator()
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .build()?;

    Ok(menu)
}

fn build_help_menu(app: &App) -> color_eyre::Result<Submenu<Wry>> {
    let help_menu_item = MenuItemBuilder::new("&Help")
        .id("help")
        .accelerator("F1")
        .build(app)?;

    let check_updates_menu_item = MenuItemBuilder::new("Check for &Updates...")
        .id("help-check-updates")
        .build(app)?;

    let about_metadata = AboutMetadata {
        name: Some(env!("CARGO_PKG_NAME").into()),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        short_version: None,
        authors: Some(vec![env!("CARGO_PKG_AUTHORS").into()]),
        comments: Some(env!("CARGO_PKG_DESCRIPTION").into()),
        copyright: Some("© 2025 Kenton Hamaluik".into()),
        license: Some(env!("CARGO_PKG_LICENSE").into()),
        website: Some(env!("CARGO_PKG_HOMEPAGE").into()),
        website_label: Some("Visit Website".into()),
        credits: Some("Kenton Hamaluik".into()),
        ..Default::default()
    };

    let menu = SubmenuBuilder::new(app, "&Help")
        .item(&help_menu_item)
        .separator()
        .item(&check_updates_menu_item)
        .separator()
        .item(&PredefinedMenuItem::about(
            app,
            Some("About Hermes"),
            Some(about_metadata),
        )?)
        .build()?;

    Ok(menu)
}

fn build_view_menu(app: &App) -> color_eyre::Result<Submenu<Wry>> {
    let menu = SubmenuBuilder::new(app, "&View")
        .item(
            &MenuItemBuilder::new("Zoom &In")
                .id("view-zoom-in")
                .accelerator("CmdOrCtrl+=")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("Zoom &Out")
                .id("view-zoom-out")
                .accelerator("CmdOrCtrl+-")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("&Reset Zoom")
                .id("view-reset-zoom")
                .accelerator("CmdOrCtrl+0")
                .build(app)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::new("&Keyboard Shortcuts")
                .id("view-keyboard-shortcuts")
                .accelerator("CmdOrCtrl+/")
                .build(app)?,
        )
        .build()?;

    Ok(menu)
}

fn build_tools_menu(
    app: &App,
    insert_timestamp_now_menu_item: &MenuItem<Wry>,
    insert_timestamp_menu_item: &MenuItem<Wry>,
) -> color_eyre::Result<Submenu<Wry>> {
    let menu = SubmenuBuilder::new(app, "&Tools")
        .item(
            &MenuItemBuilder::new("&Send Message...")
                .id("tools-send")
                .accelerator("CmdOrCtrl+T")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("&Listen for Messages...")
                .id("tools-listen")
                .accelerator("CmdOrCtrl+L")
                .build(app)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::new("&Validate Message")
                .id("tools-validate")
                .accelerator("CmdOrCtrl+Shift+V")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::new("&Compare Messages...")
                .id("tools-compare")
                .accelerator("CmdOrCtrl+D")
                .build(app)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::new("&Generate Control ID")
                .id("tools-generate-control-id")
                .accelerator("CmdOrCtrl+G")
                .build(app)?,
        )
        .item(insert_timestamp_now_menu_item)
        .item(insert_timestamp_menu_item)
        .build()?;

    Ok(menu)
}

fn build_window_menu(app: &App) -> color_eyre::Result<Submenu<Wry>> {
    let menu = SubmenuBuilder::new(app, "&Window")
        .item(&PredefinedMenuItem::minimize(app, None)?)
        .item(&PredefinedMenuItem::maximize(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::close_window(app, None)?)
        .build()?;

    Ok(menu)
}
