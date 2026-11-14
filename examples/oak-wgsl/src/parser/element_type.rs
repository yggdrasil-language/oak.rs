use crate::lexer::token_type::WgslTokenType;
use oak_core::{ElementType, UniversalElementRole};

/// WGSL element types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WgslElementType {
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
    /// Text.
    Text,

    /// Integer literal.
    IntegerLiteral,
    /// Float literal.
    FloatLiteral,
    /// String literal.
    StringLiteral,
    /// Boolean literal.
    BoolLiteral,
    /// Identifier.
    Identifier,

    /// `bool` keyword.
    BoolKw,
    /// `i32` keyword.
    I32Kw,
    /// `u32` keyword.
    U32Kw,
    /// `f32` keyword.
    F32Kw,
    /// `f16` keyword.
    F16Kw,
    /// `vec2` keyword.
    Vec2Kw,
    /// `vec3` keyword.
    Vec3Kw,
    /// `vec4` keyword.
    Vec4Kw,
    /// `mat2x2` keyword.
    Mat2x2Kw,
    /// `mat2x3` keyword.
    Mat2x3Kw,
    /// `mat2x4` keyword.
    Mat2x4Kw,
    /// `mat3x2` keyword.
    Mat3x2Kw,
    /// `mat3x3` keyword.
    Mat3x3Kw,
    /// `mat3x4` keyword.
    Mat3x4Kw,
    /// `mat4x2` keyword.
    Mat4x2Kw,
    /// `mat4x3` keyword.
    Mat4x3Kw,
    /// `mat4x4` keyword.
    Mat4x4Kw,
    /// `array` keyword.
    ArrayKw,
    /// `ptr` keyword.
    PtrKw,
    /// `atomic` keyword.
    AtomicKw,
    /// `sampler` keyword.
    SamplerKw,
    /// `sampler_comparison` keyword.
    SamplerComparisonKw,
    /// `texture_1d` keyword.
    Texture1dKw,
    /// `texture_2d` keyword.
    Texture2dKw,
    /// `texture_2d_array` keyword.
    Texture2dArrayKw,
    /// `texture_3d` keyword.
    Texture3dKw,
    /// `texture_cube` keyword.
    TextureCubeKw,
    /// `texture_cube_array` keyword.
    TextureCubeArrayKw,
    /// `texture_multisampled_2d` keyword.
    TextureMultisampled2dKw,
    /// `texture_depth_2d` keyword.
    TextureDepth2dKw,
    /// `texture_depth_cube` keyword.
    TextureDepthCubeKw,
    /// `texture_depth_multisampled_2d` keyword.
    TextureDepthMultisampled2dKw,
    /// `texture_storage_1d` keyword.
    TextureStorage1dKw,
    /// `texture_storage_2d` keyword.
    TextureStorage2dKw,
    /// `texture_storage_2d_array` keyword.
    TextureStorage2dArrayKw,
    /// `texture_storage_3d` keyword.
    TextureStorage3dKw,

    /// `fn` keyword.
    FnKw,
    /// `var` keyword.
    VarKw,
    /// `let` keyword.
    LetKw,
    /// `const` keyword.
    ConstKw,
    /// `if` keyword.
    IfKw,
    /// `else` keyword.
    ElseKw,
    /// `switch` keyword.
    SwitchKw,
    /// `case` keyword.
    CaseKw,
    /// `default` keyword.
    DefaultKw,
    /// `loop` keyword.
    LoopKw,
    /// `for` keyword.
    ForKw,
    /// `while` keyword.
    WhileKw,
    /// `break` keyword.
    BreakKw,
    /// `continue` keyword.
    ContinueKw,
    /// `return` keyword.
    ReturnKw,
    /// `discard` keyword.
    DiscardKw,

    /// `function` keyword.
    FunctionKw,
    /// `private` keyword.
    PrivateKw,
    /// `workgroup` keyword.
    WorkgroupKw,
    /// `uniform` keyword.
    UniformKw,
    /// `storage` keyword.
    StorageKw,

    /// `read` keyword.
    ReadKw,
    /// `write` keyword.
    WriteKw,
    /// `read_write` keyword.
    ReadWriteKw,

    /// `vertex` keyword.
    VertexKw,
    /// `fragment` keyword.
    FragmentKw,
    /// `compute` keyword.
    ComputeKw,
    /// `binding` keyword.
    BindingKw,
    /// `group` keyword.
    GroupKw,
    /// `location` keyword.
    LocationKw,
    /// `builtin` keyword.
    BuiltinKw,
    /// `interpolate` keyword.
    InterpolateKw,
    /// `invariant` keyword.
    InvariantKw,
    /// `size` keyword.
    SizeKw,
    /// `align` keyword.
    AlignKw,
    /// `workgroup_size` keyword.
    WorkgroupSizeKw,

    /// `position` keyword.
    PositionKw,
    /// `vertex_index` keyword.
    VertexIndexKw,
    /// `instance_index` keyword.
    InstanceIndexKw,
    /// `front_facing` keyword.
    FrontFacingKw,
    /// `frag_depth` keyword.
    FragDepthKw,
    /// `local_invocation_id` keyword.
    LocalInvocationIdKw,
    /// `local_invocation_index` keyword.
    LocalInvocationIndexKw,
    /// `global_invocation_id` keyword.
    GlobalInvocationIdKw,
    /// `workgroup_id` keyword.
    WorkgroupIdKw,
    /// `num_workgroups` keyword.
    NumWorkgroupsKw,
    /// `sample_index` keyword.
    SampleIndexKw,
    /// `sample_mask` keyword.
    SampleMaskKw,

    /// `struct` keyword.
    StructKw,
    /// `type` keyword.
    TypeKw,
    /// `alias` keyword.
    AliasKw,
    /// `enable` keyword.
    EnableKw,
    /// `requires` keyword.
    RequiresKw,
    /// `override` keyword.
    OverrideKw,

    /// Plus `+`.
    Plus,
    /// Minus `-`.
    Minus,
    /// Star `*`.
    Star,
    /// Slash `/`.
    Slash,
    /// Percent `%`.
    Percent,
    /// Ampersand `&`.
    Ampersand,
    /// Pipe `|`.
    Pipe,
    /// Caret `^`.
    Caret,
    /// Tilde `~`.
    Tilde,
    /// Left shift `<<`.
    LeftShift,
    /// Right shift `>>`.
    RightShift,
    /// Plus assign `+=`.
    PlusAssign,
    /// Minus assign `-=`.
    MinusAssign,
    /// Star assign `*=`.
    StarAssign,
    /// Slash assign `/=`.
    SlashAssign,
    /// Percent assign `%=`.
    PercentAssign,
    /// Ampersand assign `&=`.
    AmpersandAssign,
    /// Pipe assign `|=`.
    PipeAssign,
    /// Caret assign `^=`.
    CaretAssign,
    /// Left shift assign `<<=`.
    LeftShiftAssign,
    /// Right shift assign `>>=`.
    RightShiftAssign,
    /// Increment `++`.
    Increment,
    /// Decrement `--`.
    Decrement,

    /// Equal `=`.
    Eq,
    /// Not equal `!=`.
    Ne,
    /// Less than `<`.
    Lt,
    /// Less than or equal to `<=`.
    Le,
    /// Greater than `>`.
    Gt,
    /// Greater than or equal to `>=`.
    Ge,
    /// Double equal `==`.
    EqEq,
    /// Bang equal `!=`.
    BangEq,
    /// Less less `<<`.
    LtLt,
    /// Greater greater `>>`.
    GtGt,

    /// Double ampersand `&&`.
    AmpersandAmpersand,
    /// Double pipe `||`.
    PipePipe,
    /// Bang `!`.
    Bang,

    /// Assign `=`.
    Assign,
    /// Arrow `->`.
    Arrow,

    /// Left parenthesis `(`.
    LeftParen,
    /// Right parenthesis `)`.
    RightParen,
    /// Left brace `{`.
    LeftBrace,
    /// Right brace `}`.
    RightBrace,
    /// Left bracket `[`.
    LeftBracket,
    /// Right bracket `]`.
    RightBracket,
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
    /// At symbol `@`.
    At,
    /// Hash symbol `#`.
    Hash,
    /// Dollar symbol `$`.
    Dollar,

    /// Root node.
    Root,
    /// Function node.
    Function,
    /// Struct node.
    Struct,
    /// Variable node.
    Variable,
    /// Block node.
    Block,
    /// Type alias node.
    TypeAlias,
    /// Parameter node.
    Param,
    /// Struct member node.
    StructMember,

    /// Attribute node such as `@group(0)`.
    Attribute,
    /// Type reference node.
    Type,
    /// Module-scope constant node.
    GlobalConst,
    /// Module-scope directive node such as `enable f16;`.
    Directive,

    /// `var`, `let`, or `const` statement node.
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
    /// `loop` statement node.
    LoopStatement,
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
    /// Prefix expression node.
    ExprUnary,
    /// Binary expression node.
    ExprBinary,
    /// Type constructor expression node such as `vec3<f32>`.
    ExprTypeCtor,
    /// Call expression node.
    ExprCall,
    /// Member access expression node.
    ExprMember,
    /// Index expression node.
    ExprIndex,
    /// Parenthesized expression node.
    ExprParen,
}

impl ElementType for WgslElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::Root => UniversalElementRole::Root,
            Self::Function => UniversalElementRole::Definition,
            Self::Struct => UniversalElementRole::Definition,
            Self::Variable => UniversalElementRole::Definition,
            Self::Block => UniversalElementRole::Container,
            Self::TypeAlias => UniversalElementRole::Definition,
            Self::GlobalConst => UniversalElementRole::Definition,
            Self::Attribute => UniversalElementRole::Attribute,
            Self::Type => UniversalElementRole::Typing,
            Self::VarStatement
            | Self::AssignmentStatement
            | Self::IfStatement
            | Self::ForStatement
            | Self::WhileStatement
            | Self::LoopStatement
            | Self::BreakStatement
            | Self::ContinueStatement
            | Self::ReturnStatement
            | Self::DiscardStatement
            | Self::ExpressionStatement
            | Self::Directive => UniversalElementRole::Statement,
            Self::ExprParen => UniversalElementRole::Expression,
            Self::Expr | Self::ExprUnary | Self::ExprBinary | Self::ExprTypeCtor | Self::ExprMember | Self::ExprIndex => UniversalElementRole::Expression,
            Self::ExprCall => UniversalElementRole::Call,
            _ => UniversalElementRole::None,
        }
    }
}

impl From<WgslTokenType> for WgslElementType {
    fn from(token: WgslTokenType) -> Self {
        match token {
            WgslTokenType::Whitespace => Self::Whitespace,
            WgslTokenType::Newline => Self::Newline,
            WgslTokenType::Comment => Self::Comment,
            WgslTokenType::Error => Self::Error,
            WgslTokenType::Eof => Self::Eof,
            WgslTokenType::Text => Self::Text,
            WgslTokenType::IntegerLiteral => Self::IntegerLiteral,
            WgslTokenType::FloatLiteral => Self::FloatLiteral,
            WgslTokenType::StringLiteral => Self::StringLiteral,
            WgslTokenType::BoolLiteral => Self::BoolLiteral,
            WgslTokenType::Identifier => Self::Identifier,
            WgslTokenType::BoolKw => Self::BoolKw,
            WgslTokenType::I32Kw => Self::I32Kw,
            WgslTokenType::U32Kw => Self::U32Kw,
            WgslTokenType::F32Kw => Self::F32Kw,
            WgslTokenType::F16Kw => Self::F16Kw,
            WgslTokenType::Vec2Kw => Self::Vec2Kw,
            WgslTokenType::Vec3Kw => Self::Vec3Kw,
            WgslTokenType::Vec4Kw => Self::Vec4Kw,
            WgslTokenType::Mat2x2Kw => Self::Mat2x2Kw,
            WgslTokenType::Mat2x3Kw => Self::Mat2x3Kw,
            WgslTokenType::Mat2x4Kw => Self::Mat2x4Kw,
            WgslTokenType::Mat3x2Kw => Self::Mat3x2Kw,
            WgslTokenType::Mat3x3Kw => Self::Mat3x3Kw,
            WgslTokenType::Mat3x4Kw => Self::Mat3x4Kw,
            WgslTokenType::Mat4x2Kw => Self::Mat4x2Kw,
            WgslTokenType::Mat4x3Kw => Self::Mat4x3Kw,
            WgslTokenType::Mat4x4Kw => Self::Mat4x4Kw,
            WgslTokenType::ArrayKw => Self::ArrayKw,
            WgslTokenType::PtrKw => Self::PtrKw,
            WgslTokenType::AtomicKw => Self::AtomicKw,
            WgslTokenType::SamplerKw => Self::SamplerKw,
            WgslTokenType::SamplerComparisonKw => Self::SamplerComparisonKw,
            WgslTokenType::Texture1dKw => Self::Texture1dKw,
            WgslTokenType::Texture2dKw => Self::Texture2dKw,
            WgslTokenType::Texture2dArrayKw => Self::Texture2dArrayKw,
            WgslTokenType::Texture3dKw => Self::Texture3dKw,
            WgslTokenType::TextureCubeKw => Self::TextureCubeKw,
            WgslTokenType::TextureCubeArrayKw => Self::TextureCubeArrayKw,
            WgslTokenType::TextureMultisampled2dKw => Self::TextureMultisampled2dKw,
            WgslTokenType::TextureDepth2dKw => Self::TextureDepth2dKw,
            WgslTokenType::TextureDepthCubeKw => Self::TextureDepthCubeKw,
            WgslTokenType::TextureDepthMultisampled2dKw => Self::TextureDepthMultisampled2dKw,
            WgslTokenType::TextureStorage1dKw => Self::TextureStorage1dKw,
            WgslTokenType::TextureStorage2dKw => Self::TextureStorage2dKw,
            WgslTokenType::TextureStorage2dArrayKw => Self::TextureStorage2dArrayKw,
            WgslTokenType::TextureStorage3dKw => Self::TextureStorage3dKw,
            WgslTokenType::FnKw => Self::FnKw,
            WgslTokenType::VarKw => Self::VarKw,
            WgslTokenType::LetKw => Self::LetKw,
            WgslTokenType::ConstKw => Self::ConstKw,
            WgslTokenType::IfKw => Self::IfKw,
            WgslTokenType::ElseKw => Self::ElseKw,
            WgslTokenType::SwitchKw => Self::SwitchKw,
            WgslTokenType::CaseKw => Self::CaseKw,
            WgslTokenType::DefaultKw => Self::DefaultKw,
            WgslTokenType::LoopKw => Self::LoopKw,
            WgslTokenType::ForKw => Self::ForKw,
            WgslTokenType::WhileKw => Self::WhileKw,
            WgslTokenType::BreakKw => Self::BreakKw,
            WgslTokenType::ContinueKw => Self::ContinueKw,
            WgslTokenType::ReturnKw => Self::ReturnKw,
            WgslTokenType::DiscardKw => Self::DiscardKw,
            WgslTokenType::FunctionKw => Self::FunctionKw,
            WgslTokenType::PrivateKw => Self::PrivateKw,
            WgslTokenType::WorkgroupKw => Self::WorkgroupKw,
            WgslTokenType::UniformKw => Self::UniformKw,
            WgslTokenType::StorageKw => Self::StorageKw,
            WgslTokenType::ReadKw => Self::ReadKw,
            WgslTokenType::WriteKw => Self::WriteKw,
            WgslTokenType::ReadWriteKw => Self::ReadWriteKw,
            WgslTokenType::VertexKw => Self::VertexKw,
            WgslTokenType::FragmentKw => Self::FragmentKw,
            WgslTokenType::ComputeKw => Self::ComputeKw,
            WgslTokenType::BindingKw => Self::BindingKw,
            WgslTokenType::GroupKw => Self::GroupKw,
            WgslTokenType::LocationKw => Self::LocationKw,
            WgslTokenType::BuiltinKw => Self::BuiltinKw,
            WgslTokenType::InterpolateKw => Self::InterpolateKw,
            WgslTokenType::InvariantKw => Self::InvariantKw,
            WgslTokenType::SizeKw => Self::SizeKw,
            WgslTokenType::AlignKw => Self::AlignKw,
            WgslTokenType::WorkgroupSizeKw => Self::WorkgroupSizeKw,
            WgslTokenType::PositionKw => Self::PositionKw,
            WgslTokenType::VertexIndexKw => Self::VertexIndexKw,
            WgslTokenType::InstanceIndexKw => Self::InstanceIndexKw,
            WgslTokenType::FrontFacingKw => Self::FrontFacingKw,
            WgslTokenType::FragDepthKw => Self::FragDepthKw,
            WgslTokenType::LocalInvocationIdKw => Self::LocalInvocationIdKw,
            WgslTokenType::LocalInvocationIndexKw => Self::LocalInvocationIndexKw,
            WgslTokenType::GlobalInvocationIdKw => Self::GlobalInvocationIdKw,
            WgslTokenType::WorkgroupIdKw => Self::WorkgroupIdKw,
            WgslTokenType::NumWorkgroupsKw => Self::NumWorkgroupsKw,
            WgslTokenType::SampleIndexKw => Self::SampleIndexKw,
            WgslTokenType::SampleMaskKw => Self::SampleMaskKw,
            WgslTokenType::StructKw => Self::StructKw,
            WgslTokenType::TypeKw => Self::TypeKw,
            WgslTokenType::AliasKw => Self::AliasKw,
            WgslTokenType::EnableKw => Self::EnableKw,
            WgslTokenType::RequiresKw => Self::RequiresKw,
            WgslTokenType::OverrideKw => Self::OverrideKw,
            WgslTokenType::Plus => Self::Plus,
            WgslTokenType::Minus => Self::Minus,
            WgslTokenType::Star => Self::Star,
            WgslTokenType::Slash => Self::Slash,
            WgslTokenType::Percent => Self::Percent,
            WgslTokenType::Ampersand => Self::Ampersand,
            WgslTokenType::Pipe => Self::Pipe,
            WgslTokenType::Caret => Self::Caret,
            WgslTokenType::Tilde => Self::Tilde,
            WgslTokenType::LeftShift => Self::LeftShift,
            WgslTokenType::RightShift => Self::RightShift,
            WgslTokenType::PlusAssign => Self::PlusAssign,
            WgslTokenType::MinusAssign => Self::MinusAssign,
            WgslTokenType::StarAssign => Self::StarAssign,
            WgslTokenType::SlashAssign => Self::SlashAssign,
            WgslTokenType::PercentAssign => Self::PercentAssign,
            WgslTokenType::AmpersandAssign => Self::AmpersandAssign,
            WgslTokenType::PipeAssign => Self::PipeAssign,
            WgslTokenType::CaretAssign => Self::CaretAssign,
            WgslTokenType::LeftShiftAssign => Self::LeftShiftAssign,
            WgslTokenType::RightShiftAssign => Self::RightShiftAssign,
            WgslTokenType::Increment => Self::Increment,
            WgslTokenType::Decrement => Self::Decrement,
            WgslTokenType::Eq => Self::Eq,
            WgslTokenType::Ne => Self::Ne,
            WgslTokenType::Lt => Self::Lt,
            WgslTokenType::Le => Self::Le,
            WgslTokenType::Gt => Self::Gt,
            WgslTokenType::Ge => Self::Ge,
            WgslTokenType::EqEq => Self::EqEq,
            WgslTokenType::BangEq => Self::BangEq,
            WgslTokenType::LtLt => Self::LtLt,
            WgslTokenType::GtGt => Self::GtGt,
            WgslTokenType::AmpersandAmpersand => Self::AmpersandAmpersand,
            WgslTokenType::PipePipe => Self::PipePipe,
            WgslTokenType::Bang => Self::Bang,
            WgslTokenType::Assign => Self::Assign,
            WgslTokenType::Arrow => Self::Arrow,
            WgslTokenType::LeftParen => Self::LeftParen,
            WgslTokenType::RightParen => Self::RightParen,
            WgslTokenType::LeftBrace => Self::LeftBrace,
            WgslTokenType::RightBrace => Self::RightBrace,
            WgslTokenType::LeftBracket => Self::LeftBracket,
            WgslTokenType::RightBracket => Self::RightBracket,
            WgslTokenType::Semicolon => Self::Semicolon,
            WgslTokenType::Comma => Self::Comma,
            WgslTokenType::Dot => Self::Dot,
            WgslTokenType::Colon => Self::Colon,
            WgslTokenType::Question => Self::Question,
            WgslTokenType::At => Self::At,
            WgslTokenType::Hash => Self::Hash,
            WgslTokenType::Dollar => Self::Dollar,
            WgslTokenType::Root => Self::Root,
            WgslTokenType::Function => Self::Function,
            WgslTokenType::Struct => Self::Struct,
            WgslTokenType::Variable => Self::Variable,
            WgslTokenType::Block => Self::Block,
            WgslTokenType::TypeAlias => Self::TypeAlias,
        }
    }
}
