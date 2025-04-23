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
}

#[tauri::command]
fn locate_cursor(message: &str, cursor: usize) -> Option<CursorLocation> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;

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

        location
    })
}

#[tauri::command]
fn get_std_description(segment: &str, field: Option<usize>, component: Option<usize>) -> String {
    let version = "2.5.1";
    match (field, component) {
        (Some(field), Some(component)) => {
            spec::describe_component(version, segment, field, component)
        }
        (Some(field), None) => spec::describe_field(version, segment, field),
        _ => spec::segment_description(version, segment),
    }
}

#[tauri::command]
fn get_wizard_description(
    segment: &str,
    field: Option<usize>,
    component: Option<usize>,
) -> Option<String> {
    match (field, component) {
        (Some(field), Some(component)) => {
            let field_desc = ::describe(segment, field, None);
            let component_desc = ::describe(segment, field, Some(component));
            match (field_desc, component_desc) {
                (Some(field_desc), Some(component_desc)) => {
                    if field_desc == component_desc {
                        Some(field_desc.to_string())
                    } else {
                        Some(format!("{field_desc}\n\n{component_desc}"))
                    }
                }
                (Some(field_desc), None) => Some(field_desc.to_string()),
                (None, Some(component_desc)) => Some(component_desc.to_string()),
                _ => None,
            }
        }
        (Some(field), None) => {
            ::describe(segment, field, None).map(|desc| desc.to_string())
        }
        _ => None,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            syntax_highlight,
            locate_cursor,
            get_std_description,
            get_wizard_description
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
