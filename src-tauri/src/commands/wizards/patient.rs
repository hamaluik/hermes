//! Patient wizard for populating HL7 message PID segments with patient demographic data.
//!
//! This wizard provides two main capabilities:
//! 1. **Search**: Query the database for patient records by ID, MRN, or name
//! 2. **Apply**: Populate a message's PID segment with patient demographic information
//!
//! ## Workflow
//! The typical workflow is:
//! 1. User searches for a patient using `wizard_search_patients`
//! 2. User selects a patient from the search results
//! 3. User applies the patient data to a message using `wizard_apply_patient`
//! 4. The PID segment is populated with the patient's demographics

use color_eyre::eyre::Context;
use hl7_parser::builder::{FieldBuilder, MessageBuilder};
use serde::{Deserialize, Serialize};

/// Patient demographic information.
///
/// All fields use String types for consistency with HL7 text encoding, even for structured
/// data like dates (formatted as YYYYMMDD).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Patient {
    /// Internal patient identifier
    pub id: String,
    /// Patient's first name
    pub fname: String,
    /// Patient's middle name
    pub mname: String,
    /// Patient's last name
    pub lname: String,
    /// Gender code
    pub gender: String,
    /// Date of birth formatted as YYYYMMDD string
    pub dob: String,
    /// Medical record number
    pub mrn: String,
    /// Enterprise identifier
    pub enterpriseid: String,
}

/// Populate an HL7 message's PID segment with patient demographic information.
///
/// This command modifies an existing message by setting specific fields in the PID
/// segment according to the HL7 v2.x standard. The PID segment must already exist
/// in the message.
///
/// # Arguments
/// * `message` - The HL7 message as a string (with newlines or \r separators)
/// * `patient` - Patient demographic data to populate into the message
/// * `overridesegment` - If true, clears all existing PID fields before populating;
///                       if false, only overwrites the specific fields listed below
///
/// # PID Fields Populated
/// * PID.2 - Enterprise ID
/// * PID.3 - Medical Record Number (MRN)
/// * PID.5 - Patient Name (Last^First^Middle)
/// * PID.7 - Date of Birth (YYYYMMDD)
/// * PID.8 - Gender Code
///
/// # Why These Fields?
/// These are the core patient demographics required for most HL7 ADT and ORM messages.
/// They uniquely identify the patient and provide essential demographic context.
///
/// # Returns
/// * `Ok(String)` - The modified message with patient data populated
/// * `Err(String)` - Error if the message cannot be parsed or lacks a PID segment
#[tauri::command]
pub fn wizard_apply_patient(
    message: &str,
    patient: Patient,
    overridesegment: bool,
) -> Result<String, String> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message)
        .wrap_err_with(|| "Failed to parse message")
        .map_err(|e| format!("{e:#}"))?;

    let mut message: MessageBuilder = (&message).into();
    let pid = if let Some(segment) = message.segment_named_mut("PID") {
        if overridesegment {
            segment.clear();
        }
        segment
    } else {
        return Err("Message does not contain a PID segment".into());
    };

    pid.set_field_value(2, patient.enterpriseid);
    pid.set_field_value(3, patient.mrn);
    pid.set_field(
        5,
        FieldBuilder::default()
            .with_component(1, patient.lname)
            .with_component(2, patient.fname)
            .with_component(3, patient.mname),
    );
    pid.set_field_value(7, patient.dob);
    pid.set_field_value(8, patient.gender);

    Ok(message.render_with_newlines().to_string())
}

/// Search for sample patients.
///
/// Returns hardcoded sample patient data for testing purposes.
/// Supports filtering by patient ID, MRN, or name.
///
/// # Search Priority
/// If multiple search parameters are provided, the search uses this priority order:
/// * Patient ID takes precedence over all other criteria
/// * MRN is used if ID is not provided
/// * Name search is only used if neither ID nor MRN is provided
///
/// # Arguments
/// * `_db` - Database connection configuration (not used - sample data returned)
/// * `name` - Patient name to search for (empty string to skip name search)
/// * `id` - Internal patient ID (empty string to skip ID search)
/// * `mrn` - Medical record number (empty string to skip MRN search)
///
/// # Returns
/// * `Ok(Vec<Patient>)` - List of matching sample patients
/// * `Err(String)` - Error description
#[tauri::command]
pub async fn wizard_search_patients(
    db: super::WizardDatabase,
    name: &str,
    id: &str,
    mrn: &str,
) -> Result<Vec<Patient>, String> {
    // sample patient data
    let sample_patients = vec![
        Patient {
            id: "P001".to_string(),
            fname: "John".to_string(),
            mname: "Michael".to_string(),
            lname: "Doe".to_string(),
            gender: "M".to_string(),
            dob: "19850315".to_string(),
            mrn: "MRN001".to_string(),
            enterpriseid: "ENT001".to_string(),
        },
        Patient {
            id: "P002".to_string(),
            fname: "Jane".to_string(),
            mname: "Elizabeth".to_string(),
            lname: "Smith".to_string(),
            gender: "F".to_string(),
            dob: "19901122".to_string(),
            mrn: "MRN002".to_string(),
            enterpriseid: "ENT002".to_string(),
        },
        Patient {
            id: "P003".to_string(),
            fname: "Robert".to_string(),
            mname: "".to_string(),
            lname: "Johnson".to_string(),
            gender: "M".to_string(),
            dob: "19780708".to_string(),
            mrn: "MRN003".to_string(),
            enterpriseid: "ENT003".to_string(),
        },
        Patient {
            id: "P004".to_string(),
            fname: "Emily".to_string(),
            mname: "Rose".to_string(),
            lname: "Williams".to_string(),
            gender: "F".to_string(),
            dob: "20001201".to_string(),
            mrn: "MRN004".to_string(),
            enterpriseid: "ENT004".to_string(),
        },
        Patient {
            id: "P005".to_string(),
            fname: "Michael".to_string(),
            mname: "James".to_string(),
            lname: "Brown".to_string(),
            gender: "M".to_string(),
            dob: "19650430".to_string(),
            mrn: "MRN005".to_string(),
            enterpriseid: "ENT005".to_string(),
        },
    ];

    let id = if id.trim().is_empty() {
        None
    } else {
        Some(id.trim())
    };
    let mrn = if mrn.trim().is_empty() {
        None
    } else {
        Some(mrn.trim())
    };
    let name = if name.trim().is_empty() {
        None
    } else {
        Some(name.trim().to_lowercase())
    };

    // filter by ID if provided
    if let Some(patient_id) = id {
        return Ok(sample_patients
            .into_iter()
            .filter(|p| p.id == patient_id)
            .collect());
    }

    // filter by MRN if provided
    if let Some(mrn_value) = mrn {
        return Ok(sample_patients
            .into_iter()
            .filter(|p| p.mrn == mrn_value)
            .collect());
    }

    // filter by name if provided
    if let Some(name_value) = name {
        return Ok(sample_patients
            .into_iter()
            .filter(|p| {
                p.fname.to_lowercase().contains(&name_value)
                    || p.lname.to_lowercase().contains(&name_value)
                    || p.mname.to_lowercase().contains(&name_value)
            })
            .collect());
    }

    // no filter - return all
    Ok(sample_patients)
}
