//! Message validation and comparison commands.
//!
//! This module provides commands for validating HL7 messages against schema
//! definitions and comparing messages to identify differences.
//!
//! # Modules
//!
//! - [`validate`] - Schema-based validation with light/full modes
//! - [`diff`] - Semantic comparison at segment/field/component level
//!
//! # Validation Modes
//!
//! - **Light** - Fast, runs on every edit (500ms debounce). Checks parse errors
//!   and required fields only.
//! - **Full** - Comprehensive, triggered on-demand. Adds length limits, patterns,
//!   allowed values, date formats, and message structure.
//!
//! Issues include character ranges for inline highlighting via syntax_highlight.

mod diff;
mod validate;

pub use diff::*;
pub use validate::*;
