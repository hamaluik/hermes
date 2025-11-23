//! HL7 message sending via MLLP (Minimal Lower Layer Protocol).
//!
//! This module implements an MLLP client for sending HL7 messages to remote servers
//! and receiving acknowledgment responses. The implementation follows an async,
//! event-driven pattern to provide real-time progress updates to the UI.
//!
//! # MLLP Protocol
//! MLLP is a simple framing protocol for HL7 messages over TCP:
//! * Start Block (SB): 0x0B (vertical tab)
//! * HL7 Message content
//! * End Block (EB): 0x1C (file separator)
//! * Carriage Return (CR): 0x0D
//!
//! The `hl7_mllp_codec` crate handles the framing automatically.
//!
//! # Event-Driven Architecture
//! The send operation emits two types of events to the frontend:
//! * `send-log` - Progress messages for displaying in the UI
//! * `send-response` - Status updates and final result
//!
//! This allows the frontend to show real-time feedback while the async operation
//! executes in a background task.

use bytes::BytesMut;
use core::str;
use futures::{sink::SinkExt, StreamExt};
use hl7_mllp_codec::MllpCodec;
use hl7_parser::{builder::MessageBuilder, datetime::TimeStamp};
use jiff::Zoned;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use std::net::ToSocketAddrs;
use tauri::{AppHandle, Emitter};
use tokio::{net::TcpStream, time::timeout};
use tokio_util::codec::Framed;

/// Request parameters for sending an HL7 message.
///
/// Passed from the frontend to the `send_message` command.
#[derive(Deserialize)]
pub struct SendRequest {
    /// Target hostname or IP address
    pub host: String,
    /// Target port number (typically 2575 for HL7)
    pub port: u16,
    /// How long to wait for a response before timing out (in seconds)
    pub wait_timeout_seconds: f32,
    /// The HL7 message to send (may contain placeholder values)
    pub message: String,
}

/// Response events emitted during the send operation.
///
/// These variants are serialized to camelCase JSON and emitted to the frontend
/// via the `send-response` event channel. The `tag` field becomes "event" and
/// the `content` field becomes "data" in the serialized output.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum SendResponse {
    /// Failed to establish TCP connection
    FailedToConnect(String),
    /// Failed to send the message over MLLP
    FailedToSend(String),
    /// Failed to receive a response (network error, not timeout)
    FailedToReceive(String),
    /// Received response but failed to decode as UTF-8
    FailedToDecode(String),
    /// Received response but failed to parse as valid HL7
    FailedToParse {
        /// The raw response that failed to parse
        message: String,
        /// Parse error details
        error: String,
    },
    /// Final response (success case, or timeout with None)
    Final(Option<String>),
}

/// Send an HL7 message via MLLP and wait for a response.
///
/// This command executes asynchronously and emits progress events to the frontend.
/// It spawns a background task to perform the actual network operations, allowing
/// the command to return immediately to the frontend.
///
/// # Placeholder Transformations
/// Before sending, the message undergoes automatic transformations for special placeholders:
///
/// * **MSH.7 (Timestamp)**: If value is "{auto}" or "{now}", replaced with current timestamp
///   in HL7 format (YYYYMMDDHHMMSS)
///
/// * **MSH.10 (Message Control ID)**: If value is "{auto}" or "{random}", replaced with
///   a 20-character random alphanumeric string
///
/// These placeholders allow users to compose message templates without worrying
/// about generating unique control IDs or current timestamps.
///
/// # Event Flow
/// 1. Validate and resolve the target address
/// 2. Parse the message and apply placeholder transformations
/// 3. Spawn background task and return immediately
/// 4. Background task:
///    - Emit "send-log" with message being sent
///    - Connect via TCP
///    - Send message with MLLP framing
///    - Emit "send-log" awaiting response
///    - Wait for response with timeout
///    - Decode and parse response
///    - Emit "send-response" with final result
///
/// # Timeout Behavior
/// If no response is received within `wait_timeout_seconds`, a timeout log is emitted
/// and a Final(None) response is sent. This is not considered a fatal error, as some
/// HL7 systems may not send acknowledgments for certain message types.
///
/// # Arguments
/// * `request` - Send parameters including host, port, timeout, and message
/// * `app` - Tauri app handle for emitting events to the frontend
///
/// # Returns
/// * `Ok(())` - Background task spawned successfully (does not indicate send success)
/// * `Err(String)` - Failed to resolve address or parse message (before spawning task)
#[tauri::command]
pub async fn send_message(request: SendRequest, app: AppHandle) -> Result<(), String> {
    let SendRequest {
        host,
        port,
        wait_timeout_seconds,
        message,
    } = request;

    let addr = format!("{host}:{port}")
        .to_socket_addrs()
        .map_err(|_| format!("Failed to resolve address for {}:{}", host, port))?
        .next()
        .ok_or_else(|| format!("No host found in `{host}:{port}`"))?;

    let message = hl7_parser::parse_message_with_lenient_newlines(&message)
        .map_err(|e| format!("Failed to parse message: {e:#}"))?;

    let mut message: MessageBuilder = (&message).into();

    // Placeholder transformations for auto-generated values
    // TODO: more general {auto} transformations
    let msh = message
        .segment_named_mut("MSH")
        .expect("messages have MSH segments");

    // Transform {auto} or {now} in MSH.7 to current timestamp
    if let Some(timestamp) = msh.field_mut(7) {
        if let Some(value) = timestamp.value_mut() {
            if value == "{auto}" || value == "{now}" {
                let now = jiff::Zoned::now();
                let now: jiff::civil::DateTime = now.into();
                let now: TimeStamp = now.into();
                *value = now.to_string();
            }
        }
    }

    // Transform {auto} or {random} in MSH.10 to random control ID
    if let Some(control_id) = msh.field_mut(10) {
        if let Some(value) = control_id.value_mut() {
            if value == "{auto}" || value == "{random}" {
                *value = Alphanumeric.sample_string(&mut rand::rng(), 20);
            }
        }
    }

    let message = message.to_string();
    let wait_timeout = std::time::Duration::from_secs_f32(wait_timeout_seconds);

    if let Err(e) = app.emit(
        "send-log",
        format!(
            "[{now}] Sending message to {addr}:\n{message}",
            now = Zoned::now()
        ),
    ) {
        log::error!("Failed to emit send-response event: {e:#}");
    }

    tokio::spawn(async move {
        let Ok(stream) = TcpStream::connect(addr).await else {
            log::error!("Failed to connect to {addr}");
            if let Err(e) = app.emit(
                "send-response",
                SendResponse::FailedToConnect(format!("{addr}")),
            ) {
                log::error!("Failed to emit send-response event: {e:#}");
            }
            return;
        };

        let mut transport = Framed::new(stream, MllpCodec::new());

        if let Err(e) = transport.send(BytesMut::from(message.as_bytes())).await {
            log::error!("Failed to send message: {e:#}");
            if let Err(ee) = app.emit(
                "send-response",
                SendResponse::FailedToSend(format!("{e:#}")),
            ) {
                log::error!("Failed to emit send-response event: {ee:#}");
            }
            return;
        }

        if let Err(e) = app.emit(
            "send-log",
            format!(
                "[{now}] Message sent to {addr}, awaiting response...",
                now = Zoned::now()
            ),
        ) {
            log::error!("Failed to emit send-response event: {e:#}");
        }

        let Some(response) = timeout(wait_timeout, transport.next()).await.ok().flatten() else {
            log::warn!("Timeout waiting for response");
            if let Err(e) = app.emit(
                "send-log",
                format!(
                    "[{now}] Timeout waiting for response after {wait_timeout:?}",
                    now = Zoned::now()
                ),
            ) {
                log::error!("Failed to emit send-response event: {e:#}");
            }
            if let Err(ee) = app.emit("send-response", SendResponse::Final(None)) {
                log::error!("Failed to emit send-response event: {ee:#}");
            }
            return;
        };

        let response = match response {
            Ok(response) => response,
            Err(e) => {
                log::error!("Failed to receive message: {e:#}");
                if let Err(ee) = app.emit(
                    "send-response",
                    SendResponse::FailedToReceive(format!("{e:#}")),
                ) {
                    log::error!("Failed to emit send-response event: {ee:#}");
                }
                return;
            }
        };

        if let Err(e) = app.emit(
            "send-log",
            format!(
                "[{now}] Received response from {addr}: {count} bytes. Parsing...",
                count = response.len(),
                now = Zoned::now()
            ),
        ) {
            log::error!("Failed to emit send-response event: {e:#}");
        }

        let response = match str::from_utf8(&response) {
            Ok(response) => response,
            Err(e) => {
                log::error!("Failed to decode response as UTF-8: {e:#}");
                if let Err(ee) = app.emit(
                    "send-response",
                    SendResponse::FailedToDecode(format!("{e:#}")),
                ) {
                    log::error!("Failed to emit send-response event: {ee:#}");
                }
                return;
            }
        };

        if let Err(e) = app.emit(
            "send-log",
            format!("[{now}] Response:\n{response}", now = Zoned::now()),
        ) {
            log::error!("Failed to emit send-response event: {e:#}");
        }

        let response = match hl7_parser::parse_message_with_lenient_newlines(response) {
            Ok(response) => response,
            Err(e) => {
                log::error!("Failed to parse response message: {e:#}");
                if let Err(ee) = app.emit(
                    "send-response",
                    SendResponse::FailedToParse {
                        message: response.to_string(),
                        error: format!("{e:#}"),
                    },
                ) {
                    log::error!("Failed to emit send-response event: {ee:#}");
                }
                return;
            }
        };

        if let Err(ee) = app.emit(
            "send-response",
            SendResponse::Final(Some(response.raw_value().to_string())),
        ) {
            log::error!("Failed to emit send-response event: {ee:#}");
        }
    });

    Ok(())
}
