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
    /// Qualified path for namespace and using declarations.
    pub path: Option<Vec<String>>,
    /// Exact parameter or signature syntax when the declaration has one.
    pub signature: Option<VosSyntaxSlice>,
    /// Exact body syntax when the declaration has a balanced body block.
    pub body: Option<VosSyntaxSlice>,
    /// Exact return type syntax including the `->` arrow when present.
    pub return_type: Option<VosSyntaxSlice>,
    /// Structured return type syntax projected by Oak without resolving names.
    pub return_type_expr: Option<VosTypeSyntax>,
    /// Typed field syntax for table and class declarations.
    pub fields: Vec<VosField>,
    /// Byte span in the original source.
    pub span: Range<usize>,
}

/// A field declaration projected by Oak without assigning VOS semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosField {
    /// Field name as written in the source.
    pub name: String,
    /// Source span of the field name.
    pub name_span: Range<usize>,
    /// Field attributes in source order.
    pub attributes: Vec<VosFieldAttribute>,
    /// Exact type syntax and its source span.
    pub type_syntax: VosSyntaxSlice,
    /// Structured type syntax projected by Oak.
    pub type_expr: VosTypeSyntax,
    /// Exact default value syntax when present.
    pub default_value: Option<VosSyntaxSlice>,
    /// Source span covering the field declaration.
    pub span: Range<usize>,
}

/// Structured VOS type syntax with no name resolution or builtin semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VosTypeSyntax {
    /// A named type or builtin path.
    Named {
        /// Path segments as written.
        path: Vec<String>,
        /// Source span of the named syntax.
        span: Range<usize>,
    },
    /// A reference wrapper such as `&User`.
    Reference {
        /// Referenced type syntax.
        target: Box<VosTypeSyntax>,
        /// Source span including the ampersand.
        span: Range<usize>,
    },
    /// An optional wrapper such as `User?`.
    Optional {
        /// Inner type syntax.
        inner: Box<VosTypeSyntax>,
        /// Source span including the question mark.
        span: Range<usize>,
    },
    /// A list wrapper such as `[User]`.
    List {
        /// Element type syntax.
        element: Box<VosTypeSyntax>,
        /// Source span including both brackets.
        span: Range<usize>,
    },
    /// A generic type such as `vector<3>` or `list<User>`.
    Generic {
        /// Generic name path.
        path: Vec<String>,
        /// Generic arguments in source order.
        arguments: Vec<VosTypeArgument>,
        /// Source span including the angle brackets.
        span: Range<usize>,
    },
}

/// An Oak-owned generic type argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VosTypeArgument {
    /// Nested type syntax argument.
    Type(VosTypeSyntax),
    /// Literal argument preserved as text and source span.
    Literal(VosSyntaxSlice),
}

/// A field attribute preserved as syntax for VOS semantic resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosFieldAttribute {
    /// First identifier in the attribute syntax when one is present.
    pub name: Option<String>,
    /// Exact attribute source text.
    pub text: String,
    /// Source span of the attribute.
    pub span: Range<usize>,
}

/// A source slice exposed by the Oak Builder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosSyntaxSlice {
    /// Exact source text.
    pub text: String,
    /// Source span in the original document.
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
