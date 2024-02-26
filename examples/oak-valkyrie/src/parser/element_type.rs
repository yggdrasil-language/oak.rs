use oak_core::{ElementType, UniversalElementRole};

use crate::lexer::token_type::ValkyrieTokenType;

/// Element types for Valkyrie language syntax tree nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValkyrieElementType {
    /// Root node of the syntax tree.
    Root,
    /// End of file marker.
    Eof,
    /// Whitespace token.
    Whitespace,
    /// Newline token.
    Newline,
    /// Line comment token.
    LineComment,
    /// Block comment token.
    BlockComment,
    /// Error node.
    Error,

    // Nodes
    /// Attribute node `[attribute(...)]`.
    Attribute,
    /// Modifier node (`abstract` / `sealed` / arbitrary identifier before a declaration).
    Modifier,
    /// Attribute argument node.
    AttributeArgument,
    /// Name path node.
    NamePath,
    /// Type node.
    Type,
    /// Namespace node.
    Namespace,
    /// Class node.
    Class,
    /// Structure node (value type).
    Structure,
    /// Parameter list node.
    ParameterList,
    /// Block expression node.
    BlockExpression,
    /// Identifier expression node.
    IdentifierExpression,
    /// Path expression node.
    PathExpression,

    // Statements
    /// Statement node.
    Statement,
    /// Let statement node.
    LetStatement,
    /// Expression statement node.
    ExprStatement,

    // Expressions
    /// Expression node.
    Expression,
    /// Call expression node.
    CallExpression,
    /// Parameter node.
    Param,
    /// Return expression node.
    ReturnExpression,
    /// Literal expression node.
    LiteralExpression,
    /// Boolean literal node.
    BooleanLiteral,
    /// Binary expression node.
    BinaryExpression,
    /// Unary expression node.
    UnaryExpression,
    /// Parenthesized expression node.
    ParenthesizedExpression,
    /// Index expression node.
    IndexExpression,
    /// Offset expression node (cardinal indexing, 0-based).
    OffsetExpression,
    /// Field expression node.
    FieldExpression,
    /// Term 层显式泛型应用：`expr::<T>`。
    TurbofishExpression,
    /// If expression node.
    IfExpression,
    /// Match expression node.
    MatchExpression,
    /// Loop expression node.
    LoopExpression,
    /// Break expression node.
    BreakExpression,
    /// Continue expression node.
    ContinueExpression,
    /// Yield expression node.
    YieldExpression,
    /// Raise expression node.
    RaiseExpression,
    /// Catch expression node.
    CatchExpression,
    /// Resume expression node.
    ResumeExpression,
    /// With expression node (functional record update).
    WithExpression,
    /// Apply block node.
    ApplyBlock,
    /// Object expression node.
    ObjectExpression,
    /// Lambda expression node.
    LambdaExpression,

    // Definitions
    /// Micro definition node.
    Micro,
    /// Mezzo definition node.
    Mezzo,
    /// Macro definition node.
    Macro,
    /// Struct definition node.
    Struct,
    /// Enum definition node.
    Enum,
    /// Enums definition node.
    Enums,
    /// Trait definition node.
    Trait,
    /// Associated type node.
    AssociatedType,
    /// Impl definition node.
    Impl,
    /// Imply definition node.
    Imply,
    /// Field definition node.
    Field,
    /// Method definition node.
    Method,
    /// Variant definition node.
    Variant,
    /// Property definition node (getter/setter).
    Property,
    /// Flags definition node.
    Flags,
    /// Widget definition node.
    Widget,
    /// Singleton definition node.
    Singleton,
    /// Shader definition node.
    Shader,
    /// Effect definition node.
    EffectDefinition,
    /// Using statement node.
    UsingStatement,
    /// Component definition node.
    Component,
    /// System definition node.
    System,
    /// Event definition node.
    Event,

    // Template
    /// Template text node.
    TemplateText,
    /// Template control node.
    TemplateControl,
    /// Interpolation node.
    Interpolation,
    /// Template comment node.
    TemplateComment,

    // Others
    /// Pattern node.
    Pattern,
    /// Argument list node.
    ArgList,
    /// Generic parameter list node.
    GenericParameterList,
    /// Generic argument list node.
    GenericArgumentList,
    /// Match arm node.
    MatchArm,
    /// Parameter node.
    Parameter,
    /// Anonymous class node.
    AnonymousClass,
    /// Generic parameter node.
    GenericParameter,
    /// Super call expression node.
    SuperCallExpression,
}

impl ElementType for ValkyrieElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::Root => UniversalElementRole::Root,
            Self::Error => UniversalElementRole::Error,
            _ => UniversalElementRole::None,
        }
    }
}

impl From<ValkyrieTokenType> for ValkyrieElementType {
    fn from(token: ValkyrieTokenType) -> Self {
        match token {
            ValkyrieTokenType::Eof => Self::Eof,
            ValkyrieTokenType::Whitespace => Self::Whitespace,
            ValkyrieTokenType::Newline => Self::Newline,
            ValkyrieTokenType::LineComment => Self::LineComment,
            ValkyrieTokenType::BlockComment => Self::BlockComment,
            ValkyrieTokenType::Error => Self::Error,
            _ => Self::Error,
        }
    }
}
