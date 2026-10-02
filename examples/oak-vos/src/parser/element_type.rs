use oak_core::{ElementType, UniversalElementRole};

/// CST element kinds for VOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VosElementType {
    /// Root document.
    Root,
    /// Namespace declaration.
    Namespace,
    /// Table declaration.
    Table,
    /// Class declaration.
    Class,
    /// Enum declaration.
    Enums,
    /// Flags declaration.
    Flags,
    /// Obsolete declaration.
    Obsolete,
    /// Import or using declaration.
    Using,
    /// Constant declaration.
    Const,
    /// Service declaration.
    Service,
    /// Query declaration.
    Query,
    /// UDF declaration.
    Udf,
    /// Recovered syntax error.
    Error,
}

impl ElementType for VosElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::Root => UniversalElementRole::Root,
            Self::Error => UniversalElementRole::Error,
            _ => UniversalElementRole::Statement,
        }
    }
}

impl From<crate::lexer::VosTokenType> for VosElementType {
    fn from(token: crate::lexer::VosTokenType) -> Self {
        match token {
            crate::lexer::VosTokenType::Namespace => Self::Namespace,
            crate::lexer::VosTokenType::Table => Self::Table,
            crate::lexer::VosTokenType::Class => Self::Class,
            crate::lexer::VosTokenType::Enums => Self::Enums,
            crate::lexer::VosTokenType::Flags => Self::Flags,
            crate::lexer::VosTokenType::Obsolete => Self::Obsolete,
            crate::lexer::VosTokenType::Using => Self::Using,
            crate::lexer::VosTokenType::Const => Self::Const,
            crate::lexer::VosTokenType::Service => Self::Service,
            crate::lexer::VosTokenType::Query => Self::Query,
            crate::lexer::VosTokenType::Udf => Self::Udf,
            _ => Self::Error,
        }
    }
}
