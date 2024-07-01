use crate::ast::expr::{Expression, Identifier, TableName};
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};

/// Represents a SELECT statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectStatement {
    /// The `WITH` clause that defines the common table expressions the statement
    /// may reference.
    pub with: Option<WithClause>,
    /// The `ALL` or `DISTINCT` quantifier, when one was written explicitly.
    pub quantifier: Option<SelectQuantifier>,
    /// The items to select (columns, expressions, or `*`).
    pub items: Vec<SelectItem>,
    /// The row source of the `FROM` clause.
    pub from: Option<TableSource>,
    /// Any JOIN clauses.
    pub joins: Vec<JoinClause>,
    /// The WHERE clause selection expression.
    pub expr: Option<Expression>, // WHERE clause
    /// The GROUP BY clause.
    pub group_by: Option<GroupByClause>,
    /// The HAVING clause.
    pub having: Option<HavingClause>,
    /// The ORDER BY clause.
    pub order_by: Option<OrderByClause>,
    /// The LIMIT clause.
    pub limit: Option<LimitClause>,
    /// The span of the SELECT statement.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for SelectStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        if let Some(with) = &self.with {
            with.to_source(buffer);
        }
        buffer.push("SELECT");
        if let Some(quantifier) = &self.quantifier {
            quantifier.to_source(buffer);
        }
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                buffer.push(",");
            }
            item.to_source(buffer);
        }
        if let Some(from) = &self.from {
            buffer.push("FROM");
            from.to_source(buffer);
        }
        for join in &self.joins {
            join.to_source(buffer);
        }
        if let Some(expr) = &self.expr {
            buffer.push("WHERE");
            expr.to_source(buffer);
        }
        if let Some(group_by) = &self.group_by {
            group_by.to_source(buffer);
        }
        if let Some(having) = &self.having {
            having.to_source(buffer);
        }
        if let Some(order_by) = &self.order_by {
            order_by.to_source(buffer);
        }
        if let Some(limit) = &self.limit {
            limit.to_source(buffer);
        }
    }
}

/// The `WITH` clause of a query, holding one or more common table expressions.
///
/// Only non-recursive CTEs are represented: every query references tables that
/// exist before the clause, so there is no self-reference flag to model.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WithClause {
    /// The common table expressions in the order they were written.
    pub ctes: Vec<CteDefinition>,
    /// The span of the clause, from `WITH` to the last CTE.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for WithClause {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("WITH");
        for (i, cte) in self.ctes.iter().enumerate() {
            if i > 0 {
                buffer.push(",");
            }
            cte.to_source(buffer);
        }
    }
}

/// A common table expression: `name [(col, ...)] AS (query)`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CteDefinition {
    /// The name the query is referenced by.
    pub name: Identifier,
    /// The optional column names that rename the query output.
    pub columns: Vec<Identifier>,
    /// The query that produces the rows.
    pub query: Box<SelectStatement>,
    /// The span of the definition.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for CteDefinition {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.name.to_source(buffer);
        if !self.columns.is_empty() {
            buffer.push("(");
            for (i, column) in self.columns.iter().enumerate() {
                if i > 0 {
                    buffer.push(",");
                }
                column.to_source(buffer);
            }
            buffer.push(")");
        }
        buffer.push("AS");
        buffer.push("(");
        self.query.to_source(buffer);
        buffer.push(")");
    }
}

/// A set operation between two `SELECT` statements.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetOperationStatement {
    /// The left operand.
    pub left: Box<SelectStatement>,
    /// The set operator that combines both operands.
    pub operator: SetOperator,
    /// The right operand.
    pub right: Box<SelectStatement>,
    /// The `ORDER BY` clause applied to the whole result, when one was written.
    pub order_by: Option<OrderByClause>,
    /// The `LIMIT` clause applied to the whole result, when one was written.
    pub limit: Option<LimitClause>,
    /// The span of the set operation.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for SetOperationStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.left.to_source(buffer);
        self.operator.to_source(buffer);
        self.right.to_source(buffer);
        if let Some(order_by) = &self.order_by {
            order_by.to_source(buffer);
        }
        if let Some(limit) = &self.limit {
            limit.to_source(buffer);
        }
    }
}

/// The operator of a set operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SetOperator {
    /// `UNION`
    Union,
    /// `UNION ALL`
    UnionAll,
    /// `INTERSECT`
    Intersect,
    /// `EXCEPT`
    Except,
}

impl ToSource for SetOperator {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            SetOperator::Union => buffer.push("UNION"),
            SetOperator::UnionAll => {
                buffer.push("UNION");
                buffer.push("ALL");
            }
            SetOperator::Intersect => buffer.push("INTERSECT"),
            SetOperator::Except => buffer.push("EXCEPT"),
        }
    }
}

/// The explicitly written row quantifier of a `SELECT` statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelectQuantifier {
    /// `SELECT ALL`
    All {
        /// The span of the `ALL` token.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// `SELECT DISTINCT`
    Distinct {
        /// The span of the `DISTINCT` token.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// `SELECT DISTINCT ON (expr, ...)`
    DistinctOn {
        /// The expressions the distinctness is evaluated on.
        expressions: Vec<Expression>,
        /// The span of the whole quantifier.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

impl ToSource for SelectQuantifier {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            SelectQuantifier::All { .. } => buffer.push("ALL"),
            SelectQuantifier::Distinct { .. } => buffer.push("DISTINCT"),
            SelectQuantifier::DistinctOn { expressions, .. } => {
                buffer.push("DISTINCT");
                buffer.push("ON");
                buffer.push("(");
                for (i, expr) in expressions.iter().enumerate() {
                    if i > 0 {
                        buffer.push(",");
                    }
                    expr.to_source(buffer);
                }
                buffer.push(")");
            }
        }
    }
}

/// Represents an item in a SELECT list.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelectItem {
    /// Select all columns (`*`).
    Star {
        /// The span of the `*`.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Select an expression with an optional alias.
    Expression {
        /// The expression to select.
        expr: Expression,
        /// The optional alias for the expression.
        alias: Option<Identifier>,
        /// The span of the select item.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

impl ToSource for SelectItem {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            SelectItem::Star { .. } => buffer.push("*"),
            SelectItem::Expression { expr, alias, .. } => {
                expr.to_source(buffer);
                if let Some(alias) = alias {
                    buffer.push("AS");
                    alias.to_source(buffer);
                }
            }
        }
    }
}

/// A row source of a `FROM` or `JOIN` clause.
///
/// A plain table and a derived table are different shapes: the first only names
/// storage, the second carries a whole query that must be planned as a subquery,
/// so flattening both into a table name would lose the query.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TableSource {
    /// A named table, optionally aliased (`FROM users u`).
    Table(TableName),
    /// A derived table built from a subquery (`FROM (SELECT ...) AS x`).
    Derived {
        /// The subquery that produces the rows.
        query: Box<SelectStatement>,
        /// The optional alias the derived table is referenced by.
        alias: Option<Identifier>,
        /// The span of the whole derived table, including its alias.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

impl ToSource for TableSource {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            TableSource::Table(table) => table.to_source(buffer),
            TableSource::Derived { query, alias, .. } => {
                buffer.push("(");
                query.to_source(buffer);
                buffer.push(")");
                if let Some(alias) = alias {
                    buffer.push("AS");
                    alias.to_source(buffer);
                }
            }
        }
    }
}

/// JOIN clause.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JoinClause {
    /// The type of join (e.g., INNER, LEFT).
    pub join_type: JoinType,
    /// The table being joined.
    pub table: TableSource,
    /// The join condition (ON clause).
    pub on: Option<Expression>,
    /// The span of the JOIN clause.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for JoinClause {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.join_type.to_source(buffer);
        buffer.push("JOIN");
        self.table.to_source(buffer);
        if let Some(on) = &self.on {
            buffer.push("ON");
            on.to_source(buffer);
        }
    }
}

/// Type of SQL join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JoinType {
    /// INNER JOIN.
    Inner,
    /// LEFT JOIN.
    Left,
    /// RIGHT JOIN.
    Right,
    /// FULL JOIN.
    Full,
}

impl ToSource for JoinType {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            JoinType::Inner => buffer.push("INNER"),
            JoinType::Left => buffer.push("LEFT"),
            JoinType::Right => buffer.push("RIGHT"),
            JoinType::Full => buffer.push("FULL"),
        }
    }
}

/// GROUP BY clause.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupByClause {
    /// The grouping items, in source order.
    pub items: Vec<GroupByItem>,
    /// The span of the GROUP BY clause.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for GroupByClause {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("GROUP");
        buffer.push("BY");
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                buffer.push(",");
            }
            item.to_source(buffer);
        }
    }
}

/// A single `GROUP BY` item.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GroupByItem {
    /// A bare column reference such as `GROUP BY users.id`.
    Column(Identifier),
    /// A positional reference counted from 1 such as `GROUP BY 1`.
    Position {
        /// The one-based position in the select list.
        index: usize,
        /// The span of the position literal.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// Any other grouping expression.
    Expression(Expression),
}

impl ToSource for GroupByItem {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            GroupByItem::Column(identifier) => identifier.to_source(buffer),
            GroupByItem::Position { index, .. } => buffer.push(&index.to_string()),
            GroupByItem::Expression(expr) => expr.to_source(buffer),
        }
    }
}

/// HAVING clause.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HavingClause {
    /// The condition for the HAVING clause.
    pub condition: Expression,
    /// The span of the HAVING clause.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for HavingClause {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("HAVING");
        self.condition.to_source(buffer);
    }
}

/// ORDER BY clause.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderByClause {
    /// The items to order by.
    pub items: Vec<OrderByItem>,
    /// The span of the ORDER BY clause.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for OrderByClause {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("ORDER");
        buffer.push("BY");
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                buffer.push(",");
            }
            item.to_source(buffer);
        }
    }
}

/// An item in an ORDER BY clause.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderByItem {
    /// The expression to order by.
    pub expr: Expression,
    /// The direction of the ordering.
    pub direction: OrderDirection,
    /// The explicit `NULLS FIRST` / `NULLS LAST` placement, when written.
    pub nulls: Option<NullsOrder>,
    /// The span of the ORDER BY item.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for OrderByItem {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.expr.to_source(buffer);
        self.direction.to_source(buffer);
        if let Some(nulls) = &self.nulls {
            buffer.push("NULLS");
            nulls.to_source(buffer);
        }
    }
}

/// Placement of null values in an ordered result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NullsOrder {
    /// `NULLS FIRST`
    First,
    /// `NULLS LAST`
    Last,
}

impl ToSource for NullsOrder {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            NullsOrder::First => buffer.push("FIRST"),
            NullsOrder::Last => buffer.push("LAST"),
        }
    }
}

/// Direction of ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OrderDirection {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

impl ToSource for OrderDirection {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            OrderDirection::Asc => buffer.push("ASC"),
            OrderDirection::Desc => buffer.push("DESC"),
        }
    }
}

/// LIMIT clause.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LimitClause {
    /// The limit value.
    pub limit: Expression,
    /// The optional offset value.
    pub offset: Option<Expression>,
    /// The span of the LIMIT clause.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for LimitClause {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("LIMIT");
        self.limit.to_source(buffer);
        if let Some(offset) = &self.offset {
            buffer.push("OFFSET");
            offset.to_source(buffer);
        }
    }
}
