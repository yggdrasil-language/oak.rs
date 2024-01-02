#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![warn(missing_docs)]
//! CST token-gap formatting primitives for Oak language adapters.

pub extern crate alloc;
extern crate self as oak_formatter;

// Public modules
/// Formatting configuration and gap constraints.
pub mod config;
/// Shared Oak formatter errors.
pub mod errors;
/// Source edit and token-gap operations.
pub mod formatters;

pub use crate::{
    config::{GapConstraint, GapFormatOptions},
    errors::{FormatResult, OakDiagnostics, OakError, OakErrorKind},
    formatters::{TextEdit, TokenGap, apply_edits, conservative_constraint, edits_for_gaps},
};
