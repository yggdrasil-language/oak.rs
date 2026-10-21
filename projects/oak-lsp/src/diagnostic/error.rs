use core::fmt::{self, Display, Formatter};

/// Failure while converting byte offsets or structured diagnostics to LSP types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticError {
    /// The provider could not load source text.
    Provider(String),
    /// The byte index is outside the source text.
    IndexTooLarge {
        /// Requested byte index.
        given: usize,
        /// Maximum valid byte index.
        max: usize,
    },
    /// The UTF-8 column offset is not on a character boundary.
    InvalidCharBoundary {
        /// Requested byte index.
        given: usize,
    },
    /// The UTF-16 column is larger than the line allows.
    ColumnTooLarge {
        /// Requested column.
        given: usize,
        /// Maximum valid column.
        max: usize,
    },
    /// A byte range could not be converted to a terminal span.
    RangeOutOfBounds {
        /// Inclusive start offset.
        start: u64,
        /// Exclusive end offset.
        end: u64,
    },
    /// A source reference was not registered for text conversion.
    UnknownSource {
        /// Stable wire identifier for the missing source.
        wire_id: String,
    },
}

impl Display for DiagnosticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider(message) => f.write_str(message),
            Self::IndexTooLarge { given, max } => write!(f, "byte index {given} is out of range, max is {max}"),
            Self::InvalidCharBoundary { given } => write!(f, "byte index {given} is not on a UTF-8 character boundary"),
            Self::ColumnTooLarge { given, max } => write!(f, "column {given} is out of range, max is {max}"),
            Self::RangeOutOfBounds { start, end } => write!(f, "byte range [{start}, {end}) is out of bounds"),
            Self::UnknownSource { wire_id } => write!(f, "source reference {wire_id} is not registered"),
        }
    }
}

impl std::error::Error for DiagnosticError {}
