//! Hermes - Desktop application for composing, sending, and receiving HL7 messages.
//!
//! This application provides a Tauri-based GUI for HL7 message manipulation, primarily
//! targeting development and testing workflows. It supports message editing,
//! syntax highlighting, field descriptions, database wizards, and MLLP network communication.
//!
//! # Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  Frontend (Svelte + SvelteKit)                              │
//! │  - Message editor UI                                        │
//! │  - Wizard forms (patient, visit, interface)                 │
//! │  - Send/receive modals                                      │
//! └────────────────────┬────────────────────────────────────────┘
//!                      │ Tauri Commands (invoke/events)
//! ┌────────────────────▼────────────────────────────────────────┐
//! │  Backend (Rust + Tauri)                                     │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │ AppData (Managed State)                              │   │
//! │  │  - SchemaCache: Cached HL7 schemas                   │   │
//! │  │  - listen_join: MLLP listener task handle            │   │
//! │  │  - Menu item refs: Save, Undo, Redo (for enable/     │   │
//! │  │    disable sync with frontend)                       │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │ Commands (Tauri-exposed functions)                   │   │
//! │  │  - Message editing (data.rs)                         │   │
//! │  │  - Syntax highlighting (syntax_highlight.rs)         │   │
//! │  │  - Cursor tracking (locate_cursor.rs)                │   │
//! │  │  - Field descriptions (field_description.rs)         │   │
//! │  │  - Schema queries (schema.rs)                        │   │
//! │  │  - Send/receive (send_receive.rs, listen.rs)         │   │
//! │  │  - Menu state (menu.rs)                              │   │
//! │  │  - Wizards (wizards/*.rs)                            │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │ Schema Module                                        │   │
//! │  │  - Loads and caches HL7 schema from messages.toml    │   │
//! │  │  - Provides segment and message metadata             │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │ Spec Module                                          │   │
//! │  │  - HL7 standard specifications                       │   │
//! │  │  - HL7 system-specific field descriptions          │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Tauri Command-Response Pattern
//!
//! The application uses Tauri's command system for frontend-backend communication:
//!
//! 1. **Synchronous Commands**: Frontend calls `invoke("command_name", args)`, awaits result
//!    - Example: `parse_message_segment`, `get_std_description`
//!
//! 2. **Event-Driven Commands**: Commands spawn background tasks and emit events
//!    - Example: `send_message` emits `send-log` and `send-response` events
//!    - Example: `start_listening` emits `received-message` events
//!    - Frontend sets up event listeners before invoking the command
//!
//! # Plugin Initialization Order
//!
//! Plugins are initialized in a specific order to ensure dependencies are available:
//! 1. **clipboard-manager** - First, as it has no dependencies
//! 2. **store** - Persistent settings storage, used by other plugins
//! 3. **fs** - File system access for schema loading
//! 4. **persisted-scope** - Persists file access permissions across restarts
//! 5. **dialog** - File dialogs (depends on fs)
//! 6. **log** - Logging infrastructure, configured with custom formatting
//! 7. **opener** - URL/file opening (last, as it's auxiliary)
//!
//! # Schema Loading
//!
//! The HL7 schema is loaded from `messages.toml` during application setup. This schema
//! defines message types, trigger events, and segment structures. Schema loading happens
//! before the app finishes initialization to ensure all commands have access to schema data.

use color_eyre::eyre::Context;
use schema::cache::SchemaCache;
use tauri::menu::{MenuBuilder, MenuItem, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{Emitter, Manager, Wry};
use tokio::sync::Mutex;

mod commands;
mod schema;
mod spec;

/// Application-wide state managed by Tauri.
///
/// This state is initialized once during app setup and is accessible to all Tauri commands
/// via the `State<AppData>` parameter. The state is thread-safe and can be accessed from
/// multiple async tasks concurrently.
pub struct AppData {
    /// Cached HL7 schema loaded from messages.toml.
    ///
    /// Provides fast access to message and segment definitions without repeatedly
    /// parsing the schema file. Uses a read-write lock internally for thread-safe access.
    schema: SchemaCache,

    /// Handle to the MLLP listener background task.
    ///
    /// When `start_listening` is called, a background task is spawned to accept incoming
    /// HL7 messages. This handle allows the task to be aborted when `stop_listening` is
    /// called or when a new listener is started. Only one listener can be active at a time.
    ///
    /// Wrapped in a Mutex to allow the handle to be taken, replaced, or aborted from
    /// different async tasks.
    listen_join: Mutex<Option<tokio::task::JoinHandle<()>>>,

    /// Reference to the Save menu item for dynamic enable/disable.
    ///
    /// This allows the frontend to sync the menu item's enabled state with the
    /// toolbar save button. The menu item is disabled when there are no unsaved
    /// changes or no file is currently open.
    pub save_menu_item: MenuItem<Wry>,

    /// Reference to the Undo menu item for dynamic enable/disable.
    ///
    /// Enabled when there are changes that can be undone.
    pub undo_menu_item: MenuItem<Wry>,

    /// Reference to the Redo menu item for dynamic enable/disable.
    ///
    /// Enabled when there are changes that can be redone.
    pub redo_menu_item: MenuItem<Wry>,
}

/// Main entry point for the Hermes application.
///
/// This function initializes the Tauri application with all necessary plugins, sets up
/// error handling, configures logging, and registers command handlers. It blocks until
/// the application window is closed by the user.
///
/// # Initialization Flow
///
/// 1. Install `color_eyre` for enhanced error reporting in the terminal
/// 2. Configure logging level (Debug in dev, Info in production)
/// 3. Build Tauri app with plugins in dependency order
/// 4. Register all command handlers
/// 5. Load HL7 schema from messages.toml into AppData
/// 6. Open developer tools in debug builds
/// 7. Run the app event loop
///
/// # Panics
///
/// This function will panic if:
/// * `color_eyre` installation fails (unlikely)
/// * The Tauri app cannot be built (misconfiguration)
/// * The app event loop encounters a fatal error
///
/// # Schema Loading Errors
///
/// If `messages.toml` cannot be loaded or parsed, the app will fail to start with an
/// error message. This is intentional - the app cannot function without schema data.
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
            commands::parse_message_segment,
            commands::render_message_segment,
            commands::send_message,
            commands::start_listening,
            commands::stop_listening,
            commands::set_save_enabled,
            commands::set_undo_enabled,
            commands::set_redo_enabled,
            commands::wizards::wizard_apply_interface,
            commands::wizards::wizard_query_interfaces,
            commands::wizards::wizard_apply_patient,
            commands::wizards::wizard_search_patients,
            commands::wizards::wizard_apply_visit,
            commands::wizards::wizard_search_visits,
        ])
        .setup(|app| {
            // Build the Save menu item separately so we can store a reference for dynamic enable/disable
            let save_menu_item = MenuItemBuilder::new("&Save")
                .id("file-save")
                .accelerator("CmdOrCtrl+S")
                .enabled(false) // Start disabled until there are unsaved changes
                .build(app)?;

            // Build the File menu with standard file operations
            // Note: We borrow save_menu_item here so we can move it into AppData afterwards
            let file_menu = SubmenuBuilder::new(app, "&File")
                .item(
                    &MenuItemBuilder::new("&New")
                        .id("file-new")
                        .accelerator("CmdOrCtrl+N")
                        .build(app)?,
                )
                .item(
                    &MenuItemBuilder::new("&Open...")
                        .id("file-open")
                        .accelerator("CmdOrCtrl+O")
                        .build(app)?,
                )
                .separator()
                .item(&save_menu_item)
                .item(
                    &MenuItemBuilder::new("Save &As...")
                        .id("file-save-as")
                        .accelerator("CmdOrCtrl+Shift+S")
                        .build(app)?,
                )
                .build()?;

            // Build Edit menu with two categories of items:
            // 1. Custom items (Undo/Redo) - require frontend event handling and dynamic enable/disable
            //    based on history state. We keep references to these in AppData.
            // 2. Predefined items (Cut/Copy/Paste/Select All) - handled natively by the webview,
            //    no frontend code needed. These work automatically with any focused text input.
            let undo_menu_item = MenuItemBuilder::new("&Undo")
                .id("edit-undo")
                .accelerator("CmdOrCtrl+Z")
                .enabled(false) // Start disabled until there's history
                .build(app)?;

            let redo_menu_item = MenuItemBuilder::new("&Redo")
                .id("edit-redo")
                .accelerator("CmdOrCtrl+Shift+Z")
                .enabled(false) // Start disabled until there's redo history
                .build(app)?;

            let find_menu_item = MenuItemBuilder::new("&Find...")
                .id("edit-find")
                .accelerator("CmdOrCtrl+F")
                .build(app)?;

            let find_replace_menu_item = MenuItemBuilder::new("Find and &Replace...")
                .id("edit-find-replace")
                .accelerator("CmdOrCtrl+H")
                .build(app)?;

            let edit_menu = SubmenuBuilder::new(app, "&Edit")
                .item(&undo_menu_item)
                .item(&redo_menu_item)
                .separator()
                .item(&PredefinedMenuItem::cut(app, None)?)
                .item(&PredefinedMenuItem::copy(app, None)?)
                .item(&PredefinedMenuItem::paste(app, None)?)
                .separator()
                .item(&find_menu_item)
                .item(&find_replace_menu_item)
                .separator()
                .item(&PredefinedMenuItem::select_all(app, None)?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .item(&edit_menu)
                .build()?;

            app.set_menu(menu)?;

            // Create AppData after menu setup, moving menu items (not cloning)
            // so we have references to the exact menu items that are in the menu
            let app_data = AppData {
                schema: SchemaCache::new("messages.toml")
                    .wrap_err_with(|| "Failed to load messages schema from messages.toml")?,
                listen_join: Mutex::new(None),
                save_menu_item,
                undo_menu_item,
                redo_menu_item,
            };
            app.manage(app_data);

            // Handle menu events by emitting corresponding frontend events
            app.on_menu_event(move |app_handle, event| {
                let event_name = match event.id().as_ref() {
                    "file-new" => Some("menu-file-new"),
                    "file-open" => Some("menu-file-open"),
                    "file-save" => Some("menu-file-save"),
                    "file-save-as" => Some("menu-file-save-as"),
                    "edit-undo" => Some("menu-edit-undo"),
                    "edit-redo" => Some("menu-edit-redo"),
                    "edit-find" => Some("menu-edit-find"),
                    "edit-find-replace" => Some("menu-edit-find-replace"),
                    _ => None,
                };

                if let Some(name) = event_name {
                    let _ = app_handle.emit(name, ());
                }
            });

            // Open devtools automatically in dev mode
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
