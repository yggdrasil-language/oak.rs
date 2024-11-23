#![doc = include_str!("readme.md")]

/// Root node of the GLSL AST.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslRoot {
    /// List of module-level items.
    pub items: Vec<GlslItem>,
}

/// A module-level GLSL item.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum GlslItem {
    /// Function definition, including the shader entry points.
    Function(GlslFunction),
    /// Struct definition.
    Struct(GlslStruct),
    /// Module-scope variable, interface block, or layout declaration.
    GlobalVar(GlslGlobalVar),
    /// Module-scope `const` declaration.
    Const(GlslConst),
    /// Type alias.
    ///
    /// GLSL has no alias declaration of its own, so the variant exists to keep
    /// the item set aligned with the other shader frontends and is never
    /// produced by the parser.
    TypeAlias(GlslTypeAlias),
}

/// A `layout(...)` qualifier such as `binding = 0` or `std430`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslLayoutQualifier {
    /// Qualifier name, for example `local_size_x`, `binding`, or `std430`.
    pub name: String,
    /// Optional value, for example `0` in `binding = 0`.
    pub value: Option<GlslExpression>,
}

/// A GLSL function.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslFunction {
    /// `layout(...)` qualifiers attached to the function.
    pub layout: Vec<GlslLayoutQualifier>,
    /// Function name.
    pub name: String,
    /// Parameter list.
    pub params: Vec<GlslParam>,
    /// Return type.
    pub return_type: Option<GlslType>,
    /// Function body.
    pub body: Vec<GlslStatement>,
}

/// A GLSL function parameter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslParam {
    /// Prefix qualifiers such as `in`, `out`, or `const`.
    pub qualifiers: Vec<GlslQualifier>,
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub ty: GlslType,
}

/// A GLSL struct definition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslStruct {
    /// Struct name.
    pub name: String,
    /// Member list.
    pub members: Vec<GlslStructMember>,
}

/// A GLSL struct member or interface block member.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslStructMember {
    /// `layout(...)` qualifiers attached to the member.
    pub layout: Vec<GlslLayoutQualifier>,
    /// Prefix qualifiers such as `flat` or `readonly`.
    pub qualifiers: Vec<GlslQualifier>,
    /// Member name.
    pub name: String,
    /// Member type.
    pub ty: GlslType,
}

/// A module-scope GLSL declaration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslGlobalVar {
    /// `layout(...)` qualifiers attached to the declaration.
    pub layout: Vec<GlslLayoutQualifier>,
    /// Prefix qualifiers such as `uniform`, `in`, `out`, or `buffer`.
    pub qualifiers: Vec<GlslQualifier>,
    /// Declared name. Declarations without a name, such as
    /// `layout(local_size_x = 64) in;`, use an empty name.
    pub name: String,
    /// Declared type.
    pub ty: Option<GlslType>,
    /// Members of an interface block such as `buffer Data { ... };`.
    pub members: Vec<GlslStructMember>,
    /// Initializer.
    pub init: Option<GlslExpression>,
}

/// A module-scope GLSL `const` declaration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslConst {
    /// Constant name.
    pub name: String,
    /// Constant type.
    pub ty: Option<GlslType>,
    /// Constant initializer.
    pub init: GlslExpression,
}

/// A GLSL type alias.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslTypeAlias {
    /// Alias name.
    pub name: String,
    /// Aliased type.
    pub ty: GlslType,
}

/// A qualifier that may prefix a declaration or a parameter.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GlslQualifier {
    /// `const`.
    Const,
    /// `uniform`.
    Uniform,
    /// `in`.
    In,
    /// `out`.
    Out,
    /// `inout`.
    InOut,
    /// `buffer`.
    Buffer,
    /// `shared`.
    Shared,
    /// `readonly`.
    Readonly,
    /// `writeonly`.
    Writeonly,
    /// `coherent`.
    Coherent,
    /// `volatile`.
    Volatile,
    /// `restrict`.
    Restrict,
    /// `flat`.
    Flat,
    /// `smooth`.
    Smooth,
    /// `noperspective`.
    NoPerspective,
    /// `centroid`.
    Centroid,
    /// `sample`.
    Sample,
    /// `invariant`.
    Invariant,
    /// `patch`.
    Patch,
    /// `precision`.
    Precision,
    /// `highp`.
    Highp,
    /// `mediump`.
    Mediump,
    /// `lowp`.
    Lowp,
}

/// A GLSL type reference.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GlslType {
    /// Type name, for example `vec3`, `mat4`, or a user-defined struct name.
    pub name: String,
    /// Type arguments.
    pub args: Vec<GlslType>,
}

/// A GLSL statement.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum GlslStatement {
    /// A local variable or constant declaration.
    Var {
        /// Prefix qualifiers such as `const`.
        qualifiers: Vec<GlslQualifier>,
        /// Declared name.
        name: String,
        /// Declared type.
        ty: Option<GlslType>,
        /// Initializer.
        init: Option<GlslExpression>,
        /// Whether the declaration carries the `const` qualifier.
        is_const: bool,
    },
    /// An assignment, including compound assignments.
    Assign {
        /// Assignment target.
        target: GlslExpression,
        /// Assignment operator.
        op: GlslAssignOp,
        /// Assigned value.
        value: GlslExpression,
    },
    /// An `if` statement.
    If {
        /// Condition.
        cond: GlslExpression,
        /// Body executed when the condition holds.
        then_body: Vec<GlslStatement>,
        /// Body executed otherwise.
        else_body: Vec<GlslStatement>,
    },
    /// A `for` loop.
    For {
        /// Loop initializer.
        init: Option<Box<GlslStatement>>,
        /// Loop condition.
        cond: Option<GlslExpression>,
        /// Loop update.
        update: Option<Box<GlslStatement>>,
        /// Loop body.
        body: Vec<GlslStatement>,
    },
    /// A `while` loop.
    While {
        /// Loop condition.
        cond: GlslExpression,
        /// Loop body.
        body: Vec<GlslStatement>,
    },
    /// A `do ... while` loop.
    DoWhile {
        /// Loop body.
        body: Vec<GlslStatement>,
        /// Loop condition.
        cond: GlslExpression,
    },
    /// A `break` statement.
    Break,
    /// A `continue` statement.
    Continue,
    /// A `discard` statement.
    Discard,
    /// A `return` statement with an optional value.
    Return(Option<GlslExpression>),
    /// An expression used as a statement.
    Expr(GlslExpression),
    /// A nested block.
    Block(Vec<GlslStatement>),
}

/// A GLSL assignment operator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlslAssignOp {
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

/// A GLSL expression.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub enum GlslExpression {
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
        op: GlslUnaryOp,
        /// Operand.
        expr: Box<GlslExpression>,
    },
    /// A binary operation.
    Binary {
        /// Operator.
        op: GlslBinaryOp,
        /// Left operand.
        lhs: Box<GlslExpression>,
        /// Right operand.
        rhs: Box<GlslExpression>,
    },
    /// A function or constructor call.
    Call {
        /// Callee text, which may be a member path such as `texture.Sample`.
        callee: String,
        /// Call arguments.
        args: Vec<GlslExpression>,
    },
    /// A member access such as `position.x`.
    Member {
        /// Base expression.
        base: Box<GlslExpression>,
        /// Member name.
        field: String,
    },
    /// An index access such as `values[i]`.
    Index {
        /// Base expression.
        base: Box<GlslExpression>,
        /// Index expression.
        index: Box<GlslExpression>,
    },
    /// A parenthesized expression.
    Paren(Box<GlslExpression>),
}

/// A GLSL prefix or postfix operator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlslUnaryOp {
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

/// A GLSL binary operator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlslBinaryOp {
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
