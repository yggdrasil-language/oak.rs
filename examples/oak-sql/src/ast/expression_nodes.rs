use crate::ast::statements::query::{NullsOrder, OrderDirection, SelectStatement};
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
        /// The `OVER (...)` window specification, absent for plain calls.
        window: Option<WindowSpec>,
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
    /// An `[NOT] EXISTS (SELECT ...)` predicate.
    Exists {
        /// The subquery whose rows are tested for existence.
        query: Box<SelectStatement>,
        /// Whether the predicate is negated (NOT EXISTS).
        negated: bool,
        /// The span of the predicate.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A `CASE WHEN ... THEN ... [ELSE ...] END` expression.
    Case {
        /// The WHEN/THEN branches. Each pair is `(when_expr, then_expr)`.
        branches: Vec<(Box<Expression>, Box<Expression>)>,
        /// Optional ELSE branch.
        else_branch: Option<Box<Expression>>,
        /// The span of the whole CASE expression.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A `CAST(expr AS type)` expression.
    Cast {
        /// The expression being cast.
        expr: Box<Expression>,
        /// The target type name (e.g. `INTEGER`, `TEXT`, `TIMESTAMP`).
        target: Arc<str>,
        /// The span of the whole CAST expression.
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
            Expression::Exists { span, .. } => span.clone(),
            Expression::Vector { span, .. } => span.clone(),
            Expression::Case { span, .. } => span.clone(),
            Expression::Cast { span, .. } => span.clone(),
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
            Expression::FunctionCall { name, args, distinct, window, .. } => {
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
                if let Some(window) = window {
                    window.to_source(buffer);
                }
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
            Expression::Exists { query, negated, .. } => {
                if *negated {
                    buffer.push("NOT");
                }
                buffer.push("EXISTS");
                buffer.push("(");
                query.to_source(buffer);
                buffer.push(")");
            }
            Expression::Case { branches, else_branch, .. } => {
                buffer.push("CASE ");
                for (when, then) in branches {
                    buffer.push("WHEN ");
                    when.to_source(buffer);
                    buffer.push(" THEN ");
                    then.to_source(buffer);
                    buffer.push(" ");
                }
                if let Some(el) = else_branch {
                    buffer.push("ELSE ");
                    el.to_source(buffer);
                    buffer.push(" ");
                }
                buffer.push("END");
            }
            Expression::Cast { expr, target, .. } => {
                buffer.push("CAST(");
                expr.to_source(buffer);
                buffer.push(" AS ");
                buffer.push(target);
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
    /// The optional alias, as written in `FROM users u` or `FROM users AS u`.
    pub alias: Option<Identifier>,
    /// The span of the table name, including its alias when one was written.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for TableName {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.name.to_source(buffer);
        if let Some(alias) = &self.alias {
            buffer.push("AS");
            alias.to_source(buffer);
        }
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

/// The `OVER (...)` window specification of a function call.
///
/// Only column references are accepted in `PARTITION BY` and `ORDER BY`, and the
/// frame is limited to the `ROWS` / `RANGE` forms below. Anything else is
/// rejected while lowering the parse tree.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowSpec {
    /// The `PARTITION BY` columns, in source order.
    pub partition_by: Vec<Identifier>,
    /// The `ORDER BY` items, in source order.
    pub order_by: Vec<WindowOrderItem>,
    /// The explicit `ROWS` / `RANGE` frame, when written.
    pub frame: Option<WindowFrame>,
    /// The span of the whole `OVER (...)` clause.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for WindowSpec {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        // The clauses are joined in their own buffer so optional parts never leave
        // a doubled or trailing space inside `OVER (...)`.
        let mut inner = SourceBuffer::new();
        let mut empty = true;
        if !self.partition_by.is_empty() {
            inner.push("PARTITION BY ");
            for (i, column) in self.partition_by.iter().enumerate() {
                if i > 0 {
                    inner.push(", ");
                }
                column.to_source(&mut inner);
            }
            empty = false;
        }
        if !self.order_by.is_empty() {
            if !empty {
                inner.push(" ");
            }
            inner.push("ORDER BY ");
            for (i, item) in self.order_by.iter().enumerate() {
                if i > 0 {
                    inner.push(", ");
                }
                item.to_source(&mut inner);
            }
            empty = false;
        }
        if let Some(frame) = &self.frame {
            if !empty {
                inner.push(" ");
            }
            frame.to_source(&mut inner);
        }
        buffer.push(" OVER (");
        buffer.push(&inner.finish());
        buffer.push(")");
    }
}

/// One `ORDER BY` column inside a window specification.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowOrderItem {
    /// The column the window ordering is computed on.
    pub column: Identifier,
    /// The direction of the ordering.
    pub direction: OrderDirection,
    /// The explicit `NULLS FIRST` / `NULLS LAST` placement, when written.
    pub nulls: Option<NullsOrder>,
    /// The span of the item, including its direction and nulls placement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for WindowOrderItem {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.column.to_source(buffer);
        self.direction.to_source(buffer);
        if let Some(nulls) = &self.nulls {
            buffer.push("NULLS");
            nulls.to_source(buffer);
        }
    }
}

/// The rows a window function is evaluated over.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowFrame {
    /// Whether the frame counts rows or value ranges.
    pub unit: WindowFrameUnit,
    /// The bound the frame starts at.
    pub start: WindowFrameBound,
    /// The bound the frame ends at, absent for the single-bound form.
    pub end: Option<WindowFrameBound>,
    /// The span of the whole frame.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for WindowFrame {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.unit.to_source(buffer);
        match &self.end {
            Some(end) => {
                buffer.push(" BETWEEN ");
                self.start.to_source(buffer);
                buffer.push(" AND ");
                end.to_source(buffer);
            }
            None => {
                buffer.push(" ");
                self.start.to_source(buffer);
            }
        }
    }
}

/// Unit of a window frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WindowFrameUnit {
    /// `ROWS`.
    Rows,
    /// `RANGE`.
    Range,
}

impl ToSource for WindowFrameUnit {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            WindowFrameUnit::Rows => buffer.push("ROWS"),
            WindowFrameUnit::Range => buffer.push("RANGE"),
        }
    }
}

/// One bound of a window frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WindowFrameBound {
    /// `UNBOUNDED PRECEDING`.
    UnboundedPreceding,
    /// `n PRECEDING`.
    Preceding(u64),
    /// `CURRENT ROW`.
    CurrentRow,
    /// `n FOLLOWING`.
    Following(u64),
    /// `UNBOUNDED FOLLOWING`.
    UnboundedFollowing,
}

impl WindowFrameBound {
    /// Returns whether this bound sits after `end` in a `BETWEEN` frame.
    ///
    /// Offsets are compared ordinally, so `5 PRECEDING` is before `2 PRECEDING`.
    /// An `UNBOUNDED PRECEDING` end bound is never valid, so any start is after it.
    /// The parser and the builder share this rule to report the same frames.
    pub(crate) fn is_after(&self, end: &WindowFrameBound) -> bool {
        fn rank(bound: &WindowFrameBound) -> i64 {
            match bound {
                WindowFrameBound::UnboundedPreceding => i64::MIN,
                WindowFrameBound::Preceding(offset) => -(*offset as i64),
                WindowFrameBound::CurrentRow => 0,
                WindowFrameBound::Following(offset) => *offset as i64,
                WindowFrameBound::UnboundedFollowing => i64::MAX,
            }
        }
        match end {
            WindowFrameBound::UnboundedPreceding => true,
            _ => rank(self) > rank(end),
        }
    }
}

impl ToSource for WindowFrameBound {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            WindowFrameBound::UnboundedPreceding => buffer.push("UNBOUNDED PRECEDING"),
            WindowFrameBound::Preceding(offset) => buffer.push(&format!("{offset} PRECEDING")),
            WindowFrameBound::CurrentRow => buffer.push("CURRENT ROW"),
            WindowFrameBound::Following(offset) => buffer.push(&format!("{offset} FOLLOWING")),
            WindowFrameBound::UnboundedFollowing => buffer.push("UNBOUNDED FOLLOWING"),
        }
    }
}
