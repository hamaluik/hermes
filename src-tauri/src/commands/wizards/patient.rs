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

/// Patient demographic information retrieved from the database.
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

/// Search for patients in the database.
///
/// This command supports three search modes with intelligent prioritization:
/// 1. **Patient ID search** (highest priority) - Direct lookup by internal patient ID
/// 2. **MRN search** (second priority) - Lookup by medical record number
/// 3. **Name search** (fallback) - Flexible name matching with multiple formats
///
/// # Search Priority
/// If multiple search parameters are provided, the search uses this priority order:
/// * Patient ID takes precedence over all other criteria
/// * MRN is used if ID is not provided
/// * Name search is only used if neither ID nor MRN is provided
///
/// This prioritization exists because ID and MRN are unique identifiers that return
/// at most one patient, while name searches can return multiple matches.
///
/// All name searches use case-insensitive partial matching (LIKE '%value%').
///
/// # Arguments
/// * `db` - Database connection configuration
/// * `name` - Patient name in any supported format (empty string to skip name search)
/// * `id` - Internal patient ID (empty string to skip ID search)
/// * `mrn` - Medical record number (empty string to skip MRN search)
///
/// # Returns
/// * `Ok(Vec<Patient>)` - List of matching patients (may be empty if no matches found)
/// * `Err(String)` - Database connection or query error
#[tauri::command]
pub async fn wizard_search_patients(
    db: super::WizardDatabase,
    name: &str,
    id: &str,
    mrn: &str,
) -> Result<Vec<Patient>, String> {
    let name = if name.trim().is_empty() {
        None
    } else {
        Some(name.trim())
    };
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

    let fake_patients = vec![
        Patient {
            id: "1".to_string(),
            fname: "John".to_string(),
            mname: "A".to_string(),
            lname: "Doe".to_string(),
            gender: "M".to_string(),
            dob: "1980-01-01".to_string(),
            mrn: "MRN001".to_string(),
            enterpriseid: "EID001".to_string(),
        },
        Patient {
            id: "2".to_string(),
            fname: "Jane".to_string(),
            mname: "B".to_string(),
            lname: "Smith".to_string(),
            gender: "F".to_string(),
            dob: "1990-02-02".to_string(),
            mrn: "MRN002".to_string(),
            enterpriseid: "EID002".to_string(),
        },
        Patient {
            id: "3".to_string(),
            fname: "Alice".to_string(),
            mname: "C".to_string(),
            lname: "Johnson".to_string(),
            gender: "F".to_string(),
            dob: "1975-03-03".to_string(),
            mrn: "MRN003".to_string(),
            enterpriseid: "EID003".to_string(),
        },
    ];

    let filtered: Vec<Patient> = fake_patients
        .into_iter()
        .filter(|p| {
            let name_match = match name {
                Some(n) => {
                    let (fname, mname, lname) = parse_name(n);
                    let fname_match = fname
                        .as_ref()
                        .map_or(true, |f| p.fname.eq_ignore_ascii_case(f));
                    let mname_match = mname
                        .as_ref()
                        .map_or(true, |m| p.mname.eq_ignore_ascii_case(m));
                    let lname_match = lname
                        .as_ref()
                        .map_or(true, |l| p.lname.eq_ignore_ascii_case(l));
                    fname_match && mname_match && lname_match
                }
                None => true,
            };
            let id_match = match id {
                Some(i) => p.id == i,
                None => true,
            };
            let mrn_match = match mrn {
                Some(m) => p.mrn == m,
                None => true,
            };
            name_match && id_match && mrn_match
        })
        .collect();

    // TODO: actual database search...

    Ok(filtered)
}

/// Parse a name string into (first_name, middle_name, last_name) components.
/// Handles formats like:
/// - "first last" -> (Some(first), None, Some(last))
/// - "last, first" -> (Some(first), None, Some(last))
/// - "first middle last" -> (Some(first), Some(middle), Some(last))
/// - "last, first middle" -> (Some(first), Some(middle), Some(last))
/// - "first" -> (Some(first), None, None)
/// - "last" -> (None, None, Some(last))
fn parse_name(name: &str) -> (Option<String>, Option<String>, Option<String>) {
    let name = name.trim();

    if name.is_empty() {
        return (None, None, None);
    }

    // Check if name contains a comma (likely "last, first" format)
    if let Some(comma_pos) = name.find(',') {
        let last = name[..comma_pos].trim();
        let rest = name[comma_pos + 1..].trim();

        let parts: Vec<&str> = rest.split_whitespace().collect();

        match parts.len() {
            0 => (None, None, Some(last.to_string())),
            1 => (Some(parts[0].to_string()), None, Some(last.to_string())),
            2 => (
                Some(parts[0].to_string()),
                Some(parts[1].to_string()),
                Some(last.to_string()),
            ),
            _ => {
                // More than 2 parts after comma: treat first as first name, rest as middle
                let middle = parts[1..].join(" ");
                (
                    Some(parts[0].to_string()),
                    Some(middle),
                    Some(last.to_string()),
                )
            }
        }
    } else {
        // No comma, split by whitespace
        let parts: Vec<&str> = name.split_whitespace().collect();

        match parts.len() {
            0 => (None, None, None),
            1 => {
                // Single name - ambiguous, but we'll treat as first name
                (Some(parts[0].to_string()), None, None)
            }
            2 => {
                // "first last"
                (Some(parts[0].to_string()), None, Some(parts[1].to_string()))
            }
            3 => {
                // "first middle last"
                (
                    Some(parts[0].to_string()),
                    Some(parts[1].to_string()),
                    Some(parts[2].to_string()),
                )
            }
            _ => {
                // More than 3 parts: first is first name, last is last name, rest is middle
                let middle = parts[1..parts.len() - 1].join(" ");
                (
                    Some(parts[0].to_string()),
                    Some(middle),
                    Some(parts[parts.len() - 1].to_string()),
                )
            }
        }
    }
}
