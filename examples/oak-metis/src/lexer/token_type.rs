use oak_core::language::{TokenType, UniversalTokenRole};

/// Token kinds for the Metis island language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetisTokenType {
    /// `island`
    KwIsland,
    /// `namespace`
    KwNamespace,
    /// `use` (island-body legacy import; prefer top-level `using` / `:`)
    KwUse,
    /// `using` (top-level name resolution only)
    KwUsing,
    /// `node`
    KwNode,
    /// `relation`
    KwRelation,
    /// `axiom`
    KwAxiom,
    /// `theorem`
    KwTheorem,
    /// `action`
    KwAction,
    /// `rewrites`
    KwRewrites,
    /// `connection`
    KwConnection,
    /// `forall`
    KwForall,
    /// `exists`
    KwExists,
    /// `and`
    KwAnd,
    /// `or`
    KwOr,
    /// `not`
    KwNot,
    /// `let`
    KwLet,
    /// `if`
    KwIf,
    /// `in` (membership / binder)
    KwIn,
    /// Identifier.
    Ident,
    /// String literal.
    String,
    /// `::`
    PathSep,
    /// `:`
    Colon,
    /// `->`
    Arrow,
    /// `==`
    EqEq,
    /// `<=`
    OpLe,
    /// `<->`
    Iff,
    /// `=`
    Eq,
    /// `·`
    OpMul,
    /// `+`
    OpPlus,
    /// `⊆`
    OpSubseteq,
    /// `⊇`
    OpSupseteq,
    /// `≅`
    OpIso,
    /// `⁻¹`
    OpInv,
    /// `|`
    Pipe,
    /// `;`
    Semi,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// Whitespace.
    Whitespace,
    /// Comment.
    Comment,
    /// End of file.
    Eof,
    /// Error token.
    Error,
}

impl TokenType for MetisTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn role(&self) -> Self::Role {
        use MetisTokenType::*;
        match self {
            KwIsland | KwNamespace | KwUse | KwUsing | KwNode | KwRelation | KwAxiom | KwTheorem | KwAction | KwRewrites | KwConnection | KwForall | KwExists | KwAnd | KwOr | KwNot | KwLet | KwIf | KwIn => UniversalTokenRole::Keyword,
            Ident => UniversalTokenRole::Name,
            String => UniversalTokenRole::Literal,
            PathSep | Colon | Arrow | Iff | EqEq | Eq | OpLe | OpMul | OpPlus | OpSubseteq | OpSupseteq | OpIso | OpInv | Pipe | Dot => UniversalTokenRole::Operator,
            Semi | LBrace | RBrace | LParen | RParen | LBracket | RBracket | Comma => UniversalTokenRole::Punctuation,
            Whitespace => UniversalTokenRole::Whitespace,
            Comment => UniversalTokenRole::Comment,
            Eof => UniversalTokenRole::Eof,
            Error => UniversalTokenRole::Error,
        }
    }
}
