//! Hermes - Desktop application for composing, sending, and receiving HL7 messages.
//!
//! This application provides a Tauri-based GUI for HL7 message manipulation, primarily
//! targeting development and testing workflows. It supports message editing, syntax
//! highlighting, field descriptions, and MLLP network communication.
//!
//! # Module Organisation
//!
//! The backend is organised by feature:
//!
//! - [`commands`] - Tauri command handlers, grouped by feature:
//!   - `communication/` - MLLP send/receive
//!   - `editor/` - Cursor tracking, data manipulation, syntax highlighting
//!   - `validation/` - Message validation and comparison
//!   - `support/` - Field descriptions and schema queries
//! - [`menu`] - Native menu building and state management
//! - [`schema`] - HL7 schema caching from TOML files
//! - [`spec`] - HL7 standard field descriptions
//!
//! # State Management
//!
//! Application state is managed via [`AppData`], which holds:
//! - Cached HL7 schema
//! - MLLP listener task handle
//! - Menu item references for dynamic enable/disable

use color_eyre::eyre::Context;
use schema::cache::SchemaCache;
use tauri::menu::{CheckMenuItem, MenuItem, Submenu};
use tauri::{Manager, Wry};
use tokio::sync::Mutex;

mod commands;
mod menu;
mod schema;
mod spec;

/// Application-wide state managed by Tauri.
///
/// This state is initialized once during app setup and is accessible to all Tauri commands
/// via the `State<AppData>` parameter. The state is thread-safe and can be accessed from
/// multiple async tasks concurrently.
pub struct AppData {
    /// Cached HL7 schema loaded from messages.toml.
    schema: SchemaCache,

    /// Handle to the MLLP listener background task.
    listen_join: Mutex<Option<tokio::task::JoinHandle<()>>>,

    /// Reference to the Save menu item for dynamic enable/disable.
    pub save_menu_item: MenuItem<Wry>,

    /// Reference to the Auto-Save checkable menu item for sync with settings.
    pub auto_save_menu_item: CheckMenuItem<Wry>,

    /// Reference to the Undo menu item for dynamic enable/disable.
    pub undo_menu_item: MenuItem<Wry>,

    /// Reference to the Redo menu item for dynamic enable/disable.
    pub redo_menu_item: MenuItem<Wry>,

    /// Reference to the "Open Recent" submenu for dynamic population.
    pub recent_files_submenu: Submenu<Wry>,

    /// Current list of recent file paths.
    pub recent_files: Mutex<Vec<String>>,

    /// Reference to the "Insert Current Timestamp" menu item for dynamic enable/disable.
    pub insert_timestamp_now_menu_item: MenuItem<Wry>,

    /// Reference to the "Insert Timestamp..." menu item for dynamic enable/disable.
    pub insert_timestamp_menu_item: MenuItem<Wry>,
}

/// Main entry point for the Hermes application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    color_eyre::config::HookBuilder::new()
        .theme(color_eyre::config::Theme::new())
        .install()
        .expect("Failed to install `color_eyre`");

    let log_level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Warn)
                .level_for("hermes", log_level)
                .level_for("hermes_lib", log_level)
                .format(|out, message, record| {
                    let now = jiff::Zoned::now();
                    out.finish(format_args!(
                        "{now}[{target}][{level}] {message}",
                        now = now,
                        target = record.target(),
                        level = record.level(),
                        message = message
                    ))
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::syntax_highlight,
            commands::locate_cursor,
            commands::get_range_of_next_field,
            commands::get_range_of_previous_field,
            commands::get_std_description,
            commands::get_messages_schema,
            commands::get_segment_schema,
            commands::get_message_segment_names,
            commands::get_message_trigger_event,
            commands::get_message_type,
            commands::get_field_range,
            commands::parse_message_segment,
            commands::render_message_segment,
            commands::generate_control_id,
            commands::get_current_cell_range,
            commands::get_current_hl7_timestamp,
            commands::format_datetime_to_hl7,
            commands::generate_template_message,
            commands::send_message,
            commands::start_listening,
            commands::stop_listening,
            menu::set_save_enabled,
            menu::set_auto_save_checked,
            menu::set_undo_enabled,
            menu::set_redo_enabled,
            menu::update_recent_files_menu,
            menu::set_insert_timestamp_enabled,
            menu::open_help_window,
            commands::compare_messages,
            commands::validate_light,
            commands::validate_full,
            commands::export_to_json,
            commands::export_to_yaml,
            commands::export_to_toml,
            commands::import_from_json,
            commands::import_from_yaml,
            commands::import_from_toml,
            commands::get_segment_index_at_cursor,
            commands::delete_segment,
            commands::move_segment,
            commands::duplicate_segment,
        ])
        .setup(|app| {
            let menu_items =
                menu::build_menu(app).wrap_err_with(|| "Failed to build application menu")?;

            menu::setup_menu_event_handler(app);

            let app_data = AppData {
                schema: SchemaCache::new("messages.toml")
                    .wrap_err_with(|| "Failed to load messages schema from messages.toml")?,
                listen_join: Mutex::new(None),
                save_menu_item: menu_items.save_menu_item,
                auto_save_menu_item: menu_items.auto_save_menu_item,
                undo_menu_item: menu_items.undo_menu_item,
                redo_menu_item: menu_items.redo_menu_item,
                recent_files_submenu: menu_items.recent_files_submenu,
                recent_files: Mutex::new(Vec::new()),
                insert_timestamp_now_menu_item: menu_items.insert_timestamp_now_menu_item,
                insert_timestamp_menu_item: menu_items.insert_timestamp_menu_item,
            };
            app.manage(app_data);

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
