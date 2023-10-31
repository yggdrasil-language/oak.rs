use diagnostic::{DiagnosticCode, DiagnosticSeverity, Message};

/// CSS selector subset parse failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssSelectorParseError {
    /// Stable dotted diagnostic code.
    pub code: DiagnosticCode,
    /// Human-readable message.
    pub message: String,
    /// Byte offset in the selector source, when known.
    pub offset: Option<usize>,
}

impl CssSelectorParseError {
    fn new(code: impl Into<String>, message: impl Into<String>, offset: Option<usize>) -> Self {
        Self { code: DiagnosticCode::new(code.into()), message: message.into(), offset }
    }

    /// Unexpected trailing input after a complete selector list.
    #[must_use]
    pub fn unexpected_trailing(remaining: &str, offset: usize) -> Self {
        Self::new("oak.css-selector.unexpected-trailing", format!("unexpected trailing input: {remaining}"), Some(offset))
    }

    /// Unexpected end of input while parsing a selector fragment.
    #[must_use]
    pub fn unexpected_eof(context: &str, offset: usize) -> Self {
        Self::new("oak.css-selector.unexpected-eof", format!("unexpected end of input while parsing {context}"), Some(offset))
    }

    /// Invalid identifier token.
    #[must_use]
    pub fn invalid_ident(token: &str, offset: usize) -> Self {
        Self::new("oak.css-selector.invalid-ident", format!("invalid identifier: {token}"), Some(offset))
    }

    /// Returns the unified diagnostic severity for this parse error.
    #[must_use]
    pub fn severity(&self) -> DiagnosticSeverity {
        DiagnosticSeverity::Error
    }

    /// Converts this parse error into a unified diagnostic message.
    #[must_use]
    pub fn message(&self) -> Message {
        Message::new(self.code.as_str()).with_fallback(self.message.clone())
    }
}

impl std::fmt::Display for CssSelectorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CssSelectorParseError {}
