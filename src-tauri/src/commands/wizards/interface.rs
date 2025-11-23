//! Interface wizard for populating HL7 message MSH segments with interface configuration.
//!
//! This wizard provides two main capabilities:
//! 1. **Query**: Retrieve sample interface configurations
//! 2. **Apply**: Populate a message's MSH (and EVN) segments with interface settings
//!
//! ## Workflow
//! The typical workflow is:
//! 1. User queries interfaces using `wizard_query_interfaces` (filtered by message type)
//! 2. User selects an interface configuration from the results
//! 3. User applies the interface to a message using `wizard_apply_interface`
//! 4. The MSH segment is populated with sending/receiving identifiers and HL7 version
//!
//! ## Why Interface Configuration Matters
//! HL7 interfaces define the communication parameters between two systems. Each interface
//! specifies the sending/receiving applications and facilities, the HL7 version to use,
//! and processing mode. This ensures messages are formatted correctly for the receiving system.

use color_eyre::eyre::Context;
use hl7_parser::{
    builder::{FieldBuilder, MessageBuilder, SegmentBuilder},
    message::Separators,
};
use serde::{Deserialize, Serialize};

/// HL7 interface configuration.
///
/// Defines the communication parameters for HL7 message exchange between systems.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Interface {
    /// Interface name
    pub name: String,
    /// Provider ID associated with this interface
    pub provider_id: String,
    /// Sending application identifier (MSH.3)
    pub sending_app: String,
    /// Sending facility identifier (MSH.4)
    pub sending_fac: String,
    /// Receiving application identifier (MSH.5)
    pub receiving_app: String,
    /// Receiving facility identifier (MSH.6)
    pub receiving_fac: String,
    /// HL7 version literal (MSH.12) - e.g., "2.5.1"
    pub version: String,
    /// Processing code (MSH.11) - typically 'P' for production or 'T' for training
    pub processing_cd: String,
    /// Default timezone name for this interface
    pub default_timezone: String,
    /// Port number for receiving messages on this interface
    pub receive_port: u16,
}

/// Populate an HL7 message's MSH segment with interface configuration.
///
/// This command configures the message header with interface-specific settings and
/// optionally creates an EVN segment for event-driven messages.
///
/// # MSH Fields Populated
/// * MSH.1 - Field separator (only if overridesegment=true)
/// * MSH.2 - Encoding characters (only if overridesegment=true)
/// * MSH.3 - Sending application
/// * MSH.4 - Sending facility
/// * MSH.5 - Receiving application
/// * MSH.6 - Receiving facility
/// * MSH.7 - Message timestamp (set to "{now}" placeholder if overridesegment=true)
/// * MSH.9 - Message type and trigger event (e.g., "ADT^A01")
/// * MSH.10 - Message control ID (set to "{random}" placeholder if overridesegment=true)
/// * MSH.11 - Processing ID
/// * MSH.12 - HL7 version
/// * MSH.15 - Accept acknowledgment type (set to "AL" if overridesegment=true)
/// * MSH.16 - Application acknowledgment type (set to "NE" if overridesegment=true)
///
/// # Special Placeholder Values
/// When `overridesegment=true`, the command sets placeholders that are later
/// transformed by the send logic:
/// * "{now}" - Replaced with current timestamp in send_message command
/// * "{random}" - Replaced with random message control ID in send_message command
///
/// # Automatic Segment Creation
/// If the message only contains an MSH segment (blank message) and no EVN exists,
/// an EVN segment is automatically created with:
/// * EVN.1 - Trigger event code
/// * EVN.2 - "{auto}" placeholder (transformed during send)
///
/// For ADT and ORM messages, PID and PV1 segments are auto-created if missing,
/// to provide a complete message skeleton. ORM messages additionally get an ORC segment.
///
/// # Arguments
/// * `message` - The HL7 message as a string
/// * `interface` - Interface configuration to apply
/// * `messagetype` - Message type code (e.g., "ADT", "ORM")
/// * `triggerevent` - Trigger event code (e.g., "A01", "O01")
/// * `overridesegment` - If true, clears MSH and resets to defaults before populating
///
/// # Returns
/// * `Ok(String)` - The modified message with interface configuration
/// * `Err(String)` - Error if message parsing fails
#[tauri::command]
pub fn wizard_apply_interface(
    message: &str,
    interface: Interface,
    messagetype: &str,
    triggerevent: &str,
    overridesegment: bool,
) -> Result<String, String> {
    let message = hl7_parser::parse_message_with_lenient_newlines(message)
        .wrap_err_with(|| "Failed to parse message")
        .map_err(|e| format!("{e:#}"))?;

    let mut message: MessageBuilder = (&message).into();
    let msh = message
        .segment_named_mut("MSH")
        .expect("messages have MSH segments");

    if overridesegment {
        let sep = Separators::default();
        msh.clear();
        msh.set_field_value(1, sep.field);
        msh.set_field_value(
            2,
            format!(
                "{comp}{rep}{esc}{sub}",
                comp = sep.component,
                rep = sep.repetition,
                esc = sep.escape,
                sub = sep.subcomponent
            ),
        );
    }

    msh.set_field_value(3, interface.sending_app);
    msh.set_field_value(4, interface.sending_fac);
    msh.set_field_value(5, interface.receiving_app);
    msh.set_field_value(6, interface.receiving_fac);
    msh.set_field(
        9,
        FieldBuilder::default()
            .with_component_value(1, messagetype)
            .with_component_value(2, triggerevent),
    );
    msh.set_field_value(11, interface.processing_cd);
    msh.set_field_value(12, interface.version);

    if overridesegment {
        msh.set_field_value(7, "{now}");
        msh.set_field_value(10, "{random}");
        msh.set_field_value(15, "AL");
        msh.set_field_value(16, "NE");
    }

    if message.segment_named("EVN").is_none() && message.segments().len() == 1 {
        message.push_segment(
            SegmentBuilder::new("EVN")
                .with_field_value(1, triggerevent)
                .with_field_value(2, "{auto}"),
        );
    }

    // convenience additions for blank messages
    if messagetype == "ADT" || messagetype == "ORM" {
        if message.segment_named("PID").is_none() {
            message.push_segment(SegmentBuilder::new("PID").with_field_value(1, ""));
        }
        if message.segment_named("PV1").is_none() {
            message.push_segment(SegmentBuilder::new("PV1").with_field_value(1, ""));
        }

        if messagetype == "ORM" {
            message.push_segment(SegmentBuilder::new("ORC").with_field_value(1, ""));
        }
    }

    Ok(message.render_with_newlines().to_string())
}

/// Query sample interface configurations.
///
/// Returns hardcoded sample interface configurations for testing purposes.
/// The interfaces returned are filtered by message type.
///
/// # Arguments
/// * `_db` - Database connection configuration (not used - sample data returned)
/// * `messagetype` - Message type code (e.g., "ADT", "ORM")
/// * `_providerid` - Optional provider ID filter (not used)
///
/// # Returns
/// * `Ok(Vec<Interface>)` - List of sample interface configurations
/// * `Err(String)` - Error for unsupported message types
#[tauri::command]
pub async fn wizard_query_interfaces(
    db: super::WizardDatabase,
    messagetype: &str,
    providerid: Option<&str>,
) -> Result<Vec<Interface>, String> {
    log::info!(
        "wizard_query_interfaces called with messagetype={}, providerid={:?}",
        messagetype,
        providerid
    );

    Ok(vec![
        Interface {
            name: "Test Interface".to_string(),
            provider_id: providerid.unwrap_or("provider_123").to_string(),
            sending_app: "AppA".to_string(),
            sending_fac: "FacilityA".to_string(),
            receiving_app: "AppB".to_string(),
            receiving_fac: "FacilityB".to_string(),
            version: "2.5".to_string(),
            processing_cd: "P".to_string(),
            default_timezone: "UTC".to_string(),
            receive_port: 12345,
        },
        Interface {
            name: "Demo Interface".to_string(),
            provider_id: providerid.unwrap_or("provider_456").to_string(),
            sending_app: "AppC".to_string(),
            sending_fac: "FacilityC".to_string(),
            receiving_app: "AppD".to_string(),
            receiving_fac: "FacilityD".to_string(),
            version: "2.3".to_string(),
            processing_cd: "T".to_string(),
            default_timezone: "America/New_York".to_string(),
            receive_port: 23456,
        },
    ])
}
