#[cfg(feature = "oak-pretty-print")]
use crate::ast::*;
#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::{AsDocument, Document, LINE as line, NIL as nil, SOFT_LINE_SPACE as soft_space, doc, indent};

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for SqlRoot {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        Document::join(self.statements.iter().map(|it| it.as_document(&())), doc!(";", line))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for SqlStatement {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            SqlStatement::Select(it) => it.as_document(&()),
            SqlStatement::Insert(it) => it.as_document(&()),
            SqlStatement::Update(it) => it.as_document(&()),
            SqlStatement::Delete(it) => it.as_document(&()),
            SqlStatement::Create(it) => it.as_document(&()),
            SqlStatement::Drop(it) => it.as_document(&()),
            SqlStatement::Alter(it) => it.as_document(&()),
            SqlStatement::Transaction(it) => Document::text(match it.action {
                crate::ast::TransactionAction::Begin => "BEGIN",
                crate::ast::TransactionAction::Commit => "COMMIT",
                crate::ast::TransactionAction::Rollback => "ROLLBACK",
            }),
            SqlStatement::SetNames(it) => {
                if let Some(collation) = &it.collation {
                    doc!("SET NAMES ", it.character_set.as_ref(), " COLLATE ", collation.as_ref())
                }
                else {
                    doc!("SET NAMES ", it.character_set.as_ref())
                }
            }
            SqlStatement::Error { .. } => nil,
            SqlStatement::Unknown { .. } => nil,
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for SelectStatement {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        let mut parts = Vec::new();
        parts.push(Document::text("SELECT"));

        let items = Document::join(self.items.iter().map(|it| it.as_document(&())), doc!(",", soft_space));
        parts.push(indent(doc!(line, items)));

        if let Some(from) = &self.from {
            parts.push(line);
            parts.push(Document::text("FROM"));
            parts.push(soft_space);
            parts.push(from.as_document(&()));
        }

        for join in &self.joins {
            parts.push(line);
            parts.push(join.as_document(&()));
        }

        if let Some(expr) = &self.expr {
            parts.push(line);
            parts.push(Document::text("WHERE"));
            parts.push(soft_space);
            parts.push(expr.as_document(&()));
        }

        if let Some(group_by) = &self.group_by {
            parts.push(line);
            parts.push(group_by.as_document(&()));
        }

        if let Some(having) = &self.having {
            parts.push(line);
            parts.push(having.as_document(&()));
        }

        if let Some(order_by) = &self.order_by {
            parts.push(line);
            parts.push(order_by.as_document(&()));
        }

        if let Some(limit) = &self.limit {
            parts.push(line);
            parts.push(limit.as_document(&()));
        }

        Document::group(Document::Concat(parts))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for SelectItem {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            SelectItem::Star { .. } => Document::text("*"),
            SelectItem::Expression { expr, alias, .. } => {
                if let Some(alias) = alias {
                    doc!(expr.as_document(&()), soft_space, "AS", soft_space, alias.as_document(&()))
                }
                else {
                    expr.as_document(&())
                }
            }
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for Expression {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            Expression::Identifier(it) => it.as_document(&()),
            Expression::Literal(it) => it.as_document(&()),
            Expression::Binary { left, op, right, .. } => {
                doc!(left.as_document(&()), soft_space, op.as_document(&()), soft_space, right.as_document(&()))
            }
            Expression::Unary { op, expr, .. } => {
                doc!(op.as_document(&()), expr.as_document(&()))
            }
            Expression::FunctionCall { name, args, .. } => {
                doc!(name.as_document(&()), "(", Document::join(args.iter().map(|it| it.as_document(&())), doc!(",", soft_space)), ")")
            }
            Expression::InList { expr, list, negated, .. } => {
                doc!(expr.as_document(&()), if *negated { doc!(soft_space, "NOT") } else { nil }, soft_space, "IN", soft_space, "(", Document::join(list.iter().map(|it| it.as_document(&())), doc!(",", soft_space)), ")")
            }
            Expression::Between { expr, low, high, negated, .. } => {
                doc!(expr.as_document(&()), if *negated { doc!(soft_space, "NOT") } else { nil }, soft_space, "BETWEEN", soft_space, low.as_document(&()), soft_space, "AND", soft_space, high.as_document(&()))
            }
            Expression::Subquery { query, .. } => {
                doc!("(", query.as_document(&()), ")")
            }
            Expression::InSubquery { expr, query: subquery, negated, .. } => {
                doc!(expr.as_document(&()), if *negated { doc!(soft_space, "NOT") } else { nil }, soft_space, "IN", soft_space, "(", subquery.as_document(&()), ")")
            }
            Expression::Error { .. } => nil,
            Expression::Vector { elements, .. } => {
                doc!("[", Document::join(elements.iter().map(|it| it.as_document(&())), doc!(",", soft_space)), "]")
            }
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for BinaryOperator {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            BinaryOperator::Plus => Document::text("+"),
            BinaryOperator::Minus => Document::text("-"),
            BinaryOperator::Star => Document::text("*"),
            BinaryOperator::Slash => Document::text("/"),
            BinaryOperator::Percent => Document::text("%"),
            BinaryOperator::And => Document::text("AND"),
            BinaryOperator::Or => Document::text("OR"),
            BinaryOperator::Equal => Document::text("="),
            BinaryOperator::NotEqual => Document::text("<>"),
            BinaryOperator::Less => Document::text("<"),
            BinaryOperator::Greater => Document::text(">"),
            BinaryOperator::LessEqual => Document::text("<="),
            BinaryOperator::GreaterEqual => Document::text(">="),
            BinaryOperator::Like => Document::text("LIKE"),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for UnaryOperator {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            UnaryOperator::Plus => Document::text("+"),
            UnaryOperator::Minus => Document::text("-"),
            UnaryOperator::Not => Document::text("NOT"),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for Literal {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            Literal::Number(n, _) => Document::text(n.as_ref()),
            Literal::String(s, _) => doc!("'", s.as_ref(), "'"),
            Literal::Boolean(b, _) => Document::text(if *b { "TRUE" } else { "FALSE" }),
            Literal::Null(_) => Document::text("NULL"),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for Identifier {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        Document::text(self.name.as_ref())
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for TableName {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        self.name.as_document(&())
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for ColumnName {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        self.name.as_document(&())
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for JoinClause {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        doc!(self.join_type.as_document(&()), soft_space, "JOIN", soft_space, self.table.as_document(&()), if let Some(on) = &self.on { doc!(soft_space, "ON", soft_space, on.as_document(&())) } else { nil })
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for JoinType {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            JoinType::Inner => Document::text("INNER"),
            JoinType::Left => Document::text("LEFT"),
            JoinType::Right => Document::text("RIGHT"),
            JoinType::Full => Document::text("FULL"),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for GroupByClause {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        doc!("GROUP", soft_space, "BY", soft_space, Document::join(self.columns.iter().map(|it| it.as_document(&())), doc!(",", soft_space)))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for HavingClause {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        doc!("HAVING", soft_space, self.condition.as_document(&()))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for OrderByClause {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        doc!("ORDER", soft_space, "BY", soft_space, Document::join(self.items.iter().map(|it| it.as_document(&())), doc!(",", soft_space)))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for OrderByItem {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        doc!(self.expr.as_document(&()), soft_space, self.direction.as_document(&()))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for OrderDirection {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            OrderDirection::Asc => Document::text("ASC"),
            OrderDirection::Desc => Document::text("DESC"),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LimitClause {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        doc!("LIMIT", soft_space, self.limit.as_document(&()), if let Some(offset) = &self.offset { doc!(soft_space, "OFFSET", soft_space, offset.as_document(&())) } else { nil })
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for CreateStatement {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        let mut parts = Vec::new();
        parts.push(Document::text("CREATE"));
        if let CreateBody::Index { unique: true, .. } = &self.body {
            parts.push(soft_space);
            parts.push(Document::text("UNIQUE"));
        }
        parts.push(soft_space);
        parts.push(self.object_type.as_document(&()));
        if self.if_not_exists {
            parts.push(soft_space);
            parts.push(Document::text("IF NOT EXISTS"));
        }
        parts.push(soft_space);
        parts.push(self.name.as_document(&()));

        match &self.body {
            CreateBody::Table { columns, .. } => {
                if !columns.is_empty() {
                    parts.push(soft_space);
                    parts.push(Document::text("("));
                    let cols = Document::join(columns.iter().map(|it| it.as_document(&())), doc!(",", line));
                    parts.push(indent(doc!(line, cols)));
                    parts.push(line);
                    parts.push(Document::text(")"));
                }
            }
            CreateBody::View { query, .. } => {
                parts.push(line);
                parts.push(Document::text("AS"));
                parts.push(soft_space);
                parts.push(query.as_document(&()));
            }
            CreateBody::Index { table_name, columns, unique: _, .. } => {
                parts.push(soft_space);
                parts.push(Document::text("ON"));
                parts.push(soft_space);
                parts.push(table_name.as_document(&()));
                parts.push(soft_space);
                parts.push(Document::text("("));
                let cols = Document::join(columns.iter().map(|it| it.as_document(&())), doc!(",", soft_space));
                parts.push(cols);
                parts.push(Document::text(")"));
            }
            CreateBody::Database { .. } => {}
        }

        Document::group(Document::Concat(parts))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for ColumnDefinition {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        let data_type_str = self.data_type.to_string();
        doc!(self.name.as_document(&()), soft_space, data_type_str, Document::join(self.constraints.iter().map(|it| doc!(soft_space, it.as_document(&()))), nil))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for ColumnConstraint {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            ColumnConstraint::PrimaryKey { .. } => Document::text("PRIMARY KEY"),
            ColumnConstraint::NotNull { .. } => Document::text("NOT NULL"),
            ColumnConstraint::Nullable { .. } => Document::text("NULL"),
            ColumnConstraint::Unique { .. } => Document::text("UNIQUE"),
            ColumnConstraint::Default(expr, _) => doc!("DEFAULT", soft_space, expr.as_document(&())),
            ColumnConstraint::Check(expr, _) => doc!("CHECK", soft_space, "(", expr.as_document(&()), ")"),
            ColumnConstraint::AutoIncrement { .. } => Document::text("AUTOINCREMENT"),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for CreateObjectType {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            CreateObjectType::Table => Document::text("TABLE"),
            CreateObjectType::View => Document::text("VIEW"),
            CreateObjectType::Index => Document::text("INDEX"),
            CreateObjectType::Database => Document::text("DATABASE"),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for InsertStatement {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        let mut parts = Vec::new();
        parts.push(Document::text("INSERT INTO"));
        parts.push(soft_space);
        parts.push(self.table_name.as_document(&()));

        if !self.columns.is_empty() {
            parts.push(soft_space);
            parts.push(Document::text("("));
            parts.push(Document::join(self.columns.iter().map(|it| it.as_document(&())), doc!(",", soft_space)));
            parts.push(Document::text(")"));
        }

        parts.push(line);
        parts.push(Document::text("VALUES"));
        parts.push(soft_space);
        parts.push(Document::text("("));
        parts.push(Document::join(self.values.iter().map(|it| it.as_document(&())), doc!(",", soft_space)));
        parts.push(Document::text(")"));

        Document::group(Document::Concat(parts))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for UpdateStatement {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        let mut parts = Vec::new();
        parts.push(Document::text("UPDATE"));
        parts.push(soft_space);
        parts.push(self.table_name.as_document(&()));
        parts.push(line);
        parts.push(Document::text("SET"));
        parts.push(soft_space);
        parts.push(Document::join(self.assignments.iter().map(|it| it.as_document(&())), doc!(",", line)));

        if let Some(selection) = &self.selection {
            parts.push(line);
            parts.push(Document::text("WHERE"));
            parts.push(soft_space);
            parts.push(selection.as_document(&()));
        }

        Document::group(Document::Concat(parts))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for Assignment {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        doc!(self.column.as_document(&()), soft_space, "=", soft_space, self.value.as_document(&()))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for DeleteStatement {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        let mut parts = Vec::new();
        parts.push(Document::text("DELETE FROM"));
        parts.push(soft_space);
        parts.push(self.table_name.as_document(&()));

        if let Some(selection) = &self.selection {
            parts.push(line);
            parts.push(Document::text("WHERE"));
            parts.push(soft_space);
            parts.push(selection.as_document(&()));
        }

        Document::group(Document::Concat(parts))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for DropStatement {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        doc!("DROP", soft_space, self.object_type.as_document(&()), if self.if_exists { doc!(soft_space, "IF EXISTS") } else { nil }, soft_space, self.name.as_document(&()))
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for DropObjectType {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            DropObjectType::Table => Document::text("TABLE"),
            DropObjectType::View => Document::text("VIEW"),
            DropObjectType::Index => Document::text("INDEX"),
            DropObjectType::Database => Document::text("DATABASE"),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for AlterStatement {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        let mut d = doc!("ALTER TABLE", soft_space, self.table_name.as_document(&()));
        if let Some(action) = &self.action {
            d = doc!(d, soft_space, action.as_document(&()));
        }
        d
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for AlterAction {
    fn as_document(&self, _params: &Self::Params) -> Document<'_> {
        match self {
            AlterAction::AddColumn { name, data_type, .. } => {
                let mut d = doc!("ADD COLUMN", soft_space, name.as_document(&()));
                if let Some(dt) = data_type {
                    let dt_str = dt.to_string();
                    d = doc!(d, soft_space, dt_str);
                }
                d
            }
            AlterAction::DropColumn { name, .. } => {
                doc!("DROP COLUMN", soft_space, name.as_document(&()))
            }
            AlterAction::RenameTo { new_name, .. } => {
                doc!("RENAME TO", soft_space, new_name.as_document(&()))
            }
        }
    }
}
