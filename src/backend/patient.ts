import { invoke } from "@tauri-apps/api/core";

export interface PatientName {
  first?: string;
  last?: string;
  middle?: string;
  suffix?: string;
  prefix?: string;
}

export interface PatientAddress {
  address1?: string;
  address2?: string;
  city?: string;
  state?: string;
  zip?: string;
  country?: string;
  address_type_code?: string;
}

export interface Patient {
  mrn?: string;
  eid?: string;
  name: PatientName;
  /** Date as YYYY-MM-DD */
  date_of_birth?: string;
  gender_code?: string;
  ethnicity_code?: string;
  address: PatientAddress;
  home_phone?: string;
  business_phone?: string;
  account_number?: string;
  ssn?: string;
  status_code?: string;
}

export async function parsePatient(message: string): Promise<Patient | null> {
  return invoke("parse_patient", { message });
}

export async function renderPatient(
  message: string,
  patient: Patient,
): Promise<string> {
  return invoke("render_patient", { message, patient });
}

export function defaultPatient(): Patient {
  return {
    mrn: "",
    eid: "",
    name: {
      first: "",
      last: "",
      middle: "",
      suffix: "",
      prefix: "",
    },
    date_of_birth: "",
    gender_code: "",
    ethnicity_code: "",
    address: {
      address1: "",
      address2: "",
      city: "",
      state: "",
      zip: "",
      country: "",
      address_type_code: "",
    },
    home_phone: "",
    business_phone: "",
    account_number: "",
    ssn: "",
    status_code: "",
  };
}
