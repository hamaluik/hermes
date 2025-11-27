//! UI support commands for field descriptions, schema queries, and utilities.
//!
//! This module provides commands that support the UI with lookup functionality
//! for HL7 field descriptions and schema data.
//!
//! # Modules
//!
//! - [`field_description`] - Human-readable descriptions from HL7 specs
//! - [`open_url`] - Open URLs in OS default browser
//! - [`schema`] - Message and segment schema queries
//!
//! # Usage
//!
//! These commands provide context to the user:
//! - Field descriptions appear in tooltips when cursor moves
//! - Schema data populates segment editing forms and validates structure

mod field_description;
mod open_url;
mod schema;

pub use field_description::*;
pub use open_url::*;
pub use schema::*;
