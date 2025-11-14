use color_eyre::eyre::Context;
use hl7_parser::{
    builder::{FieldBuilder, MessageBuilder, SegmentBuilder},
    message::Separators,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,
    pub provider_id: String,
    pub sending_app: String,
    pub sending_fac: String,
    pub receiving_app: String,
    pub receiving_fac: String,
    pub version: String,
    pub processing_cd: String,
    pub default_timezone: String,
    pub receive_port: u16,
}

#[tauri::command]
pub fn wizard_apply_interface(
    message: &str,
    interface: Interface,
    messagetype: &str, // tauri breaks when I use snake case -_-
    triggerevent: &str,
    overridesegment: bool,
) -> Result<String, String> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message)
        .wrap_err_with(|| "Failed to parse message")
        .map_err(|e| format!("{e:#}"))?;

    let mut message: MessageBuilder = (&message).into();
    let msh = message
        .segment_named_mut("MSH")
        .expect("messages have MSH segments");

    if overridesegment {
        let sep = Separators::default();
        msh.clear();
        msh.set_field_value(1, sep.field);
        msh.set_field_value(
            2,
            format!(
                "{comp}{rep}{esc}{sub}",
                comp = sep.component,
                rep = sep.repetition,
                esc = sep.escape,
                sub = sep.subcomponent
            ),
        );
    }

    msh.set_field_value(3, interface.sending_app);
    msh.set_field_value(4, interface.sending_fac);
    msh.set_field_value(5, interface.receiving_app);
    msh.set_field_value(6, interface.receiving_fac);
    msh.set_field(
        9,
        FieldBuilder::default()
            .with_component_value(1, messagetype)
            .with_component_value(2, triggerevent),
    );
    msh.set_field_value(11, interface.processing_cd);
    msh.set_field_value(12, interface.version);

    if overridesegment {
        msh.set_field_value(7, "{now}");
        msh.set_field_value(10, "{random}");
        msh.set_field_value(15, "AL");
        msh.set_field_value(16, "NE");
    }

    if message.segment_named("EVN").is_none() && message.segments().len() == 1 {
        message.push_segment(
            SegmentBuilder::new("EVN")
                .with_field_value(1, triggerevent)
                .with_field_value(2, "{auto}"),
        );
    }

    // convenience additions for blank messages
    if messagetype == "ADT" || messagetype == "ORM" {
        if message.segment_named("PID").is_none() {
            message.push_segment(SegmentBuilder::new("PID").with_field_value(1, ""));
        }
        if message.segment_named("PV1").is_none() {
            message.push_segment(SegmentBuilder::new("PV1").with_field_value(1, ""));
        }

        if messagetype == "ORM" {
            message.push_segment(SegmentBuilder::new("ORC").with_field_value(1, ""));
        }
    }

    Ok(message.render_with_newlines().to_string())
}

#[tauri::command]
pub async fn wizard_query_interfaces(
    _db: super::WizardDatabase,
    messagetype: &str,
    providerid: Option<&str>,
) -> Result<Vec<Interface>, String> {
    log::info!(
        "wizard_query_interfaces called with messagetype={}, providerid={:?}",
        messagetype,
        providerid
    );

    Ok(vec![
        Interface {
            name: "Test Interface".to_string(),
            provider_id: providerid.unwrap_or("provider_123").to_string(),
            sending_app: "AppA".to_string(),
            sending_fac: "FacilityA".to_string(),
            receiving_app: "AppB".to_string(),
            receiving_fac: "FacilityB".to_string(),
            version: "2.5".to_string(),
            processing_cd: "P".to_string(),
            default_timezone: "UTC".to_string(),
            receive_port: 12345,
        },
        Interface {
            name: "Demo Interface".to_string(),
            provider_id: providerid.unwrap_or("provider_456").to_string(),
            sending_app: "AppC".to_string(),
            sending_fac: "FacilityC".to_string(),
            receiving_app: "AppD".to_string(),
            receiving_fac: "FacilityD".to_string(),
            version: "2.3".to_string(),
            processing_cd: "T".to_string(),
            default_timezone: "America/New_York".to_string(),
            receive_port: 23456,
        },
    ])
}
