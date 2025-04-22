use hl7_parser::parser::ParseError;
use syntax_highlight::html_escape;

mod syntax_highlight;

#[tauri::command]
fn syntax_highlight(message: &str) -> String {
    match hl7_parser::parse_message_with_lenient_newlines(message) {
        Ok(msg) => syntax_highlight::syntax_highlight(&msg),
        Err(ParseError::FailedToParse { position, .. }) => {
            let before = html_escape(&message[..position]).replace('\n', "<br/>");
            let after = html_escape(&message[position..]).replace('\n', "<br/>");
            format!(r#"{before}<span class="err">{after}</span>"#)
        }
        Err(ParseError::IncompleteInput(position)) => {
            let position = position.unwrap_or(0);
            let before = html_escape(&message[..position]).replace('\n', "<br/>");
            let after = html_escape(&message[position..]).replace('\n', "<br/>");
            format!(r#"{before}<span class="err">{after}</span>"#)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![syntax_highlight])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
