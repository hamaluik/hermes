use hl7_parser::builder::{FieldBuilder, MessageBuilder, SegmentBuilder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PatientName {
    pub first: Option<String>,
    pub last: Option<String>,
    pub middle: Option<String>,
    pub suffix: Option<String>,
    pub prefix: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PatientAddress {
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
    pub address_type_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Patient {
    pub mrn: Option<String>,
    pub eid: Option<String>,
    pub name: PatientName,
    /// Date as YYYY-MM-DD
    pub date_of_birth: Option<String>,
    pub gender_code: Option<String>,
    pub ethnicity_code: Option<String>,
    pub address: PatientAddress,
    pub home_phone: Option<String>,
    pub business_phone: Option<String>,
    pub account_number: Option<String>,
    pub ssn: Option<String>,
    pub status_code: Option<String>,
}

#[tauri::command]
pub fn parse_patient(message: &str) -> Option<Patient> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message).ok()?;

    let mrn = message
        .query("PID.3.1")
        .map(|mrn| message.separators.decode(mrn.raw_value()).to_string());
    let eid = message
        .query("PID.2.1")
        .map(|eid| message.separators.decode(eid.raw_value()).to_string());

    let name = {
        let last = message
            .query("PID.5.1")
            .map(|last| message.separators.decode(last.raw_value()).to_string());
        let first = message
            .query("PID.5.2")
            .map(|first| message.separators.decode(first.raw_value()).to_string());
        let middle = message
            .query("PID.5.3")
            .map(|middle| message.separators.decode(middle.raw_value()).to_string());
        let suffix = message
            .query("PID.5.4")
            .map(|suffix| message.separators.decode(suffix.raw_value()).to_string());
        let prefix = message
            .query("PID.5.5")
            .map(|prefix| message.separators.decode(prefix.raw_value()).to_string());
        PatientName {
            last,
            first,
            middle,
            suffix,
            prefix,
        }
    };

    let date_of_birth = message
        .query("PID.7")
        .map(|dob| dob.raw_value())
        .and_then(|dob| hl7_parser::datetime::parse_date(dob, true).ok())
        .map(|dob| {
            format!(
                "{:04}-{:02}-{:02}",
                dob.year,
                dob.month.unwrap_or(1),
                dob.day.unwrap_or(1)
            )
        });

    let gender_code = message
        .query("PID.8")
        .map(|gender| message.separators.decode(gender.raw_value()).to_string());

    let ethnicity_code = message
        .query("PID.10")
        .map(|ethnicity| message.separators.decode(ethnicity.raw_value()).to_string());

    let address = {
        let address1 = message
            .query("PID.11.1")
            .map(|address1| message.separators.decode(address1.raw_value()).to_string());
        let address2 = message
            .query("PID.11.2")
            .map(|address2| message.separators.decode(address2.raw_value()).to_string());
        let city = message
            .query("PID.11.3")
            .map(|city| message.separators.decode(city.raw_value()).to_string());
        let state = message
            .query("PID.11.4")
            .map(|state| message.separators.decode(state.raw_value()).to_string());
        let zip = message
            .query("PID.11.5")
            .map(|zip| message.separators.decode(zip.raw_value()).to_string());
        let country = message
            .query("PID.11.6")
            .map(|country| message.separators.decode(country.raw_value()).to_string());
        let address_type_code = message.query("PID.11.7").map(|address_type_code| {
            message
                .separators
                .decode(address_type_code.raw_value())
                .to_string()
        });

        PatientAddress {
            address1,
            address2,
            city,
            state,
            zip,
            country,
            address_type_code,
        }
    };

    let home_phone = message.query("PID.13").map(|home_phone| {
        message
            .separators
            .decode(home_phone.raw_value())
            .to_string()
    });

    let business_phone = message.query("PID.14").map(|business_phone| {
        message
            .separators
            .decode(business_phone.raw_value())
            .to_string()
    });

    let account_number = message.query("PID.18.1").map(|account_number| {
        message
            .separators
            .decode(account_number.raw_value())
            .to_string()
    });

    let ssn = message
        .query("PID.19")
        .map(|ssn| message.separators.decode(ssn.raw_value()).to_string());

    let status_code = message.query("PID.30").map(|status_code| {
        message
            .separators
            .decode(status_code.raw_value())
            .to_string()
    });

    Some(Patient {
        mrn,
        eid,
        name,
        date_of_birth,
        gender_code,
        ethnicity_code,
        address,
        home_phone,
        business_phone,
        account_number,
        ssn,
        status_code,
    })
}

#[tauri::command]
pub fn render_patient(message: &str, patient: Patient) -> String {
    let Ok(message) = hl7_parser::parse_message_with_lenient_newlines(message) else {
        return message.to_string();
    };

    let mut message: MessageBuilder = (&message).into();
    let has_pid = message.segment_named("PID").is_some();
    if !has_pid {
        message.push_segment(SegmentBuilder::new("PID"));
    }
    let pid = message
        .segment_named_mut("PID")
        .expect("message has PID segment");

    pid.set_field_value(3, patient.mrn.unwrap_or_default());
    pid.set_field_value(2, patient.eid.unwrap_or_default());

    let name = FieldBuilder::default()
        .with_component_value(1, patient.name.last.unwrap_or_default())
        .with_component_value(2, patient.name.first.unwrap_or_default())
        .with_component_value(3, patient.name.middle.unwrap_or_default())
        .with_component_value(4, patient.name.suffix.unwrap_or_default())
        .with_component_value(5, patient.name.prefix.unwrap_or_default());
    pid.set_field(5, name);

    // a bit naive, but fine for now
    let date_of_birth = patient
        .date_of_birth
        .as_deref()
        .unwrap_or_default()
        .replace("-", "");
    pid.set_field_value(7, date_of_birth);

    pid.set_field_value(8, patient.gender_code.unwrap_or_default());
    pid.set_field_value(10, patient.ethnicity_code.unwrap_or_default());

    let address = FieldBuilder::default()
        .with_component_value(1, patient.address.address1.unwrap_or_default())
        .with_component_value(2, patient.address.address2.unwrap_or_default())
        .with_component_value(3, patient.address.city.unwrap_or_default())
        .with_component_value(4, patient.address.state.unwrap_or_default())
        .with_component_value(5, patient.address.zip.unwrap_or_default())
        .with_component_value(6, patient.address.country.unwrap_or_default())
        .with_component_value(7, patient.address.address_type_code.unwrap_or_default());
    pid.set_field(11, address);

    pid.set_field_value(13, patient.home_phone.unwrap_or_default());
    pid.set_field_value(14, patient.business_phone.unwrap_or_default());
    pid.set_field_value(18, patient.account_number.unwrap_or_default());
    pid.set_field_value(19, patient.ssn.unwrap_or_default());
    pid.set_field_value(30, patient.status_code.unwrap_or_default());

    message.render_with_newlines().to_string()
}
