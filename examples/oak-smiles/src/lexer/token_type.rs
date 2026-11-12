//! Token types for SMILES.

use oak_core::language::{TokenType, UniversalTokenRole};

/// Token kinds emitted by [`crate::lexer::SmilesLexer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SmilesTokenType {
    /// An unbracketed organic-subset atom, for example `C` or `Cl`.
    OrganicAtom,
    /// A bracketed atom including its brackets, for example `[Na+]`.
    BracketAtom,
    /// A bond symbol: `-`, `=`, `#`, `:`, `/`, `\`, or `~`.
    Bond,
    /// A `(` branch opening.
    BranchOpen,
    /// A `)` branch closing.
    BranchClose,
    /// A `.` fragment separator.
    Dot,
    /// A ring label: one digit or a `%nn` pair.
    RingLabel,
    /// A reaction arrow: `>` or `>>`.
    Arrow,
    /// End of input.
    Eof,
}

impl TokenType for SmilesTokenType {
    type Role = UniversalTokenRole;

    const END_OF_STREAM: Self = Self::Eof;

    fn role(&self) -> Self::Role {
        match self {
            Self::OrganicAtom | Self::BracketAtom => UniversalTokenRole::Literal,
            Self::Bond => UniversalTokenRole::Operator,
            Self::BranchOpen | Self::BranchClose | Self::Dot => UniversalTokenRole::Punctuation,
            Self::RingLabel => UniversalTokenRole::Literal,
            Self::Arrow => UniversalTokenRole::Operator,
            Self::Eof => UniversalTokenRole::Eof,
        }
    }
}
