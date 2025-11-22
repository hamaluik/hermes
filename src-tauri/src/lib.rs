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
//! │  - Communication drawer (send/listen for messages)          │
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
//! │  │  - auto_save_menu_item: Checkable Auto-Save toggle   │   │
//! │  │  - recent_files_submenu: Dynamic "Open Recent" menu  │   │
//! │  │  - recent_files: File paths for menu event lookup    │   │
//! │  └──────────────────────────────────────────────────────┘   │
//! │  ┌──────────────────────────────────────────────────────┐   │
//! │  │ Commands (Tauri-exposed functions)                   │   │
//! │  │  - Message editing (data.rs)                         │   │
//! │  │  - Syntax highlighting (syntax_highlight.rs)         │   │
//! │  │  - Cursor tracking (locate_cursor.rs)                │   │
//! │  │  - Field descriptions (field_description.rs)         │   │
//! │  │  - Schema queries (schema.rs)                        │   │
//! │  │  - Send/receive (send_receive.rs, listen.rs)         │   │
//! │  │  - Menu state & help window (menu.rs)                │   │
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
use tauri::menu::{
    AboutMetadata, CheckMenuItem, CheckMenuItemBuilder, MenuBuilder, MenuItem, MenuItemBuilder,
    PredefinedMenuItem, Submenu, SubmenuBuilder,
};
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

    /// Reference to the Auto-Save checkable menu item for sync with settings.
    ///
    /// The frontend toggles this when the user changes the auto-save setting
    /// in the settings modal or clicks this menu item.
    pub auto_save_menu_item: CheckMenuItem<Wry>,

    /// Reference to the Undo menu item for dynamic enable/disable.
    ///
    /// Enabled when there are changes that can be undone.
    pub undo_menu_item: MenuItem<Wry>,

    /// Reference to the Redo menu item for dynamic enable/disable.
    ///
    /// Enabled when there are changes that can be redone.
    pub redo_menu_item: MenuItem<Wry>,

    /// Reference to the "Open Recent" submenu for dynamic population.
    ///
    /// The frontend calls `update_recent_files_menu` to rebuild this menu
    /// with the current list of recent files.
    pub recent_files_submenu: Submenu<Wry>,

    /// Current list of recent file paths.
    ///
    /// Updated via `update_recent_files_menu` and used to look up the file path
    /// when a recent file menu item is clicked. Stored here because menu event
    /// handlers need access to the paths to emit the correct event payload.
    pub recent_files: Mutex<Vec<String>>,

    /// Reference to the "Insert Current Timestamp" menu item for dynamic enable/disable.
    ///
    /// Disabled when the cursor is not within a valid field/component (e.g., on segment name).
    pub insert_timestamp_now_menu_item: MenuItem<Wry>,

    /// Reference to the "Insert Timestamp..." menu item for dynamic enable/disable.
    ///
    /// Disabled when the cursor is not within a valid field/component (e.g., on segment name).
    pub insert_timestamp_menu_item: MenuItem<Wry>,
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
            commands::set_save_enabled,
            commands::set_auto_save_checked,
            commands::set_undo_enabled,
            commands::set_redo_enabled,
            commands::update_recent_files_menu,
            commands::set_insert_timestamp_enabled,
            commands::open_help_window,
            commands::wizards::wizard_apply_interface,
            commands::wizards::wizard_query_interfaces,
            commands::wizards::wizard_apply_patient,
            commands::wizards::wizard_search_patients,
            commands::wizards::wizard_apply_visit,
            commands::wizards::wizard_search_visits,
            commands::compare_messages,
        ])
        .setup(|app| {
            // Build the Save menu item separately so we can store a reference for dynamic enable/disable
            let save_menu_item = MenuItemBuilder::new("&Save")
                .id("file-save")
                .accelerator("CmdOrCtrl+S")
                .enabled(false) // Start disabled until there are unsaved changes
                .build(app)?;

            // Build the Auto-Save checkable menu item (initial state synced from settings by frontend)
            let auto_save_menu_item = CheckMenuItemBuilder::new("Auto-Save")
                .id("file-auto-save")
                .checked(false) // Start unchecked, frontend will sync from settings
                .build(app)?;

            // Build the "Open Recent" submenu (starts empty, populated by frontend)
            let recent_files_submenu = SubmenuBuilder::new(app, "Open &Recent")
                .id("file-open-recent")
                .enabled(false) // Disabled until populated with files
                .build()?;

            // Build the "New from Template" submenu with pre-populated message types
            // Templates are generated dynamically from messages.toml schema
            let template_submenu = SubmenuBuilder::new(app, "New from &Template")
                .id("file-new-from-template")
                // ADT messages
                .item(&MenuItemBuilder::new("ADT^A01 (Admit)").id("template-adt_a01").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A02 (Transfer)").id("template-adt_a02").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A03 (Discharge)").id("template-adt_a03").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A04 (Register)").id("template-adt_a04").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A05 (Pre-admit)").id("template-adt_a05").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A08 (Update)").id("template-adt_a08").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A11 (Cancel Admit)").id("template-adt_a11").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A12 (Cancel Transfer)").id("template-adt_a12").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A13 (Cancel Discharge)").id("template-adt_a13").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A23 (Delete)").id("template-adt_a23").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A34 (Merge Patient - ID Only)").id("template-adt_a34").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A40 (Merge Patient)").id("template-adt_a40").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A49 (Change Patient ID)").id("template-adt_a49").build(app)?)
                .item(&MenuItemBuilder::new("ADT^A50 (Change Visit ID)").id("template-adt_a50").build(app)?)
                .separator()
                // Order messages
                .item(&MenuItemBuilder::new("ORM^O01 (Order)").id("template-orm_o01").build(app)?)
                .item(&MenuItemBuilder::new("ORU^R01 (Results)").id("template-oru_r01").build(app)?)
                .item(&MenuItemBuilder::new("ORR^O02 (Order Response)").id("template-orr_o02").build(app)?)
                .item(&MenuItemBuilder::new("DFT^P03 (Financial)").id("template-dft_p03").build(app)?)
                .build()?;

            // Build the File menu with standard file operations
            // Note: We borrow save_menu_item here so we can move it into AppData afterwards
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
                .separator()
                .item(&auto_save_menu_item)
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

            let jump_to_field_menu_item = MenuItemBuilder::new("&Jump to Field...")
                .id("edit-jump-to-field")
                .accelerator("CmdOrCtrl+J")
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
                .item(&jump_to_field_menu_item)
                .separator()
                .item(&PredefinedMenuItem::select_all(app, None)?)
                .build()?;

            // Build the Help menu with documentation and about items
            let help_menu_item = MenuItemBuilder::new("&Help")
                .id("help")
                .accelerator("F1")
                .build(app)?;

            // About dialog metadata - pulls values from Cargo.toml via env! macros
            // Platform support:
            // - macOS: name, version, short_version, copyright, credits, icon
            // - Windows/Linux: name, version, short_version, authors, comments, copyright, license, website, website_label
            // Note: [package.metadata.about] fields (copyright, credits, website_label) require
            // manual sync - only [package] fields have env! macros available.
            let about_metadata = AboutMetadata {
                name: Some(env!("CARGO_PKG_NAME").into()),
                version: Some(env!("CARGO_PKG_VERSION").into()),
                short_version: None,
                authors: Some(vec![env!("CARGO_PKG_AUTHORS").into()]), // Windows/Linux only
                comments: Some(env!("CARGO_PKG_DESCRIPTION").into()),  // Windows/Linux only
                copyright: Some("© 2025 Kenton Hamaluik".into()), // Sync with Cargo.toml [package.metadata.about]
                license: Some(env!("CARGO_PKG_LICENSE").into()),       // Windows/Linux only
                website: Some(env!("CARGO_PKG_HOMEPAGE").into()),      // Windows/Linux only
                website_label: Some("Visit Website".into()),           // Sync with Cargo.toml [package.metadata.about]
                credits: Some("Kenton Hamaluik".into()),               // Sync with Cargo.toml [package.metadata.about]
                ..Default::default()
            };

            let help_menu = SubmenuBuilder::new(app, "&Help")
                .item(&help_menu_item)
                .separator()
                .item(&PredefinedMenuItem::about(app, Some("About Hermes"), Some(about_metadata))?)
                .build()?;

            // Build the View menu for zoom controls.
            // Uses standard keyboard shortcuts (Cmd+/Ctrl+ +/-/0) that users expect from
            // browsers and desktop applications. Zoom levels are persisted to settings.
            let view_menu = SubmenuBuilder::new(app, "&View")
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
                .build()?;

            // Build the Tools menu for HL7 communication operations.
            // Provides keyboard shortcuts (Cmd+T, Cmd+L) to quickly access send/listen
            // functionality without using the mouse. This is particularly useful during
            // rapid edit-send-review cycles when testing HL7 messages.
            // Both items open the communication drawer with the appropriate tab selected.
            //
            // Timestamp insertion items start disabled and are enabled by the frontend when
            // the cursor is within a valid field/component that can be replaced.
            let insert_timestamp_now_menu_item = MenuItemBuilder::new("Insert &Current Timestamp")
                .id("tools-insert-timestamp-now")
                .accelerator("CmdOrCtrl+Shift+T")
                .enabled(false)
                .build(app)?;

            let insert_timestamp_menu_item = MenuItemBuilder::new("Insert &Timestamp...")
                .id("tools-insert-timestamp")
                .enabled(false)
                .build(app)?;

            let tools_menu = SubmenuBuilder::new(app, "&Tools")
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
                .item(&insert_timestamp_now_menu_item)
                .item(&insert_timestamp_menu_item)
                .build()?;

            // Build the Window menu with standard window operations.
            // This provides Cmd+W (close) and Cmd+M (minimize) shortcuts that macOS users expect.
            // Using PredefinedMenuItem types ensures proper platform integration without
            // requiring frontend event handling—these actions work natively.
            // Note: close_window is unsupported on Linux but works on macOS and Windows.
            let window_menu = SubmenuBuilder::new(app, "&Window")
                .item(&PredefinedMenuItem::minimize(app, None)?)
                .item(&PredefinedMenuItem::maximize(app, None)?)
                .separator()
                .item(&PredefinedMenuItem::close_window(app, None)?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .item(&edit_menu)
                .item(&view_menu)
                .item(&tools_menu)
                .item(&window_menu)
                .item(&help_menu)
                .build()?;

            app.set_menu(menu)?;

            // Create AppData after menu setup, moving menu items (not cloning)
            // so we have references to the exact menu items that are in the menu
            let app_data = AppData {
                schema: SchemaCache::new("messages.toml")
                    .wrap_err_with(|| "Failed to load messages schema from messages.toml")?,
                listen_join: Mutex::new(None),
                save_menu_item,
                auto_save_menu_item,
                undo_menu_item,
                redo_menu_item,
                recent_files_submenu,
                recent_files: Mutex::new(Vec::new()),
                insert_timestamp_now_menu_item,
                insert_timestamp_menu_item,
            };
            app.manage(app_data);

            // Handle menu events by emitting corresponding frontend events
            app.on_menu_event(move |app_handle, event| {
                let event_id = event.id().as_ref();

                // Handle standard menu events (emit empty payload)
                let event_name = match event_id {
                    "file-new" => Some("menu-file-new"),
                    "file-open" => Some("menu-file-open"),
                    "file-save" => Some("menu-file-save"),
                    "file-save-as" => Some("menu-file-save-as"),
                    "file-auto-save" => Some("menu-file-auto-save"),
                    "edit-undo" => Some("menu-edit-undo"),
                    "edit-redo" => Some("menu-edit-redo"),
                    "edit-find" => Some("menu-edit-find"),
                    "edit-find-replace" => Some("menu-edit-find-replace"),
                    "edit-jump-to-field" => Some("menu-edit-jump-to-field"),
                    "view-zoom-in" => Some("menu-view-zoom-in"),
                    "view-zoom-out" => Some("menu-view-zoom-out"),
                    "view-reset-zoom" => Some("menu-view-reset-zoom"),
                    "tools-send" => Some("menu-tools-send"),
                    "tools-listen" => Some("menu-tools-listen"),
                    "tools-compare" => Some("menu-tools-compare"),
                    "tools-generate-control-id" => Some("menu-tools-generate-control-id"),
                    "tools-insert-timestamp-now" => Some("menu-tools-insert-timestamp-now"),
                    "tools-insert-timestamp" => Some("menu-tools-insert-timestamp"),
                    "recent-clear" => Some("menu-clear-recent"),
                    "help" => Some("menu-help"),
                    _ => None,
                };

                if let Some(name) = event_name {
                    let _ = app_handle.emit(name, ());
                    return;
                }

                // Handle recent file menu items (emit file path as payload)
                if let Some(index_str) = event_id.strip_prefix("recent-file-") {
                    if let Ok(index) = index_str.parse::<usize>() {
                        // Look up the file path from stored recent files
                        if let Some(state) = app_handle.try_state::<AppData>() {
                            // Use blocking lock since we're in a sync callback
                            let recent_files = state.recent_files.blocking_lock();
                            if let Some(file_path) = recent_files.get(index) {
                                let _ = app_handle.emit("menu-open-recent", file_path.clone());
                            }
                        }
                    }
                    return;
                }

                // Handle template menu items (emit template name as payload)
                if let Some(template_name) = event_id.strip_prefix("template-") {
                    let _ = app_handle.emit("menu-new-from-template", template_name);
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
