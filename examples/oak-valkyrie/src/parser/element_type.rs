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
    /// 函数约束子句。
    WhereClause,
    /// 单个受约束类型及其 bounds。
    WhereConstraint,
    /// 一个完整 trait bound。
    TraitBound,
    /// bound 的关联类型等式。
    AssociatedTypeEquation,
    /// Module-scoped type alias.
    TypeAlias,
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
    /// 数组字面量，元素按源码顺序保留。
    ArrayExpression,
    /// Boolean literal node.
    BooleanLiteral,
    /// Binary expression node.
    BinaryExpression,
    /// Unary expression node.
    UnaryExpression,
    /// Parenthesized expression node.
    ParenthesizedExpression,
    /// Tuple literal expression node: `(a, b, ...)`.
    TupleExpression,
    /// Explicit cast expression: `expr as Type`.
    CastExpression,
    /// Ordinal index node: `receiver[index]` (1-based).
    IndexExpression,
    /// Cardinal index node: `receiver⁅cardinal⁆` (0-based; alias `receiver::[cardinal]`).
    /// Type name is historically `OffsetExpression`; semantics are cardinal indexing.
    OffsetExpression,
    /// Field expression node.
    FieldExpression,
    /// Term-level explicit generic application: `expr::<T>`.
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
    /// Union definition node.
    Union,
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
    // Template (TGrammar)
    /// `<% stmt %>` single-line meta statement.
    TemplateStatement,
    /// `<% if ... %>...<% end if %>` block.
    TemplateIfStatement,
    /// `<% if ... %>` branch.
    TemplateIfFragment,
    /// `<% else if ... %>` branch.
    TemplateElseIfFragment,
    /// `<% else %>` branch.
    TemplateElseFragment,
    /// `<% loop ... %>...<% end loop %>` block.
    TemplateLoop,
    /// `<% loop ... %>` branch.
    TemplateLoopFragment,
    /// `<% match %>...<% end %>` block.
    TemplateMatch,
    /// `<% match ... %>` header.
    TemplateMatchFragment,
    /// `<% case ... %>` branch.
    TemplateCaseFragment,
    /// `<% end keyword %>` terminator.
    TemplateEndFragment,
    /// Template body between `%>` and the next `<%`.
    TemplateText,
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
