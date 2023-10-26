use core::fmt;

/// Error returned by [`super::format_source`] when formatting cannot complete safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    message: String,
}

impl FormatError {
    /// Create a formatting error from a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Error message text.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<String> for FormatError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for FormatError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}
