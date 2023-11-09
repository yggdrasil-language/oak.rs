use core::range::Range;

/// Top-level VOS declaration kind emitted by the Oak Builder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VosDeclarationKind {
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
    /// Using declaration.
    Using,
    /// Constant declaration.
    Const,
    /// Service declaration.
    Service,
    /// Query declaration.
    Query,
    /// UDF declaration.
    Udf,
    /// Session-local micro declaration.
    Micro,
}

/// A source-spanned top-level VOS declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosDeclaration {
    /// Declaration category.
    pub kind: VosDeclarationKind,
    /// First declaration name when the syntax has one.
    pub name: Option<String>,
    /// Byte span in the original source.
    pub span: Range<usize>,
}

/// The initial Oak-built VOS AST root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosRoot {
    /// The exact source passed to the Oak frontend.
    pub source: String,
    /// Top-level declarations in source order.
    pub declarations: Vec<VosDeclaration>,
}
