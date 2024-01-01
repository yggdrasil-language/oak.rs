use crate::{SqlElementType, SqlLanguage, SqlParser, ast, ast::*, lexer::token_type::SqlTokenType};
use oak_core::{Builder, BuilderCache, GreenNode, OakDiagnostics, OakError, Parser, Range, RedNode, RedTree, SourceText, TextEdit, builder::BuildOutput, source::Source};
use std::sync::Arc;

mod build_expression_tree;

/// AST builder for SQL.
///
/// This builder is responsible for converting the generic syntax tree (green/red trees)
/// into a strongly-typed SQL Abstract Syntax Tree (AST). It implements the [`Builder`]
/// trait, allowing it to be used within the Oak incremental parsing framework.
///
/// # Usage
///
/// ```rust
/// use oak_core::{Builder, source::SourceText};
/// use oak_sql::{SqlBuilder, SqlLanguage};
///
/// let config = SqlLanguage::default();
/// let builder = SqlBuilder::new(&config);
/// // ... use builder to build AST from source
/// ```
#[derive(Clone)]
pub struct SqlBuilder<'config> {
    config: &'config SqlLanguage,
}

impl<'config> SqlBuilder<'config> {
    /// Creates a new `SqlBuilder` with the specified language configuration.
    pub fn new(config: &'config SqlLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<SqlLanguage> for SqlBuilder<'config> {
    /// Builds the SQL AST from the source text and edits.
    ///
    /// This is the main entry point for the builder. It performs the following steps:
    /// 1. Initializes a [`SqlParser`] with the current configuration.
    /// 2. Executes the parser to produce a [`RedTree`] (Concrete Syntax Tree).
    /// 3. Converts the resulting green tree into a typed [`SqlRoot`] AST.
    /// 4. Handles incremental updates by passing `edits` to the parser.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use oak_core::{Builder, ParseSession, source::SourceText};
    /// use oak_sql::{SqlBuilder, SqlLanguage};
    ///
    /// let config = SqlLanguage::default();
    /// let builder = SqlBuilder::new(&config);
    /// let mut cache = ParseSession::default();
    /// let source = "SELECT * FROM users";
    /// let output = builder.build(&source, &[], &mut cache);
    ///
    /// if let Ok(root) = output.result {
    ///     assert_eq!(root.statements.len(), 1);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails or if the CST cannot be lowered to a valid AST.
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], cache: &'a mut impl BuilderCache<SqlLanguage>) -> BuildOutput<SqlLanguage> {
        let parser = SqlParser::new(self.config);
        let parse_result = parser.parse(source, edits, cache);
        let OakDiagnostics { result, diagnostics } = parse_result;

        match result {
            Ok(green_tree) => {
                let source_text = SourceText::new(source.get_text_in((0..source.length()).into()).into_owned());
                match self.build_root(green_tree, &source_text) {
                    Ok(ast_root) => OakDiagnostics { result: Ok(ast_root), diagnostics },
                    Err(build_error) => {
                        let mut diagnostics = diagnostics;
                        diagnostics.push(build_error.clone());
                        OakDiagnostics { result: Err(build_error), diagnostics }
                    }
                }
            }
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics },
        }
    }
}

impl<'config> SqlBuilder<'config> {
    /// Builds the root SQL AST node from a green tree.
    ///
    /// This method performs the "lowering" phase where it traverses the untyped [`GreenNode`]
    /// tree and constructs a structured [`SqlRoot`]. It maps each top-level child node
    /// to its corresponding SQL statement type (SELECT, INSERT, etc.).
    pub(crate) fn build_root<'a>(&self, green_tree: &'a GreenNode<'a, SqlLanguage>, source: &SourceText) -> Result<SqlRoot, OakError> {
        let root_node = RedNode::new(green_tree, 0);
        let mut statements = Vec::new();

        for child in root_node.children() {
            if let RedTree::Node(n) = child {
                let res = match n.green.kind {
                    SqlElementType::SelectStatement => self.build_select_statement(n, source).map(SqlStatement::Select),
                    SqlElementType::InsertStatement => self.build_insert_statement(n, source).map(SqlStatement::Insert),
                    SqlElementType::UpdateStatement => self.build_update_statement(n, source).map(SqlStatement::Update),
                    SqlElementType::DeleteStatement => self.build_delete_statement(n, source).map(SqlStatement::Delete),
                    SqlElementType::CreateStatement => self.build_create_statement(n, source).map(SqlStatement::Create),
                    SqlElementType::DropStatement => self.build_drop_statement(n, source).map(SqlStatement::Drop),
                    SqlElementType::AlterStatement => self.build_alter_statement(n, source).map(SqlStatement::Alter),
                    SqlElementType::TransactionStatement => self.build_transaction_statement(n).map(SqlStatement::Transaction),
                    SqlElementType::SetNamesStatement => self.build_set_names_statement(n, source).map(SqlStatement::SetNames),
                    _ => continue,
                };

                match res {
                    Ok(stmt) => statements.push(stmt),
                    Err(e) => {
                        statements.push(SqlStatement::Error { message: Arc::from(e.to_string()), span: n.span() });
                    }
                }
            }
        }

        Ok(SqlRoot { statements, span: root_node.span() })
    }

    /// Builds a `SELECT` statement AST node.
    fn build_select_statement<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<SelectStatement, OakError> {
        let mut items = Vec::new();
        let mut from = None;
        let mut joins = Vec::new();
        let mut expr = None;
        let mut group_by = None;
        let mut having = None;
        let mut order_by = None;
        let mut limit = None;

        let mut where_found = false;

        for child in node.children() {
            match child {
                RedTree::Node(n) => match n.green.kind {
                    SqlElementType::SelectItem => items.push(self.build_select_item(n, source)?),
                    SqlElementType::TableName => from = Some(self.build_table_name(n, source)?),
                    SqlElementType::JoinClause => joins.push(self.build_join_clause(n, source)?),
                    SqlElementType::GroupByClause => group_by = Some(self.build_group_by_clause(n, source)?),
                    SqlElementType::HavingClause => having = Some(self.build_having_clause(n, source)?),
                    SqlElementType::OrderByClause => order_by = Some(self.build_order_by_clause(n, source)?),
                    SqlElementType::LimitClause => limit = Some(self.build_limit_clause(n, source)?),
                    SqlElementType::Expression => {
                        if where_found && expr.is_none() {
                            expr = Some(self.build_expression(n, source)?);
                        }
                    }
                    _ => {}
                },
                RedTree::Leaf(t) => {
                    if t.kind == SqlTokenType::Where {
                        where_found = true;
                    }
                }
            }
        }

        Ok(SelectStatement { items, from, joins, expr, group_by, having, order_by, limit, span: node.span() })
    }

    /// Builds a select item (column or expression) for a `SELECT` statement.
    fn build_select_item<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<SelectItem, OakError> {
        let mut expr = None;
        let mut alias = None;
        let mut is_star = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    SqlTokenType::Star => is_star = true,
                    SqlTokenType::Identifier_ => {
                        if expr.is_none() {
                            expr = Some(Expression::Identifier(Identifier { name: self.get_text(t.span.clone(), source), span: t.span.clone() }));
                        }
                        else {
                            alias = Some(Identifier { name: self.get_text(t.span.clone(), source), span: t.span.clone() });
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    SqlElementType::Expression => expr = Some(self.build_expression(n, source)?),
                    SqlElementType::Identifier => {
                        if expr.is_none() {
                            expr = Some(Expression::Identifier(self.build_identifier(n, source)?));
                        }
                        else {
                            alias = Some(self.build_identifier(n, source)?);
                        }
                    }
                    SqlElementType::Alias => alias = Some(self.build_identifier(n, source)?),
                    _ => {}
                },
            }
        }

        if is_star { Ok(SelectItem::Star { span: node.span() }) } else { Ok(SelectItem::Expression { expr: expr.ok_or_else(|| OakError::custom_error("Missing expression in select item"))?, alias, span: node.span() }) }
    }

    /// Builds an `INSERT` statement AST node.
    fn build_insert_statement<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<InsertStatement, OakError> {
        let mut table_name = None;
        let mut columns = Vec::new();
        let mut values = Vec::new();

        for child in node.children() {
            if let RedTree::Node(n) = child {
                match n.green.kind {
                    SqlElementType::TableName => table_name = Some(self.build_table_name(n, source)?),
                    SqlElementType::ColumnName => {
                        for sub in n.children() {
                            match sub {
                                RedTree::Node(sn) if sn.green.kind == SqlElementType::Identifier => {
                                    columns.push(self.build_identifier(sn, source)?);
                                }
                                RedTree::Leaf(st) if st.kind == SqlTokenType::Identifier_ => {
                                    columns.push(Identifier { name: self.get_text(st.span.clone(), source), span: st.span.clone() });
                                }
                                _ => {}
                            }
                        }
                    }
                    SqlElementType::ValueList => {
                        self.collect_expressions(n, source, &mut values)?;
                    }
                    _ => {}
                }
            }
        }

        Ok(InsertStatement { table_name: table_name.ok_or_else(|| OakError::custom_error("Missing table name in INSERT"))?, columns, values, span: node.span() })
    }

    /// Collects expressions from a node into a vector.
    fn collect_expressions<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText, out: &mut Vec<Expression>) -> Result<(), OakError> {
        for child in node.children() {
            if let RedTree::Node(n) = child {
                match n.green.kind {
                    SqlElementType::Expression => out.push(self.build_expression(n, source)?),
                    SqlElementType::Identifier => out.push(Expression::Identifier(self.build_identifier(n, source)?)),
                    SqlElementType::ValueList => self.collect_expressions(n, source, out)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Builds an `UPDATE` statement AST node.
    fn build_update_statement<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<UpdateStatement, OakError> {
        let mut table_name: Option<TableName> = None;
        let mut assignments = Vec::new();
        let mut selection = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    SqlTokenType::Set => {}
                    SqlTokenType::Identifier_ => {
                        if table_name.is_none() {
                            let ident = Identifier { name: self.get_text(t.span.clone(), source), span: t.span.clone() };
                            table_name = Some(TableName { name: ident, span: t.span.clone() });
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    SqlElementType::TableName => table_name = Some(self.build_table_name(n, source)?),
                    SqlElementType::Assignment => assignments.push(self.build_assignment(n, source)?),
                    SqlElementType::Expression => selection = Some(self.build_expression(n, source)?),
                    _ => {}
                },
            }
        }

        Ok(UpdateStatement { table_name: table_name.ok_or_else(|| OakError::custom_error("Missing table name in UPDATE"))?, assignments, selection, span: node.span() })
    }

    /// Builds an assignment (column = expression) for an `UPDATE` statement.
    fn build_assignment<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Assignment, OakError> {
        let mut column = None;
        let mut value = None;

        for child in node.children() {
            if let RedTree::Node(n) = child {
                match n.green.kind {
                    SqlElementType::ColumnName => column = Some(self.build_identifier(n, source)?),
                    SqlElementType::Expression => value = Some(self.build_expression(n, source)?),
                    _ => {}
                }
            }
        }

        Ok(Assignment { column: column.ok_or_else(|| OakError::custom_error("Missing column in assignment"))?, value: value.ok_or_else(|| OakError::custom_error("Missing value in assignment"))?, span: node.span() })
    }

    /// Builds a `DELETE` statement AST node.
    fn build_delete_statement<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<DeleteStatement, OakError> {
        let mut table_name = None;
        let mut selection = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => {
                    if t.kind == SqlTokenType::From {
                        // skip
                    }
                }
                RedTree::Node(n) => match n.green.kind {
                    SqlElementType::TableName => table_name = Some(self.build_table_name(n, source)?),
                    SqlElementType::Expression => selection = Some(self.build_expression(n, source)?),
                    _ => {}
                },
            }
        }

        Ok(DeleteStatement { table_name: table_name.ok_or_else(|| OakError::custom_error("Missing table name in DELETE"))?, selection, span: node.span() })
    }

    /// Builds a `CREATE` statement AST node.
    fn build_create_statement<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<CreateStatement, OakError> {
        let mut object_type = CreateObjectType::Table;
        let mut name = None;
        let mut if_not_exists = false;
        let mut columns = Vec::new();
        let mut query = None;
        let mut table_name = None;
        let mut index_columns = Vec::new();
        let mut unique = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    SqlTokenType::Table => object_type = CreateObjectType::Table,
                    SqlTokenType::View => object_type = CreateObjectType::View,
                    SqlTokenType::Index => object_type = CreateObjectType::Index,
                    SqlTokenType::Database => object_type = CreateObjectType::Database,
                    SqlTokenType::Exists => if_not_exists = true,
                    SqlTokenType::Unique => unique = true,
                    _ => {
                        if object_type == CreateObjectType::Index && name.is_some() && table_name.is_some() && t.kind == SqlTokenType::Identifier_ {
                            index_columns.push(ast::Identifier { name: self.get_text(t.span.clone(), source), span: t.span.clone() });
                        }
                    }
                },
                RedTree::Node(n) => match n.green.kind {
                    SqlElementType::TableName => {
                        if object_type == CreateObjectType::Index && name.is_some() {
                            table_name = Some(self.build_table_name(n, source)?);
                        }
                        else {
                            name = Some(self.build_identifier(n, source)?);
                        }
                    }
                    SqlElementType::Identifier => {
                        if object_type == CreateObjectType::Index && name.is_some() && table_name.is_none() {
                            table_name = Some(ast::TableName { name: self.build_identifier(n, source)?, span: n.span() });
                        }
                        else if name.is_none() {
                            name = Some(self.build_identifier(n, source)?);
                        }
                        else if object_type == CreateObjectType::Index {
                            index_columns.push(self.build_identifier(n, source)?);
                        }
                    }
                    SqlElementType::ColumnDefinition => {
                        columns.push(self.build_column_definition(n, source)?);
                    }
                    SqlElementType::SelectStatement => {
                        query = Some(self.build_select_statement(n, source)?);
                    }
                    _ => {}
                },
            }
        }

        let body = match object_type {
            CreateObjectType::Table => CreateBody::Table { columns, span: node.span() },
            CreateObjectType::View => CreateBody::View { query: Box::new(query.ok_or_else(|| OakError::custom_error("Missing query in CREATE VIEW"))?), span: node.span() },
            CreateObjectType::Index => CreateBody::Index { table_name: table_name.ok_or_else(|| OakError::custom_error("Missing table name in CREATE INDEX"))?, columns: index_columns, unique, span: node.span() },
            CreateObjectType::Database => CreateBody::Database { span: node.span() },
        };

        let result = CreateStatement { object_type, name: name.ok_or_else(|| OakError::custom_error("Missing name in CREATE"))?, if_not_exists, body, span: node.span() };
        Ok(result)
    }

    fn build_column_definition<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<ColumnDefinition, OakError> {
        let mut name = None;
        let mut data_type_str = String::new();
        let mut constraints = Vec::new();

        // Use a flag to track if we are currently parsing the data type
        let mut parsing_data_type = false;

        for child in node.children() {
            match child {
                RedTree::Node(n) => match n.green.kind {
                    SqlElementType::ColumnName | SqlElementType::Identifier => {
                        if name.is_none() {
                            name = Some(self.build_identifier(n, source)?);
                            parsing_data_type = true;
                        }
                        else if parsing_data_type {
                            if !data_type_str.is_empty() {
                                data_type_str.push(' ');
                            }
                            data_type_str.push_str(self.get_text(n.span(), source).trim());
                        }
                    }
                    _ => {}
                },
                RedTree::Leaf(t) => {
                    match t.kind {
                        SqlTokenType::Primary => {
                            constraints.push(ColumnConstraint::PrimaryKey { span: t.span.clone() });
                            parsing_data_type = false;
                        }
                        SqlTokenType::Key => {
                            // Part of PRIMARY KEY, stop parsing data type if we haven't already
                            parsing_data_type = false;
                        }
                        SqlTokenType::Not => {
                            constraints.push(ColumnConstraint::NotNull { span: t.span.clone() });
                            parsing_data_type = false;
                        }
                        SqlTokenType::Null => {
                            // Check if NOT NULL was just added
                            let mut is_not_null = false;
                            if let Some(ColumnConstraint::NotNull { .. }) = constraints.last() {
                                is_not_null = true;
                            }

                            if !is_not_null {
                                constraints.push(ColumnConstraint::Nullable { span: t.span.clone() });
                            }
                            parsing_data_type = false;
                        }
                        SqlTokenType::Unique => {
                            constraints.push(ColumnConstraint::Unique { span: t.span.clone() });
                            parsing_data_type = false;
                        }
                        SqlTokenType::AutoIncrement => {
                            constraints.push(ColumnConstraint::AutoIncrement { span: t.span.clone() });
                            parsing_data_type = false;
                        }
                        SqlTokenType::Default => {
                            parsing_data_type = false;
                            // The expression follows Default token in column definition
                            if let Some(expr_node) = self.find_next_node_in_parent(node.clone(), SqlElementType::Expression, t.span.end) {
                                constraints.push(ColumnConstraint::Default(self.build_expression(expr_node, source)?, t.span.clone()));
                            }
                        }
                        SqlTokenType::Check => {
                            parsing_data_type = false;
                            // The expression follows Check token in column definition
                            if let Some(expr_node) = self.find_next_node_in_parent(node.clone(), SqlElementType::Expression, t.span.end) {
                                constraints.push(ColumnConstraint::Check(self.build_expression(expr_node, source)?, t.span.clone()));
                            }
                        }
                        _ if parsing_data_type => {
                            if !data_type_str.is_empty() {
                                data_type_str.push(' ');
                            }
                            data_type_str.push_str(self.get_text(t.span.clone(), source).trim());
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(ColumnDefinition { name: name.ok_or_else(|| OakError::custom_error("Missing name in column definition"))?, data_type: Arc::from(data_type_str), constraints, span: node.span() })
    }

    /// Helper to find the next node of a specific kind after a certain position
    fn find_next_node_in_parent<'a>(&self, parent: RedNode<'a, SqlLanguage>, kind: SqlElementType, after_pos: usize) -> Option<RedNode<'a, SqlLanguage>> {
        for child in parent.children() {
            if let RedTree::Node(n) = child {
                if n.green.kind == kind && n.span().start >= after_pos {
                    return Some(n);
                }
            }
        }
        None
    }

    fn build_drop_statement<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<DropStatement, OakError> {
        let mut object_type = DropObjectType::Table;
        let mut name = None;
        let mut if_exists = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    SqlTokenType::Table => object_type = DropObjectType::Table,
                    SqlTokenType::View => object_type = DropObjectType::View,
                    SqlTokenType::Index => object_type = DropObjectType::Index,
                    SqlTokenType::Database => object_type = DropObjectType::Database,
                    SqlTokenType::Exists => if_exists = true,
                    _ => {}
                },
                RedTree::Node(n) => {
                    if n.green.kind == SqlElementType::TableName || n.green.kind == SqlElementType::Identifier {
                        name = Some(self.build_identifier(n, source)?);
                    }
                }
            }
        }

        let result = DropStatement { object_type, name: name.ok_or_else(|| OakError::custom_error("Missing name in DROP"))?, if_exists, span: node.span() };
        Ok(result)
    }

    fn build_alter_statement<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<AlterStatement, OakError> {
        let mut table_name = None;
        let mut action = None;

        for child in node.children() {
            if let RedTree::Node(n) = child {
                if n.green.kind == SqlElementType::TableName {
                    table_name = Some(self.build_table_name(n, source)?);
                }
                else if n.green.kind == SqlElementType::AlterAction {
                    action = Some(self.build_alter_action(n, source)?);
                }
            }
        }

        let result = AlterStatement { table_name: table_name.ok_or_else(|| OakError::custom_error("Missing table name in ALTER"))?, action, span: node.span() };
        Ok(result)
    }

    fn build_alter_action<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<ast::AlterAction, OakError> {
        use SqlTokenType::*;
        let mut is_add = false;
        let mut is_drop = false;
        let mut is_rename = false;
        let mut identifier: Option<ast::Identifier> = None;
        let mut data_type_tokens = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    Add => is_add = true,
                    Drop => is_drop = true,
                    Rename => is_rename = true,
                    Identifier_ => {
                        if identifier.is_none() {
                            identifier = Some(ast::Identifier { name: self.get_text(t.span.clone(), source), span: t.span.clone() });
                        }
                        else if is_add {
                            data_type_tokens.push(t.span.clone());
                        }
                    }
                    LeftParen | RightParen | NumberLiteral | Comma | Int | Integer | Varchar | Char | Text | Date | Time | Timestamp | Decimal | Float | Double | Boolean => {
                        if is_add && identifier.is_some() {
                            data_type_tokens.push(t.span.clone());
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) if n.green.kind == SqlElementType::Identifier => {
                    identifier = Some(self.build_identifier(n, source)?);
                }
                _ => {}
            }
        }

        let data_type = if data_type_tokens.is_empty() {
            None
        }
        else {
            let start = data_type_tokens[0].start;
            let end = data_type_tokens.last().unwrap().end;
            Some(self.get_text(Range { start, end }, source))
        };

        if is_add {
            Ok(ast::AlterAction::AddColumn { name: identifier.ok_or_else(|| OakError::custom_error("Missing column name in ALTER TABLE ADD"))?, data_type, span: node.span() })
        }
        else if is_drop {
            Ok(ast::AlterAction::DropColumn { name: identifier.ok_or_else(|| OakError::custom_error("Missing column name in ALTER TABLE DROP"))?, span: node.span() })
        }
        else if is_rename {
            Ok(ast::AlterAction::RenameTo { new_name: identifier.ok_or_else(|| OakError::custom_error("Missing new name in ALTER TABLE RENAME"))?, span: node.span() })
        }
        else {
            Err(OakError::custom_error("Unknown ALTER action"))
        }
    }

    fn build_table_name<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<TableName, OakError> {
        let mut name = None;
        for child in node.children() {
            match child {
                RedTree::Leaf(t) if t.kind == SqlTokenType::Identifier_ => {
                    name = Some(Identifier { name: self.get_text(t.span.clone(), source), span: t.span.clone() });
                }
                RedTree::Node(n) if n.green.kind == SqlElementType::Identifier => {
                    name = Some(self.build_identifier(n, source)?);
                }
                _ => {}
            }
        }
        Ok(TableName { name: name.ok_or_else(|| OakError::custom_error("Missing table name"))?, span: node.span() })
    }

    fn build_transaction_statement<'a>(&self, node: RedNode<'a, SqlLanguage>) -> Result<TransactionStatement, OakError> {
        let action = node
            .children()
            .find_map(|child| match child {
                RedTree::Leaf(token) => match token.kind {
                    SqlTokenType::Begin => Some(TransactionAction::Begin),
                    SqlTokenType::Commit => Some(TransactionAction::Commit),
                    SqlTokenType::Rollback => Some(TransactionAction::Rollback),
                    _ => None,
                },
                _ => None,
            })
            .ok_or_else(|| OakError::custom_error("Missing transaction action"))?;
        Ok(TransactionStatement { action, span: node.span() })
    }

    fn build_set_names_statement<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<SetNamesStatement, OakError> {
        let tokens = node
            .children()
            .filter_map(|child| match child {
                RedTree::Leaf(token) if token.kind == SqlTokenType::Identifier_ => Some(self.get_text(token.span.clone(), source)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if tokens.first().is_none_or(|token| !token.eq_ignore_ascii_case("NAMES")) {
            return Err(OakError::custom_error("SET NAMES requires a character set and optional collation"));
        }
        let (character_set, collation) = match tokens.as_slice() {
            [_, character_set] => (character_set.clone(), None),
            [_, character_set, collate, collation] if collate.eq_ignore_ascii_case("COLLATE") => (character_set.clone(), Some(collation.clone())),
            _ => return Err(OakError::custom_error("SET NAMES requires a character set and optional collation")),
        };
        Ok(SetNamesStatement { character_set, collation, span: node.span() })
    }

    fn build_identifier<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Identifier, OakError> {
        Ok(Identifier { name: self.get_text(node.span(), source), span: node.span() })
    }

    fn build_join_clause<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<JoinClause, OakError> {
        let mut join_type = JoinType::Inner;
        let mut table = None;
        let mut on = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    SqlTokenType::Inner => join_type = JoinType::Inner,
                    SqlTokenType::Left => join_type = JoinType::Left,
                    SqlTokenType::Right => join_type = JoinType::Right,
                    SqlTokenType::Full => join_type = JoinType::Full,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    SqlElementType::TableName => table = Some(self.build_table_name(n, source)?),
                    SqlElementType::Expression => on = Some(self.build_expression(n, source)?),
                    _ => {}
                },
            }
        }

        Ok(JoinClause { join_type, table: table.ok_or_else(|| OakError::custom_error("Missing table in JOIN"))?, on, span: node.span() })
    }

    fn build_group_by_clause<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<GroupByClause, OakError> {
        let mut columns = Vec::new();
        for child in node.children() {
            if let RedTree::Node(n) = child {
                if n.green.kind == SqlElementType::Expression {
                    columns.push(self.build_expression(n, source)?);
                }
            }
        }
        Ok(GroupByClause { columns, span: node.span() })
    }

    fn build_having_clause<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<HavingClause, OakError> {
        let mut condition = None;
        for child in node.children() {
            if let RedTree::Node(n) = child {
                if n.green.kind == SqlElementType::Expression {
                    condition = Some(self.build_expression(n, source)?);
                }
            }
        }
        Ok(HavingClause { condition: condition.ok_or_else(|| OakError::custom_error("Missing condition in HAVING"))?, span: node.span() })
    }

    fn build_order_by_clause<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<OrderByClause, OakError> {
        let mut items = Vec::new();
        let mut current_expr: Option<(Expression, Range<usize>)> = None;

        for child in node.children() {
            match child {
                RedTree::Node(n) => {
                    if n.green.kind == SqlElementType::Expression {
                        if let Some((expr, span)) = current_expr.take() {
                            items.push(OrderByItem { expr, direction: OrderDirection::Asc, span });
                        }
                        current_expr = Some((self.build_expression(n.clone(), source)?, n.span()));
                    }
                }
                RedTree::Leaf(t) => match t.kind {
                    SqlTokenType::Asc => {
                        if let Some((expr, span)) = current_expr.take() {
                            let total_span = Range { start: span.start, end: t.span.end };
                            items.push(OrderByItem { expr, direction: OrderDirection::Asc, span: total_span });
                        }
                    }
                    SqlTokenType::Desc => {
                        if let Some((expr, span)) = current_expr.take() {
                            let total_span = Range { start: span.start, end: t.span.end };
                            items.push(OrderByItem { expr, direction: OrderDirection::Desc, span: total_span });
                        }
                    }
                    _ => {}
                },
            }
        }

        if let Some((expr, span)) = current_expr {
            items.push(OrderByItem { expr, direction: OrderDirection::Asc, span });
        }

        Ok(OrderByClause { items, span: node.span() })
    }

    fn build_limit_clause<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<LimitClause, OakError> {
        let mut limit = None;
        let mut offset = None;

        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                if t.kind == SqlTokenType::NumberLiteral {
                    let expr = Expression::Literal(Literal::Number(self.get_text(t.span.clone(), source), t.span.clone()));
                    if limit.is_none() {
                        limit = Some(expr);
                    }
                    else {
                        offset = Some(expr);
                    }
                }
            }
            else if let RedTree::Node(n) = child {
                if n.green.kind == SqlElementType::Expression {
                    let expr = self.build_expression(n, source)?;
                    if limit.is_none() {
                        limit = Some(expr);
                    }
                    else {
                        offset = Some(expr);
                    }
                }
            }
        }

        Ok(LimitClause { limit: limit.ok_or_else(|| OakError::custom_error("Missing limit value"))?, offset, span: node.span() })
    }

    fn get_text(&self, span: core::range::Range<usize>, source: &SourceText) -> Arc<str> {
        Arc::from(source.get_text_in(span))
    }

    fn map_binary_op(&self, kind: SqlTokenType) -> Option<BinaryOperator> {
        match kind {
            SqlTokenType::Plus => Some(BinaryOperator::Plus),
            SqlTokenType::Minus => Some(BinaryOperator::Minus),
            SqlTokenType::Star => Some(BinaryOperator::Star),
            SqlTokenType::Slash => Some(BinaryOperator::Slash),
            SqlTokenType::Equal => Some(BinaryOperator::Equal),
            SqlTokenType::NotEqual => Some(BinaryOperator::NotEqual),
            SqlTokenType::Less => Some(BinaryOperator::Less),
            SqlTokenType::LessEqual => Some(BinaryOperator::LessEqual),
            SqlTokenType::Greater => Some(BinaryOperator::Greater),
            SqlTokenType::GreaterEqual => Some(BinaryOperator::GreaterEqual),
            SqlTokenType::And => Some(BinaryOperator::And),
            SqlTokenType::Or => Some(BinaryOperator::Or),
            _ => None,
        }
    }

    fn map_unary_op(&self, kind: SqlTokenType) -> Option<UnaryOperator> {
        match kind {
            SqlTokenType::Plus => Some(UnaryOperator::Plus),
            SqlTokenType::Minus => Some(UnaryOperator::Minus),
            SqlTokenType::Not => Some(UnaryOperator::Not),
            _ => None,
        }
    }

    fn build_statement<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<SqlStatement, OakError> {
        match node.green.kind {
            SqlElementType::SelectStatement => Ok(SqlStatement::Select(self.build_select_statement(node, source)?)),
            SqlElementType::InsertStatement => Ok(SqlStatement::Insert(self.build_insert_statement(node, source)?)),
            SqlElementType::UpdateStatement => Ok(SqlStatement::Update(self.build_update_statement(node, source)?)),
            SqlElementType::DeleteStatement => Ok(SqlStatement::Delete(self.build_delete_statement(node, source)?)),
            SqlElementType::CreateStatement => Ok(SqlStatement::Create(self.build_create_statement(node, source)?)),
            SqlElementType::DropStatement => Ok(SqlStatement::Drop(self.build_drop_statement(node, source)?)),
            SqlElementType::AlterStatement => Ok(SqlStatement::Alter(self.build_alter_statement(node, source)?)),
            SqlElementType::TransactionStatement => Ok(SqlStatement::Transaction(self.build_transaction_statement(node)?)),
            SqlElementType::SetNamesStatement => Ok(SqlStatement::SetNames(self.build_set_names_statement(node, source)?)),
            _ => Err(OakError::custom_error("Unknown statement type")),
        }
    }
}
