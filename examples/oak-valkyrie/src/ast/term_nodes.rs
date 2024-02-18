//! Term nodes for the Valkyrie language AST.
//!
//! This module defines expressions and control flow nodes for the Valkyrie language, including:
//! - Arithmetic expressions (binary and unary operations)
//! - Literals (strings, booleans)
//! - Function calls and field access
//! - Control flow (if, match, loop)
//! - Control transfer (return, break, continue)
//! - Error handling (raise, catch, resume)
//! - Blocks and lambda expressions
//! - Object and class expressions

use super::{Identifier, LoopKind, MatchArm, NamePath, Param, Pattern, Span, StringLiteral, TypeExpression};
use crate::{ast::structure_nodes::AnonymousClass, lexer::token_type::ValkyrieTokenType};

/// An expression in the Valkyrie language.
///
/// Expressions represent values and computations in the language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TermExpression {
    /// A binary operation expression.
    ///
    /// # V Language Example
    /// ```v
    /// let result = 10 + 5  // Addition
    /// let result = 10 - 5  // Subtraction
    /// let result = 10 * 5  // Multiplication
    /// let result = 10 / 5  // Division
    /// let result = 10 % 5  // Modulo
    /// let result = 10 == 5 // Equality
    /// let result = 10 != 5 // Inequality
    /// let result = 10 > 5  // Greater than
    /// let result = 10 < 5  // Less than
    /// let result = 10 >= 5 // Greater than or equal
    /// let result = 10 <= 5 // Less than or equal
    /// let result = true && false // Logical AND
    /// let result = true || false // Logical OR
    /// ```
    Binary(Box<TermBinaryNode>),
    /// A unary operation expression.
    ///
    /// # V Language Example
    /// ```v
    /// let result = -5      // Negation
    /// let result = !true   // Logical NOT
    /// let result = *ptr    // Dereference
    /// let result = &value  // Reference
    /// ```
    Unary(Box<TermUnaryNode>),

    /// A string literal expression.
    ///
    /// # V Language Example
    /// ```v
    /// let name = "John"
    /// let message = "Hello, world!"
    /// ```
    StringLiteral(StringLiteral),
    /// An integer literal preserving its lexical text and source span.
    IntegerLiteral {
        value: String,
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A floating-point literal preserving its lexical text and source span.
    FloatLiteral {
        value: String,
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A boolean literal expression.
    ///
    /// # V Language Example
    /// ```v
    /// let is_true = true
    /// let is_false = false
    /// ```
    Bool {
        /// The boolean value.
        value: bool,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// Term 层显式泛型应用（turbofish），如 `identity::<i32>` 或 `T::<i64>()`。
    Turbofish {
        /// 被应用泛型的表达式。
        expr: Box<TermExpression>,
        /// 类型实参。
        arguments: Vec<TypeExpression>,
        /// 源码区间。
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A function call expression.
    ///
    /// # V Language Example
    /// ```v
    /// let result = add(10, 5)
    /// let result = Math::sqrt(25)
    /// ```
    ApplyCall {
        callee: Box<TermExpression>,
        /// The call arguments.
        args: Vec<TermExpression>,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A field access expression.
    ///
    /// # V Language Example
    /// ```v
    /// let name = person.name
    /// let length = array.length
    /// ```
    DotCall {
        /// The receiver expression.
        receiver: Box<TermExpression>,
        /// The field name.
        field: Identifier,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// An index expression.
    ///
    /// # V Language Example
    /// ```v
    /// let element = array[0]
    /// let value = matrix[2][3]
    /// ```
    Index {
        /// The receiver expression.
        receiver: Box<TermExpression>,
        /// The index expression.
        index: Box<TermExpression>,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// An offset expression (pointer arithmetic).
    ///
    /// # V Language Example
    /// ```v
    /// let next = ptr + 1
    /// let prev = ptr - 1
    /// ```
    Offset {
        /// The receiver expression.
        receiver: Box<TermExpression>,
        /// The offset expression.
        offset: Box<TermExpression>,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A parenthesized expression.
    ///
    /// # V Language Example
    /// ```v
    /// let result = (10 + 5) * 2
    /// let result = (a && b) || (c && d)
    /// ```
    Paren {
        /// The inner expression.
        expr: Box<TermExpression>,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A block expression.
    ///
    /// # V Language Example
    /// ```v
    /// let result = {
    ///     let x = 10
    ///     let y = 20
    ///     x + y
    /// }
    /// ```
    Block(Block),
    /// A lambda expression.
    ///
    /// # V Language Example
    /// ```v
    /// let add = micro(x, y) { x + y }
    /// let double = micro(x) { x * 2 }
    /// let greet = micro(name: String) -> String { "Hello, " + name }
    /// ```
    Micro(AnonymousMicro),
    /// An object expression.
    ///
    /// Creates a new object instance with specified field values.
    ///
    /// # V Language Example
    /// ```v
    /// let p = Point { x: 10, y: 20 }
    /// let shorthand = Point { x, y }  // shorthand syntax
    /// ```
    Object {
        /// The callee expression.
        callee: Box<TermExpression>,
        /// The field-value pairs. None for shorthand syntax.
        fields: Vec<(Identifier, Option<TermExpression>)>,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// Anonymous class expression.
    ///
    /// # V Language Example
    /// ```v
    /// let obj = class { x: 10, y: 20 }
    /// let impl_trait = class: Trait { ... }
    /// ```
    AnonymousClass(Box<AnonymousClass>),
    /// An if expression.
    ///
    /// # V Language Example
    /// ```v
    /// let result = if x > 0 {
    ///     "Positive"
    /// } else {
    ///     "Non-positive"
    /// }
    ///
    /// // With pattern matching
    /// let result = if let Some(value) = optional {
    ///     value
    /// } else {
    ///     0
    /// }
    /// ```
    If {
        /// Optional pattern for pattern-matching the condition.
        pattern: Option<Pattern>,
        /// The condition expression.
        condition: Box<TermExpression>,
        /// The then branch block.
        then_branch: Block,
        /// The optional else branch block.
        else_branch: Option<Block>,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A match expression.
    ///
    /// # V Language Example
    /// ```v
    /// let result = match value {
    ///     0 => "Zero",
    ///     1 => "One",
    ///     2 => "Two",
    ///     _ => "Other"
    /// }
    ///
    /// // With patterns
    /// let result = match optional {
    ///     Some(x) => "Found: " + x,
    ///     None => "Not found"
    /// }
    /// ```
    Match {
        /// The expression being matched.
        scrutinee: Box<TermExpression>,
        /// The match arms.
        arms: Vec<MatchArm>,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A loop expression.
    ///
    /// # V Language Example
    /// ```v
    /// // Infinite loop
    /// loop {
    ///     println("Hello")
    /// }
    ///
    /// // Loop with condition
    /// let mut i = 0
    /// loop while i < 10 {
    ///     println(i)
    ///     i += 1
    /// }
    ///
    /// // Loop with pattern
    /// loop item in items {
    ///     println(item)
    /// }
    ///
    /// // Labeled loop
    /// 'outer: loop {
    ///     loop {
    ///         break 'outer
    ///     }
    /// }
    /// ```
    Loop {
        /// The loop keyword kind.
        kind: LoopKind,
        /// Optional label for the loop.
        label: Option<String>,
        /// Optional pattern for loop variable binding.
        pattern: Option<Pattern>,
        /// Optional condition for conditional loops.
        condition: Option<Box<TermExpression>>,
        /// The loop body.
        body: Block,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A return expression.
    ///
    /// # V Language Example
    /// ```v
    /// micro add(x: i32, y: i32) -> i32 {
    ///     return x + y
    /// }
    ///
    /// micro greet() {
    ///     println("Hello")
    ///     return  // No value
    /// }
    /// ```
    Return(Box<Return>),
    /// A break expression.
    ///
    /// # V Language Example
    /// ```v
    /// loop i in 0..10 {
    ///     if i == 5 {
    ///         break
    ///     }
    ///     println(i)
    /// }
    ///
    /// // Break with value
    /// let result = loop {
    ///     let value = compute()
    ///     if value > 100 {
    ///         break value
    ///     }
    /// }
    ///
    /// // Break from labeled loop
    /// 'outer: loop {
    ///     loop {
    ///         break 'outer
    ///     }
    /// }
    /// ```
    Break(Box<Break>),
    /// A continue expression.
    ///
    /// # V Language Example
    /// ```v
    /// loop i in 0..10 {
    ///     if i % 2 == 0 {
    ///         continue
    ///     }
    ///     println(i)  // Only prints odd numbers
    /// }
    ///
    /// // Continue labeled loop
    /// 'outer: loop i in 0..10 {
    ///     loop j in 0..10 {
    ///         if j == 5 {
    ///             continue 'outer
    ///         }
    ///     }
    /// }
    /// ```
    Continue(Box<Continue>),
    /// A raise (throw) expression.
    ///
    /// # V Language Example
    /// ```v
    /// micro divide(a: f64, b: f64) -> f64 {
    ///     if b == 0.0 {
    ///         raise "Division by zero"
    ///     }
    ///     return a / b
    /// }
    /// ```
    Raise(Box<Raise>),
    /// A resume expression.
    ///
    /// Resumes execution from an effect handler with a value.
    /// Only valid inside a catch block.
    ///
    /// # V Language Example
    /// ```v
    /// catch process() {
    ///     case Read { prompt }: resume "input data"
    /// }
    /// ```
    Resume(Box<Resume>),
    /// A yield expression.
    ///
    /// # V Language Example
    /// ```v
    /// micro generator() {
    ///     yield 1
    ///     yield 2
    ///     yield 3
    /// }
    ///
    /// // Yield from
    /// micro generator() {
    ///     yield from another_generator()
    /// }
    /// ```
    Yield {
        /// The optional value to yield.
        expr: Option<Box<TermExpression>>,
        /// Whether this is a yield from expression.
        yield_from: bool,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A catch (try-catch) expression.
    ///
    /// # V Language Example
    /// ```v
    /// let result = catch {
    ///     risky_operation()
    /// } {
    ///     case Error { message }: "Error: " + message
    ///     case _: "Unknown error"
    /// }
    /// ```
    Catch {
        /// The expression to try.
        expr: Box<TermExpression>,
        /// The catch arms.
        arms: Vec<MatchArm>,
        /// The source code span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// With expression for functional record updates.
    ///
    /// Creates a new record by copying an existing one and updating specified fields.
    ///
    /// # V Language Example
    /// ```v
    /// let p2 = p1.with { x: 20.0, y: 30.0 }
    /// let updated = config.with { timeout: 60 }
    /// ```
    With {
        /// The base expression to copy from.
        base: Box<TermExpression>,
        /// Field updates to apply.
        updates: Vec<(Identifier, TermExpression)>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// Super call expression for constructor chaining.
    ///
    /// Represents a call to a parent class constructor within a subclass constructor.
    ///
    /// # V Language Example
    /// ```v
    /// class Derived(Base) {
    ///     initiate(mut self, x: i32, y: i32) {
    ///         super.initiate(x)  // Call parent constructor
    ///         self.y = y
    ///     }
    /// }
    /// ```
    SuperCall {
        /// Optional parent alias for renamed inheritance.
        ///
        /// In renamed inheritance, specifies which parent to call:
        /// # V Language Example
        /// ```v
        /// class Child(primary: ParentA, secondary: ParentB) {
        ///     initiate(mut self) {
        ///         super.primary.initiate()  // alias: "primary"
        ///     }
        /// }
        /// ```
        parent_alias: Option<Identifier>,
        /// The method name to call (usually "initiate").
        method: Identifier,
        /// Arguments passed to the parent constructor.
        args: Vec<TermExpression>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// A name path expression (e.g., `std::collections::HashMap`).
    ///
    /// # V Language Example
    /// ```v
    /// let map = std::collections::HashMap::new()
    /// let value = SomeModule::some_function()
    /// ```
    NamePath(Box<NamePath>),
}

/// A unary operation node.
///
/// Represents operations like negation, logical NOT, dereference, and reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TermUnaryNode {
    /// The unary operator.
    pub operator: ValkyrieTokenType,
    /// The operand expression.
    pub base: TermExpression,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A binary operation node.
///
/// Represents operations like addition, subtraction, multiplication, division, and logical operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TermBinaryNode {
    /// The binary operator.
    pub operator: ValkyrieTokenType,
    /// The left operand.
    pub lhs: TermExpression,
    /// The right operand.
    pub rhs: TermExpression,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A return expression node.
///
/// Represents a return statement with an optional value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Return {
    /// The optional return value expression.
    pub base: Option<TermExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A break expression node.
///
/// Represents a break statement with an optional label and value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Break {
    /// Optional label of the loop to break from.
    pub label: Option<String>,
    /// Optional value to break with.
    pub base: Option<TermExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A resume expression node.
///
/// Represents a resume statement in a catch block.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Resume {
    /// The value to resume with.
    pub base: Option<TermExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A raise expression node.
///
/// Represents a raise (throw) statement for error handling.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Raise {
    /// The expression to raise.
    pub base: Option<TermExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A continue expression node.
///
/// Represents a continue statement with an optional label.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Continue {
    /// Optional label of the loop to continue.
    pub label: Option<String>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A block of statements.
///
/// Represents a sequence of statements enclosed in braces.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    /// The statements in the block.
    pub statements: Vec<super::Statement>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A lambda expression.
///
/// Represents an anonymous function with parameters and a body.
///
/// # V Language Example
/// ```v
/// let add = micro(x, y) { x + y }
/// let double = micro(x: i32) -> i32 { x * 2 }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnonymousMicro {
    /// The lambda parameters.
    pub params: Vec<Param>,
    /// Optional return type annotation.
    pub return_type: Option<TypeExpression>,
    /// The lambda body.
    pub body: Block,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

impl TermExpression {
    /// Returns the source code span of the expression.
    pub fn span(&self) -> Span {
        match self {
            TermExpression::NamePath(path) => path.span,
            TermExpression::StringLiteral(string) => string.span,
            TermExpression::IntegerLiteral { span, .. } => *span,
            TermExpression::FloatLiteral { span, .. } => *span,
            TermExpression::Bool { span, .. } => *span,
            TermExpression::Binary(node) => node.span,
            TermExpression::Unary(node) => node.span,
            TermExpression::Turbofish { span, .. } => *span,
            TermExpression::ApplyCall { span, .. } => *span,
            TermExpression::DotCall { span, .. } => *span,
            TermExpression::Index { span, .. } => *span,
            TermExpression::Offset { span, .. } => *span,
            TermExpression::Paren { span, .. } => *span,
            TermExpression::Block(block) => block.span,
            TermExpression::Micro(lambda) => lambda.span,
            TermExpression::Object { span, .. } => *span,
            TermExpression::AnonymousClass(node) => node.span,
            TermExpression::If { span, .. } => *span,
            TermExpression::Match { span, .. } => *span,
            TermExpression::Loop { span, .. } => *span,
            TermExpression::Return(node) => node.span,
            TermExpression::Break(node) => node.span,
            TermExpression::Continue(node) => node.span,
            TermExpression::Yield { span, .. } => *span,
            TermExpression::Raise(raise) => raise.span,
            TermExpression::Resume(resume) => resume.span,
            TermExpression::Catch { span, .. } => *span,
            TermExpression::With { span, .. } => *span,
            TermExpression::SuperCall { span, .. } => *span,
        }
    }
}

impl Block {
    /// Returns the source code span of the block.
    pub fn span(&self) -> Span {
        self.span
    }
}

impl AnonymousMicro {
    /// Returns the source code span of the lambda expression.
    pub fn span(&self) -> Span {
        self.span
    }
}
