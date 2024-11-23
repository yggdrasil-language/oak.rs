use oak_core::{Token, TokenType, UniversalTokenRole};

/// HLSL token type alias.
pub type HlslToken = Token<HlslTokenType>;

impl TokenType for HlslTokenType {
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
            Self::StringLiteral | Self::NumberLiteral | Self::BooleanLiteral => UniversalTokenRole::Literal,
            Self::Error => UniversalTokenRole::Error,

            Self::Bool
            | Self::Int
            | Self::Uint
            | Self::Half
            | Self::Float
            | Self::Double
            | Self::Min16float
            | Self::Min10float
            | Self::Min16int
            | Self::Min12int
            | Self::Min16uint
            | Self::Bool2
            | Self::Bool3
            | Self::Bool4
            | Self::Int2
            | Self::Int3
            | Self::Int4
            | Self::Uint2
            | Self::Uint3
            | Self::Uint4
            | Self::Half2
            | Self::Half3
            | Self::Half4
            | Self::Float2
            | Self::Float3
            | Self::Float4
            | Self::Double2
            | Self::Double3
            | Self::Double4
            | Self::Float2x2
            | Self::Float2x3
            | Self::Float2x4
            | Self::Float3x2
            | Self::Float3x3
            | Self::Float3x4
            | Self::Float4x2
            | Self::Float4x3
            | Self::Float4x4
            | Self::Double2x2
            | Self::Double2x3
            | Self::Double2x4
            | Self::Double3x2
            | Self::Double3x3
            | Self::Double3x4
            | Self::Double4x2
            | Self::Double4x3
            | Self::Double4x4
            | Self::Texture1D
            | Self::Texture2D
            | Self::Texture3D
            | Self::TextureCube
            | Self::Texture1DArray
            | Self::Texture2DArray
            | Self::TextureCubeArray
            | Self::Texture2DMS
            | Self::Texture2DMSArray
            | Self::Sampler
            | Self::SamplerState
            | Self::SamplerComparisonState
            | Self::Buffer
            | Self::StructuredBuffer
            | Self::ByteAddressBuffer
            | Self::RWBuffer
            | Self::RWStructuredBuffer
            | Self::RWByteAddressBuffer
            | Self::AppendStructuredBuffer
            | Self::ConsumeStructuredBuffer
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
            | Self::Static
            | Self::Const
            | Self::Uniform
            | Self::Varying
            | Self::In
            | Self::Out
            | Self::Inout
            | Self::Inline
            | Self::Extern
            | Self::Shared
            | Self::Groupshared
            | Self::Volatile
            | Self::Precise
            | Self::Nointerpolation
            | Self::Linear
            | Self::Centroid
            | Self::Sample
            | Self::Noperspective
            | Self::Target
            | Self::Register
            | Self::Packoffset
            | Self::Struct
            | Self::Cbuffer
            | Self::Tbuffer
            | Self::Technique
            | Self::Pass
            | Self::Interface
            | Self::Class
            | Self::Namespace
            | Self::Typedef
            | Self::Template
            | Self::Typename
            | Self::Using
            | Self::Sizeof
            | Self::Undef
            | Self::Include
            | Self::Define
            | Self::If_
            | Self::Ifdef
            | Self::Ifndef
            | Self::Else_
            | Self::Elif
            | Self::Endif
            | Self::Line
            | Self::Pragma => UniversalTokenRole::Keyword,

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

            Self::Dot
            | Self::Conditional
            | Self::LeftParen
            | Self::RightParen
            | Self::LeftBracket
            | Self::RightBracket
            | Self::LeftBrace
            | Self::RightBrace
            | Self::Semicolon
            | Self::Comma
            | Self::Colon
            | Self::DoubleColon
            | Self::Question
            | Self::Hash
            | Self::At
            | Self::Backslash => UniversalTokenRole::Punctuation,

            _ => UniversalTokenRole::None,
        }
    }
}

/// Token types for the HLSL (High-Level Shading Language) lexer.
///
/// This enum represents all possible token types in HLSL,
/// including data types, texture types, control flow keywords, operators, and punctuation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HlslTokenType {
    /// Whitespace token.
    Whitespace,
    /// Newline token.
    Newline,

    /// Comment token.
    Comment,

    /// String literal token.
    StringLiteral,
    /// Number literal token.
    NumberLiteral,
    /// Boolean literal token.
    BooleanLiteral,

    /// Identifier token.
    Identifier,

    /// Boolean data type `bool`.
    Bool,
    /// Integer data type `int`.
    Int,
    /// Unsigned integer data type `uint`.
    Uint,
    /// Half-precision floating-point data type `half`.
    Half,
    /// Single-precision floating-point data type `float`.
    Float,
    /// Double-precision floating-point data type `double`.
    Double,
    /// Minimum 16-bit float data type `min16float`.
    Min16float,
    /// Minimum 10-bit float data type `min10float`.
    Min10float,
    /// Minimum 16-bit integer data type `min16int`.
    Min16int,
    /// Minimum 12-bit integer data type `min12int`.
    Min12int,
    /// Minimum 16-bit unsigned integer data type `min16uint`.
    Min16uint,

    /// 2-component boolean vector `bool2`.
    Bool2,
    /// 3-component boolean vector `bool3`.
    Bool3,
    /// 4-component boolean vector `bool4`.
    Bool4,
    /// 2-component integer vector `int2`.
    Int2,
    /// 3-component integer vector `int3`.
    Int3,
    /// 4-component integer vector `int4`.
    Int4,
    /// 2-component unsigned integer vector `uint2`.
    Uint2,
    /// 3-component unsigned integer vector `uint3`.
    Uint3,
    /// 4-component unsigned integer vector `uint4`.
    Uint4,
    /// 2-component half-precision vector `half2`.
    Half2,
    /// 3-component half-precision vector `half3`.
    Half3,
    /// 4-component half-precision vector `half4`.
    Half4,
    /// 2-component float vector `float2`.
    Float2,
    /// 3-component float vector `float3`.
    Float3,
    /// 4-component float vector `float4`.
    Float4,
    /// 2-component double vector `double2`.
    Double2,
    /// 3-component double vector `double3`.
    Double3,
    /// 4-component double vector `double4`.
    Double4,

    /// 2x2 float matrix `float2x2`.
    Float2x2,
    /// 2x3 float matrix `float2x3`.
    Float2x3,
    /// 2x4 float matrix `float2x4`.
    Float2x4,
    /// 3x2 float matrix `float3x2`.
    Float3x2,
    /// 3x3 float matrix `float3x3`.
    Float3x3,
    /// 3x4 float matrix `float3x4`.
    Float3x4,
    /// 4x2 float matrix `float4x2`.
    Float4x2,
    /// 4x3 float matrix `float4x3`.
    Float4x3,
    /// 4x4 float matrix `float4x4`.
    Float4x4,
    /// 2x2 double matrix `double2x2`.
    Double2x2,
    /// 2x3 double matrix `double2x3`.
    Double2x3,
    /// 2x4 double matrix `double2x4`.
    Double2x4,
    /// 3x2 double matrix `double3x2`.
    Double3x2,
    /// 3x3 double matrix `double3x3`.
    Double3x3,
    /// 3x4 double matrix `double3x4`.
    Double3x4,
    /// 4x2 double matrix `double4x2`.
    Double4x2,
    /// 4x3 double matrix `double4x3`.
    Double4x3,
    /// 4x4 double matrix `double4x4`.
    Double4x4,

    /// 1D texture type `Texture1D`.
    Texture1D,
    /// 2D texture type `Texture2D`.
    Texture2D,
    /// 3D texture type `Texture3D`.
    Texture3D,
    /// Cubemap texture type `TextureCube`.
    TextureCube,
    /// 1D texture array type `Texture1DArray`.
    Texture1DArray,
    /// 2D texture array type `Texture2DArray`.
    Texture2DArray,
    /// Cubemap array type `TextureCubeArray`.
    TextureCubeArray,
    /// Multisampled 2D texture type `Texture2DMS`.
    Texture2DMS,
    /// Multisampled 2D texture array type `Texture2DMSArray`.
    Texture2DMSArray,

    /// Sampler type.
    Sampler,
    /// Sampler state type `SamplerState`.
    SamplerState,
    /// Comparison sampler state type `SamplerComparisonState`.
    SamplerComparisonState,

    /// Buffer type.
    Buffer,
    /// Structured buffer type `StructuredBuffer`.
    StructuredBuffer,
    /// Byte address buffer type `ByteAddressBuffer`.
    ByteAddressBuffer,
    /// Read-write buffer type `RWBuffer`.
    RWBuffer,
    /// Read-write structured buffer type `RWStructuredBuffer`.
    RWStructuredBuffer,
    /// Read-write byte address buffer type `RWByteAddressBuffer`.
    RWByteAddressBuffer,
    /// Append structured buffer type `AppendStructuredBuffer`.
    AppendStructuredBuffer,
    /// Consume structured buffer type `ConsumeStructuredBuffer`.
    ConsumeStructuredBuffer,

    /// If keyword `if`.
    If,
    /// Else keyword `else`.
    Else,
    /// For keyword `for`.
    For,
    /// While keyword `while`.
    While,
    /// Do keyword `do`.
    Do,
    /// Switch keyword `switch`.
    Switch,
    /// Case keyword `case`.
    Case,
    /// Default keyword `default`.
    Default,
    /// Break keyword `break`.
    Break,
    /// Continue keyword `continue`.
    Continue,
    /// Return keyword `return`.
    Return,
    /// Discard keyword `discard`.
    Discard,

    /// Static modifier `static`.
    Static,
    /// Const modifier `const`.
    Const,
    /// Uniform modifier `uniform`.
    Uniform,
    /// Varying modifier `varying`.
    Varying,
    /// Input parameter modifier `in`.
    In,
    /// Output parameter modifier `out`.
    Out,
    /// Input-output parameter modifier `inout`.
    Inout,
    /// Inline modifier `inline`.
    Inline,
    /// Extern modifier `extern`.
    Extern,
    /// Shared modifier `shared`.
    Shared,
    /// Group-shared modifier `groupshared`.
    Groupshared,
    /// Volatile modifier `volatile`.
    Volatile,
    /// Precise modifier `precise`.
    Precise,
    /// No interpolation modifier `nointerpolation`.
    Nointerpolation,
    /// Linear interpolation modifier `linear`.
    Linear,
    /// Centroid interpolation modifier `centroid`.
    Centroid,
    /// Sample interpolation modifier `sample`.
    Sample,
    /// No perspective modifier `noperspective`.
    Noperspective,
    /// Target modifier `target`.
    Target,

    /// Register semantic modifier `register`.
    Register,
    /// Pack offset modifier `packoffset`.
    Packoffset,

    /// Struct keyword `struct`.
    Struct,
    /// Constant buffer keyword `cbuffer`.
    Cbuffer,
    /// Texture buffer keyword `tbuffer`.
    Tbuffer,
    /// Technique keyword `technique`.
    Technique,
    /// Pass keyword `pass`.
    Pass,
    /// Interface keyword `interface`.
    Interface,
    /// Class keyword `class`.
    Class,
    /// Namespace keyword `namespace`.
    Namespace,
    /// Typedef keyword `typedef`.
    Typedef,
    /// Template keyword `template`.
    Template,
    /// Typename keyword `typename`.
    Typename,
    /// Using keyword `using`.
    Using,
    /// Sizeof keyword `sizeof`.
    Sizeof,
    /// Undef keyword `undef`.
    Undef,

    /// Include preprocessor directive `#include`.
    Include,
    /// Define preprocessor directive `#define`.
    Define,
    /// If preprocessor directive `#if`.
    If_,
    /// Ifdef preprocessor directive `#ifdef`.
    Ifdef,
    /// Ifndef preprocessor directive `#ifndef`.
    Ifndef,
    /// Else preprocessor directive `#else`.
    Else_,
    /// Elif preprocessor directive `#elif`.
    Elif,
    /// Endif preprocessor directive `#endif`.
    Endif,
    /// Line preprocessor directive `#line`.
    Line,
    /// Pragma preprocessor directive `#pragma`.
    Pragma,

    /// Plus operator `+`.
    Plus,
    /// Minus operator `-`.
    Minus,
    /// Multiply operator `*`.
    Multiply,
    /// Divide operator `/`.
    Divide,
    /// Modulo operator `%`.
    Modulo,
    /// Assignment operator `=`.
    Assign,
    /// Plus assignment operator `+=`.
    PlusAssign,
    /// Minus assignment operator `-=`.
    MinusAssign,
    /// Multiply assignment operator `*=`.
    MultiplyAssign,
    /// Divide assignment operator `/=`.
    DivideAssign,
    /// Modulo assignment operator `%=`.
    ModuloAssign,
    /// Equality operator `==`.
    Equal,
    /// Inequality operator `!=`.
    NotEqual,
    /// Less than operator `<`.
    Less,
    /// Greater than operator `>`.
    Greater,
    /// Less than or equal operator `<=`.
    LessEqual,
    /// Greater than or equal operator `>=`.
    GreaterEqual,
    /// Logical and operator `&&`.
    LogicalAnd,
    /// Logical or operator `||`.
    LogicalOr,
    /// Logical not operator `!`.
    LogicalNot,
    /// Bitwise and operator `&`.
    BitwiseAnd,
    /// Bitwise or operator `|`.
    BitwiseOr,
    /// Bitwise xor operator `^`.
    BitwiseXor,
    /// Bitwise not operator `~`.
    BitwiseNot,
    /// Left shift operator `<<`.
    LeftShift,
    /// Right shift operator `>>`.
    RightShift,
    /// Left shift assignment operator `<<=`.
    LeftShiftAssign,
    /// Right shift assignment operator `>>=`.
    RightShiftAssign,
    /// Bitwise and assignment operator `&=`.
    BitwiseAndAssign,
    /// Bitwise or assignment operator `|=`.
    BitwiseOrAssign,
    /// Bitwise xor assignment operator `^=`.
    BitwiseXorAssign,
    /// Increment operator `++`.
    Increment,
    /// Decrement operator `--`.
    Decrement,
    /// Dot operator `.`.
    Dot,
    /// Arrow operator `->`.
    Arrow,
    /// Conditional ternary operator `?:`.
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
    /// Colon `:`.
    Colon,
    /// Double colon `::`.
    DoubleColon,
    /// Question mark `?`.
    Question,
    /// Hash symbol `#`.
    Hash,
    /// At symbol `@`.
    At,
    /// Backslash symbol `\`.
    Backslash,

    /// End of file marker.
    Eof,
    /// Root node of the AST.
    Root,
    /// Function declaration node.
    FunctionDeclaration,
    /// Struct declaration node.
    StructDeclaration,
    /// Variable declaration node.
    VariableDeclaration,
    /// Parameter list node.
    ParameterList,
    /// Parameter node.
    Parameter,
    /// Code block node.
    Block,
    /// Statement node.
    Statement,
    /// Expression node.
    Expression,
    /// Error node.
    Error,
}
