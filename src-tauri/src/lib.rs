mod commands;
mod spec;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO: configure logging, color-eyre?

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::syntax_highlight,
            commands::locate_cursor,
            commands::get_std_description,
            commands::get_wizard_description,
            commands::parse_header,
            commands::render_header,
            commands::parse_patient,
            commands::render_patient,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
