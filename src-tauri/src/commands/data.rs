use std::collections::HashMap;

use color_eyre::eyre::Context;
use hl7_parser::builder::{FieldBuilder, MessageBuilder, SegmentBuilder};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppData;

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SegmentData {
    fields: HashMap<String, Option<String>>,
}

#[tauri::command]
pub fn get_message_segment_names(message: &str) -> Vec<String> {
    let Ok(message) = hl7_parser::parse_message_with_lenient_newlines(message) else {
        return vec!["MSH".to_string()];
    };
    message
        .segments()
        .map(|segment| segment.name.to_string())
        .collect()
}

#[tauri::command]
pub fn get_message_trigger_event(message: &str) -> Option<String> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    message
        .query("MSH.9.2")
        .map(|value| message.separators.decode(value.raw_value()).to_string())
}

#[tauri::command]
pub fn get_message_type(message: &str) -> Option<String> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;
    message
        .query("MSH.9.1")
        .map(|value| message.separators.decode(value.raw_value()).to_string())
}

#[tauri::command]
pub fn parse_message_segment(
    message: &str,
    segment: &str,
    segment_repeat: usize,
    state: State<'_, AppData>,
) -> Result<SegmentData, String> {
    let schema = state
        .schema
        .get_segment(segment)
        .wrap_err_with(|| format!("Failed to load segment {segment} schema"))
        .map_err(|e| format!("{e:#}"))?;

    let message = hl7_parser::parse_message_with_lenient_newlines(message)
        .wrap_err_with(|| "Failed to parse message")
        .map_err(|e| format!("{e:#}"))?;

    Ok(SegmentData {
        fields: schema
            .into_iter()
            .map(|field| {
                let field_id = format!(
                    "{segment}.{field}{component}",
                    // segment_repeat = segment_repeat + 1,
                    field = field.field,
                    component = if let Some(comp) = field.component {
                        format!(".{comp}")
                    } else {
                        String::new()
                    }
                );
                let field_value = message
                    .query(&field_id)
                    .map(|value| message.separators.decode(value.raw_value()).to_string());

                (field_id, field_value)
            })
            .collect(),
    })
}

#[tauri::command]
pub fn render_message_segment(
    message: &str,
    segment: &str,
    segment_repeat: usize,
    data: SegmentData,
) -> String {
    let Ok(message) = hl7_parser::parse_message_with_lenient_newlines(message) else {
        return message.to_string();
    };

    let mut message: MessageBuilder = (&message).into();
    // ensure the message has at least `segment_repeat + 1` segments of this type
    // while message.segment_n(segment, segment_repeat + 1).is_none() {
    //     message.push_segment(SegmentBuilder::new(segment));
    // }
    // let seg = message
    //     .segment_n_mut(segment, segment_repeat + 1)
    //     .expect("message has segment");
    if !message.segment_named(segment).is_some() {
        message.push_segment(SegmentBuilder::new(segment));
    }
    let seg = message
        .segment_named_mut(segment)
        .expect("message has segment");

    for (field_id, field_value) in data.fields.into_iter() {
        let Some((field_id, component_id)) = parse_field_id(&field_id, segment) else {
            continue;
        };

        if let Some(component_id) = component_id {
            if !seg.has_field(field_id) {
                seg.set_field(field_id, FieldBuilder::default());
            }
            let field = seg.field_mut(field_id).expect("field exists");
            field.set_component(component_id, field_value.unwrap_or_default());
        } else {
            seg.set_field_value(field_id, field_value.unwrap_or_default());
        }
    }

    // TODO: rearrange the segments if needed

    message.render_with_newlines().to_string()
}

fn parse_field_id(field_id: &str, segment: &str) -> Option<(usize, Option<usize>)> {
    // split the field_id into segment, field, and component
    let parts: Vec<&str> = field_id.split('.').collect();
    if parts.len() < 2 {
        log::warn!("Invalid field_id: {field_id}");
        return None;
    }

    let Ok(segment_name) = parts[0].parse::<String>(); // split always has at least 1 part
    if segment_name != segment {
        log::warn!("Segment name does not match: {segment_name} != {segment}");
        return None;
    }

    let Ok(field) = parts[1].parse::<usize>() else {
        log::warn!("Invalid field number in field_id: {field_id}");
        return None;
    };
    if field < 1 {
        log::warn!("Field number must be greater than 0: {field_id}");
        return None;
    }

    let component = if parts.len() > 2 {
        let Ok(component) = parts[2].parse::<usize>() else {
            log::warn!("Invalid component number in field_id: {field_id}");
            return None;
        };
        if component < 1 {
            log::warn!("Component number must be greater than 0: {field_id}");
            return None;
        }
        Some(component)
    } else {
        None
    };

    Some((field, component))
}
