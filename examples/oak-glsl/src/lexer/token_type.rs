use oak_core::{Token, TokenType, UniversalTokenRole};

/// GLSL token.
pub type GlslToken = Token<GlslTokenType>;

impl TokenType for GlslTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Eof => UniversalTokenRole::Eof,
            Self::Whitespace | Self::Newline => UniversalTokenRole::Whitespace,
            Self::Comment => UniversalTokenRole::Comment,
            Self::Identifier => UniversalTokenRole::Name,
            Self::IntegerLiteral | Self::FloatLiteral | Self::StringLiteral | Self::BooleanLiteral => UniversalTokenRole::Literal,
            Self::Error => UniversalTokenRole::Error,

            Self::Void
            | Self::Bool
            | Self::Int
            | Self::Uint
            | Self::Float
            | Self::Double
            | Self::Vec2
            | Self::Vec3
            | Self::Vec4
            | Self::IVec2
            | Self::IVec3
            | Self::IVec4
            | Self::UVec2
            | Self::UVec3
            | Self::UVec4
            | Self::BVec2
            | Self::BVec3
            | Self::BVec4
            | Self::DVec2
            | Self::DVec3
            | Self::DVec4
            | Self::Mat2
            | Self::Mat3
            | Self::Mat4
            | Self::Mat2x2
            | Self::Mat2x3
            | Self::Mat2x4
            | Self::Mat3x2
            | Self::Mat3x3
            | Self::Mat3x4
            | Self::Mat4x2
            | Self::Mat4x3
            | Self::Mat4x4
            | Self::Sampler2D
            | Self::Sampler3D
            | Self::SamplerCube
            | Self::Sampler2DArray
            | Self::SamplerCubeArray
            | Self::Sampler2DShadow
            | Self::SamplerCubeShadow
            | Self::Sampler2DArrayShadow
            | Self::ISampler2D
            | Self::USampler2D
            | Self::Image2D
            | Self::AtomicUint
            | Self::Const
            | Self::Uniform
            | Self::In
            | Self::Out
            | Self::Inout
            | Self::Buffer
            | Self::Shared
            | Self::Layout
            | Self::Precision
            | Self::Highp
            | Self::Mediump
            | Self::Lowp
            | Self::Flat
            | Self::Smooth
            | Self::Noperspective
            | Self::Centroid
            | Self::Sample
            | Self::Invariant
            | Self::Patch
            | Self::Readonly
            | Self::Writeonly
            | Self::Coherent
            | Self::Volatile
            | Self::Restrict
            | Self::If
            | Self::Else
            | Self::For
            | Self::While
            | Self::Do
            | Self::Switch
            | Self::Case
            | Self::Default
            | Self::Break
            | Self::Continue
            | Self::Return
            | Self::Discard
            | Self::Struct
            | Self::Directive => UniversalTokenRole::Keyword,

            Self::Plus
            | Self::Minus
            | Self::Multiply
            | Self::Divide
            | Self::Modulo
            | Self::Assign
            | Self::PlusAssign
            | Self::MinusAssign
            | Self::MultiplyAssign
            | Self::DivideAssign
            | Self::ModuloAssign
            | Self::Equal
            | Self::NotEqual
            | Self::Less
            | Self::Greater
            | Self::LessEqual
            | Self::GreaterEqual
            | Self::LogicalAnd
            | Self::LogicalOr
            | Self::LogicalNot
            | Self::BitwiseAnd
            | Self::BitwiseOr
            | Self::BitwiseXor
            | Self::BitwiseNot
            | Self::LeftShift
            | Self::RightShift
            | Self::LeftShiftAssign
            | Self::RightShiftAssign
            | Self::BitwiseAndAssign
            | Self::BitwiseOrAssign
            | Self::BitwiseXorAssign
            | Self::Increment
            | Self::Decrement
            | Self::Arrow => UniversalTokenRole::Operator,

            Self::Conditional | Self::LeftParen | Self::RightParen | Self::LeftBracket | Self::RightBracket | Self::LeftBrace | Self::RightBrace | Self::Semicolon | Self::Comma | Self::Dot | Self::Colon | Self::Question | Self::Hash => {
                UniversalTokenRole::Punctuation
            }

            _ => UniversalTokenRole::None,
        }
    }
}

/// GLSL token types.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GlslTokenType {
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
}
