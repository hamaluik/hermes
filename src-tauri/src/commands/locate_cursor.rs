use serde::Serialize;

#[derive(Default, Serialize)]
pub struct CursorLocation {
    segment: Option<String>,
    field: Option<usize>,
    repeat: Option<usize>,
    component: Option<usize>,
    subcomponent: Option<usize>,
}

#[tauri::command]
pub fn locate_cursor(message: &str, cursor: usize) -> Option<CursorLocation> {
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
