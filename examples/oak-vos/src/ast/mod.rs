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
    /// Durable macro declaration.
    Macro,
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
    /// Structured parameters projected from the declaration signature.
    pub parameters: Vec<VosParameter>,
    /// Exact body syntax when the declaration has a balanced body block.
    pub body: Option<VosSyntaxSlice>,
    /// Expression program projected from the body block when it is a program.
    pub body_program: Option<VosProgram>,
    /// Exact return type syntax including the `->` arrow when present.
    pub return_type: Option<VosSyntaxSlice>,
    /// Structured return type syntax projected by Oak without resolving names.
    pub return_type_expr: Option<VosTypeSyntax>,
    /// Typed field syntax for table and class declarations.
    pub fields: Vec<VosField>,
    /// Byte span in the original source.
    pub span: Range<usize>,
}

/// One Oak-projected operation parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosParameter {
    /// Parameter name as written.
    pub name: String,
    /// Source span of the parameter name.
    pub name_span: Range<usize>,
    /// Exact parameter type syntax.
    pub type_syntax: VosSyntaxSlice,
    /// Structured parameter type syntax.
    pub type_expr: VosTypeSyntax,
    /// Span covering the parameter.
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

/// One Oak-projected expression program: `micro` declarations, statements and
/// an optional trailing result expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosProgram {
    /// Session-local `micro` declarations in source order.
    pub micros: Vec<VosFnDecl>,
    /// Leading statements (`let`, expression statements).
    pub statements: Vec<VosStmt>,
    /// Optional trailing expression used as the program result.
    pub result: Option<VosExpr>,
    /// Span covering the program, including body braces for nested programs.
    pub span: Range<usize>,
}

/// Kind of Oak-projected function declaration inside a program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VosFnKind {
    /// Session-local `micro` function.
    Micro,
    /// Durable `macro` function.
    Macro,
}

/// An Oak-projected `micro` or `macro` declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosFnDecl {
    /// Function category.
    pub kind: VosFnKind,
    /// Function name as written.
    pub name: String,
    /// Source span of the function name.
    pub name_span: Range<usize>,
    /// Parameters projected from the signature.
    pub parameters: Vec<VosParameter>,
    /// Structured return type syntax after `->`.
    pub return_type: Option<VosTypeSyntax>,
    /// Body program projected from the balanced body block.
    pub body: VosProgram,
    /// Span covering the whole declaration.
    pub span: Range<usize>,
}

/// One Oak-projected program statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VosStmt {
    /// An immutable `let` binding.
    Let(VosLet),
    /// An expression used as a statement.
    Expr(VosExpr),
}

/// An Oak-projected `let name [: Type] = expression` binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosLet {
    /// Bound name as written.
    pub name: String,
    /// Source span of the bound name.
    pub name_span: Range<usize>,
    /// Optional type annotation syntax.
    pub type_expr: Option<VosTypeSyntax>,
    /// Initializer expression.
    pub value: VosExpr,
    /// Span covering the whole binding.
    pub span: Range<usize>,
}

/// An Oak-projected expression without name resolution or typing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VosExpr {
    /// A literal value.
    Literal(VosLiteral),
    /// A local, type or table name.
    Name {
        /// Identifier text.
        name: String,
        /// Span of the identifier.
        span: Range<usize>,
    },
    /// `Type { field: expression, ... }` construction.
    TypedObject {
        /// Constructed type name as written.
        ty: String,
        /// Source span of the type name.
        ty_span: Range<usize>,
        /// Field initializers in source order.
        fields: Vec<VosFieldInit>,
        /// Span covering the construction.
        span: Range<usize>,
    },
    /// `{ field: expression, ... }` anonymous struct.
    AnonObject {
        /// Field initializers in source order.
        fields: Vec<VosFieldInit>,
        /// Span covering the object.
        span: Range<usize>,
    },
    /// `[ expression, ... ]`.
    List {
        /// Element expressions in source order.
        items: Vec<VosExpr>,
        /// Span covering the list.
        span: Range<usize>,
    },
    /// `x => expression` or `(a, b) => expression`.
    Lambda(VosLambda),
    /// `expression.name` or `expression::name`.
    Member {
        /// Receiver expression.
        object: Box<VosExpr>,
        /// Member name as written.
        name: String,
        /// Source span of the member name.
        name_span: Range<usize>,
        /// Separator as written.
        sep: VosPathSep,
        /// Source span of the member name.
        span: Range<usize>,
    },
    /// `callee(arguments...)`.
    Call {
        /// Callee expression.
        callee: Box<VosExpr>,
        /// Argument expressions in source order.
        args: Vec<VosExpr>,
        /// Span from the callee's own start token through the closing parenthesis.
        span: Range<usize>,
    },
    /// `!expression`.
    Unary {
        /// Unary operator.
        op: VosUnaryOp,
        /// Operand expression.
        expr: Box<VosExpr>,
        /// Span covering the operator and operand.
        span: Range<usize>,
    },
    /// `left op right`.
    Binary {
        /// Binary operator.
        op: VosBinaryOp,
        /// Left operand.
        left: Box<VosExpr>,
        /// Right operand.
        right: Box<VosExpr>,
        /// Span covering both operands and the operator.
        span: Range<usize>,
    },
    /// `expression.*` full projection.
    StarProj {
        /// Receiver expression.
        receiver: Box<VosExpr>,
        /// Span covering the receiver and `.*`.
        span: Range<usize>,
    },
    /// `expression.{ items }` selective projection.
    StructProj {
        /// Receiver expression.
        receiver: Box<VosExpr>,
        /// Projection items in source order.
        items: Vec<VosProjItem>,
        /// Span covering the receiver and the projection block.
        span: Range<usize>,
    },
    /// `expression?` optional unwrap or error propagation.
    Try {
        /// Operand expression.
        expr: Box<VosExpr>,
        /// Span covering the operand and `?`.
        span: Range<usize>,
    },
}

impl VosExpr {
    /// Returns the source span of this expression.
    pub fn span(&self) -> Range<usize> {
        match self {
            Self::Literal(_) => (0..0).into(),
            Self::Name { span, .. }
            | Self::TypedObject { span, .. }
            | Self::AnonObject { span, .. }
            | Self::List { span, .. }
            | Self::Member { span, .. }
            | Self::Call { span, .. }
            | Self::Unary { span, .. }
            | Self::Binary { span, .. }
            | Self::StarProj { span, .. }
            | Self::StructProj { span, .. }
            | Self::Try { span, .. } => span.clone(),
            Self::Lambda(lambda) => lambda.span.clone(),
        }
    }
}

/// An Oak-projected literal value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VosLiteral {
    /// String literal without the surrounding quotes.
    String(String),
    /// Integer digits as written.
    Int(String),
    /// Floating point literal text as written.
    Float(String),
    /// Boolean literal.
    Bool(bool),
    /// `null`.
    Null,
}

/// An Oak-projected lambda expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosLambda {
    /// Parameter names in source order.
    pub params: Vec<String>,
    /// Source spans of the parameter names.
    pub param_spans: Vec<Range<usize>>,
    /// Body expression.
    pub body: Box<VosExpr>,
    /// Span covering the whole lambda.
    pub span: Range<usize>,
}

/// One Oak-projected field initializer inside an object or projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosFieldInit {
    /// Output or field name as written.
    pub name: String,
    /// Source span of the field name.
    pub name_span: Range<usize>,
    /// Explicit value when written as `name: expression`.
    pub value: Option<VosExpr>,
    /// Span covering the initializer.
    pub span: Range<usize>,
}

/// One item inside a `.{ ... }` projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VosProjItem {
    /// Star spread `*`.
    Star {
        /// Span of the `*`.
        span: Range<usize>,
    },
    /// A named or shorthand field.
    Field(VosFieldInit),
}

/// Unary operators supported by the VOS expression grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VosUnaryOp {
    /// `!expression`.
    Not,
}

/// Binary operators supported by the VOS expression grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VosBinaryOp {
    /// `==`.
    Eq,
    /// `!=`.
    Ne,
    /// `<`.
    Lt,
    /// `<=`.
    Le,
    /// `>`.
    Gt,
    /// `>=`.
    Ge,
    /// `&&`.
    And,
    /// `||`.
    Or,
    /// `+`.
    Add,
    /// `-`.
    Sub,
    /// `*`.
    Mul,
    /// `/`.
    Div,
}

/// Separator between a receiver and a member.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VosPathSep {
    /// `.` value member access.
    Dot,
    /// `::` static path separator.
    ColonColon,
}
