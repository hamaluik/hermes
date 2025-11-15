/**
 * Visit Wizard Backend Module
 *
 * Provides TypeScript bindings for Tauri backend commands that search for
 * HL7 system patient visits and apply visit data to HL7 message PV1 segments.
 *
 * Workflow:
 * 1. wizardSearchVisits: Context-aware search - extracts patient from message,
 *    then queries database for that patient's visits
 * 2. User selects visit from results
 * 3. wizardApplyVisit: Updates PV1 segment with visit/encounter data
 *
 * This module bridges the frontend visit wizard UI and the Rust backend
 * that performs SQL queries and HL7 message manipulation.
 */
import { invoke } from "@tauri-apps/api/core";
import type { WizardDatabase } from "./wizard_database";

/**
 * Visit/Encounter Record from HL7 system
 *
 * Represents a patient visit/encounter from the database with
 * all information needed to populate a PV1 (Patient Visit) segment.
 *
 * @property seqno - Visit sequence number (unique identifier)
 * @property external_visit_number - External visit/encounter number (PV1.19)
 * @property location_id - Location identifier for the visit
 * @property providers - Map of provider types to provider IDs (PV1.7, PV1.8, PV1.9)
 * @property patient_type_cd - Patient type code (PV1.2, "I"=Inpatient, "O"=Outpatient, "E"=Emergency)
 * @property hospital_service - Hospital service code (PV1.10)
 * @property account_number - Billing account number (PV1.50)
 * @property admission_date - Admission date/time in HL7 format (PV1.44)
 * @property discharge_date - Discharge date/time in HL7 format (PV1.45)
 * @property facility_id - Facility identifier
 * @property building_id - Building identifier within facility
 * @property point_of_care - Point of care (PV1.3.1)
 * @property room - Room number (PV1.3.2)
 * @property floor - Floor number (PV1.3.4)
 * @property bed - Bed identifier (PV1.3.3)
 */
export interface WizardVisit {
  seqno: number;
  external_visit_number: string;
  location_id: string;
  providers: Record<string, string>;
  patient_type_cd: string;
  hospital_service: string;
  account_number: string;
  admission_date: string;
  discharge_date: string;
  facility_id: string;
  building_id: string;
  point_of_care: string;
  room: string;
  floor: string;
  bed: string;
}

/**
 * Searches database for patient visits
 *
 * Context-aware search that automatically extracts patient information from
 * the current HL7 message's PID segment and queries for that patient's visits.
 *
 * This eliminates the need for manual search input - the backend:
 * 1. Parses the message to extract patient ID/MRN from PID segment
 * 2. Queries the database for visits matching that patient
 * 3. Returns all visits ordered by admission date (most recent first)
 *
 * If no valid patient is found in the message, returns an empty array.
 *
 * @param db - Database connection configuration
 * @param message - Current HL7 message (patient extracted from PID segment)
 * @returns Promise resolving to array of visits for the patient in the message
 * @throws Error if database connection fails or query errors
 */
export async function wizardSearchVisits(
  db: WizardDatabase,
  message: string,
): Promise<WizardVisit[]> {
  return invoke("wizard_search_visits", {
    db,
    message,
  });
}

/**
 * Applies selected visit data to HL7 message
 *
 * Updates the PV1 (Patient Visit) segment with visit/encounter information
 * from the selected visit. Can either merge with existing PV1 data or
 * completely replace it based on override_segment flag.
 *
 * Backend operation:
 * 1. Parses existing HL7 message
 * 2. Extracts or creates PV1 segment
 * 3. Populates fields from visit record (location, type, dates, providers)
 * 4. May query database for additional provider information
 * 5. Reconstructs complete HL7 message
 *
 * Note: Unlike other wizards, this function requires database access during
 * apply (not just search) because provider data may need to be looked up
 * from additional database tables.
 *
 * @param db - Database connection configuration (needed for provider lookups)
 * @param message - Current HL7 message text
 * @param visit - Selected visit record to apply
 * @param override_segment - If true, replaces entire PV1; if false, merges data
 * @returns Promise resolving to updated HL7 message text
 * @throws Error if message parsing fails, visit data is invalid, or database query fails
 */
export async function wizardApplyVisit(
  db: WizardDatabase,
  message: string,
  visit: WizardVisit,
  override_segment: boolean,
): Promise<string> {
  return invoke("wizard_apply_visit", {
    db,
    message,
    visit,
    overridesegment: override_segment,
  });
}
