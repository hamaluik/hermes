use serde::Serialize;
use std::ops::Range;

#[derive(Default, Serialize)]
pub struct CursorLocation {
    segment: Option<String>,
    segment_number: Option<usize>,
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

        if let Some((segment, segment_n, _)) = loc.segment {
            location.segment = Some(segment.to_string());
            location.segment_number = Some(segment_n);
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

#[derive(Serialize)]
pub struct CursorRange {
    start: usize,
    end: usize,
}

fn flatten_message(message: &hl7_parser::Message) -> Vec<Range<usize>> {
    let mut cells = Vec::new();
    for segment in message.segments() {
        // include segment name as a navigable cell
        cells.push(Range {
            start: segment.range.start,
            end: segment.range.start + segment.name.len(),
        });

        if segment.fields.is_empty() {
            continue;
        }

        for field in segment.fields() {
            if field.repeats.is_empty() {
                cells.push(field.range.clone());
                continue;
            }

            for repeat in field.repeats.iter() {
                if repeat.components.is_empty() {
                    cells.push(repeat.range.clone());
                    continue;
                }

                for component in repeat.components.iter() {
                    if component.subcomponents.is_empty() {
                        cells.push(component.range.clone());
                        continue;
                    }

                    for subcomponent in component.subcomponents.iter() {
                        cells.push(subcomponent.range.clone());
                    }
                }
            }
        }
    }
    cells
}

#[tauri::command]
pub fn get_range_of_next_field(message: &str, cursor: usize) -> Option<CursorRange> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    let cells = flatten_message(&message);

    let mut cells_iter = cells.iter();
    while let Some(cell) = cells_iter.next() {
        if cursor >= cell.start && cursor <= cell.end {
            return cells_iter.next().map(|next_cell| CursorRange {
                start: next_cell.start,
                end: next_cell.end,
            });
        }
    }

    None
}

#[tauri::command]
pub fn get_range_of_previous_field(message: &str, cursor: usize) -> Option<CursorRange> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    let cells = flatten_message(&message);

    let mut cells_iter = cells.iter().rev();
    while let Some(cell) = cells_iter.next() {
        if cursor >= cell.start && cursor <= cell.end {
            return cells_iter.next().map(|prev_cell| CursorRange {
                start: prev_cell.start,
                end: prev_cell.end,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_range_of_next_field_in_component_next_component() {
        let message = r#"MSH|^~\&|a^b"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 12);
    }

    #[test]
    fn can_get_range_of_next_field_in_component_next_repeat_component() {
        let message = r#"MSH|^~\&|a~b^c"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 12);
    }

    #[test]
    fn can_get_range_of_next_field_in_component_next_repeat() {
        let message = r#"MSH|^~\&|a~bc"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 13);
    }

    #[test]
    fn can_get_range_of_next_field_in_field_next_component() {
        let message = r#"MSH|^~\&|a|b^c"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 12);
    }

    #[test]
    fn can_get_range_of_next_field_in_field_next_field() {
        let message = r#"MSH|^~\&|a|bc"#;
        let cursor = 9;
        let range = get_range_of_next_field(message, cursor).expect("range exists");
        assert_eq!(range.start, 11);
        assert_eq!(range.end, 13);
    }
}
