use crate::{
    schema::{message::MessagesSchema, segment::Field},
    AppData,
};
use color_eyre::eyre::Context;
use tauri::State;

#[tauri::command]
pub fn get_segment_schema(segment: &str, state: State<'_, AppData>) -> Result<Vec<Field>, String> {
    state
        .schema
        .get_segment(segment)
        .wrap_err_with(|| format!("Failed to load segment {segment} data"))
        .map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub fn get_messages_schema(state: State<'_, AppData>) -> Result<MessagesSchema, String> {
    state
        .schema
        .get_messages()
        .wrap_err_with(|| "Failed to load messages schema")
        .map_err(|e| format!("{e:#}"))
}
