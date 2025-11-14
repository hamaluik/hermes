import { invoke } from "@tauri-apps/api/core";
import type { WizardDatabase } from "./wizard_database";

export interface WizardInterface {
  name: string;
  provider_id: string;
  sending_app: string;
  sending_fac: string;
  receiving_app: string;
  receiving_fac: string;
  version: string;
  processing_cd: string;
  default_timezone: string;
  receive_port: number;
}

export async function wizardQueryInterfaces(
  db: WizardDatabase,
  messageType: "ADT" | "ORM",
): Promise<WizardInterface[]> {
  return invoke("wizard_query_interfaces", {
    db,
    messagetype: messageType,
    providerid: null,
  });
}

export async function wizardApplyInterface(
  message: string,
  _interface: WizardInterface,
  message_type: string,
  trigger_event: string,
  override_segment: boolean,
): Promise<string> {
  const args = {
    message,
    interface: _interface,
    messagetype: message_type,
    triggerevent: trigger_event,
    overridesegment: override_segment,
  };
  console.debug(args);
  return invoke("wizard_apply_interface", args);
}
