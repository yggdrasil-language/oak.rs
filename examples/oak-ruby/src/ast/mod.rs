#![doc = include_str!("readme.md")]
use core::range::Range;

/// Ruby AST root node
pub type RubyAst = RubyRoot;

/// Program node
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RubyRoot {
    /// List of statements
    pub statements: Vec<StatementNode>,
    /// Source code span
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

/// Ruby statement node
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatementNode {
    /// Expression statement
    Expression(ExpressionNode),
    /// Method definition
    MethodDef {
        /// Method name
        name: String,
        /// Parameter list
        params: Vec<String>,
        /// Method body
        body: Vec<StatementNode>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Class definition
    ClassDef {
        /// Class name
        name: String,
        /// Superclass name
        superclass: Option<String>,
        /// Class body
        body: Vec<StatementNode>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Assignment statement
    Assignment {
        /// Assignment target
        target: String,
        /// Assignment value
        value: ExpressionNode,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Conditional statement
    If {
        /// Condition expression
        condition: ExpressionNode,
        /// then branch
        then_body: Vec<StatementNode>,
        /// else branch
        else_body: Option<Vec<StatementNode>>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Loop statement
    While {
        /// Condition expression
        condition: ExpressionNode,
        /// Loop body
        body: Vec<StatementNode>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Until loop statement
    Until {
        /// Condition expression
        condition: ExpressionNode,
        /// Loop body
        body: Vec<StatementNode>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// `for x in expr`（builder 可再降成 while）
    For {
        /// 循环变量
        var: String,
        /// 可枚举表达式
        iterable: ExpressionNode,
        /// 循环体
        body: Vec<StatementNode>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Case statement
    Case {
        /// Value expression
        value: ExpressionNode,
        /// When clauses
        when_clauses: Vec<(ExpressionNode, Vec<StatementNode>)>,
        /// Else clause
        else_clause: Option<Vec<StatementNode>>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Return statement
    Return {
        /// Return value
        value: Option<ExpressionNode>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Next statement
    Next {
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Redo statement
    Redo {
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// `break`
    Break {
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

/// Ruby expression node
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExpressionNode {
    /// Identifier
    Identifier {
        /// Identifier name
        name: String,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Literal
    Literal(LiteralNode),
    /// Method call
    MethodCall {
        /// Receiver
        receiver: Option<Box<ExpressionNode>>,
        /// Method name
        method: String,
        /// Argument list
        args: Vec<ExpressionNode>,
        /// 块参数（`{|x|` / `do |x|`）
        block_params: Vec<String>,
        /// 块体；无块时为 `None`
        block_body: Option<Vec<StatementNode>>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Binary operation
    BinaryOp {
        /// Left operand
        left: Box<ExpressionNode>,
        /// Operator
        operator: String,
        /// Right operand
        right: Box<ExpressionNode>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Unary operation
    UnaryOp {
        /// Operator
        operator: String,
        /// Operand
        operand: Box<ExpressionNode>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Array
    Array {
        /// Array element list
        elements: Vec<ExpressionNode>,
        /// Source code span
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Hash
    Hash {
        /// The key-value pairs in the hash.
        pairs: Vec<(ExpressionNode, ExpressionNode)>,
        /// The source span of the hash.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

/// Literal node
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LiteralNode {
    /// Integer literal
    Integer {
        /// The integer value.
        value: i64,
        /// The source span of the integer.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Float literal
    Float {
        /// The floating-point value.
        value: f64,
        /// The source span of the float.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// String literal
    String {
        /// The string value.
        value: String,
        /// The source span of the string.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Symbol literal
    Symbol {
        /// The symbol value.
        value: String,
        /// The source span of the symbol.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Boolean literal
    Boolean {
        /// The boolean value.
        value: bool,
        /// The source span of the boolean.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// nil
    Nil {
        /// The source span of the nil literal.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

/// Ruby AST visitor trait
pub trait RubyAstVisitor {
    /// Visits the root program node.
    fn visit_program(&mut self, node: &RubyRoot);
    /// Visits a statement node.
    fn visit_statement(&mut self, stmt: &StatementNode);
    /// Visits an expression node.
    fn visit_expression(&mut self, expr: &ExpressionNode);
    /// Visits a literal node.
    fn visit_literal(&mut self, literal: &LiteralNode);
}
