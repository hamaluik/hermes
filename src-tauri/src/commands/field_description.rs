use crate::spec::std_spec::{describe_component, describe_field, segment_description};

#[tauri::command]
pub fn get_std_description(
    segment: &str,
    field: Option<usize>,
    component: Option<usize>,
) -> String {
    let version = "2.5.1";
    match (field, component) {
        (Some(field), Some(component)) => describe_component(version, segment, field, component),
        (Some(field), None) => describe_field(version, segment, field),
        _ => segment_description(version, segment),
    }
}
