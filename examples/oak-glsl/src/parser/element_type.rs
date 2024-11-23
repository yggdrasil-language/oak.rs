use crate::lexer::token_type::GlslTokenType;
use oak_core::{ElementType, UniversalElementRole};

/// Element types for the GLSL parser.
///
/// The token variants come first so that [`GlslElementType::from`] can map a
/// token kind; syntax node kinds are appended after them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum GlslElementType {
    /// Whitespace.
    Whitespace,
    /// Newline.
    Newline,
    /// Comment.
    Comment,
    /// Error.
    Error,
    /// End of file.
    Eof,
    /// Plain text.
    Text,

    /// Integer literal.
    IntegerLiteral,
    /// Floating-point literal.
    FloatLiteral,
    /// String literal.
    StringLiteral,
    /// Boolean literal.
    BooleanLiteral,
    /// Identifier.
    Identifier,

    /// `void` keyword.
    Void,
    /// `bool` keyword.
    Bool,
    /// `int` keyword.
    Int,
    /// `uint` keyword.
    Uint,
    /// `float` keyword.
    Float,
    /// `double` keyword.
    Double,
    /// `vec2` keyword.
    Vec2,
    /// `vec3` keyword.
    Vec3,
    /// `vec4` keyword.
    Vec4,
    /// `ivec2` keyword.
    IVec2,
    /// `ivec3` keyword.
    IVec3,
    /// `ivec4` keyword.
    IVec4,
    /// `uvec2` keyword.
    UVec2,
    /// `uvec3` keyword.
    UVec3,
    /// `uvec4` keyword.
    UVec4,
    /// `bvec2` keyword.
    BVec2,
    /// `bvec3` keyword.
    BVec3,
    /// `bvec4` keyword.
    BVec4,
    /// `dvec2` keyword.
    DVec2,
    /// `dvec3` keyword.
    DVec3,
    /// `dvec4` keyword.
    DVec4,
    /// `mat2` keyword.
    Mat2,
    /// `mat3` keyword.
    Mat3,
    /// `mat4` keyword.
    Mat4,
    /// `mat2x2` keyword.
    Mat2x2,
    /// `mat2x3` keyword.
    Mat2x3,
    /// `mat2x4` keyword.
    Mat2x4,
    /// `mat3x2` keyword.
    Mat3x2,
    /// `mat3x3` keyword.
    Mat3x3,
    /// `mat3x4` keyword.
    Mat3x4,
    /// `mat4x2` keyword.
    Mat4x2,
    /// `mat4x3` keyword.
    Mat4x3,
    /// `mat4x4` keyword.
    Mat4x4,
    /// `sampler2D` keyword.
    Sampler2D,
    /// `sampler3D` keyword.
    Sampler3D,
    /// `samplerCube` keyword.
    SamplerCube,
    /// `sampler2DArray` keyword.
    Sampler2DArray,
    /// `samplerCubeArray` keyword.
    SamplerCubeArray,
    /// `sampler2DShadow` keyword.
    Sampler2DShadow,
    /// `samplerCubeShadow` keyword.
    SamplerCubeShadow,
    /// `sampler2DArrayShadow` keyword.
    Sampler2DArrayShadow,
    /// `isampler2D` keyword.
    ISampler2D,
    /// `usampler2D` keyword.
    USampler2D,
    /// `image2D` keyword.
    Image2D,
    /// `atomic_uint` keyword.
    AtomicUint,

    /// `const` keyword.
    Const,
    /// `uniform` keyword.
    Uniform,
    /// `in` keyword.
    In,
    /// `out` keyword.
    Out,
    /// `inout` keyword.
    Inout,
    /// `buffer` keyword.
    Buffer,
    /// `shared` keyword.
    Shared,
    /// `layout` keyword.
    Layout,
    /// `precision` keyword.
    Precision,
    /// `highp` keyword.
    Highp,
    /// `mediump` keyword.
    Mediump,
    /// `lowp` keyword.
    Lowp,
    /// `flat` keyword.
    Flat,
    /// `smooth` keyword.
    Smooth,
    /// `noperspective` keyword.
    Noperspective,
    /// `centroid` keyword.
    Centroid,
    /// `sample` keyword.
    Sample,
    /// `invariant` keyword.
    Invariant,
    /// `patch` keyword.
    Patch,
    /// `readonly` keyword.
    Readonly,
    /// `writeonly` keyword.
    Writeonly,
    /// `coherent` keyword.
    Coherent,
    /// `volatile` keyword.
    Volatile,
    /// `restrict` keyword.
    Restrict,

    /// `if` keyword.
    If,
    /// `else` keyword.
    Else,
    /// `for` keyword.
    For,
    /// `while` keyword.
    While,
    /// `do` keyword.
    Do,
    /// `switch` keyword.
    Switch,
    /// `case` keyword.
    Case,
    /// `default` keyword.
    Default,
    /// `break` keyword.
    Break,
    /// `continue` keyword.
    Continue,
    /// `return` keyword.
    Return,
    /// `discard` keyword.
    Discard,
    /// `struct` keyword.
    Struct,

    /// Preprocessor directive line.
    Directive,

    /// Plus `+`.
    Plus,
    /// Minus `-`.
    Minus,
    /// Multiply `*`.
    Multiply,
    /// Divide `/`.
    Divide,
    /// Modulo `%`.
    Modulo,
    /// Assign `=`.
    Assign,
    /// Plus assign `+=`.
    PlusAssign,
    /// Minus assign `-=`.
    MinusAssign,
    /// Multiply assign `*=`.
    MultiplyAssign,
    /// Divide assign `/=`.
    DivideAssign,
    /// Modulo assign `%=`.
    ModuloAssign,
    /// Equal `==`.
    Equal,
    /// Not equal `!=`.
    NotEqual,
    /// Less than `<`.
    Less,
    /// Greater than `>`.
    Greater,
    /// Less than or equal `<=`.
    LessEqual,
    /// Greater than or equal `>=`.
    GreaterEqual,
    /// Logical and `&&`.
    LogicalAnd,
    /// Logical or `||`.
    LogicalOr,
    /// Logical not `!`.
    LogicalNot,
    /// Bitwise and `&`.
    BitwiseAnd,
    /// Bitwise or `|`.
    BitwiseOr,
    /// Bitwise xor `^`.
    BitwiseXor,
    /// Bitwise not `~`.
    BitwiseNot,
    /// Left shift `<<`.
    LeftShift,
    /// Right shift `>>`.
    RightShift,
    /// Left shift assign `<<=`.
    LeftShiftAssign,
    /// Right shift assign `>>=`.
    RightShiftAssign,
    /// Bitwise and assign `&=`.
    BitwiseAndAssign,
    /// Bitwise or assign `|=`.
    BitwiseOrAssign,
    /// Bitwise xor assign `^=`.
    BitwiseXorAssign,
    /// Increment `++`.
    Increment,
    /// Decrement `--`.
    Decrement,
    /// Arrow `->`.
    Arrow,
    /// Conditional `?`.
    Conditional,

    /// Left parenthesis `(`.
    LeftParen,
    /// Right parenthesis `)`.
    RightParen,
    /// Left bracket `[`.
    LeftBracket,
    /// Right bracket `]`.
    RightBracket,
    /// Left brace `{`.
    LeftBrace,
    /// Right brace `}`.
    RightBrace,
    /// Semicolon `;`.
    Semicolon,
    /// Comma `,`.
    Comma,
    /// Dot `.`.
    Dot,
    /// Colon `:`.
    Colon,
    /// Question mark `?`.
    Question,
    /// Hash `#`.
    Hash,

    /// Root node.
    Root,
    /// Function declaration node.
    FunctionDeclaration,
    /// Struct declaration node.
    StructDeclaration,
    /// Variable declaration node.
    VariableDeclaration,
    /// Module-scope `const` declaration node.
    GlobalConstDeclaration,
    /// Type alias declaration node.
    TypeAliasDeclaration,
    /// Struct or interface block member node.
    StructMemberDeclaration,
    /// `layout(...)` qualifier node.
    LayoutQualifier,
    /// Type reference node.
    TypeReference,
    /// Parameter list node.
    ParameterList,
    /// Parameter node.
    Parameter,
    /// Block node.
    Block,
    /// Local variable declaration statement node.
    VarStatement,
    /// Assignment statement node.
    AssignmentStatement,
    /// `if` statement node.
    IfStatement,
    /// `for` statement node.
    ForStatement,
    /// The initializer part of a `for` header.
    ForInit,
    /// The condition part of a `for` header.
    ForCond,
    /// The update part of a `for` header.
    ForUpdate,
    /// `while` statement node.
    WhileStatement,
    /// `do ... while` statement node.
    DoWhileStatement,
    /// `break` statement node.
    BreakStatement,
    /// `continue` statement node.
    ContinueStatement,
    /// `return` statement node.
    ReturnStatement,
    /// `discard` statement node.
    DiscardStatement,
    /// Expression statement node.
    ExpressionStatement,
    /// Plain expression node such as a literal or a name.
    Expr,
    /// Prefix or postfix expression node.
    ExprUnary,
    /// Binary expression node.
    ExprBinary,
    /// Call expression node.
    ExprCall,
    /// Member access expression node.
    ExprMember,
    /// Index expression node.
    ExprIndex,
    /// Parenthesized expression node.
    ExprParen,
}

impl ElementType for GlslElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::Root => UniversalElementRole::Root,
            Self::FunctionDeclaration | Self::StructDeclaration | Self::VariableDeclaration | Self::GlobalConstDeclaration | Self::TypeAliasDeclaration => UniversalElementRole::Definition,
            Self::Block => UniversalElementRole::Container,
            Self::TypeReference => UniversalElementRole::Typing,
            Self::LayoutQualifier => UniversalElementRole::Attribute,
            Self::ParameterList | Self::Parameter | Self::StructMemberDeclaration => UniversalElementRole::Detail,
            Self::VarStatement
            | Self::AssignmentStatement
            | Self::IfStatement
            | Self::ForStatement
            | Self::WhileStatement
            | Self::DoWhileStatement
            | Self::BreakStatement
            | Self::ContinueStatement
            | Self::ReturnStatement
            | Self::DiscardStatement
            | Self::ExpressionStatement => UniversalElementRole::Statement,
            Self::Expr | Self::ExprUnary | Self::ExprBinary | Self::ExprMember | Self::ExprIndex | Self::ExprParen => UniversalElementRole::Expression,
            Self::ExprCall => UniversalElementRole::Call,
            _ => UniversalElementRole::None,
        }
    }
}

impl From<GlslTokenType> for GlslElementType {
    fn from(token: GlslTokenType) -> Self {
        // Both enums are `#[repr(u8)]` with the token variants in the same order,
        // and node-only variants are appended after them, so the discriminants
        // of every token variant line up.
        unsafe { std::mem::transmute(token) }
    }
}
