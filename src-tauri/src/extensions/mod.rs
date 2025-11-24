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

pub mod host;
mod process;
pub mod protocol;
pub mod types;

// primary public API
pub use host::ExtensionHost;
