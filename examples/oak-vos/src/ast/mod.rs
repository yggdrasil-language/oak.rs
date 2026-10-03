use core::range::Range;

use crate::{lexer::VosTokenType, parser::VosElementType};

/// One lossless Oak-built VOS syntax element.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VosSyntaxElement {
    /// Nested syntax node.
    Node(VosSyntaxNode),
    /// Source token with its exact text.
    Token(VosSyntaxToken),
}

/// A source-spanned VOS CST node produced by Oak Builder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosSyntaxNode {
    /// Oak element kind.
    pub kind: VosElementType,
    /// Byte span in the original source.
    pub span: Range<usize>,
    /// Child nodes and tokens in source order.
    pub children: Vec<VosSyntaxElement>,
}

/// A source token preserved by the Oak VOS CST.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosSyntaxToken {
    /// Oak token kind.
    pub kind: VosTokenType,
    /// Byte span in the original source.
    pub span: Range<usize>,
    /// Exact token text from the source.
    pub text: String,
}

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
    /// Lossless CST emitted by Oak Builder.
    pub syntax: VosSyntaxNode,
    /// Top-level declarations in source order.
    pub declarations: Vec<VosDeclaration>,
}
