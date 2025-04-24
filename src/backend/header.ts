import { invoke } from "@tauri-apps/api/core";

export interface Header {
  sending_application?: string;
  sending_facility?: string;
  receiving_application?: string;
  receiving_facility?: string;
  date_time_of_message?: string;
  message_type?: string;
  trigger_event?: string;
  control_id?: string;
  processing_id?: string;
  version_id?: string;
  accept_acknowledgment_type?: string;
  application_acknowledgment_type?: string;
  character_set?: string;
}

export async function parseHeader(message: string): Promise<Header | null> {
  return invoke("parse_header", { message });
}

export async function renderHeader(
  message: string,
  header: Header,
): Promise<string> {
  return invoke("render_header", { message, header });
}

export function defaultHeader(): Header {
  return {
    sending_application: "",
    sending_facility: "",
    receiving_application: "",
    receiving_facility: "",
    date_time_of_message: "",
    message_type: "",
    trigger_event: "",
    control_id: "",
    processing_id: "",
    version_id: "",
    accept_acknowledgment_type: "",
    application_acknowledgment_type: "",
    character_set: "",
  };
}
