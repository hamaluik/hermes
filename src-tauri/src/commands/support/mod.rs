//! UI support commands for field descriptions and schema queries.
//!
//! This module provides commands that support the UI with lookup functionality
//! for HL7 field descriptions and schema data.
//!
//! # Modules
//!
//! - [`field_description`] - Human-readable descriptions from HL7 specs
//! - [`schema`] - Message and segment schema queries
//!
//! # Usage
//!
//! These commands provide context to the user:
//! - Field descriptions appear in tooltips when cursor moves
//! - Schema data populates segment editing forms and validates structure

mod field_description;
mod schema;

pub use field_description::*;
pub use schema::*;
