use color_eyre::eyre::Context;
use schema::cache::SchemaCache;
use tauri::Manager;

mod commands;
mod schema;
mod spec;

struct AppData {
    schema: SchemaCache,
}

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

    // TODO: file menu

    tauri::Builder::default()
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
            commands::parse_message_segment,
            commands::render_message_segment,
        ])
        .setup(|app| {
            let app_data = AppData {
                schema: SchemaCache::new("messages.toml")
                    .wrap_err_with(|| "Failed to load messages schema from messages.toml")?,
            };
            app.manage(app_data);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
