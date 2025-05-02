use bytes::BytesMut;
use core::str;
use futures::{sink::SinkExt, StreamExt};
use hl7_mllp_codec::MllpCodec;
use hl7_parser::{builder::MessageBuilder, datetime::TimeStamp};
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use std::net::ToSocketAddrs;
use tauri::{AppHandle, Emitter};
use tokio::{net::TcpStream, time::timeout};
use tokio_util::codec::Framed;

#[derive(Deserialize)]
pub struct SendTransformationsRequest {
    pub control_id: bool,
    pub timestamp: bool,
}

#[derive(Deserialize)]
pub struct SendRequest {
    pub host: String,
    pub port: u16,
    pub transformations: SendTransformationsRequest,
    pub wait_timeout_seconds: f32,
    pub message: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum SendResponse {
    FailedToConnect(String),
    FailedToSend(String),
    FailedToReceive(String),
    FailedToDecode(String),
    FailedToParse { message: String, error: String },
    Final(Option<String>),
}

#[tauri::command]
pub async fn send_message(request: SendRequest, app: AppHandle) -> Result<(), String> {
    let SendRequest {
        host,
        port,
        transformations,
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

    if transformations.control_id {
        use rand::distr::{Alphanumeric, SampleString};
        let value = Alphanumeric.sample_string(&mut rand::rng(), 20);

        message
            .segment_named_mut("MSH")
            .expect("messages have MSH segments")
            .set_field_value(10, value);
    }

    if transformations.timestamp {
        let now = jiff::Zoned::now();
        let now: jiff::civil::DateTime = now.into();
        let now: TimeStamp = now.into();

        message
            .segment_named_mut("MSH")
            .expect("messages have MSH segments")
            .set_field_value(7, now);
    }

    // TODO: {auto} transformations

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
                SendResponse::FailedToConnect(format!("Failed to connect to {addr}")),
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
