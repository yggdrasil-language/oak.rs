/// Element types for SQL.
pub mod element_type;

use crate::{SqlElementType, SqlLanguage};
use oak_core::{
    GreenNode, OakError, Parser, ParserState, TextEdit, TokenType,
    parser::{
        ParseCache, ParseOutput, parse_with_lexer,
        pratt::{Associativity, Pratt, PrattParser, binary, unary},
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

/// Returns whether the current token is the bare word `word`, ignoring case.
///
/// `OVER`, `PARTITION`, `ROWS`, `RANGE`, `UNBOUNDED`, `PRECEDING`, `FOLLOWING`
/// and `CURRENT` are not keywords in this lexer: they stay `Identifier_` tokens
/// and only act as window syntax where a window specification is expected, so
/// they keep working as ordinary column names everywhere else.
fn at_word<S: Source + ?Sized>(state: &State<'_, S>, word: &str) -> bool {
    state.peek_kind() == Some(crate::lexer::SqlTokenType::Identifier_) && state.peek_text().is_some_and(|text| text.eq_ignore_ascii_case(word))
}

/// Consumes the current token when it is the bare word `word`.
fn eat_word<S: Source + ?Sized>(state: &mut State<'_, S>, word: &str) -> bool {
    if at_word(state, word) {
        state.bump();
        true
    }
    else {
        false
    }
}

/// Returns whether the current token can start a column of a window clause.
///
/// Inside `OVER (...)` the clause words are syntax, so `ROWS`, `RANGE`,
/// `PARTITION`, `ORDER`, `)`, `,` and the end of the stream all close the list.
fn at_window_item_start<S: Source + ?Sized>(state: &State<'_, S>) -> bool {
    use crate::lexer::SqlTokenType::*;
    matches!(state.peek_kind(), Some(kind) if !matches!(kind, RightParen | Comma | Semicolon | Eof | Order | Asc | Desc | Nulls)) && !at_word(state, "ROWS") && !at_word(state, "RANGE") && !at_word(state, "PARTITION")
}

impl<'config> Pratt<SqlLanguage> for SqlParser<'config> {
    fn primary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, SqlLanguage> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        match state.peek_kind() {
            Some(Identifier_) => {
                state.bump();
                // Qualified column names (`table.column`) are merged into one
                // `Identifier` node so the AST keeps the original spelling. The
                // merge only happens when `Dot` is directly followed by another
                // identifier, so other uses of `.` stay untouched.
                while state.peek_kind() == Some(Dot) && state.peek_kind_at(1) == Some(Identifier_) {
                    state.bump(); // Dot
                    state.bump(); // Identifier_
                }
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
                // `(SELECT ...)` and `(WITH ... SELECT ...)` in an expression position
                // are scalar subqueries. The parenthesized query becomes its own
                // `Subquery` child node, the same shape `EXISTS` and `IN` already use.
                // Any other content stays a grouped expression so `(a + b)`, nested
                // parentheses and `(a, b)` row constructors keep their reading.
                if matches!(state.peek_non_trivia_kind_at(1), Some(Select | With)) {
                    let inner_cp = state.checkpoint();
                    state.expect(LeftParen).ok();
                    if let Err(error) = self.parse_select(state) {
                        state.errors.push(error);
                    }
                    state.expect(RightParen).ok();
                    state.finish_at(inner_cp, SqlElementType::Subquery);
                    return state.finish_at(cp, SqlElementType::Expression);
                }
                state.bump();
                while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                    PrattParser::parse(state, 0, self);
                    if !state.eat(Comma) {
                        break;
                    }
                }
                state.expect(RightParen).ok();
                state.finish_at(cp, SqlElementType::Expression)
            }
            Some(Distinct) => {
                // `DISTINCT` is only meaningful in the SELECT quantifier position,
                // so any other occurrence is reported instead of parsed as a bare
                // column reference.
                state.errors.push(OakError::custom_error("`DISTINCT` is only allowed immediately after `SELECT`"));
                state.bump();
                state.finish_at(cp, SqlElementType::ErrorNode)
            }
            Some(Case) => {
                // CASE WHEN ... THEN ... [ELSE ...] END — bare CASE form only
                state.bump(); // CASE
                while state.peek_kind() == Some(When) {
                    state.bump(); // WHEN
                    PrattParser::parse(state, 0, self);
                    state.expect(Then).ok();
                    PrattParser::parse(state, 0, self);
                }
                if state.eat(Else) {
                    PrattParser::parse(state, 0, self);
                }
                state.expect(End).ok();
                state.finish_at(cp, SqlElementType::Expression)
            }
            Some(Exists) => {
                // `EXISTS (SELECT ...)`: the parenthesized query is its own child
                // node so the builder reads the subquery without scanning leaves.
                // A leading `NOT` is handled by the prefix branch as unary `NOT`
                // and folded back into the predicate by the builder.
                state.bump(); // EXISTS
                let inner_cp = state.checkpoint();
                state.expect(LeftParen).ok();
                if state.peek_kind() != Some(Select) {
                    state.errors.push(OakError::custom_error("`EXISTS` requires a subquery"));
                }
                else if let Err(error) = self.parse_select(state) {
                    state.errors.push(error);
                }
                state.expect(RightParen).ok();
                state.finish_at(inner_cp, SqlElementType::Subquery);
                state.finish_at(cp, SqlElementType::Expression)
            }
            Some(Cast) => {
                // CAST(expr AS type_name) — type_name may be multi-token (e.g. VARCHAR(32))
                state.bump(); // CAST
                state.expect(LeftParen).ok();
                PrattParser::parse(state, 0, self);
                state.expect(As).ok();
                // Consume type tokens until RightParen
                while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                    state.bump();
                }
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
        use crate::lexer::SqlTokenType::*;
        // Unary signs bind tighter than multiplication and division, hence the
        // precedence of 12. Without this branch `SELECT -3` cannot form an
        // `Expression` node and `a > -1` misreads `-` as a binary minus.
        //
        // `NOT` sits between `AND` and the comparison/predicate level: parsing
        // its operand at precedence 3 makes `NOT a = 1` read as `NOT (a = 1)`
        // while `NOT a = 1 AND b = 2` keeps `AND` outside the negation.
        match state.peek_kind() {
            Some(kind @ (Minus | Plus)) => unary(state, kind, 12, SqlElementType::Expression, |s, p| PrattParser::parse(s, p, self)),
            Some(Not) => unary(state, Not, 3, SqlElementType::Expression, |s, p| PrattParser::parse(s, p, self)),
            _ => self.primary(state),
        }
    }

    fn infix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, SqlLanguage>, min_precedence: u8) -> Option<&'a GreenNode<'a, SqlLanguage>> {
        use crate::lexer::SqlTokenType::*;
        let kind = state.peek_kind()?;

        // `NOT IN`, `NOT BETWEEN` and `NOT LIKE` are infix forms: the `NOT` belongs
        // to the operator rather than to a prefix expression. Detect it without
        // consuming anything so a rejected precedence check leaves the state intact.
        let negated_operator = kind == Not && matches!(state.peek_non_trivia_kind_at(1), Some(In | Between | Like));
        let operator = if negated_operator { state.peek_non_trivia_kind_at(1)? } else { kind };

        // Function call postfix: `Identifier '(' args? ')'`.
        if operator == LeftParen && left.kind() == SqlElementType::Identifier {
            let cp = state.checkpoint_before(left);
            state.expect(LeftParen).ok();
            let mut first = true;
            while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                // `DISTINCT` quantifier: skip the token so the following argument is
                // parsed as a normal expression. Only meaningful on COUNT/SUM/AVG;
                // downstream validation handles unsupported functions.
                if state.peek_kind() == Some(Distinct) {
                    state.bump();
                    continue;
                }
                if first {
                    // `COUNT(*)` — the bare Star stays a leaf under the call.
                    if state.peek_kind() == Some(Star) {
                        state.bump();
                        first = false;
                        if !state.eat(Comma) {
                            break;
                        }
                        continue;
                    }
                    first = false;
                }
                PrattParser::parse(state, 0, self);
                if !state.eat(Comma) {
                    break;
                }
            }
            state.expect(RightParen).ok();
            // `OVER (...)` turns the call into a window function. A bare `OVER`
            // that is not followed by `(` keeps its old reading as an alias, so
            // `SELECT COUNT(*) over` does not start a window specification.
            if at_word(state, "OVER") && state.peek_non_trivia_kind_at(1) == Some(LeftParen) {
                self.parse_window_spec(state);
            }
            return Some(state.finish_at(cp, SqlElementType::FunctionCall));
        }

        // `IS [NOT] NULL` — the optional `NOT` is consumed here so the CST keeps the
        // negation next to the operator instead of swallowing the following range.
        if operator == Is {
            if min_precedence > 5 {
                return None;
            }
            let cp = state.checkpoint_before(left);
            state.expect(Is).ok();
            state.eat(Not);
            // `IS [NOT] DISTINCT FROM` is a null-safe comparison with its own
            // semantics. It must not be accepted as a partial `IS` predicate, so
            // report it next to the offending token instead of failing later with
            // an unrelated "unsupported token in WHERE predicate" message.
            if state.peek_kind() == Some(Distinct) {
                state.errors.push(OakError::custom_error("`IS DISTINCT FROM` is not supported"));
            }
            PrattParser::parse(state, 6, self);
            return Some(state.finish_at(cp, SqlElementType::Expression));
        }

        // `BETWEEN low AND high` consumes its own `AND` at a precedence above the
        // logical connectives, so the folded `AND` never escapes into `low`.
        if operator == Between {
            if min_precedence > 5 {
                return None;
            }
            let cp = state.checkpoint_before(left);
            if negated_operator {
                state.bump();
            }
            state.expect(Between).ok();
            PrattParser::parse(state, 6, self);
            state.expect(And).ok();
            PrattParser::parse(state, 6, self);
            return Some(state.finish_at(cp, SqlElementType::Expression));
        }

        // `IN (expr, ...)` and, later, `IN (SELECT ...)`. The parenthesized payload
        // becomes a child node so the builder never mistakes a single element for
        // the whole predicate.
        if operator == In {
            if min_precedence > 5 {
                return None;
            }
            let paren_offset = if negated_operator { 2 } else { 1 };
            if state.peek_non_trivia_kind_at(paren_offset) != Some(LeftParen) {
                return None;
            }
            let cp = state.checkpoint_before(left);
            if negated_operator {
                state.bump();
            }
            state.expect(In).ok();
            let inner_cp = state.checkpoint();
            state.expect(LeftParen).ok();
            let is_subquery = state.peek_kind() == Some(Select);
            if is_subquery {
                if let Err(error) = self.parse_select(state) {
                    // `infix` cannot return an error, so keep the diagnostic with the
                    // parse state instead of silently continuing with a partial query.
                    state.errors.push(error);
                }
            }
            else {
                while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                    PrattParser::parse(state, 0, self);
                    if !state.eat(Comma) {
                        break;
                    }
                }
            }
            state.expect(RightParen).ok();
            state.finish_at(inner_cp, if is_subquery { SqlElementType::Subquery } else { SqlElementType::Expression });
            return Some(state.finish_at(cp, SqlElementType::Expression));
        }

        let (prec, assoc) = match operator {
            Or => (1, Associativity::Left),
            And => (2, Associativity::Left),
            Equal | NotEqual | Less | Greater | LessEqual | GreaterEqual | Like => (4, Associativity::Left),
            Concat => (6, Associativity::Left),
            Plus | Minus => (10, Associativity::Left),
            Star | Slash | Percent => (11, Associativity::Left),
            DoubleColon => (15, Associativity::Left),
            _ => return None,
        };

        if prec < min_precedence {
            return None;
        }

        if negated_operator {
            state.bump();
        }
        let expr_kind = SqlElementType::Expression;
        Some(binary(state, left, operator, prec, assoc, expr_kind, |s, p| PrattParser::parse(s, p, self)))
    }
}

impl<'config> SqlParser<'config> {
    /// Parses `OVER (PARTITION BY ... ORDER BY ... [ROWS|RANGE frame])`.
    ///
    /// Each clause becomes its own child node so the builder reads the window
    /// without scanning leaves. Forms the AST cannot express are reported here
    /// rather than silently accepted.
    fn parse_window_spec<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, SqlLanguage> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // OVER
        state.expect(LeftParen).ok();

        if state.peek_kind() == Some(RightParen) {
            state.errors.push(OakError::custom_error("`OVER` requires `PARTITION BY`, `ORDER BY`, or a window frame"));
        }
        else {
            if at_word(state, "PARTITION") {
                self.parse_window_partition(state);
            }
            // A second `PARTITION BY` is not part of the syntax. Report it and
            // consume it so the later clauses still parse as their own nodes.
            if at_word(state, "PARTITION") {
                state.errors.push(OakError::custom_error("Duplicate `PARTITION BY` in window specification"));
                self.parse_window_partition(state);
            }
            if state.at(Order) {
                self.parse_window_order_by(state);
            }
            if at_word(state, "ROWS") || at_word(state, "RANGE") {
                self.parse_window_frame(state);
            }
            if state.peek_kind() != Some(RightParen) {
                state.errors.push(OakError::custom_error("Unexpected token in window specification"));
                // Keep the tree covering the source so later statements keep
                // their absolute offsets.
                state.advance_until(RightParen);
            }
        }

        state.expect(RightParen).ok();
        state.finish_at(cp, SqlElementType::WindowSpec)
    }

    /// Parses `PARTITION BY column, ...` into a window clause node.
    fn parse_window_partition<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // PARTITION
        state.expect(By).ok();
        self.parse_window_columns(state, "`PARTITION BY`");
        state.finish_at(cp, SqlElementType::WindowPartition);
    }

    /// Parses `ORDER BY column [ASC|DESC] [NULLS FIRST|LAST], ...`.
    fn parse_window_order_by<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // ORDER
        state.expect(By).ok();
        loop {
            if !at_window_item_start(state) {
                break;
            }
            let item = PrattParser::parse(state, 0, self);
            if item.kind() != SqlElementType::Identifier {
                state.errors.push(OakError::custom_error("`ORDER BY` in a window accepts only column references"));
            }
            if matches!(state.peek_kind(), Some(Asc | Desc)) {
                state.bump();
            }
            if state.at(Nulls) {
                state.bump();
                if matches!(state.peek_kind(), Some(First | Last)) {
                    state.bump();
                }
            }
            if !state.eat(Comma) {
                break;
            }
        }
        state.finish_at(cp, SqlElementType::WindowOrderBy);
    }

    /// Parses the comma separated column list of `PARTITION BY`.
    fn parse_window_columns<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, clause: &str) {
        let mut count = 0usize;
        loop {
            if !at_window_item_start(state) {
                break;
            }
            let item = PrattParser::parse(state, 0, self);
            if item.kind() != SqlElementType::Identifier {
                state.errors.push(OakError::custom_error(format!("{clause} in a window accepts only column references")));
            }
            count += 1;
            if !state.eat(crate::lexer::SqlTokenType::Comma) {
                break;
            }
        }
        if count == 0 {
            state.errors.push(OakError::custom_error(format!("{clause} requires at least one column")));
        }
    }

    /// Parses a `ROWS` / `RANGE` frame, including its optional `BETWEEN` pair.
    ///
    /// The bounds it reads also drive the frame validation: a frame may not start
    /// with `UNBOUNDED FOLLOWING` and may not start after its end bound.
    fn parse_window_frame<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::{ast::WindowFrameBound, lexer::SqlTokenType::*};
        let cp = state.checkpoint();
        state.bump(); // ROWS | RANGE
        if state.at(Between) {
            state.bump();
            let start = self.parse_window_frame_bound(state);
            state.expect(And).ok();
            let end = self.parse_window_frame_bound(state);
            if let (Some(start), Some(end)) = (&start, &end) {
                if matches!(start, WindowFrameBound::UnboundedFollowing) {
                    state.errors.push(OakError::custom_error("`UNBOUNDED FOLLOWING` cannot start a window frame"));
                }
                else if start.is_after(end) {
                    state.errors.push(OakError::custom_error("A window frame cannot start after its end bound"));
                }
            }
        }
        else {
            if let Some(start) = self.parse_window_frame_bound(state) {
                if matches!(start, WindowFrameBound::UnboundedFollowing) {
                    state.errors.push(OakError::custom_error("`UNBOUNDED FOLLOWING` cannot start a window frame"));
                }
            }
        }
        if at_word(state, "EXCLUDE") {
            state.errors.push(OakError::custom_error("`EXCLUDE` is not supported in a window frame"));
            state.advance_until(RightParen);
        }
        state.finish_at(cp, SqlElementType::WindowFrame);
    }

    /// Parses one `UNBOUNDED PRECEDING` / `n PRECEDING` / `CURRENT ROW` /
    /// `n FOLLOWING` / `UNBOUNDED FOLLOWING` frame bound.
    ///
    /// Returns the bound when it is one the AST can express, so the caller can
    /// validate the resulting frame before it reaches the builder.
    fn parse_window_frame_bound<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Option<crate::ast::WindowFrameBound> {
        use crate::{ast::WindowFrameBound, lexer::SqlTokenType::*};
        if at_word(state, "UNBOUNDED") {
            state.bump();
            if eat_word(state, "PRECEDING") {
                return Some(WindowFrameBound::UnboundedPreceding);
            }
            if eat_word(state, "FOLLOWING") {
                return Some(WindowFrameBound::UnboundedFollowing);
            }
            state.errors.push(OakError::custom_error("`UNBOUNDED` must be followed by `PRECEDING` or `FOLLOWING`"));
            return None;
        }
        if at_word(state, "CURRENT") {
            state.bump();
            if state.expect(Row).is_err() {
                return None;
            }
            return Some(WindowFrameBound::CurrentRow);
        }
        if state.at(NumberLiteral) {
            let offset = state.peek_text().and_then(|text| text.parse::<u64>().ok());
            state.bump();
            let Some(offset) = offset
            else {
                state.errors.push(OakError::custom_error("A window frame offset must be a non-negative integer"));
                return None;
            };
            if eat_word(state, "PRECEDING") {
                return Some(WindowFrameBound::Preceding(offset));
            }
            if eat_word(state, "FOLLOWING") {
                return Some(WindowFrameBound::Following(offset));
            }
            state.errors.push(OakError::custom_error("A window frame offset must be followed by `PRECEDING` or `FOLLOWING`"));
            return None;
        }
        state.errors.push(OakError::custom_error("Unsupported window frame bound"));
        None
    }

    fn parse_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        match state.peek_kind() {
            Some(Select) | Some(With) => self.parse_select(state)?,
            Some(Insert) => self.parse_insert(state)?,
            Some(Update) => self.parse_update(state)?,
            Some(Delete) => self.parse_delete(state)?,
            Some(Create) => self.parse_create(state)?,
            Some(Drop) => self.parse_drop(state)?,
            Some(Alter) => self.parse_alter(state)?,
            Some(Explain) => self.parse_explain(state)?,
            Some(Begin) | Some(Commit) | Some(Rollback) | Some(Savepoint) | Some(Release) | Some(Transaction) => self.parse_transaction(state)?,
            Some(Pragma) => self.parse_pragma(state)?,
            Some(Show) => self.parse_show(state)?,
            Some(Set) => self.parse_set(state)?,
            _ => {
                if state.peek_kind() == Some(Distinct) {
                    state.errors.push(OakError::custom_error("`DISTINCT` is only allowed immediately after `SELECT`"));
                }
                // Every skipped token is pushed into the error node, so the tree
                // still covers the whole source and later statements keep their
                // absolute offsets.
                let cp = state.checkpoint();
                while state.not_at_end() && state.peek_kind() != Some(Semicolon) {
                    state.bump();
                }
                state.eat(Semicolon);
                state.finish_at(cp, SqlElementType::ErrorNode);
            }
        }
        Self::reject_unexpected_tail(state, "statement");
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
        // `SET [GLOBAL|SESSION|LOCAL] <name> = <expression>` is how a driver applies
        // character set, time zone, `sql_mode` and similar connection settings.
        let scoped = state.peek_kind() == Some(Identifier_) && state.peek_text().is_some_and(|text| crate::ast::SessionScope::from_word(&text).is_some());
        if scoped {
            state.bump();
        }
        state.expect(Identifier_).ok();
        state.expect(Equal).ok();
        PrattParser::parse(state, 0, self);
        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::SetVariableStatement);
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
                if state.eat(To) {
                    // `ROLLBACK TO [SAVEPOINT] name`
                    state.eat(Savepoint);
                    state.expect(Identifier_).ok();
                }
                else {
                    // plain `ROLLBACK` / `ROLLBACK TRANSACTION`
                    state.eat(Transaction);
                }
            }
            Some(Savepoint) => {
                // `SAVEPOINT name`
                state.bump();
                state.expect(Identifier_).ok();
            }
            Some(Release) => {
                // `RELEASE [SAVEPOINT] name`
                state.bump();
                state.eat(Savepoint);
                state.expect(Identifier_).ok();
            }
            Some(Transaction) => {
                state.bump();
            }
            _ => return Err(OakError::custom_error("Expected BEGIN, COMMIT, ROLLBACK, SAVEPOINT, or RELEASE")),
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
        let scoped = state.peek_kind() == Some(Identifier_) && state.peek_text().is_some_and(|text| crate::ast::SessionScope::from_word(&text).is_some());
        if scoped {
            state.bump();
        }
        state.expect(Identifier_).ok();
        if state.eat(Like) {
            state.expect(StringLiteral).ok();
        }
        state.eat(Semicolon);
        state.finish_at(cp, SqlElementType::ShowStatement);
        Ok(())
    }

    /// Tokens that may legitimately follow a `WHERE` predicate. Anything else means
    /// the predicate was only partially parsed, which must surface as an error
    /// instead of silently accepting the truncated expression as the whole filter.
    fn is_where_terminator(kind: Option<crate::lexer::SqlTokenType>) -> bool {
        use crate::lexer::SqlTokenType::*;
        matches!(kind, None | Some(Group | Having | Order | Limit | Offset | Semicolon | Eof | RightParen | Returning | Union | Intersect | Except))
    }

    /// Parses the explicit `ALL` / `DISTINCT` quantifier that may follow `SELECT`.
    ///
    /// The quantifier gets its own node so the AST never loses whether the query
    /// asks for duplicate elimination (`DISTINCT`) or `DISTINCT ON` evaluation.
    fn parse_select_quantifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::SqlTokenType::*;
        if !matches!(state.peek_kind(), Some(All | Distinct)) {
            return;
        }
        let cp = state.checkpoint();
        if state.eat(All) {
            state.finish_at(cp, SqlElementType::SelectQuantifier);
            return;
        }
        state.expect(Distinct).ok();
        if state.eat(On) {
            if !state.eat(LeftParen) {
                state.errors.push(OakError::custom_error("`DISTINCT ON` requires a parenthesized expression list"));
            }
            else {
                if state.peek_kind() == Some(RightParen) {
                    state.errors.push(OakError::custom_error("`DISTINCT ON` requires at least one expression"));
                }
                while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                    PrattParser::parse(state, 0, self);
                    if !state.eat(Comma) {
                        break;
                    }
                }
                state.expect(RightParen).ok();
            }
        }
        state.finish_at(cp, SqlElementType::SelectQuantifier);
    }

    /// Tokens that may legitimately follow the last `GROUP BY` item.
    fn is_group_by_terminator(kind: Option<crate::lexer::SqlTokenType>) -> bool {
        use crate::lexer::SqlTokenType::*;
        matches!(kind, None | Some(Having | Order | Limit | Offset | Semicolon | Eof | RightParen | Returning | Union | Intersect | Except))
    }

    /// Tokens that may legitimately follow the `HAVING` predicate.
    fn is_having_terminator(kind: Option<crate::lexer::SqlTokenType>) -> bool {
        use crate::lexer::SqlTokenType::*;
        matches!(kind, None | Some(Order | Limit | Offset | Semicolon | Eof | RightParen | Returning | Union | Intersect | Except))
    }

    /// Tokens that may legitimately follow the last `ORDER BY` item.
    fn is_order_by_terminator(kind: Option<crate::lexer::SqlTokenType>) -> bool {
        use crate::lexer::SqlTokenType::*;
        matches!(kind, None | Some(Limit | Offset | Semicolon | Eof | RightParen | Returning))
    }

    /// Tokens that may start another statement.
    fn is_statement_start(kind: Option<crate::lexer::SqlTokenType>) -> bool {
        use crate::lexer::SqlTokenType::*;
        matches!(kind, Some(Select | With | Insert | Update | Delete | Create | Drop | Alter | Explain | Begin | Commit | Rollback | Transaction | Pragma | Show | Set))
    }

    /// Reports tokens that no statement or clause accepted and consumes them, so
    /// no tail can be silently dropped while the tree still covers the source.
    fn reject_unexpected_tail<'a, S: Source + ?Sized>(state: &mut State<'a, S>, context: &str) {
        use crate::lexer::SqlTokenType::*;
        let Some(kind) = state.peek_kind()
        else {
            return;
        };
        if matches!(kind, Eof | Semicolon) || Self::is_statement_start(Some(kind)) {
            return;
        }
        state.errors.push(OakError::custom_error(format!("unexpected token after {context}: {kind:?}")));
        while state.not_at_end() && !matches!(state.peek_kind(), Some(Semicolon | Eof)) && !Self::is_statement_start(state.peek_kind()) {
            state.bump();
        }
        state.eat(Semicolon);
    }

    /// Parses a query expression: an optional `WITH` clause, the `SELECT` body, an
    /// optional set operation, and the `ORDER BY` / `LIMIT` clauses that apply to
    /// the whole result.
    fn parse_select<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        if state.peek_kind() == Some(With) {
            self.parse_with_clause(state)?;
        }
        self.parse_select_terms(state)?;

        let mut set_operation = false;
        if Self::set_operator_kind(state.peek_kind()).is_some() {
            set_operation = true;
            // The first operand ends before its operator, so closing it here keeps
            // everything outside the set-operation node cleanly separated.
            state.finish_at(cp, SqlElementType::SelectStatement);

            // Collect the rest of the chain: operator + operand, repeated.
            while let Some(operator) = Self::set_operator_kind(state.peek_kind()) {
                state.bump();
                if operator == Union {
                    state.eat(All);
                }
                let operand_cp = state.checkpoint();
                self.parse_select_terms(state)?;
                state.finish_at(operand_cp, SqlElementType::SelectStatement);
            }
        }

        self.parse_order_by_limit(state)?;
        state.eat(Semicolon);
        state.finish_at(cp, if set_operation { SqlElementType::SetOperationStatement } else { SqlElementType::SelectStatement });
        Ok(())
    }

    /// Parses the non-recursive `WITH name [(col, ...)] AS (query), ...` prefix.
    ///
    /// `RECURSIVE` is not a keyword, so it is only treated as the modifier when a
    /// CTE name follows it. That keeps an ordinary CTE named `recursive` usable
    /// while `WITH RECURSIVE` is still rejected, because a self-referencing CTE
    /// cannot be evaluated by the current AST.
    fn parse_with_clause<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        let cp = state.checkpoint();
        state.expect(With).ok();
        if state.peek_kind() == Some(Identifier_) && state.peek_text().is_some_and(|text| text.eq_ignore_ascii_case("RECURSIVE")) && state.peek_non_trivia_kind_at(1) == Some(Identifier_) {
            return Err(OakError::custom_error("recursive CTEs are not supported"));
        }
        loop {
            let cte_cp = state.checkpoint();
            state.expect(Identifier_).ok();
            if state.eat(LeftParen) {
                while state.peek_kind() == Some(Identifier_) {
                    state.bump();
                    if !state.eat(Comma) {
                        break;
                    }
                }
                state.expect(RightParen).ok();
            }
            state.expect(As).ok();
            state.expect(LeftParen).ok();
            self.parse_select(state)?;
            state.expect(RightParen).ok();
            state.finish_at(cte_cp, SqlElementType::CteDefinition);
            if !state.eat(Comma) {
                break;
            }
        }
        state.finish_at(cp, SqlElementType::WithClause);
        Ok(())
    }

    /// Returns the token kind that starts a set operation, if one is next.
    fn set_operator_kind(kind: Option<crate::lexer::SqlTokenType>) -> Option<crate::lexer::SqlTokenType> {
        use crate::lexer::SqlTokenType::*;
        match kind {
            Some(operator @ (Union | Intersect | Except)) => Some(operator),
            _ => None,
        }
    }

    /// Parses the `SELECT ... [HAVING]` body.
    ///
    /// `ORDER BY` and `LIMIT` are left to the caller: after a set operation they
    /// apply to the whole result instead of the last branch.
    fn parse_select_terms<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        state.expect(Select).ok();
        self.parse_select_quantifier(state);

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
            self.parse_table_source(state);

            // Parse JOIN clauses
            while let Some(kind) = state.peek_kind() {
                if matches!(kind, Join | Inner | Left | Right | Full | Cross) {
                    let join_cp = state.checkpoint();
                    if kind != Join {
                        state.bump(); // Inner, Left, Right, Full, Cross
                        if kind != Cross {
                            state.eat(Outer);
                        }
                    }
                    state.expect(Join).ok();

                    self.parse_table_source(state);

                    // CROSS JOIN has no ON clause; other join types may have one.
                    if kind != Cross && state.eat(On) {
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
            if !Self::is_where_terminator(state.peek_kind()) {
                return Err(OakError::custom_error("unsupported token in WHERE predicate"));
            }
        }

        if state.eat(Group) {
            let group_cp = state.checkpoint();
            state.expect(By).ok();
            let mut first = true;
            while state.not_at_end() {
                if Self::is_group_by_terminator(state.peek_kind()) {
                    state.errors.push(OakError::custom_error(if first { "`GROUP BY` requires at least one item" } else { "trailing comma in `GROUP BY`" }));
                    break;
                }
                first = false;
                // GROUPING SETS ((a), (b), (a, b)) — two-keyword entry
                if state.peek_kind() == Some(Grouping) {
                    state.bump(); // GROUPING (Leaf stays as child of GroupByClause)
                    if state.eat(Sets) {
                        state.expect(LeftParen).ok();
                        // Inner groups: ((a), (b), ...)
                        if state.peek_kind() == Some(LeftParen) {
                            while state.not_at_end() {
                                if state.peek_kind() != Some(LeftParen) {
                                    break;
                                }
                                state.bump(); // inner (
                                // items inside ( ... ) — may be empty
                                if state.peek_kind() != Some(RightParen) {
                                    let mut item_first = true;
                                    while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                                        if !item_first {
                                            if !state.eat(Comma) {
                                                break;
                                            }
                                        }
                                        PrattParser::parse(state, 0, self);
                                        item_first = false;
                                    }
                                }
                                state.expect(RightParen).ok(); // inner )
                                if !state.eat(Comma) {
                                    break;
                                }
                            }
                        }
                        state.expect(RightParen).ok(); // outer )
                    }
                }
                // ROLLUP(a, b, c)
                else if state.peek_kind() == Some(Rollup) {
                    state.bump(); // ROLLUP
                    state.expect(LeftParen).ok();
                    if state.peek_kind() != Some(RightParen) {
                        let mut item_first = true;
                        while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                            if !item_first {
                                if !state.eat(Comma) {
                                    break;
                                }
                            }
                            PrattParser::parse(state, 0, self);
                            item_first = false;
                        }
                    }
                    state.expect(RightParen).ok();
                }
                // CUBE(a, b, c)
                else if state.peek_kind() == Some(Cube) {
                    state.bump(); // CUBE
                    state.expect(LeftParen).ok();
                    if state.peek_kind() != Some(RightParen) {
                        let mut item_first = true;
                        while state.not_at_end() && state.peek_kind() != Some(RightParen) {
                            if !item_first {
                                if !state.eat(Comma) {
                                    break;
                                }
                            }
                            PrattParser::parse(state, 0, self);
                            item_first = false;
                        }
                    }
                    state.expect(RightParen).ok();
                }
                // Plain expression or identifier or position literal
                else {
                    PrattParser::parse(state, 0, self);
                }
                if !state.eat(Comma) {
                    break;
                }
            }
            if !Self::is_group_by_terminator(state.peek_kind()) {
                return Err(OakError::custom_error("unsupported token in GROUP BY clause"));
            }
            state.finish_at(group_cp, SqlElementType::GroupByClause);
        }

        if state.eat(Having) {
            let having_cp = state.checkpoint();
            if Self::is_having_terminator(state.peek_kind()) {
                state.errors.push(OakError::custom_error("`HAVING` requires a predicate"));
            }
            else {
                PrattParser::parse(state, 0, self);
                if !Self::is_having_terminator(state.peek_kind()) {
                    return Err(OakError::custom_error("unsupported token in HAVING predicate"));
                }
            }
            state.finish_at(having_cp, SqlElementType::HavingClause);
        }

        Ok(())
    }

    /// Parses the trailing `ORDER BY`, `LIMIT` and `OFFSET` clauses.
    fn parse_order_by_limit<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::SqlTokenType::*;
        if state.eat(Order) {
            let order_cp = state.checkpoint();
            state.expect(By).ok();
            let mut first = true;
            while state.not_at_end() {
                if Self::is_order_by_terminator(state.peek_kind()) {
                    state.errors.push(OakError::custom_error(if first { "`ORDER BY` requires at least one item" } else { "trailing comma in `ORDER BY`" }));
                    break;
                }
                first = false;
                PrattParser::parse(state, 0, self);
                if state.eat(Asc) || state.eat(Desc) {
                    // Handled
                }
                if state.eat(Nulls) {
                    if !state.eat(First) && !state.eat(Last) {
                        state.errors.push(OakError::custom_error("`NULLS` must be followed by `FIRST` or `LAST`"));
                    }
                }
                if !state.eat(Comma) {
                    break;
                }
            }
            if !Self::is_order_by_terminator(state.peek_kind()) {
                return Err(OakError::custom_error("unsupported token in ORDER BY clause"));
            }
            state.finish_at(order_cp, SqlElementType::OrderByClause);
        }

        // The checkpoint is taken before the leading keyword so the `LIMIT` /
        // `OFFSET` token stays inside the node: a standalone `OFFSET` is a valid
        // clause and the builder must dispatch on the keyword, not on position.
        let clause_cp = state.checkpoint();
        if state.eat(Limit) {
            PrattParser::parse(state, 0, self);
            if state.eat(Offset) {
                PrattParser::parse(state, 0, self);
            }
            state.finish_at(clause_cp, SqlElementType::LimitClause);
        }
        else if state.eat(Offset) {
            PrattParser::parse(state, 0, self);
            state.finish_at(clause_cp, SqlElementType::LimitClause);
        }

        Ok(())
    }

    /// Parses one `FROM` / `JOIN` row source: a named table or a derived table.
    ///
    /// Both forms keep their optional alias, and a derived table is recognised by
    /// the `(` that opens its subquery. The node kind stays `TableName` so the
    /// builder tells `FROM users u` from `FROM (SELECT ...) AS x` by the nested
    /// `SELECT` rather than by a second CST node shape.
    fn parse_table_source<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::SqlTokenType::*;
        if state.peek_kind() != Some(LeftParen) {
            self.parse_table_name(state);
            return;
        }
        let table_cp = state.checkpoint();
        state.expect(LeftParen).ok();
        if let Err(error) = self.parse_select(state) {
            // `parse_table_source` has no error channel of its own, so the
            // diagnostic is kept with the parse state like the `IN` subquery.
            state.errors.push(error);
        }
        state.expect(RightParen).ok();
        if state.eat(As) {
            state.expect(Identifier_).ok();
        }
        else if state.peek_kind() == Some(Identifier_) {
            state.bump();
        }
        state.finish_at(table_cp, SqlElementType::TableName);
    }

    /// Parses one `FROM` / `JOIN` table name, including its optional alias.
    ///
    /// Both `FROM users AS u` and `FROM users u` are accepted, so a self join and
    /// any other aliased reference can be written. The alias is only taken when a
    /// bare identifier follows, which keeps `FROM users WHERE ...` unambiguous.
    fn parse_table_name<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::SqlTokenType::*;
        let table_cp = state.checkpoint();
        state.expect(Identifier_).ok(); // TableName
        if state.eat(As) {
            state.expect(Identifier_).ok();
        }
        else if state.peek_kind() == Some(Identifier_) {
            state.bump();
        }
        state.finish_at(table_cp, SqlElementType::TableName);
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
                        while state.not_at_end() && !matches!(state.peek_kind(), Some(Where | Semicolon | Returning | Eof)) {
                            let assign_cp = state.checkpoint();

                            let col_cp = state.checkpoint();
                            state.expect(Identifier_).ok();
                            state.finish_at(col_cp, SqlElementType::ColumnName);

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
            while state.not_at_end() && !matches!(state.peek_kind(), Some(Semicolon | Eof)) {
                // A returned item has the same shape as a select item, so the
                // builder can reuse `build_select_item` for both.
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
            if !Self::is_where_terminator(state.peek_kind()) {
                return Err(OakError::custom_error("unsupported token in WHERE predicate"));
            }
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
            if !Self::is_where_terminator(state.peek_kind()) {
                return Err(OakError::custom_error("unsupported token in WHERE predicate"));
            }
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
                // Leading trivia has to be recorded as leaves as well. Advancing
                // without pushing it would shorten the tree, which shifts every
                // later absolute offset away from the real source position.
                if state.current().map(|t| t.kind.is_ignored()).unwrap_or(false) {
                    state.bump();
                    continue;
                }
                self.parse_statement(state)?
            }
            Ok(state.finish_at(cp, SqlElementType::Root))
        })
    }
}
