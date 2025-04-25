use hl7_parser::{
    builder::{FieldBuilder, MessageBuilder, SegmentBuilder},
    Message,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct VisitLocation {
    pub point_of_care: Option<String>,
    pub room: Option<String>,
    pub facility: Option<String>,
    pub building: Option<String>,
    pub floor: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Provider {
    pub member_id: Option<String>,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Visit {
    pub sequence_number: Option<String>,
    pub patient_class: Option<String>,
    pub location: VisitLocation,
    pub old_location: VisitLocation,
    pub attending_doctor: Provider,
    pub referring_doctor: Provider,
    pub consulting_doctor: Provider,
    pub admitting_doctor: Provider,
    pub external_visit_number: Option<String>,
}

fn parse_location(message: &Message, field: &str) -> VisitLocation {
    let point_of_care = message.query(format!("{field}.1")).map(|point_of_care| {
        message
            .separators
            .decode(point_of_care.raw_value())
            .to_string()
    });

    let room = message
        .query(format!("{field}.2"))
        .map(|room| message.separators.decode(room.raw_value()).to_string());

    let facility = message
        .query(format!("{field}.3"))
        .map(|facility| message.separators.decode(facility.raw_value()).to_string());

    let building = message
        .query(format!("{field}.4"))
        .map(|building| message.separators.decode(building.raw_value()).to_string());

    let floor = message
        .query(format!("{field}.5"))
        .map(|floor| message.separators.decode(floor.raw_value()).to_string());

    VisitLocation {
        point_of_care,
        room,
        facility,
        building,
        floor,
    }
}

fn parse_provider(message: &Message, field: &str) -> Provider {
    let member_id = message
        .query(format!("{field}.1"))
        .map(|member_id| message.separators.decode(member_id.raw_value()).to_string());

    let family_name = message.query(format!("{field}.2")).map(|family_name| {
        message
            .separators
            .decode(family_name.raw_value())
            .to_string()
    });

    let given_name = message.query(format!("{field}.3")).map(|given_name| {
        message
            .separators
            .decode(given_name.raw_value())
            .to_string()
    });

    let middle_name = message.query(format!("{field}.4")).map(|middle_name| {
        message
            .separators
            .decode(middle_name.raw_value())
            .to_string()
    });

    Provider {
        member_id,
        family_name,
        given_name,
        middle_name,
    }
}

#[tauri::command]
pub fn parse_visit(message: &str) -> Option<Visit> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;

    let sequence_number = message.query("PV1.1").map(|sequence_number| {
        message
            .separators
            .decode(sequence_number.raw_value())
            .to_string()
    });

    let patient_class = message.query("PV1.2").map(|patient_class| {
        message
            .separators
            .decode(patient_class.raw_value())
            .to_string()
    });

    let location = parse_location(&message, "PV1.3");
    let old_location = parse_location(&message, "PV1.6");

    let attending_doctor = parse_provider(&message, "PV1.7");
    let referring_doctor = parse_provider(&message, "PV1.8");
    let consulting_doctor = parse_provider(&message, "PV1.9");
    let admitting_doctor = parse_provider(&message, "PV1.10");

    let external_visit_number = message.query("PV1.19").map(|external_visit_number| {
        message
            .separators
            .decode(external_visit_number.raw_value())
            .to_string()
    });

    Some(Visit {
        sequence_number,
        patient_class,
        location,
        old_location,
        attending_doctor,
        referring_doctor,
        consulting_doctor,
        admitting_doctor,
        external_visit_number,
    })
}

#[tauri::command]
pub fn render_visit(message: &str, visit: Visit) -> String {
    let Ok(message) = hl7_parser::parse_message_with_lenient_newlines(message) else {
        return message.to_string();
    };

    let mut message: MessageBuilder = (&message).into();
    let has_pv1 = message.segment_named("PV1").is_some();
    if !has_pv1 {
        message.push_segment(SegmentBuilder::new("PV1"));
    }
    let pv1 = message
        .segment_named_mut("PV1")
        .expect("message has PV1 segment");

    pv1.set_field_value(1, visit.sequence_number.unwrap_or_default());
    pv1.set_field_value(2, visit.patient_class.unwrap_or_default());
    pv1.set_field(
        3,
        FieldBuilder::default()
            .with_component_value(1, visit.location.point_of_care.unwrap_or_default())
            .with_component_value(2, visit.location.room.unwrap_or_default())
            .with_component_value(3, visit.location.facility.unwrap_or_default())
            .with_component_value(4, visit.location.building.unwrap_or_default())
            .with_component_value(5, visit.location.floor.unwrap_or_default()),
    );
    pv1.set_field(
        6,
        FieldBuilder::default()
            .with_component_value(1, visit.old_location.point_of_care.unwrap_or_default())
            .with_component_value(2, visit.old_location.room.unwrap_or_default())
            .with_component_value(3, visit.old_location.facility.unwrap_or_default())
            .with_component_value(4, visit.old_location.building.unwrap_or_default())
            .with_component_value(5, visit.old_location.floor.unwrap_or_default()),
    );

    pv1.set_field(
        7,
        FieldBuilder::default()
            .with_component_value(1, visit.attending_doctor.member_id.unwrap_or_default())
            .with_component_value(2, visit.attending_doctor.family_name.unwrap_or_default())
            .with_component_value(3, visit.attending_doctor.given_name.unwrap_or_default())
            .with_component_value(4, visit.attending_doctor.middle_name.unwrap_or_default()),
    );

    pv1.set_field(
        8,
        FieldBuilder::default()
            .with_component_value(1, visit.referring_doctor.member_id.unwrap_or_default())
            .with_component_value(2, visit.referring_doctor.family_name.unwrap_or_default())
            .with_component_value(3, visit.referring_doctor.given_name.unwrap_or_default())
            .with_component_value(4, visit.referring_doctor.middle_name.unwrap_or_default()),
    );

    pv1.set_field(
        9,
        FieldBuilder::default()
            .with_component_value(1, visit.consulting_doctor.member_id.unwrap_or_default())
            .with_component_value(2, visit.consulting_doctor.family_name.unwrap_or_default())
            .with_component_value(3, visit.consulting_doctor.given_name.unwrap_or_default())
            .with_component_value(4, visit.consulting_doctor.middle_name.unwrap_or_default()),
    );

    pv1.set_field(
        10,
        FieldBuilder::default()
            .with_component_value(1, visit.admitting_doctor.member_id.unwrap_or_default())
            .with_component_value(2, visit.admitting_doctor.family_name.unwrap_or_default())
            .with_component_value(3, visit.admitting_doctor.given_name.unwrap_or_default())
            .with_component_value(4, visit.admitting_doctor.middle_name.unwrap_or_default()),
    );

    pv1.set_field_value(19, visit.external_visit_number.unwrap_or_default());

    message.render_with_newlines().to_string()
}
