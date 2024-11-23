#![doc = include_str!("readme.md")]

/// Root node of the HLSL AST.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslRoot {
    /// List of module-level items.
    pub items: Vec<HlslItem>,
}

/// A module-level HLSL item.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum HlslItem {
    /// Function definition, including the shader entry points.
    Function(HlslFunction),
    /// Struct definition.
    Struct(HlslStruct),
    /// Module-scope variable, resource, or buffer declaration.
    GlobalVar(HlslGlobalVar),
    /// Module-scope `const` declaration.
    Const(HlslConst),
    /// Type alias declared through `typedef` or `using`.
    TypeAlias(HlslTypeAlias),
}

/// An HLSL attribute such as `[numthreads(64, 1, 1)]` or `[unroll]`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslAttribute {
    /// Attribute name without the surrounding brackets.
    pub name: String,
    /// Attribute arguments.
    pub args: Vec<HlslExpression>,
}

/// An HLSL function.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslFunction {
    /// Attributes attached to the function.
    pub attributes: Vec<HlslAttribute>,
    /// Function name.
    pub name: String,
    /// Parameter list.
    pub params: Vec<HlslParam>,
    /// Return type.
    pub return_type: Option<HlslType>,
    /// Return semantic such as `SV_Target`.
    pub semantic: Option<String>,
    /// Function body.
    pub body: Vec<HlslStatement>,
}

/// An HLSL function parameter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslParam {
    /// Prefix qualifiers such as `in`, `out`, `inout`, or `const`.
    pub qualifiers: Vec<HlslQualifier>,
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub ty: HlslType,
    /// Parameter semantic such as `SV_Position`.
    pub semantic: Option<String>,
}

/// An HLSL struct definition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslStruct {
    /// Struct name.
    pub name: String,
    /// Member list.
    pub members: Vec<HlslStructMember>,
}

/// An HLSL struct member.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslStructMember {
    /// Attributes attached to the member.
    pub attributes: Vec<HlslAttribute>,
    /// Prefix qualifiers such as `nointerpolation`.
    pub qualifiers: Vec<HlslQualifier>,
    /// Member name.
    pub name: String,
    /// Member type.
    pub ty: HlslType,
    /// Member semantic such as `TEXCOORD0`.
    pub semantic: Option<String>,
}

/// A module-scope HLSL variable or resource declaration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslGlobalVar {
    /// Attributes attached to the variable.
    pub attributes: Vec<HlslAttribute>,
    /// Prefix qualifiers such as `uniform`, `static`, or `groupshared`.
    pub qualifiers: Vec<HlslQualifier>,
    /// Resource register declared through `register(...)`.
    pub register: Option<HlslRegister>,
    /// Variable semantic, if any.
    pub semantic: Option<String>,
    /// Variable name. Layout-only declarations have an empty name.
    pub name: String,
    /// Variable type.
    pub ty: Option<HlslType>,
    /// Initializer.
    pub init: Option<HlslExpression>,
}

/// A module-scope HLSL `const` declaration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslConst {
    /// Constant name.
    pub name: String,
    /// Constant type.
    pub ty: Option<HlslType>,
    /// Constant initializer.
    pub init: HlslExpression,
}

/// An HLSL type alias declared through `typedef` or `using`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslTypeAlias {
    /// Alias name.
    pub name: String,
    /// Aliased type.
    pub ty: HlslType,
}

/// A resource register such as `register(b0)` or `register(t0)`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslRegister {
    /// Register class, for example `b`, `t`, `u`, or `s`.
    pub class: Option<String>,
    /// Register slot number.
    pub index: Option<u32>,
}

/// A qualifier that may prefix a declaration or a parameter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HlslQualifier {
    /// `uniform`.
    Uniform,
    /// `const`.
    Const,
    /// `static`.
    Static,
    /// `extern`.
    Extern,
    /// `inline`.
    Inline,
    /// `in`.
    In,
    /// `out`.
    Out,
    /// `inout`.
    InOut,
    /// `shared`.
    Shared,
    /// `groupshared`.
    Groupshared,
    /// `varying`.
    Varying,
    /// `volatile`.
    Volatile,
    /// `precise`.
    Precise,
    /// `nointerpolation`.
    NoInterpolation,
    /// `linear`.
    Linear,
    /// `centroid`.
    Centroid,
    /// `sample`.
    Sample,
    /// `noperspective`.
    NoPerspective,
    /// `target`.
    Target,
}

/// An HLSL type reference.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct HlslType {
    /// Type name, for example `float4`, `Texture2D`, or `VS_OUTPUT`.
    pub name: String,
    /// Type arguments, for example `<float4>` in `RWStructuredBuffer<float4>`.
    pub args: Vec<HlslType>,
}

/// An HLSL statement.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum HlslStatement {
    /// A local variable or constant declaration.
    Var {
        /// Prefix qualifiers such as `const`, `static`, or `groupshared`.
        qualifiers: Vec<HlslQualifier>,
        /// Declared name.
        name: String,
        /// Declared type.
        ty: Option<HlslType>,
        /// Initializer.
        init: Option<HlslExpression>,
        /// Whether the declaration carries the `const` qualifier.
        is_const: bool,
    },
    /// An assignment, including compound assignments.
    Assign {
        /// Assignment target.
        target: HlslExpression,
        /// Assignment operator.
        op: HlslAssignOp,
        /// Assigned value.
        value: HlslExpression,
    },
    /// An `if` statement.
    If {
        /// Condition.
        cond: HlslExpression,
        /// Body executed when the condition holds.
        then_body: Vec<HlslStatement>,
        /// Body executed otherwise.
        else_body: Vec<HlslStatement>,
    },
    /// A `for` loop.
    For {
        /// Loop initializer.
        init: Option<Box<HlslStatement>>,
        /// Loop condition.
        cond: Option<HlslExpression>,
        /// Loop update.
        update: Option<Box<HlslStatement>>,
        /// Loop body.
        body: Vec<HlslStatement>,
    },
    /// A `while` loop.
    While {
        /// Loop condition.
        cond: HlslExpression,
        /// Loop body.
        body: Vec<HlslStatement>,
    },
    /// A `do ... while` loop.
    DoWhile {
        /// Loop body.
        body: Vec<HlslStatement>,
        /// Loop condition.
        cond: HlslExpression,
    },
    /// A `break` statement.
    Break,
    /// A `continue` statement.
    Continue,
    /// A `discard` statement.
    Discard,
    /// A `return` statement with an optional value.
    Return(Option<HlslExpression>),
    /// An expression used as a statement.
    Expr(HlslExpression),
    /// A nested block.
    Block(Vec<HlslStatement>),
}

/// An HLSL assignment operator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HlslAssignOp {
    /// `=`.
    Assign,
    /// `+=`.
    AddAssign,
    /// `-=`.
    SubAssign,
    /// `*=`.
    MulAssign,
    /// `/=`.
    DivAssign,
    /// `%=`.
    ModAssign,
    /// `&=`.
    AndAssign,
    /// `|=`.
    OrAssign,
    /// `^=`.
    XorAssign,
    /// `<<=`.
    ShlAssign,
    /// `>>=`.
    ShrAssign,
}

/// An HLSL expression.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum HlslExpression {
    /// An integer literal.
    IntegerLiteral(i64),
    /// A floating-point literal.
    FloatLiteral(f64),
    /// A boolean literal.
    BoolLiteral(bool),
    /// A name reference such as a variable, function, or type constructor.
    Ident(String),
    /// A prefix or postfix operation.
    Unary {
        /// Operator.
        op: HlslUnaryOp,
        /// Operand.
        expr: Box<HlslExpression>,
    },
    /// A binary operation.
    Binary {
        /// Operator.
        op: HlslBinaryOp,
        /// Left operand.
        lhs: Box<HlslExpression>,
        /// Right operand.
        rhs: Box<HlslExpression>,
    },
    /// A function or constructor call.
    Call {
        /// Callee text, which may be a member path such as `texture.Sample`.
        callee: String,
        /// Call arguments.
        args: Vec<HlslExpression>,
    },
    /// A member access such as `gid.x`.
    Member {
        /// Base expression.
        base: Box<HlslExpression>,
        /// Member name.
        field: String,
    },
    /// An index access such as `values[i]`.
    Index {
        /// Base expression.
        base: Box<HlslExpression>,
        /// Index expression.
        index: Box<HlslExpression>,
    },
    /// A parenthesized expression.
    Paren(Box<HlslExpression>),
}

/// An HLSL prefix or postfix operator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HlslUnaryOp {
    /// `-`.
    Neg,
    /// `!`.
    Not,
    /// `~`.
    BitNot,
    /// `++` before the operand.
    PreIncrement,
    /// `--` before the operand.
    PreDecrement,
    /// `++` after the operand.
    PostIncrement,
    /// `--` after the operand.
    PostDecrement,
}

/// An HLSL binary operator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HlslBinaryOp {
    /// `+`.
    Add,
    /// `-`.
    Sub,
    /// `*`.
    Mul,
    /// `/`.
    Div,
    /// `%`.
    Mod,
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
    /// `&`.
    BitAnd,
    /// `|`.
    BitOr,
    /// `^`.
    BitXor,
    /// `<<`.
    Shl,
    /// `>>`.
    Shr,
}
