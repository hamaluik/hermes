use hl7_parser::parser::ParseError;
use serde::Serialize;
use spec::get_version_with_fallback;
use syntax_highlight::html_escape;

mod spec;
mod ;
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

#[derive(Default, Serialize)]
struct CursorLocation {
    segment: Option<String>,
    field: Option<usize>,
    repeat: Option<usize>,
    component: Option<usize>,
    subcomponent: Option<usize>,
    description_standard: Option<String>,
    description_wizard: Option<&'static str>,
}

#[tauri::command]
fn locate_cursor(message: &str, cursor: usize) -> Option<CursorLocation> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    let version = get_version_with_fallback(&message);

    message.locate_cursor(cursor).map(|loc| {
        let mut location = CursorLocation::default();

        if let Some((segment, _, _)) = loc.segment {
            location.segment = Some(segment.to_string());
            if let Some((field_i, field)) = loc.field {
                location.field = Some(field_i);
                if let Some((repeat_i, repeat)) = loc.repeat {
                    if field.has_repeats() {
                        location.repeat = Some(repeat_i);
                    }
                    if let Some((component_i, component)) = loc.component {
                        if repeat.has_components() {
                            location.component = Some(component_i);
                        }
                        if let Some((subcomponent_i, _)) = loc.sub_component {
                            if component.has_subcomponents() {
                                location.subcomponent = Some(subcomponent_i);
                                location.component = Some(component_i);
                            }
                        }
                    }
                }
            }
        }

        match (
            &location.segment,
            location.field.as_ref(),
            location.component.as_ref(),
        ) {
            (Some(segment), Some(field), Some(component)) => {
                location.description_standard = Some(spec::describe_component(
                    version, segment, *field, *component,
                ));
                location.description_wizard = ::describe(segment, *field, Some(*component));
            }
            (Some(segment), Some(field), None) => {
                location.description_standard =
                    Some(spec::describe_field(version, segment, *field));
                location.description_wizard = ::describe(segment, *field, None);
            }
            (Some(segment), None, None) => {
                location.description_standard = Some(spec::segment_description(version, segment));
            }
            _ => {}
        }

        location
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![syntax_highlight, locate_cursor])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
