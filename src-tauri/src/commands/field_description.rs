use crate::spec::{
    std_spec::{describe_component, describe_field, segment_description},
    ::describe,
};

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

#[tauri::command]
pub fn get_wizard_description(
    segment: &str,
    field: Option<usize>,
    component: Option<usize>,
) -> Option<String> {
    match (field, component) {
        (Some(field), Some(component)) => {
            let field_desc = describe(segment, field, None);
            let component_desc = describe(segment, field, Some(component));
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
        (Some(field), None) => describe(segment, field, None).map(|desc| desc.to_string()),
        _ => None,
    }
}
