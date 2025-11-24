//! Extension system for Hermes.
//!
//! This module implements the extension host that manages third-party extensions
//! communicating over JSON-RPC 2.0 via stdio.
//!
//! # Architecture
//!
//! Extensions are standalone executables that communicate with Hermes using
//! LSP-style message framing (Content-Length headers) over stdin/stdout.
//!
//! - [`protocol`] - JSON-RPC 2.0 message types and framing
//! - [`types`] - Shared type definitions
//! - [`process`] - Single extension process management
//! - [`host`] - Multi-extension orchestration

mod host;
mod process;
mod protocol;
mod types;

// primary public API
pub use host::ExtensionHost;

// types needed by Tauri commands (Phase 2)
#[allow(unused_imports)]
pub use host::{ExtensionStatus, ToolbarButtonInfo};
#[allow(unused_imports)]
pub use types::ExtensionConfig;
