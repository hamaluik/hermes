use crate::{schema::Field, AppData};
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
