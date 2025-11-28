//! HL7 schema definitions embedded at compile time.
//!
//! This module provides schema data for HL7 segments and message types. Schemas define
//! field names, validation rules, allowed values, and template defaults used throughout
//! the application for validation, form generation, and message templates.
//!
//! # Embedded Schemas
//!
//! Schema TOML files in `data/` are embedded into the binary at compile time via
//! `build.rs`. This enables single-binary distribution without external data files.
//! The trade-off is that schema changes require recompilation.
//!
//! # Extension Overrides
//!
//! Extensions can provide runtime schema overrides via the extension API. These are
//! merged with the base schemas and applied via `SchemaCache::set_extension_overrides()`.
//! See `merge.rs` for the merging semantics.

pub mod cache;
pub mod merge;
pub mod message;
pub mod segment;
