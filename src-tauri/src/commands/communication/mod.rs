//! HL7 message communication via MLLP (Minimal Lower Layer Protocol).
//!
//! This module provides commands for sending and receiving HL7 messages over TCP
//! using the MLLP framing protocol.
//!
//! # Modules
//!
//! - [`send`] - MLLP client for sending messages and receiving ACKs
//! - [`listen`] - MLLP server for receiving messages and sending ACKs
//!
//! # Event-Driven Architecture
//!
//! Both send and listen operations use Tauri events to communicate progress:
//! - `send-log` / `send-response` - Progress and results from send operations
//! - `received-message` - Incoming messages from the listener
//!
//! This allows the UI to show real-time feedback while async operations run.

mod listen;
mod send;

pub use listen::*;
pub use send::*;
