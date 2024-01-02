//! Formatter errors use Oak's shared diagnostic type.

pub use oak_core::errors::{OakDiagnostics, OakError, OakErrorKind};

/// Result returned by the CST formatter algorithm.
pub type FormatResult<T> = Result<T, OakError>;
