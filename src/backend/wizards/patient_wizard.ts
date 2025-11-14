import { invoke } from "@tauri-apps/api/core";
import type { WizardDatabase } from "./wizard_database";

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
