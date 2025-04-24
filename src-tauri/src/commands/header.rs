use hl7_parser::builder::{FieldBuilder, MessageBuilder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Header {
    pub sending_application: Option<String>,
    pub sending_facility: Option<String>,
    pub receiving_application: Option<String>,
    pub receiving_facility: Option<String>,
    pub date_time_of_message: Option<String>,
    pub message_type: Option<String>,
    pub trigger_event: Option<String>,
    pub control_id: Option<String>,
    pub processing_id: Option<String>,
    pub version_id: Option<String>,
    pub accept_acknowledgment_type: Option<String>,
    pub application_acknowledgment_type: Option<String>,
    pub character_set: Option<String>,
}

#[tauri::command]
pub fn parse_header(message: &str) -> Option<Header> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;

    let sending_application = message.query("MSH.3").map(|sending_application| {
        message
            .separators
            .decode(sending_application.raw_value())
            .to_string()
    });
    let sending_facility = message.query("MSH.4").map(|sending_facility| {
        message
            .separators
            .decode(sending_facility.raw_value())
            .to_string()
    });
    let receiving_application = message.query("MSH.5").map(|receiving_application| {
        message
            .separators
            .decode(receiving_application.raw_value())
            .to_string()
    });
    let receiving_facility = message.query("MSH.6").map(|receiving_facility| {
        message
            .separators
            .decode(receiving_facility.raw_value())
            .to_string()
    });
    let date_time_of_message = message.query("MSH.7").map(|date_time_of_message| {
        message
            .separators
            .decode(date_time_of_message.raw_value())
            .to_string()
    });
    let message_type = message.query("MSH.9.1").map(|message_type| {
        message
            .separators
            .decode(message_type.raw_value())
            .to_string()
    });
    let trigger_event = message.query("MSH.9.2").map(|trigger_event| {
        message
            .separators
            .decode(trigger_event.raw_value())
            .to_string()
    });
    let control_id = message.query("MSH.10").map(|control_id| {
        message
            .separators
            .decode(control_id.raw_value())
            .to_string()
    });
    let processing_id = message.query("MSH.11.1").map(|processing_id| {
        message
            .separators
            .decode(processing_id.raw_value())
            .to_string()
    });
    let version_id = message.query("MSH.12").map(|version_id| {
        message
            .separators
            .decode(version_id.raw_value())
            .to_string()
    });
    let accept_acknowledgment_type = message.query("MSH.15").map(|accept_acknowledgment_type| {
        message
            .separators
            .decode(accept_acknowledgment_type.raw_value())
            .to_string()
    });
    let application_acknowledgment_type =
        message
            .query("MSH.16")
            .map(|application_acknowledgment_type| {
                message
                    .separators
                    .decode(application_acknowledgment_type.raw_value())
                    .to_string()
            });
    let character_set = message.query("MSH.18").map(|character_set| {
        message
            .separators
            .decode(character_set.raw_value())
            .to_string()
    });

    Some(Header {
        sending_application,
        sending_facility,
        receiving_application,
        receiving_facility,
        date_time_of_message,
        message_type,
        trigger_event,
        control_id,
        processing_id,
        version_id,
        accept_acknowledgment_type,
        application_acknowledgment_type,
        character_set,
    })
}

#[tauri::command]
pub fn render_header(message: &str, header: Header) -> String {
    let Ok(message) = hl7_parser::parse_message_with_lenient_newlines(message) else {
        return message.to_string();
    };

    let mut message: MessageBuilder = (&message).into();
    if !message.segment_named("MSH").is_some() {
        return message.to_string(); // shouldn't be possible but just in case
    }
    let msh = message
        .segment_named_mut("MSH")
        .expect("MSH segment not found");

    msh.set_field_value(3, header.sending_application.unwrap_or_default());
    msh.set_field_value(4, header.sending_facility.unwrap_or_default());
    msh.set_field_value(5, header.receiving_application.unwrap_or_default());
    msh.set_field_value(6, header.receiving_facility.unwrap_or_default());
    msh.set_field_value(7, header.date_time_of_message.unwrap_or_default());
    let message_type = FieldBuilder::default()
        .with_component_value(1, header.message_type.unwrap_or_default())
        .with_component_value(2, header.trigger_event.unwrap_or_default());
    msh.set_field(9, message_type);
    msh.set_field_value(10, header.control_id.unwrap_or_default());
    msh.set_field_value(11, header.processing_id.unwrap_or_default());
    msh.set_field_value(12, header.version_id.unwrap_or_default());
    msh.set_field_value(15, header.accept_acknowledgment_type.unwrap_or_default());
    msh.set_field_value(
        16,
        header.application_acknowledgment_type.unwrap_or_default(),
    );
    msh.set_field_value(18, header.character_set.unwrap_or_default());

    message.render_with_newlines().to_string()
}
