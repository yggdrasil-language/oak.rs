use crate::ast::{FunctionParam, JsxElement, JsxFragment, JsxSelfClosingElement, Modifier, Statement, TypeAnnotation, TypeParameter};
use core::range::Range;

/// Represents a TypeScript expression.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Expression {
    /// The kind of expression.
    pub kind: Box<ExpressionKind>,
    /// Source span of the expression.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl Expression {
    /// Creates a new `Expression`.
    pub fn new(kind: ExpressionKind, span: Range<usize>) -> Self {
        Self { kind: Box::new(kind), span }
    }

    /// Gets the span of the expression.
    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

/// Represents the different kinds of expressions in TypeScript.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExpressionKind {
    /// An identifier.
    Identifier(String),
    /// A numeric literal.
    NumericLiteral(f64),
    /// A string literal.
    StringLiteral(String),
    /// A BigInt literal.
    BigIntLiteral(String),
    /// A boolean literal.
    BooleanLiteral(bool),
    /// A null literal.
    NullLiteral,
    /// A regular expression literal.
    RegexLiteral(String),
    /// A template string literal.
    TemplateString(String),
    /// A unary expression.
    UnaryExpression {
        /// Unary operator.
        operator: String,
        /// Argument expression.
        argument: Box<Expression>,
    },
    /// An update expression.
    UpdateExpression {
        /// Update operator.
        operator: String,
        /// Argument expression.
        argument: Box<Expression>,
        /// Whether the operator is a prefix.
        prefix: bool,
    },
    /// A binary expression.
    BinaryExpression {
        /// Left-hand side expression.
        left: Box<Expression>,
        /// Binary operator.
        operator: String,
        /// Right-hand side expression.
        right: Box<Expression>,
    },
    /// A conditional (ternary) expression.
    ConditionalExpression {
        /// Condition expression.
        test: Box<Expression>,
        /// Expression to evaluate if the condition is true.
        consequent: Box<Expression>,
        /// Expression to evaluate if the condition is false.
        alternate: Box<Expression>,
    },
    /// A member expression.
    MemberExpression {
        /// Object expression.
        object: Box<Expression>,
        /// Property expression.
        property: Box<Expression>,
        /// Whether the property is computed (using `[]`).
        computed: bool,
        /// Whether the access is optional (using `?.`).
        optional: bool,
    },
    /// A meta property such as `import.meta`.
    MetaProperty {
        /// The meta keyword, e.g. `import`.
        meta: String,
        /// The meta property name, e.g. `meta`.
        property: String,
    },
    /// A call expression.
    CallExpression {
        /// Function to call.
        func: Box<Expression>,
        /// Arguments to the function call.
        args: Vec<Expression>,
    },
    /// A new expression.
    NewExpression {
        /// Constructor function to call.
        func: Box<Expression>,
        /// Arguments to the constructor call.
        args: Vec<Expression>,
    },
    /// An assignment expression.
    AssignmentExpression {
        /// Left-hand side expression.
        left: Box<Expression>,
        /// Assignment operator.
        operator: String,
        /// Right-hand side expression.
        right: Box<Expression>,
    },
    /// An 'as' expression (type assertion).
    AsExpression {
        /// Expression being asserted.
        expression: Box<Expression>,
        /// Type annotation being asserted to.
        type_annotation: TypeAnnotation,
    },
    /// An arrow function expression.
    ArrowFunction {
        /// Type parameters of the arrow function.
        type_params: Vec<TypeParameter>,
        /// Parameters of the arrow function.
        params: Vec<FunctionParam>,
        /// Return type of the arrow function.
        return_type: Option<TypeAnnotation>,
        /// Body of the arrow function.
        body: Box<Statement>,
        /// Whether the arrow function is async.
        async_: bool,
    },
    /// An object literal.
    ObjectLiteral {
        /// Properties within the object literal.
        properties: Vec<ObjectProperty>,
    },
    /// An array literal.
    ArrayLiteral {
        /// Elements within the array literal.
        elements: Vec<Expression>,
    },
    /// A spread element.
    SpreadElement(Box<Expression>),
    /// An await expression.
    AwaitExpression(Box<Expression>),
    /// A yield expression.
    YieldExpression(Option<Box<Expression>>),
    /// An import expression.
    ImportExpression {
        /// Module specifier being imported.
        module_specifier: Box<Expression>,
    },
    /// A function expression.
    FunctionExpression {
        /// Optional name of the function.
        name: Option<String>,
        /// Type parameters of the function.
        type_params: Vec<TypeParameter>,
        /// Parameters of the function.
        params: Vec<FunctionParam>,
        /// Return type of the function.
        return_type: Option<TypeAnnotation>,
        /// Body of the function.
        body: Vec<Statement>,
        /// Whether the function is async.
        async_: bool,
        /// Whether the function is a generator.
        generator: bool,
    },
    /// A tagged template expression.
    TaggedTemplateExpression {
        /// Tag function expression.
        tag: Box<Expression>,
        /// Template literal expression.
        template: Box<Expression>,
    },
    /// A type assertion expression (e.g., `<Type>expression`).
    TypeAssertionExpression {
        /// Expression being asserted.
        expression: Box<Expression>,
        /// Type annotation being asserted to.
        type_annotation: TypeAnnotation,
    },
    /// A non-null expression (e.g., `expression!`).
    NonNullExpression(Box<Expression>),
    /// A JSX element.
    JsxElement(Box<JsxElement>),
    /// A JSX fragment.
    JsxFragment(Box<JsxFragment>),
    /// A self-closing JSX element.
    JsxSelfClosingElement(Box<JsxSelfClosingElement>),
}

/// Represents a property in an object literal.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectProperty {
    /// A property declaration.
    Property {
        /// Name of the property.
        name: String,
        /// Value of the property.
        value: Expression,
        /// Whether the property is shorthand.
        shorthand: bool,
        /// Source span of the property.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A method or accessor member (`{ m() {} }`, `{ get x() {} }`).
    Method {
        /// Name of the member; a computed key keeps its raw inner text.
        name: String,
        /// Parameters of the method.
        params: Vec<FunctionParam>,
        /// Return type annotation of the method.
        return_type: Option<TypeAnnotation>,
        /// Body statements of the method.
        body: Vec<Statement>,
        /// Keyword modifiers written before the name (`async`, `get`, `set`).
        modifiers: Vec<Modifier>,
        /// Whether the member is a generator (`*`).
        generator: bool,
        /// Source span of the member.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A spread property.
    Spread(Expression),
}

/// Represents a literal type.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LiteralType {
    /// A string literal type.
    String(String),
    /// A numeric literal type.
    Number(f64),
    /// A boolean literal type.
    Boolean(bool),
    /// A BigInt literal type.
    BigInt(String),
}
