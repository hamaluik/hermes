/**
 * Patient Wizard Backend Module
 *
 * Provides TypeScript bindings for Tauri backend commands that search for
 * sample patients and apply patient data to HL7 message PID segments.
 *
 * Workflow:
 * 1. wizardSearchPatients: Queries database for patients matching search criteria
 * 2. User selects patient from results
 * 3. wizardApplyPatient: Updates PID segment with patient demographic data
 *
 * This module bridges the frontend patient wizard UI and the Rust backend
 * that performs SQL queries and HL7 message manipulation.
 */
import { invoke } from "@tauri-apps/api/core";
import type { WizardDatabase } from "./wizard_database";

/**
 * Patient Record from HL7 system
 *
 * Represents a patient from the database with demographic
 * information needed to populate a PID (Patient Identification) segment.
 *
 * @property id - Patient ID (PID.3, internal database ID)
 * @property fname - First/given name (PID.5.2)
 * @property mname - Middle name (PID.5.3)
 * @property lname - Last/family name (PID.5.1)
 * @property gender - Gender code (PID.8, e.g., "M", "F", "U")
 * @property dob - Date of birth in HL7 format (PID.7, YYYYMMDD)
 * @property mrn - Medical Record Number (PID.3, alternate ID)
 * @property enterpriseid - Enterprise-wide patient identifier
 */
export interface WizardPatient {
  id: string;
  fname: string;
  mname: string;
  lname: string;
  gender: string;
  dob: string;
  mrn: string;
  enterpriseid: string;
}

/**
 * Searches database for patients
 *
 * Performs flexible patient search supporting partial and exact matches:
 * - name: Partial match on last name (wildcards applied by backend)
 * - id: Exact match on patient ID
 * - mrn: Exact match on medical record number
 *
 * Empty parameters are ignored, allowing users to search by any combination
 * of criteria. At least one criterion must be provided (validated by UI).
 *
 * @param db - Database connection configuration
 * @param name - Patient last name (partial match, case-insensitive)
 * @param id - Patient ID (exact match)
 * @param mrn - Medical Record Number (exact match)
 * @returns Promise resolving to array of matching patients
 * @throws Error if database connection fails or query errors
 */
export async function wizardSearchPatients(
  db: WizardDatabase,
  name: string,
  id: string,
  mrn: string,
): Promise<WizardPatient[]> {
  return invoke("wizard_search_patients", {
    db,
    name,
    id,
    mrn,
  });
}

/**
 * Applies selected patient data to HL7 message
 *
 * Updates the PID (Patient Identification) segment with demographic information
 * from the selected patient. Can either merge with existing PID data or
 * completely replace it based on override_segment flag.
 *
 * Backend operation:
 * 1. Parses existing HL7 message
 * 2. Extracts or creates PID segment
 * 3. Populates fields from patient record (name, ID, DOB, gender, MRN)
 * 4. Reconstructs complete HL7 message
 *
 * @param message - Current HL7 message text
 * @param patient - Selected patient record to apply
 * @param override_segment - If true, replaces entire PID; if false, merges data
 * @returns Promise resolving to updated HL7 message text
 * @throws Error if message parsing fails or patient data is invalid
 */
export async function wizardApplyPatient(
  message: string,
  patient: WizardPatient,
  override_segment: boolean,
): Promise<string> {
  return invoke("wizard_apply_patient", {
    message,
    patient,
    overridesegment: override_segment,
  });
}
