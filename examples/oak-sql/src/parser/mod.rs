/// Element types for SQL.
pub mod element_type;

use crate::{SqlElementType, SqlLanguage};
use oak_core::{
    GreenNode, OakError, Parser, ParserState, TextEdit, TokenType,
    parser::{
        ParseCache, ParseOutput, parse_with_lexer,
        pratt::{Associativity, Pratt, PrattParser, binary},
    },
    source::Source,
};

/// Parser for SQL.
///
/// This parser implements the [`Parser`] trait and uses a recursive descent
/// approach combined with a Pratt parser for expressions. It handles various
/// SQL dialects based on the provided [`SqlLanguage`] configuration.
///
/// # Supported Dialects
///
/// - Standard SQL
/// - MySQL (with backticks and case-insensitive matching)
/// - PostgreSQL
/// - SQLite
/// - SQL Server
pub struct SqlParser<'config> {
    pub(crate) config: &'config SqlLanguage,
}

impl<'config> SqlParser<'config> {
    /// Creates a new `SqlParser` with the given configuration.
    pub fn new(config: &'config SqlLanguage) -> Self {
        Self { config }
    }
}

pub(crate) type State<'a, S> = ParserState<'a, SqlLanguage, S>;

impl<'config> Pratt<SqlLanguage> for SqlParser<'config> {
    fn primary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, SqlLanguage> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        match state.peek_kind() {
            Some(Identifier_) => {
                state.bump();
                state.finish_at(cp, SqlElementType::Identifier)
            }
            Some(LeftBracket) => {
                state.bump();
                while state.not_at_end() && state.peek_kind() != Some(RightBracket) {
                    PrattParser::parse(state, 0, self);
                    if !state.eat(Comma) {
                        break;
                    }
                }
                state.expect(RightBracket).ok();
                state.finish_at(cp, SqlElementType::Expression)
            }
            Some(NumberLiteral) | Some(FloatLiteral) | Some(StringLiteral) | Some(BooleanLiteral) | Some(NullLiteral) | Some(True) | Some(False) | Some(Null) => {
                state.bump();
                state.finish_at(cp, SqlElementType::Expression)
            }
            Some(LeftParen) => {
                state.bump();
                PrattParser::parse(state, 0, self);
                state.expect(RightParen).ok();
                state.finish_at(cp, SqlElementType::Expression)
            }
            _ => {
                state.bump();
                state.finish_at(cp, SqlElementType::ErrorNode)
            }
        }
    }

    fn prefix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, SqlLanguage> {
        self.primary(state)
    }

    fn infix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, SqlLanguage>, min_precedence: u8) -> Option<&'a GreenNode<'a, SqlLanguage>> {
        use crate::lexer::SqlTokenType::*;
        let kind = state.peek_kind()?;

        let (prec, assoc) = match kind {
            Or => (1, Associativity::Left),
            And => (2, Associativity::Left),
            Equal | NotEqual | Less | Greater | LessEqual | GreaterEqual | Like | In | Between | Is => (3, Associativity::Left),
            Concat => (4, Associativity::Left),
            Plus | Minus => (10, Associativity::Left),
            Star | Slash | Percent => (11, Associativity::Left),
            DoubleColon => (15, Associativity::Left),
            _ => return None,
        };

        if prec < min_precedence {
            return None;
        }

        let expr_kind = SqlElementType::Expression;
        Some(binary(state, left, kind, prec, assoc, expr_kind, |s, p| PrattParser::parse(s, p, self)))
    }
}

impl<'config> SqlParser<'config> {
    fn parse_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        match state.peek_kind() {
            Some(Select) => self.parse_select(state)?,
            Some(Insert) => self.parse_insert(state)?,
            Some(Update) => self.parse_update(state)?,
            Some(Delete) => self.parse_delete(state)?,
            Some(Create) => self.parse_create(state)?,
            Some(Drop) => self.parse_drop(state)?,
            Some(Alter) => self.parse_alter(state)?,
            Some(Explain) => self.parse_explain(state)?,
            Some(Begin) | Some(Commit) | Some(Rollback) | Some(Transaction) => self.parse_transaction(state)?,
            Some(Pragma) => self.parse_pragma(state)?,
            Some(Show) => self.parse_show(state)?,
            Some(Set) => self.parse_set(state)?,
            _ => {
                let cp = state.checkpoint();
                state.advance_until(Semicolon);
                state.eat(Semicolon);
                state.finish_at(cp, SqlElementType::ErrorNode);
            }
        }
        Ok(())
    }

    fn parse_set<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Set).ok();
        if state.peek_kind() == Some(Identifier_) && state.peek_text().is_some_and(|text| text.eq_ignore_ascii_case("NAMES")) {
            state.bump();
            state.expect(Identifier_).ok();
            if state.peek_kind() == Some(Identifier_) && state.peek_text().is_some_and(|text| text.eq_ignore_ascii_case("COLLATE")) {
                state.bump();
                state.expect(Identifier_).ok();
            }
            state.eat(Semicolon);
            state.finish_at(cp, SqlElementType::SetNamesStatement);
            return Ok(());
        }
        state.expect(Identifier_).ok();
        state.expect(Equal).ok();
        PrattParser::parse(state, 0, self);
        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::Expression);
        Ok(())
    }

    fn parse_explain<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Explain).ok();
        self.parse_statement(state)?;
        state.finish_at(cp, SqlElementType::ExplainStatement);
        Ok(())
    }

    fn parse_transaction<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        match state.peek_kind() {
            Some(Begin) => {
                state.bump();
                state.eat(Transaction);
            }
            Some(Commit) => {
                state.bump();
                state.eat(Transaction);
            }
            Some(Rollback) => {
                state.bump();
                state.eat(Transaction);
            }
            Some(Transaction) => {
                state.bump();
            }
            _ => return Err(OakError::custom_error("Expected BEGIN, COMMIT, or ROLLBACK")),
        }
        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::TransactionStatement);
        Ok(())
    }

    fn parse_pragma<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Pragma).ok();
        state.expect(Identifier_).ok();
        if state.eat(Equal) || state.eat(LeftParen) {
            PrattParser::parse(state, 0, self);
            state.eat(RightParen);
        }
        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::PragmaStatement);
        Ok(())
    }

    fn parse_show<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Show).ok();
        state.expect(Identifier_).ok();
        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::ShowStatement);
        Ok(())
    }

    fn parse_select<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Select).ok();
        state.eat(Distinct);
        state.eat(All);

        // Parse Select Items
        while state.not_at_end() && state.peek_kind() != Some(From) {
            let item_cp = state.checkpoint();
            if state.eat(Star) {
                // All columns
            }
            else {
                PrattParser::parse(state, 0, self);
                if state.eat(As) {
                    state.expect(Identifier_).ok();
                }
                else if state.peek_kind() == Some(Identifier_) {
                    state.bump();
                }
            }
            state.finish_at(item_cp, SqlElementType::SelectItem);

            if !state.eat(Comma) {
                break;
            }
        }

        if state.eat(From) {
            let table_cp = state.checkpoint();
            state.expect(Identifier_).ok(); // TableName
            state.finish_at(table_cp, SqlElementType::TableName);

            // Parse JOIN clauses
            while let Some(kind) = state.peek_kind() {
                if matches!(kind, Join | Inner | Left | Right | Full) {
                    let join_cp = state.checkpoint();
                    if kind != Join {
                        state.bump(); // Inner, Left, etc.
                        state.eat(Outer);
                    }
                    state.expect(Join).ok();

                    let table_cp = state.checkpoint();
                    state.expect(Identifier_).ok(); // Joined TableName
                    state.finish_at(table_cp, SqlElementType::TableName);

                    if state.eat(On) {
                        PrattParser::parse(state, 0, self); // Join condition
                    }
                    state.finish_at(join_cp, SqlElementType::JoinClause);
                }
                else {
                    break;
                }
            }
        }

        if state.eat(Where) {
            PrattParser::parse(state, 0, self);
        }

        if state.eat(Group) {
            let group_cp = state.checkpoint();
            state.expect(By).ok();
            while state.not_at_end() {
                PrattParser::parse(state, 0, self);
                if !state.eat(Comma) {
                    break;
                }
            }
            state.finish_at(group_cp, SqlElementType::GroupByClause);
        }

        if state.eat(Having) {
            let having_cp = state.checkpoint();
            PrattParser::parse(state, 0, self);
            state.finish_at(having_cp, SqlElementType::HavingClause);
        }

        if state.eat(Order) {
            let order_cp = state.checkpoint();
            state.expect(By).ok();
            while state.not_at_end() {
                PrattParser::parse(state, 0, self);
                if state.eat(Asc) || state.eat(Desc) {
                    // Handled
                }
                if !state.eat(Comma) {
                    break;
                }
            }
            state.finish_at(order_cp, SqlElementType::OrderByClause);
        }

        if state.eat(Limit) {
            let limit_cp = state.checkpoint();
            state.expect(NumberLiteral).ok();
            if state.eat(Offset) {
                state.expect(NumberLiteral).ok();
            }
            state.finish_at(limit_cp, SqlElementType::LimitClause);
        }
        else if state.eat(Offset) {
            let offset_cp = state.checkpoint();
            state.expect(NumberLiteral).ok();
            state.finish_at(offset_cp, SqlElementType::LimitClause);
        }

        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::SelectStatement);
        Ok(())
    }

    fn parse_insert<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Insert).ok();
        state.eat(Into);

        let table_cp = state.checkpoint();
        state.expect(Identifier_).ok(); // TableName
        state.finish_at(table_cp, SqlElementType::TableName);

        if state.eat(LeftParen) {
            while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                let col_cp = state.checkpoint();
                state.expect(Identifier_).ok();
                state.finish_at(col_cp, SqlElementType::ColumnName);
                if !state.eat(Comma) {
                    break;
                }
            }
            state.expect(RightParen).ok();
        }

        if state.eat(Values) {
            let values_cp = state.checkpoint();
            while state.eat(LeftParen) {
                let value_list_cp = state.checkpoint();
                while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                    PrattParser::parse(state, 0, self);
                    if !state.eat(Comma) {
                        break;
                    }
                }
                state.expect(RightParen).ok();
                state.finish_at(value_list_cp, SqlElementType::ValueList);

                if !state.eat(Comma) {
                    break;
                }
            }
            state.finish_at(values_cp, SqlElementType::ValueList);
        }
        else if state.peek_kind() == Some(Select) {
            self.parse_select(state)?;
        }

        self.parse_on_conflict(state);
        self.parse_returning_clause(state);

        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::InsertStatement);
        Ok(())
    }

    fn parse_on_conflict<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::SqlTokenType::*;
        if state.eat(On) {
            if state.eat(Conflict) {
                let cp = state.checkpoint();
                // (col1, col2)
                if state.eat(LeftParen) {
                    while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                        state.expect(Identifier_).ok();
                        if !state.eat(Comma) {
                            break;
                        }
                    }
                    state.expect(RightParen).ok();
                }

                if state.eat(Do) {
                    if state.eat(Nothing) {
                        // DO NOTHING
                    }
                    else if state.eat(Update) {
                        state.expect(Set).ok();
                        while state.not_at_end() && state.peek_kind() != Some(Where) && state.peek_kind() != Some(Semicolon) && state.peek_kind() != Some(Returning) {
                            let assign_cp = state.checkpoint();
                            state.expect(Identifier_).ok();
                            state.expect(Equal).ok();
                            PrattParser::parse(state, 0, self);
                            state.finish_at(assign_cp, SqlElementType::Assignment);
                            if !state.eat(Comma) {
                                break;
                            }
                        }
                        if state.eat(Where) {
                            PrattParser::parse(state, 0, self);
                        }
                    }
                }
                state.finish_at(cp, SqlElementType::ConflictClause);
            }
        }
    }

    fn parse_returning_clause<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::SqlTokenType::*;
        if state.eat(Returning) {
            let cp = state.checkpoint();
            while state.not_at_end() && state.peek_kind() != Some(Semicolon) {
                if state.eat(Star) {
                    // All columns
                }
                else {
                    PrattParser::parse(state, 0, self);
                    if state.eat(As) {
                        state.expect(Identifier_).ok();
                    }
                    else if state.peek_kind() == Some(Identifier_) {
                        state.bump();
                    }
                }
                if !state.eat(Comma) {
                    break;
                }
            }
            state.finish_at(cp, SqlElementType::ReturningClause);
        }
    }

    fn parse_update<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Update).ok();

        let table_cp = state.checkpoint();
        state.expect(Identifier_).ok(); // TableName
        state.finish_at(table_cp, SqlElementType::TableName);

        if state.eat(Set) {
            while state.not_at_end() && state.peek_kind() != Some(Where) && state.peek_kind() != Some(Semicolon) {
                let assign_cp = state.checkpoint();

                let col_cp = state.checkpoint();
                state.expect(Identifier_).ok(); // Column
                state.finish_at(col_cp, SqlElementType::ColumnName);

                state.expect(Equal).ok();
                PrattParser::parse(state, 0, self);
                state.finish_at(assign_cp, SqlElementType::Assignment);

                if !state.eat(Comma) {
                    break;
                }
            }
        }

        if state.eat(Where) {
            PrattParser::parse(state, 0, self);
        }

        self.parse_returning_clause(state);

        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::UpdateStatement);
        Ok(())
    }

    fn parse_delete<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Delete).ok();
        state.eat(From);

        let table_cp = state.checkpoint();
        state.expect(Identifier_).ok(); // TableName
        state.finish_at(table_cp, SqlElementType::TableName);

        if state.eat(Where) {
            PrattParser::parse(state, 0, self);
        }

        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::DeleteStatement);
        Ok(())
    }

    fn parse_create<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Create).ok();

        if state.eat(Table) {
            state.eat(If);
            state.eat(Not);
            state.eat(Exists);

            let table_cp = state.checkpoint();
            state.expect(Identifier_).ok(); // TableName
            state.finish_at(table_cp, SqlElementType::TableName);

            if state.eat(LeftParen) {
                while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                    let col_cp = state.checkpoint();

                    let name_cp = state.checkpoint();
                    state.expect(Identifier_).ok(); // Column Name
                    state.finish_at(name_cp, SqlElementType::ColumnName);

                    // Type
                    self.parse_data_type(state);

                    // Constraints
                    while state.not_at_end() && !matches!(state.peek_kind(), Some(Comma) | Some(RightParen)) {
                        if state.eat(Primary) {
                            state.expect(Key).ok();
                        }
                        else if state.eat(Not) {
                            state.expect(Null).ok();
                        }
                        else if state.eat(Null) {
                        }
                        else if state.eat(Unique) {
                        }
                        else if state.eat(Default) {
                            let expr_cp = state.checkpoint();
                            PrattParser::parse(state, 0, self);
                            state.finish_at(expr_cp, SqlElementType::Expression);
                        }
                        else if state.eat(Check) {
                            if state.eat(LeftParen) {
                                let expr_cp = state.checkpoint();
                                PrattParser::parse(state, 0, self);
                                state.finish_at(expr_cp, SqlElementType::Expression);
                                state.expect(RightParen).ok();
                            }
                        }
                        else if state.eat(AutoIncrement) {
                        }
                        else {
                            state.bump();
                        }
                    }

                    state.finish_at(col_cp, SqlElementType::ColumnDefinition);
                    if !state.eat(Comma) {
                        break;
                    }
                }
                state.expect(RightParen).ok();
            }
        }
        else if state.eat(View) {
            let name_cp = state.checkpoint();
            state.expect(Identifier_).ok();
            state.finish_at(name_cp, SqlElementType::Identifier);

            state.expect(As).ok();
            self.parse_select(state)?;
        }
        else if state.peek_kind() == Some(Index) || state.peek_kind() == Some(Unique) {
            state.eat(Unique);
            if state.eat(Index) {
                let name_cp = state.checkpoint();
                state.expect(Identifier_).ok(); // Index Name
                state.finish_at(name_cp, SqlElementType::Identifier);

                state.expect(On).ok();

                let table_cp = state.checkpoint();
                state.expect(Identifier_).ok(); // Table Name
                state.finish_at(table_cp, SqlElementType::TableName);

                if state.eat(LeftParen) {
                    while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                        let col_cp = state.checkpoint();
                        state.expect(Identifier_).ok();
                        state.finish_at(col_cp, SqlElementType::Identifier);
                        state.eat(Comma);
                    }
                    state.expect(RightParen).ok();
                }
            }
        }
        else if state.eat(Database) || state.eat(Schema) {
            state.eat(If);
            state.eat(Not);
            state.eat(Exists);
            let name_cp = state.checkpoint();
            state.expect(Identifier_).ok();
            state.finish_at(name_cp, SqlElementType::Identifier);
        }

        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::CreateStatement);
        Ok(())
    }

    fn parse_drop<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Drop).ok();

        if state.eat(Table) || state.eat(View) || state.eat(Index) || state.eat(Database) || state.eat(Schema) {
            state.eat(If);
            state.eat(Exists);
            let table_cp = state.checkpoint();
            state.expect(Identifier_).ok(); // Object Name
            state.finish_at(table_cp, SqlElementType::TableName);
        }

        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::DropStatement);
        Ok(())
    }

    fn parse_alter<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(Alter).ok();

        if state.eat(Table) {
            let table_cp = state.checkpoint();
            state.expect(Identifier_).ok(); // TableName
            state.finish_at(table_cp, SqlElementType::TableName);

            // Simplified ALTER TABLE actions
            if state.peek_kind() == Some(Add) || state.peek_kind() == Some(Drop) || state.peek_kind() == Some(Rename) {
                let action_cp = state.checkpoint();
                if state.eat(Add) {
                    state.eat(Column);
                    state.expect(Identifier_).ok();
                    // Optional data type
                    self.parse_data_type(state);
                }
                else if state.eat(Drop) {
                    state.eat(Column);
                    state.expect(Identifier_).ok();
                }
                else if state.eat(Rename) {
                    state.eat(To);
                    state.expect(Identifier_).ok();
                }
                state.finish_at(action_cp, SqlElementType::AlterAction);
            }
        }

        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::AlterStatement);
        Ok(())
    }

    fn parse_data_type<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::SqlTokenType::*;
        if state.not_at_end() && !matches!(state.peek_kind(), Some(Comma) | Some(RightParen) | Some(Primary) | Some(Not) | Some(Null) | Some(Unique) | Some(Default) | Some(Check) | Some(Foreign) | Some(References) | Some(Semicolon)) {
            state.bump(); // Type name
            if state.eat(LeftParen) {
                state.expect(NumberLiteral).ok();
                if state.eat(Comma) {
                    state.expect(NumberLiteral).ok();
                }
                state.expect(RightParen).ok();
            }
        }
    }
}

impl<'config> Parser<SqlLanguage> for SqlParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<SqlLanguage>) -> ParseOutput<'a, SqlLanguage> {
        let lexer = crate::lexer::SqlLexer::new(&self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let cp = state.checkpoint();
            while state.not_at_end() {
                if state.current().map(|t| t.kind.is_ignored()).unwrap_or(false) {
                    state.advance();
                    continue;
                }
                self.parse_statement(state)?
            }
            Ok(state.finish_at(cp, SqlElementType::Root))
        })
    }
}
