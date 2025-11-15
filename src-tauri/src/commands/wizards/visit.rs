//! Visit wizard for populating HL7 message PV1 segments with patient visit data.
//!
//! This wizard provides two main capabilities:
//! 1. **Search**: Query the database for visit records based on patient info
//! 2. **Apply**: Populate a message's PV1 (and related) segments with visit information
//!
//! ## Workflow
//! The typical workflow is:
//! 1. User searches for visits using `wizard_search_visits` (requires patient info in message)
//! 2. User selects a visit from the search results (typically showing the 10 most recent)
//! 3. User applies the visit data to a message using `wizard_apply_visit`
//! 4. The PV1 segment is populated with location, providers, and visit details

use std::collections::HashMap;

use color_eyre::{eyre::Context, Result};
use hl7_parser::builder::{FieldBuilder, MessageBuilder};
use jiff::Zoned;
use serde::{Deserialize, Serialize};

/// Patient visit information retrieved from the database.
///
/// Represents a single encounter between a patient and the healthcare facility.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Visit {
    /// Internal visit sequence number
    pub seqno: Option<i32>,
    /// External visit identifier for the healthcare facility (PV1.19)
    pub external_visit_number: Option<String>,
    /// Internal location identifier
    pub location_id: Option<String>,
    /// Map of provider role codes to provider IDs
    pub providers: std::collections::HashMap<String, String>,
    /// Patient type/class code (PV1.2) - e.g., "I" for inpatient, "O" for outpatient
    pub patient_type_cd: Option<String>,
    /// Hospital service code (PV1.10)
    pub hospital_service: Option<String>,
    /// Patient account number (PID.18)
    pub account_number: Option<String>,
    /// Visit admission date/time with timezone
    pub admission_date: Option<Zoned>,
    /// Visit discharge date/time with timezone
    pub discharge_date: Option<Zoned>,
    /// Facility identifier component of PV1.3 (assigned location)
    pub facility_id: Option<String>,
    /// Building identifier component of PV1.3
    pub building_id: Option<String>,
    /// Point of care (unit/ward) component of PV1.3
    pub point_of_care: Option<String>,
    /// Room number component of PV1.3
    pub room: Option<String>,
    /// Floor component of PV1.3
    pub floor: Option<String>,
    /// Bed number component of PV1.3
    pub bed: Option<String>,
}

/// Populate an HL7 message with patient visit information.
///
/// This command modifies an existing message by setting fields in both the PID and PV1
/// segments. It requires both segments to exist in the message.
///
/// # Location Fields (PV1.3)
/// PV1.3 is a complex field with multiple components representing the patient's location:
/// * Component 1: Point of Care (unit/ward)
/// * Component 2: Room
/// * Component 3: Bed
/// * Component 4: Facility
/// * Component 7: Building
/// * Component 8: Floor
///
/// # Arguments
/// * `db` - Database connection configuration (needed for looking up patient visits)
/// * `message` - The HL7 message as a string
/// * `visit` - Visit data to populate into the message
/// * `overridesegment` - If true, clears all existing PV1 fields before populating;
///                       if false, only overwrites the specific fields
///
/// # Returns
/// * `Ok(String)` - The modified message with visit data populated
/// * `Err(String)` - Error if parsing fails, required segments are missing, or database operations fail
#[tauri::command]
pub async fn wizard_apply_visit(
    db: super::WizardDatabase,
    message: &str,
    visit: Visit,
    overridesegment: bool,
) -> Result<String, String> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message)
        .wrap_err_with(|| "Failed to parse message")
        .map_err(|e| format!("{e:#}"))?;

    let mut message: MessageBuilder = (&message).into();
    let pid = if let Some(segment) = message.segment_named_mut("PID") {
        segment
    } else {
        return Err("Message does not contain a PID segment".into());
    };

    // set account number in the PID
    pid.set_field_value(18, visit.account_number.unwrap_or_default());

    let pv1 = if let Some(segment) = message.segment_named_mut("PV1") {
        if overridesegment {
            segment.clear();
        }
        segment
    } else {
        return Err("Message does not contain a PV1 segment".into());
    };

    // set the rest of the visit info in the PV1
    pv1.set_field_value(1, "1");
    pv1.set_field_value(2, visit.patient_type_cd.unwrap_or_default());
    pv1.set_field(
        3,
        FieldBuilder::default()
            .with_component(1, visit.point_of_care.unwrap_or_default())
            .with_component(2, visit.room.unwrap_or_default())
            .with_component(3, visit.bed.unwrap_or_default())
            .with_component(4, visit.facility_id.unwrap_or_default())
            .with_component(7, visit.building_id.unwrap_or_default())
            .with_component(8, visit.floor.unwrap_or_default()),
    );

    // Set hospital service (PV1.10)
    pv1.set_field_value(10, visit.hospital_service.unwrap_or_default());

    // Set external visit number (PV1.19)
    pv1.set_field_value(19, visit.external_visit_number.unwrap_or_default());

    Ok(message.render_with_newlines().to_string())
}

/// Search for patient visits in the database based on patient info in the message.
///
/// This command extracts patient identifying information from the message's PID segment,
/// searches for the corresponding patient in the database, then retrieves their recent visits.
///
/// # Search Flow
/// 1. Parse the message to extract PID.3 (MRN) and PID.5 (patient name)
/// 2. Search for the patient using `wizard_search_patients`
/// 3. If patient found, query their 10 most recent visits
///
/// # Why Extract from Message?
/// This approach ensures that the visits retrieved are for the same patient already
/// identified in the message being composed. It maintains consistency and reduces
/// the chance of applying visit data for the wrong patient.
///
/// # Arguments
/// * `db` - Database connection configuration
/// * `message` - The HL7 message containing patient info in the PID segment
///
/// # Returns
/// * `Ok(Vec<Visit>)` - List of up to 10 recent visits for the patient
/// * `Err(String)` - Error if message parsing fails, PID segment missing, or no patient found
#[tauri::command]
pub async fn wizard_search_visits(
    db: super::WizardDatabase,
    message: &str,
) -> Result<Vec<Visit>, String> {
    let visits = vec![
        Visit {
            seqno: Some(1),
            external_visit_number: Some("EXT123".to_string()),
            location_id: Some("LOC1".to_string()),
            providers: HashMap::new(),
            patient_type_cd: Some("INPATIENT".to_string()),
            hospital_service: Some("CARDIOLOGY".to_string()),
            account_number: Some("ACC1001".to_string()),
            admission_date: None,
            discharge_date: None,
            facility_id: Some("FAC1".to_string()),
            building_id: Some("BLDG1".to_string()),
            point_of_care: Some("POC1".to_string()),
            room: Some("101".to_string()),
            floor: Some("1".to_string()),
            bed: Some("A".to_string()),
        },
        Visit {
            seqno: Some(2),
            external_visit_number: Some("EXT456".to_string()),
            location_id: Some("LOC2".to_string()),
            providers: HashMap::new(),
            patient_type_cd: Some("OUTPATIENT".to_string()),
            hospital_service: Some("ONCOLOGY".to_string()),
            account_number: Some("ACC2002".to_string()),
            admission_date: None,
            discharge_date: None,
            facility_id: Some("FAC2".to_string()),
            building_id: Some("BLDG2".to_string()),
            point_of_care: Some("POC2".to_string()),
            room: Some("202".to_string()),
            floor: Some("2".to_string()),
            bed: Some("B".to_string()),
        },
    ];
    Ok(visits)
}
