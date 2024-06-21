use crate::ast::statements::query::SelectStatement;
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};
use std::sync::Arc;

/// Represents an SQL expression.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Expression {
    /// An identifier expression.
    Identifier(Identifier),
    /// A wildcard argument such as the `*` in `COUNT(*)`.
    Wildcard {
        /// The span of the wildcard token.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A literal value expression.
    Literal(Literal),
    /// A binary operation expression.
    Binary {
        /// The left-hand side of the binary operation.
        left: Box<Expression>,
        /// The binary operator.
        op: BinaryOperator,
        /// The right-hand side of the binary operation.
        right: Box<Expression>,
        /// The span of the binary operation.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A unary operation expression.
    Unary {
        /// The unary operator.
        op: UnaryOperator,
        /// The expression being operated on.
        expr: Box<Expression>,
        /// The span of the unary operation.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A function call expression.
    FunctionCall {
        /// The name of the function being called.
        name: Identifier,
        /// The arguments passed to the function.
        args: Vec<Expression>,
        /// Whether the aggregate uses `DISTINCT` quantifier (`COUNT(DISTINCT x)`).
        distinct: bool,
        /// The span of the function call.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// Vector/Array literal.
    Vector {
        /// The vector elements.
        elements: Vec<Expression>,
        /// The span of the vector.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// An IN expression.
    InList {
        /// The expression being checked.
        expr: Box<Expression>,
        /// The list of values to check against.
        list: Vec<Expression>,
        /// Whether the IN condition is negated (NOT IN).
        negated: bool,
        /// The span of the IN expression.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A BETWEEN expression.
    Between {
        /// The expression being checked.
        expr: Box<Expression>,
        /// The lower bound of the range.
        low: Box<Expression>,
        /// The upper bound of the range.
        high: Box<Expression>,
        /// Whether the BETWEEN condition is negated (NOT BETWEEN).
        negated: bool,
        /// The span of the BETWEEN expression.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// An `IS [NOT] NULL` predicate.
    ///
    /// This is a three-valued predicate, not a comparison. `NULL = NULL` evaluates
    /// to `NULL` while `NULL IS NULL` evaluates to `TRUE`, so representing it as a
    /// binary comparison would lose the distinction.
    IsNull {
        /// The expression being tested.
        expr: Box<Expression>,
        /// Whether the predicate is negated (IS NOT NULL).
        negated: bool,
        /// The span of the predicate.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// An `IS [NOT] TRUE` or `IS [NOT] FALSE` predicate.
    IsTruth {
        /// The expression being tested.
        expr: Box<Expression>,
        /// The boolean value the expression is tested against.
        value: bool,
        /// Whether the predicate is negated (IS NOT TRUE / IS NOT FALSE).
        negated: bool,
        /// The span of the predicate.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A subquery expression.
    Subquery {
        /// The subquery SELECT statement.
        query: Box<SelectStatement>,
        /// The span of the subquery.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// An IN expression with a subquery.
    InSubquery {
        /// The expression being checked.
        expr: Box<Expression>,
        /// The subquery to check against.
        query: Box<SelectStatement>,
        /// Whether the IN condition is negated (NOT IN).
        negated: bool,
        /// The span of the IN expression.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// An error occurred during expression parsing or building.
    Error {
        /// The error message.
        message: Arc<str>,
        /// The span where the error occurred.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

impl Expression {
    /// Returns the source span of this expression.
    pub fn span(&self) -> Range<usize> {
        match self {
            Expression::Identifier(id) => id.span.clone(),
            Expression::Wildcard { span } => span.clone(),
            Expression::Literal(lit) => lit.span().clone(),
            Expression::Binary { span, .. } => span.clone(),
            Expression::Unary { span, .. } => span.clone(),
            Expression::FunctionCall { span, .. } => span.clone(),
            Expression::InList { span, .. } => span.clone(),
            Expression::Between { span, .. } => span.clone(),
            Expression::IsNull { span, .. } => span.clone(),
            Expression::IsTruth { span, .. } => span.clone(),
            Expression::Subquery { span, .. } => span.clone(),
            Expression::InSubquery { span, .. } => span.clone(),
            Expression::Vector { span, .. } => span.clone(),
            Expression::Error { span, .. } => span.clone(),
        }
    }
}

impl ToSource for Expression {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            Expression::Identifier(id) => id.to_source(buffer),
            Expression::Wildcard { .. } => buffer.push("*"),
            Expression::Literal(lit) => lit.to_source(buffer),
            Expression::Binary { left, op, right, .. } => {
                left.to_source(buffer);
                op.to_source(buffer);
                right.to_source(buffer);
            }
            Expression::Unary { op, expr, .. } => {
                op.to_source(buffer);
                expr.to_source(buffer);
            }
            Expression::FunctionCall { name, args, distinct, .. } => {
                name.to_source(buffer);
                buffer.push("(");
                if *distinct {
                    buffer.push("DISTINCT ");
                }
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        buffer.push(",");
                    }
                    arg.to_source(buffer);
                }
                buffer.push(")");
            }
            Expression::Vector { elements, .. } => {
                buffer.push("[");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        buffer.push(", ");
                    }
                    elem.to_source(buffer);
                }
                buffer.push("]");
            }
            Expression::InList { expr, list, negated, .. } => {
                expr.to_source(buffer);
                if *negated {
                    buffer.push("NOT");
                }
                buffer.push("IN");
                buffer.push("(");
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        buffer.push(",");
                    }
                    item.to_source(buffer);
                }
                buffer.push(")");
            }
            Expression::Between { expr, low, high, negated, .. } => {
                expr.to_source(buffer);
                if *negated {
                    buffer.push("NOT");
                }
                buffer.push("BETWEEN");
                low.to_source(buffer);
                buffer.push("AND");
                high.to_source(buffer);
            }
            Expression::IsNull { expr, negated, .. } => {
                to_source_is_operand(expr, buffer);
                buffer.push(" IS");
                if *negated {
                    buffer.push("NOT");
                }
                buffer.push("NULL");
            }
            Expression::IsTruth { expr, value, negated, .. } => {
                to_source_is_operand(expr, buffer);
                buffer.push(" IS");
                if *negated {
                    buffer.push("NOT");
                }
                buffer.push(if *value { "TRUE" } else { "FALSE" });
            }
            Expression::Subquery { query, .. } => {
                buffer.push("(");
                query.to_source(buffer);
                buffer.push(")");
            }
            Expression::InSubquery { expr, query, negated, .. } => {
                expr.to_source(buffer);
                if *negated {
                    buffer.push("NOT");
                }
                buffer.push("IN");
                buffer.push("(");
                query.to_source(buffer);
                buffer.push(")");
            }
            Expression::Error { message, .. } => {
                buffer.push("/* EXPR ERROR: ");
                buffer.push(message);
                buffer.push(" */");
            }
        }
    }
}

/// Returns whether `expr` must stay parenthesized when it is the left operand of
/// an `IS` predicate.
///
/// `IS` binds tighter than the logical connectives, `NOT`, `LIKE` and the
/// comparison operators, so printing those without parentheses would reparse the
/// predicate against a smaller operand.
pub(crate) fn needs_is_operand_parentheses(expr: &Expression) -> bool {
    match expr {
        Expression::Unary { op: UnaryOperator::Not, .. } => true,
        Expression::Binary { op, .. } => {
            matches!(op, BinaryOperator::Or | BinaryOperator::And | BinaryOperator::Equal | BinaryOperator::NotEqual | BinaryOperator::Less | BinaryOperator::Greater | BinaryOperator::LessEqual | BinaryOperator::GreaterEqual | BinaryOperator::Like)
        }
        _ => false,
    }
}

/// Writes the left operand of an `IS` predicate, adding parentheses when the
/// operand would otherwise reparse at a lower precedence.
fn to_source_is_operand(expr: &Expression, buffer: &mut SourceBuffer) {
    if needs_is_operand_parentheses(expr) {
        buffer.push("(");
        expr.to_source(buffer);
        buffer.push(")");
    }
    else {
        expr.to_source(buffer);
    }
}

/// Binary operators in SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BinaryOperator {
    /// Addition (+).
    Plus,
    /// Subtraction (-).
    Minus,
    /// Multiplication (*).
    Star,
    /// Division (/).
    Slash,
    /// Modulo (%).
    Percent,
    /// Logical AND.
    And,
    /// Logical OR.
    Or,
    /// Equality (=).
    Equal,
    /// Inequality (<> or !=).
    NotEqual,
    /// Less than (<).
    Less,
    /// Greater than (>).
    Greater,
    /// Less than or equal to (<=).
    LessEqual,
    /// Greater than or equal to (>=).
    GreaterEqual,
    /// Pattern matching (LIKE).
    Like,
}

impl ToSource for BinaryOperator {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            BinaryOperator::Plus => buffer.push("+"),
            BinaryOperator::Minus => buffer.push("-"),
            BinaryOperator::Star => buffer.push("*"),
            BinaryOperator::Slash => buffer.push("/"),
            BinaryOperator::Percent => buffer.push("%"),
            BinaryOperator::And => buffer.push("AND"),
            BinaryOperator::Or => buffer.push("OR"),
            BinaryOperator::Equal => buffer.push("="),
            BinaryOperator::NotEqual => buffer.push("<>"),
            BinaryOperator::Less => buffer.push("<"),
            BinaryOperator::Greater => buffer.push(">"),
            BinaryOperator::LessEqual => buffer.push("<="),
            BinaryOperator::GreaterEqual => buffer.push(">="),
            BinaryOperator::Like => buffer.push("LIKE"),
        }
    }
}

/// Unary operators in SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnaryOperator {
    /// Unary plus (+).
    Plus,
    /// Unary minus (-).
    Minus,
    /// Logical NOT.
    Not,
}

impl ToSource for UnaryOperator {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            UnaryOperator::Plus => buffer.push("+"),
            UnaryOperator::Minus => buffer.push("-"),
            UnaryOperator::Not => buffer.push("NOT"),
        }
    }
}

/// SQL literals.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Literal {
    /// Numeric literal.
    Number(Arc<str>, #[serde(with = "oak_core::serde_range")] Range<usize>),
    /// String literal.
    String(Arc<str>, #[serde(with = "oak_core::serde_range")] Range<usize>),
    /// Boolean literal.
    Boolean(bool, #[serde(with = "oak_core::serde_range")] Range<usize>),
    /// NULL literal.
    Null(#[serde(with = "oak_core::serde_range")] Range<usize>),
}

impl Literal {
    /// Returns the source span of this literal.
    pub fn span(&self) -> Range<usize> {
        match self {
            Literal::Number(_, span) => span.clone(),
            Literal::String(_, span) => span.clone(),
            Literal::Boolean(_, span) => span.clone(),
            Literal::Null(span) => span.clone(),
        }
    }
}

impl ToSource for Literal {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            Literal::Number(n, _) => buffer.push(n),
            Literal::String(s, _) => {
                buffer.push("'");
                buffer.push(s);
                buffer.push("'");
            }
            Literal::Boolean(b, _) => buffer.push(if *b { "TRUE" } else { "FALSE" }),
            Literal::Null(_) => buffer.push("NULL"),
        }
    }
}

/// SQL identifier.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Identifier {
    /// The name of the identifier.
    pub name: Arc<str>,
    /// The span of the identifier.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for Identifier {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(&self.name);
    }
}

/// SQL table name.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TableName {
    /// The name of the table.
    pub name: Identifier,
    /// The span of the table name.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for TableName {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.name.to_source(buffer);
    }
}

/// Column name.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColumnName {
    /// The name of the column.
    pub name: Identifier,
    /// The span of the column name in the source.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for ColumnName {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.name.to_source(buffer);
    }
}
