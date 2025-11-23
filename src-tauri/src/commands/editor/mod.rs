//! Core message editor commands.
//!
//! This module provides commands for cursor tracking, message data manipulation,
//! and syntax highlighting.
//!
//! # Modules
//!
//! - [`cursor`] - Cursor position tracking and field navigation (Tab/Shift-Tab)
//! - [`data`] - Segment parsing/rendering, field queries, timestamps, templates
//! - [`syntax_highlight`] - HTML generation with CSS classes for HL7 elements
//!
//! # Editing Flow
//!
//! The editor uses a textarea + HTML overlay pattern:
//! 1. User types in the textarea
//! 2. Frontend calls `syntax_highlight` to get styled HTML
//! 3. HTML overlay renders on top of the textarea
//! 4. Cursor position tracked via `locate_cursor` for context display

mod cursor;
mod data;
mod export;
mod syntax_highlight;

pub use cursor::*;
pub use data::*;
pub use export::*;
pub use syntax_highlight::*;
