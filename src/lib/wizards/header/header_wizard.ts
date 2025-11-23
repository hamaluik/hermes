/**
 * Header Wizard Backend Module
 *
 * Provides TypeScript bindings for Tauri backend commands that query and apply
 * the database interface configurations to HL7 message headers (MSH segment).
 *
 * Workflow:
 * 1. wizardQueryInterfaces: Queries database for configured interfaces
 * 2. User selects interface from results
 * 3. wizardApplyInterface: Updates MSH segment with interface routing data
 *
 * This module bridges the frontend header wizard UI and the Rust backend
 * that performs SQL queries and HL7 message manipulation.
 */
import { invoke } from "@tauri-apps/api/core";
import type { WizardDatabase } from "../shared/wizard_database";

/**
 * Interface Configuration from the database
 *
 * Represents a configured HL7 interface in the database, containing all
 * routing and connection information needed to populate an MSH segment.
 *
 * @property name - Human-readable interface name (e.g., "Lab Results Interface")
 * @property provider_id - Provider identifier
 * @property sending_app - Sending application name (MSH.3)
 * @property sending_fac - Sending facility name (MSH.4)
 * @property receiving_app - Receiving application name (MSH.5)
 * @property receiving_fac - Receiving facility name (MSH.6)
 * @property version - HL7 version (MSH.12, e.g., "2.5")
 * @property processing_cd - Processing ID (MSH.11, e.g., "P" for Production)
 * @property default_timezone - Time zone for timestamps
 * @property receive_port - MLLP port for receiving messages
 */
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

/**
 * Queries the database database for configured interfaces
 *
 * Fetches interfaces matching the specified message type from the database.
 * The backend filters by message type to show only relevant interfaces
 * (ADT interfaces for patient messages, ORM interfaces for order messages).
 *
 * @param db - Database connection configuration
 * @param messageType - Type of message to filter interfaces by ("ADT" or "ORM")
 * @returns Promise resolving to array of matching interfaces
 * @throws Error if database connection fails or query errors
 */
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

/**
 * Applies selected interface configuration to HL7 message
 *
 * Updates the MSH (Message Header) segment with routing information from
 * the selected interface. Can either merge with existing MSH data or
 * completely replace it based on override_segment flag.
 *
 * Backend operation:
 * 1. Parses existing HL7 message
 * 2. Extracts or creates MSH segment
 * 3. Populates fields from interface configuration
 * 4. Updates message type and trigger event
 * 5. Reconstructs complete HL7 message
 *
 * @param message - Current HL7 message text
 * @param _interface - Selected interface configuration to apply
 * @param message_type - Message type to set in MSH.9.1 (e.g., "ADT", "ORM")
 * @param trigger_event - Trigger event to set in MSH.9.2 (e.g., "A01", "O01")
 * @param override_segment - If true, replaces entire MSH; if false, merges data
 * @returns Promise resolving to updated HL7 message text
 * @throws Error if message parsing fails or interface data is invalid
 */
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
