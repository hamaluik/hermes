use core::str;
use std::net::ToSocketAddrs;

use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use hl7_mllp_codec::MllpCodec;
use hl7_parser::{
    builder::{FieldBuilder, MessageBuilder, SegmentBuilder},
    message::Separators,
};
use rand::distr::{Alphanumeric, SampleString};
use tauri::{AppHandle, Emitter, State};
use tokio::net::TcpListener;
use tokio_util::codec::Framed;

use crate::AppData;

#[tauri::command]
pub async fn start_listening(
    host: Option<&str>,
    port: u16,
    app: AppHandle,
    state: State<'_, AppData>,
) -> Result<(), String> {
    let host = host.unwrap_or("0.0.0.0");

    let addr = format!("{host}:{port}")
        .to_socket_addrs()
        .map_err(|_| format!("Failed to resolve address for {}:{}", host, port))?
        .next()
        .ok_or_else(|| format!("No host found in `{host}:{port}`"))?;

    log::info!("Listening on {addr}");

    let mut listen_join = state.listen_join.lock().await;
    if let Some(listener) = listen_join.as_mut() {
        listener.abort();
        *listen_join = None;
    }
    drop(listen_join);

    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to start listening on {addr}: {e:#}"))?;

    let handle = tokio::spawn(async move {
        'accept: loop {
            let (stream, remote) = match listener.accept().await {
                Ok((stream, remote)) => (stream, remote),
                Err(e) => {
                    log::error!("Failed to accept connection: {e:#}");
                    continue 'accept;
                }
            };
            log::info!("Accepted connection from {remote}");

            let mut transport = Framed::new(stream, MllpCodec::new());
            'messages: while let Some(result) = transport.next().await {
                let message = match result {
                    Ok(message) => message,
                    Err(e) => {
                        log::error!("Failed to receive message: {e:#}");
                        continue 'messages;
                    }
                };
                let Ok(message) = str::from_utf8(&message) else {
                    log::error!("Failed to decode message: invalid UTF-8");
                    continue 'messages;
                };

                let message = match hl7_parser::parse_message(message) {
                    Ok(message) => message,
                    Err(e) => {
                        log::error!("Failed to parse HL7 message: {e:#}");
                        continue 'messages;
                    }
                };

                // emit the message
                if let Err(e) =
                    app.emit("received-message", message.raw_value().replace('\r', "\n"))
                {
                    log::error!("Failed to emit received-message event: {e:#}");
                }

                // extract info from the message
                let msh = message
                    .segment("MSH")
                    .expect("Valid messages have MSH segments");

                let sending_app = msh
                    .field(3)
                    .map(|f| message.separators.decode(f.raw_value()).to_string())
                    .unwrap_or_default();
                let sending_facility = msh
                    .field(4)
                    .map(|f| message.separators.decode(f.raw_value()).to_string())
                    .unwrap_or_default();
                let receiving_app = msh
                    .field(5)
                    .map(|f| message.separators.decode(f.raw_value()).to_string())
                    .unwrap_or_default();
                let receiving_facility = msh
                    .field(6)
                    .map(|f| message.separators.decode(f.raw_value()).to_string())
                    .unwrap_or_default();
                let trigger_event = msh
                    .field(9)
                    .map(|f| f.component(2))
                    .flatten()
                    .map(|f| message.separators.decode(f.raw_value()).to_string())
                    .unwrap_or_default();
                let control_id = msh
                    .field(10)
                    .map(|f| message.separators.decode(f.raw_value()).to_string())
                    .unwrap_or_default();
                let processing_id = msh
                    .field(11)
                    .map(|f| message.separators.decode(f.raw_value()).to_string())
                    .unwrap_or_default();
                let version_id = msh
                    .field(12)
                    .map(|f| message.separators.decode(f.raw_value()).to_string())
                    .unwrap_or_else(|| "2.5.1".to_string());

                let accept_ack = msh.field(15);
                let application_ack = msh.field(16);

                let is_enhanced_mode = accept_ack.is_some() || application_ack.is_some();
                let ack_level = if is_enhanced_mode { 'C' } else { 'A' };

                let new_cid = Alphanumeric.sample_string(&mut rand::rng(), 20);

                let ack = MessageBuilder::new(Separators::default())
                    .with_segment(
                        SegmentBuilder::new("MSH")
                            .with_field_value(3, receiving_app)
                            .with_field_value(4, receiving_facility)
                            .with_field_value(5, sending_app)
                            .with_field_value(6, sending_facility)
                            .with_field(
                                9,
                                FieldBuilder::default()
                                    .with_component_value(1, "ACK")
                                    .with_component_value(2, trigger_event)
                                    .with_component_value(3, "ACK"),
                            )
                            .with_field_value(10, new_cid)
                            .with_field_value(11, processing_id)
                            .with_field_value(12, version_id),
                    )
                    .with_segment(
                        SegmentBuilder::new("MSA")
                            .with_field_value(1, format!("{ack_level}A"))
                            .with_field_value(2, control_id)
                            .with_field_value(3, "Message accepted"),
                    );
                let ack = ack.to_string();

                if let Err(e) = transport.send(BytesMut::from(ack.as_bytes())).await {
                    log::error!("Failed to send ACK: {e:#}");
                    continue 'messages;
                }
            }
        }
    });

    let mut listen_join = state.listen_join.lock().await;
    *listen_join = Some(handle);
    drop(listen_join);

    Ok(())
}

#[tauri::command]
pub async fn stop_listening(state: State<'_, AppData>) -> Result<(), String> {
    let mut listen_join = state.listen_join.lock().await;
    if let Some(listener) = listen_join.take() {
        listener.abort();
    }
    Ok(())
}
