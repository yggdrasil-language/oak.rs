#![doc = include_str!("readme.md")]

/// WGSL root node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslRoot {
    /// List of items.
    pub items: Vec<WgslItem>,
}

/// A top-level WGSL item.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum WgslItem {
    /// Function definition.
    Function(WgslFunction),
    /// Struct definition.
    Struct(WgslStruct),
    /// Module-scope variable.
    GlobalVar(WgslGlobalVar),
    /// Module-scope constant.
    Const(WgslConst),
    /// Type alias.
    TypeAlias(WgslTypeAlias),
}

/// A WGSL attribute such as `@group(0)` or `@compute`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslAttribute {
    /// Attribute name without the leading `@`.
    pub name: String,
    /// Attribute arguments.
    pub args: Vec<WgslExpression>,
}

/// A WGSL function.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslFunction {
    /// Attributes attached to the function.
    pub attributes: Vec<WgslAttribute>,
    /// Function name.
    pub name: String,
    /// Parameter list.
    pub params: Vec<WgslParam>,
    /// Return type.
    pub return_type: Option<WgslType>,
    /// Function body.
    pub body: Vec<WgslStatement>,
}

/// A WGSL function parameter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslParam {
    /// Attributes attached to the parameter.
    pub attributes: Vec<WgslAttribute>,
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub ty: WgslType,
}

/// A WGSL struct definition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslStruct {
    /// Struct name.
    pub name: String,
    /// Member list.
    pub members: Vec<WgslStructMember>,
}

/// A WGSL struct member.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslStructMember {
    /// Attributes attached to the member.
    pub attributes: Vec<WgslAttribute>,
    /// Member name.
    pub name: String,
    /// Member type.
    pub ty: WgslType,
}

/// A module-scope WGSL variable.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslGlobalVar {
    /// Attributes attached to the variable.
    pub attributes: Vec<WgslAttribute>,
    /// Address space declared through `var<...>`.
    pub address_space: Option<WgslAddressSpace>,
    /// Access mode declared through `var<...>`.
    pub access: Option<WgslAccessMode>,
    /// Variable name.
    pub name: String,
    /// Variable type.
    pub ty: Option<WgslType>,
    /// Initializer.
    pub init: Option<WgslExpression>,
}

/// A module-scope WGSL constant.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslConst {
    /// Constant name.
    pub name: String,
    /// Constant type.
    pub ty: Option<WgslType>,
    /// Constant initializer.
    pub init: WgslExpression,
}

/// A WGSL type alias.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslTypeAlias {
    /// Alias name.
    pub name: String,
    /// Aliased type.
    pub ty: WgslType,
}

/// Address space of a module-scope variable.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WgslAddressSpace {
    /// `function`.
    Function,
    /// `private`.
    Private,
    /// `workgroup`.
    Workgroup,
    /// `uniform`.
    Uniform,
    /// `storage`.
    Storage,
}

/// Access mode of a module-scope variable.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WgslAccessMode {
    /// `read`.
    Read,
    /// `write`.
    Write,
    /// `read_write`.
    ReadWrite,
}

/// A WGSL type reference.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct WgslType {
    /// Type name, for example `f32` or `array`.
    pub name: String,
    /// Type arguments, for example `[f32, 64]` for `array<f32, 64>`.
    pub args: Vec<WgslType>,
}

/// A WGSL statement.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum WgslStatement {
    /// A `var`, `let`, or `const` declaration.
    Var {
        /// Declared name.
        name: String,
        /// Declared type.
        ty: Option<WgslType>,
        /// Initializer.
        init: Option<WgslExpression>,
        /// Whether the declaration is immutable (`let` or `const`).
        is_const: bool,
    },
    /// An assignment, including compound assignments.
    Assign {
        /// Assignment target.
        target: WgslExpression,
        /// Assignment operator.
        op: WgslAssignOp,
        /// Assigned value.
        value: WgslExpression,
    },
    /// An `if` statement.
    If {
        /// Condition.
        cond: WgslExpression,
        /// Body executed when the condition holds.
        then_body: Vec<WgslStatement>,
        /// Body executed otherwise.
        else_body: Vec<WgslStatement>,
    },
    /// A `for` loop.
    For {
        /// Loop initializer.
        init: Option<Box<WgslStatement>>,
        /// Loop condition.
        cond: Option<WgslExpression>,
        /// Loop update.
        update: Option<Box<WgslStatement>>,
        /// Loop body.
        body: Vec<WgslStatement>,
    },
    /// A `while` loop.
    While {
        /// Loop condition.
        cond: WgslExpression,
        /// Loop body.
        body: Vec<WgslStatement>,
    },
    /// A `loop` statement.
    Loop {
        /// Loop body.
        body: Vec<WgslStatement>,
    },
    /// A `break` statement.
    Break,
    /// A `continue` statement.
    Continue,
    /// A `discard` statement.
    Discard,
    /// A `return` statement with an optional value.
    Return(Option<WgslExpression>),
    /// An expression used as a statement.
    Expr(WgslExpression),
    /// A nested block.
    Block(Vec<WgslStatement>),
}

/// A WGSL assignment operator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WgslAssignOp {
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

/// A WGSL expression.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum WgslExpression {
    /// An integer literal.
    IntegerLiteral(i64),
    /// A floating-point literal.
    FloatLiteral(f64),
    /// A boolean literal.
    BoolLiteral(bool),
    /// A name reference such as a variable, function, or type constructor.
    Ident(String),
    /// A prefix operation.
    Unary {
        /// Operator.
        op: WgslUnaryOp,
        /// Operand.
        expr: Box<WgslExpression>,
    },
    /// A binary operation.
    Binary {
        /// Operator.
        op: WgslBinaryOp,
        /// Left operand.
        lhs: Box<WgslExpression>,
        /// Right operand.
        rhs: Box<WgslExpression>,
    },
    /// A function or constructor call.
    Call {
        /// Callee name, possibly including type arguments such as `vec3<f32>`.
        callee: String,
        /// Call arguments.
        args: Vec<WgslExpression>,
    },
    /// A member access such as `gid.x`.
    Member {
        /// Base expression.
        base: Box<WgslExpression>,
        /// Field name.
        field: String,
    },
    /// An index access such as `out[i]`.
    Index {
        /// Base expression.
        base: Box<WgslExpression>,
        /// Index expression.
        index: Box<WgslExpression>,
    },
    /// A parenthesized expression.
    Paren(Box<WgslExpression>),
}

/// A WGSL prefix operator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WgslUnaryOp {
    /// `-`.
    Neg,
    /// `!`.
    Not,
    /// `~`.
    BitNot,
}

/// A WGSL binary operator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WgslBinaryOp {
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
