//! Tauri command handlers.
//!
//! Commands are organised by feature to mirror the frontend structure and make
//! related code easy to find. Each subdirectory groups commands that serve a
//! common purpose:
//!
//! - [`communication`] - MLLP send/receive over TCP
//! - [`editor`] - Cursor tracking, data manipulation, syntax highlighting
//! - [`validation`] - Message validation and semantic comparison
//! - [`support`] - Field descriptions and schema queries
//!
//! # Adding New Commands
//!
//! 1. Identify which feature group the command belongs to
//! 2. Add the command to the appropriate subdirectory
//! 3. Re-export it from the subdirectory's `mod.rs`
//! 4. Register it in `lib.rs` under `invoke_handler`
//!
//! If the command doesn't fit existing groups, create a new subdirectory.

mod communication;
mod editor;
mod support;
mod validation;

pub use communication::*;
pub use editor::*;
pub use support::*;
pub use validation::*;
