use oak_core::TokenType;
/// Element types for the Ruby language.
pub mod element_type;

use crate::{
    language::RubyLanguage,
    lexer::{RubyLexer, token_type::RubyTokenType},
};
use oak_core::{
    GreenNode, OakError, TextEdit,
    parser::{
        ParseCache, ParseOutput, Parser, ParserState, parse_with_lexer,
        pratt::{Associativity, Pratt, PrattParser},
    },
    source::Source,
};

pub(crate) type State<'a, S> = ParserState<'a, RubyLanguage, S>;

/// A parser for the Ruby language.
pub struct RubyParser<'config> {
    pub(crate) config: &'config RubyLanguage,
}

impl<'config> RubyParser<'config> {
    /// Creates a new `RubyParser` with the given configuration.
    pub fn new(config: &'config RubyLanguage) -> Self {
        Self { config }
    }

    /// 语句间隙：空格 / 注释 / 换行。
    fn skip_stmt_gap<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::RubyTokenType::*;
        while matches!(state.peek_kind(), Some(Whitespace | Comment | Newline)) {
            state.bump();
        }
    }

    /// 表达式内间隙：运算符后允许换行续行。
    fn skip_expr_gap<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::RubyTokenType::*;
        while matches!(state.peek_kind(), Some(Whitespace | Comment | Newline)) {
            state.bump();
        }
    }

    /// 仅同行修饰符：跳过空格/注释后若是 `if`/`unless`。
    fn peek_same_line_modifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Option<bool> {
        use crate::lexer::token_type::RubyTokenType::*;
        while matches!(state.peek_kind(), Some(Whitespace | Comment)) {
            state.bump();
        }
        match state.peek_kind() {
            Some(If) => Some(false),
            Some(Unless) => Some(true),
            _ => None,
        }
    }

    fn parse_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        self.skip_stmt_gap(state);
        // 体循环边界：不要把 `end`/`else` 当表达式吞掉。
        if matches!(
            state.peek_kind(),
            Some(End | Else | Elsif | When | Rescue | Ensure) | None
        ) {
            return Ok(());
        }
        if matches!(
            state.peek_kind(),
            Some(Identifier | Constant | GlobalVariable | InstanceVariable | ClassVariable)
        ) && matches!(
            state.peek_non_trivia_kind_at(1),
            Some(
                Assign | PlusAssign | MinusAssign | MultiplyAssign | DivideAssign | OrOrAssign
                    | AndAndAssign
            )
        ) {
            let checkpoint = state.checkpoint();
            state.bump();
            state.bump();
            self.skip_expr_gap(state);
            PrattParser::parse(state, 0, self);
            // 行尾修饰符：同一行的 `if`/`unless`（换行后的 if 是新语句）。
            if let Some(is_unless) = self.peek_same_line_modifier(state) {
                state.bump();
                self.skip_expr_gap(state);
                PrattParser::parse(state, 0, self);
                state.eat(Semicolon);
                state.eat(Newline);
                state.finish_at(
                    checkpoint,
                    if is_unless {
                        crate::parser::element_type::RubyElementType::UnlessStatement
                    } else {
                        crate::parser::element_type::RubyElementType::IfStatement
                    },
                );
            } else {
                state.eat(Semicolon);
                state.eat(Newline);
                state.finish_at(checkpoint, crate::parser::element_type::RubyElementType::AssignmentStatement);
            }
            return Ok(());
        }
        match state.peek_kind() {
            Some(Def) => self.parse_method_def(state)?,
            Some(Class) => self.parse_class_def(state)?,
            Some(Module) => self.parse_module_def(state)?,
            Some(If) => self.parse_if_stmt(state)?,
            Some(Unless) => self.parse_unless_stmt(state)?,
            Some(While) => self.parse_while_stmt(state)?,
            Some(Until) => self.parse_until_stmt(state)?,
            Some(For) => self.parse_for_stmt(state)?,
            Some(Case) => self.parse_case_stmt(state)?,
            Some(Begin) => self.parse_begin_stmt(state)?,
            Some(Return) => self.parse_return_stmt(state)?,
            Some(Break) => {
                let cp = state.checkpoint();
                state.bump();
                // `break if cond` / `break unless cond`
                if let Some(is_unless) = self.peek_same_line_modifier(state) {
                    state.bump();
                    self.skip_expr_gap(state);
                    PrattParser::parse(state, 0, self);
                    state.eat(Semicolon);
                    state.eat(Newline);
                    state.finish_at(
                        cp,
                        if is_unless {
                            crate::parser::element_type::RubyElementType::UnlessStatement
                        } else {
                            crate::parser::element_type::RubyElementType::IfStatement
                        },
                    );
                } else {
                    state.eat(Semicolon);
                    state.eat(Newline);
                    state.finish_at(cp, crate::parser::element_type::RubyElementType::BreakStatement);
                }
            }
            Some(Next) => {
                let cp = state.checkpoint();
                state.bump();
                // `next if cond`：先记成 if，体里是空 next（宿主过渡当 continue 近似）。
                if let Some(is_unless) = self.peek_same_line_modifier(state) {
                    state.bump();
                    self.skip_expr_gap(state);
                    PrattParser::parse(state, 0, self);
                    state.eat(Semicolon);
                    state.eat(Newline);
                    state.finish_at(
                        cp,
                        if is_unless {
                            crate::parser::element_type::RubyElementType::UnlessStatement
                        } else {
                            crate::parser::element_type::RubyElementType::IfStatement
                        },
                    );
                } else {
                    state.eat(Semicolon);
                    state.eat(Newline);
                    state.finish_at(cp, crate::parser::element_type::RubyElementType::BreakStatement);
                }
            }
            _ => {
                let cp = state.checkpoint();
                PrattParser::parse(state, 0, self);
                // 语句修饰符：`stmt if cond` / `stmt unless cond`
                if let Some(is_unless) = self.peek_same_line_modifier(state) {
                    state.bump();
                    self.skip_expr_gap(state);
                    PrattParser::parse(state, 0, self);
                    state.eat(Semicolon);
                    state.eat(Newline);
                    state.finish_at(
                        cp,
                        if is_unless {
                            crate::parser::element_type::RubyElementType::UnlessStatement
                        } else {
                            crate::parser::element_type::RubyElementType::IfStatement
                        },
                    );
                } else {
                    state.eat(Semicolon);
                    state.eat(Newline);
                }
            }
        }
        Ok(())
    }

    fn parse_unless_stmt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // unless
        PrattParser::parse(state, 0, self);
        self.parse_conditional_body(state)?;
        if state.eat(Else) {
            self.parse_conditional_body(state)?;
        }
        state.expect(End).ok();
        state.finish_at(cp, crate::parser::element_type::RubyElementType::UnlessStatement);
        Ok(())
    }

    fn parse_until_stmt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // until
        PrattParser::parse(state, 0, self);
        self.parse_body(state)?;
        state.finish_at(cp, crate::parser::element_type::RubyElementType::UntilStatement);
        Ok(())
    }

    fn parse_for_stmt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // for
        state.expect(Identifier).ok();
        state.expect(In).ok();
        PrattParser::parse(state, 0, self);
        self.parse_body(state)?;
        state.finish_at(cp, crate::parser::element_type::RubyElementType::ForStatement);
        Ok(())
    }

    fn parse_case_stmt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // case
        if !state.at(When) {
            PrattParser::parse(state, 0, self);
        }

        while state.at(When) {
            let when_cp = state.checkpoint();
            state.bump(); // when
            PrattParser::parse(state, 0, self);
            while state.eat(Comma) {
                PrattParser::parse(state, 0, self);
            }
            state.eat(Then);
            self.parse_case_body(state)?;
            state.finish_at(when_cp, crate::parser::element_type::RubyElementType::WhenClause);
        }

        if state.eat(Else) {
            self.parse_case_body(state)?;
        }

        state.expect(End).ok();
        state.finish_at(cp, crate::parser::element_type::RubyElementType::CaseStatement);
        Ok(())
    }

    fn parse_case_body<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        while state.not_at_end() && !state.at(End) && !state.at(When) && !state.at(Else) {
            self.parse_statement(state)?
        }
        Ok(())
    }

    /// `{|a,b| ...}` 或 `do |a| ... end`；无块则不动。
    fn parse_optional_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::RubyTokenType::*;
        while matches!(state.peek_kind(), Some(Whitespace | Comment)) {
            state.bump();
        }
        let brace = state.at(LeftBrace);
        let do_kw = state.at(Do);
        if !brace && !do_kw {
            return;
        }
        let cp = state.checkpoint();
        state.bump(); // { or do
        if state.eat(BitOr) {
            while state.not_at_end() && !state.at(BitOr) {
                if matches!(state.peek_kind(), Some(Whitespace | Comment | Newline)) {
                    state.bump();
                    continue;
                }
                if matches!(state.peek_kind(), Some(Identifier | Constant)) {
                    state.bump();
                }
                state.eat(Comma);
            }
            let _ = state.expect(BitOr);
        }
        if brace {
            while state.not_at_end() && !state.at(RightBrace) {
                let _ = self.parse_statement(state);
            }
            let _ = state.expect(RightBrace);
        } else {
            while state.not_at_end() && !state.at(End) {
                let _ = self.parse_statement(state);
            }
            let _ = state.expect(End);
        }
        state.finish_at(cp, crate::parser::element_type::RubyElementType::BlockExpression);
    }

    fn parse_begin_stmt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // begin

        while state.not_at_end() && !state.at(End) && !state.at(Rescue) && !state.at(Ensure) && !state.at(Else) {
            self.parse_statement(state)?
        }

        while state.at(Rescue) {
            let rescue_cp = state.checkpoint();
            state.bump(); // rescue
            if !state.at(Then) && !state.at(Newline) && !state.at(Semicolon) {
                PrattParser::parse(state, 0, self); // Exception class
                if state.at(EqualGreater) {
                    state.bump();
                    state.expect(Identifier).ok();
                }
            }
            state.eat(Then);
            while state.not_at_end() && !state.at(End) && !state.at(Rescue) && !state.at(Ensure) && !state.at(Else) {
                self.parse_statement(state)?
            }
            state.finish_at(rescue_cp, crate::parser::element_type::RubyElementType::RescueClause);
        }

        if state.eat(Else) {
            while state.not_at_end() && !state.at(End) && !state.at(Ensure) {
                self.parse_statement(state)?
            }
        }

        if state.at(Ensure) {
            let ensure_cp = state.checkpoint();
            state.bump(); // ensure
            while state.not_at_end() && !state.at(End) {
                self.parse_statement(state)?
            }
            state.finish_at(ensure_cp, crate::parser::element_type::RubyElementType::EnsureClause);
        }

        state.expect(End).ok();
        state.finish_at(cp, crate::parser::element_type::RubyElementType::BeginStatement);
        Ok(())
    }

    fn parse_method_def<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // def
        // `def self.foo` / `def Foo.bar`
        if matches!(state.peek_kind(), Some(Self_ | Identifier | Constant)) {
            state.bump();
            if state.eat(Dot) {
                if matches!(state.peek_kind(), Some(Identifier | Constant)) {
                    state.bump();
                }
            }
        }
        if state.eat(LeftParen) {
            while state.not_at_end() && !state.at(RightParen) {
                if state.eat(Comma) {
                    continue;
                }
                if matches!(state.peek_kind(), Some(Identifier | Constant)) {
                    state.bump();
                    // 默认参数 `a = expr`：吃掉默认值表达式，arity 仍按有名参数计。
                    if state.eat(Assign) {
                        PrattParser::parse(state, 0, self);
                    }
                    continue;
                }
                break;
            }
            let _ = state.expect(RightParen);
        }
        self.parse_body(state)?;
        state.finish_at(cp, crate::parser::element_type::RubyElementType::MethodDefinition);
        Ok(())
    }

    fn parse_body<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        loop {
            self.skip_stmt_gap(state);
            if !state.not_at_end()
                || state.at(End)
                || state.at(Else)
                || state.at(Elsif)
                || state.at(Rescue)
                || state.at(Ensure)
                || state.at(When)
            {
                break;
            }
            self.parse_statement(state)?
        }
        state.eat(End);
        Ok(())
    }

    /// if/unless 的 then/else 体：停在 `else`/`elsif`/`end`，不吞 `end`。
    fn parse_conditional_body<'a, S: Source + ?Sized>(
        &self,
        state: &mut State<'a, S>,
    ) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        loop {
            self.skip_stmt_gap(state);
            if !state.not_at_end()
                || state.at(End)
                || state.at(Else)
                || state.at(Elsif)
                || state.at(Rescue)
                || state.at(Ensure)
            {
                break;
            }
            self.parse_statement(state)?
        }
        Ok(())
    }

    fn parse_class_def<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // class
        state.expect(Constant).ok();
        if state.eat(Less) {
            state.expect(Constant).ok();
        }
        self.parse_body(state)?;
        state.finish_at(cp, crate::parser::element_type::RubyElementType::ClassDefinition);
        Ok(())
    }

    fn parse_module_def<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // module
        state.expect(Constant).ok();
        self.parse_body(state)?;
        state.finish_at(cp, crate::parser::element_type::RubyElementType::ModuleDefinition);
        Ok(())
    }

    fn parse_if_stmt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // if
        PrattParser::parse(state, 0, self);
        self.parse_conditional_body(state)?;
        while state.eat(Elsif) {
            PrattParser::parse(state, 0, self);
            self.parse_conditional_body(state)?;
        }
        if state.eat(Else) {
            self.parse_conditional_body(state)?;
        }
        state.expect(End).ok();
        state.finish_at(cp, crate::parser::element_type::RubyElementType::IfStatement);
        Ok(())
    }

    fn parse_while_stmt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // while
        PrattParser::parse(state, 0, self);
        self.parse_body(state)?;
        state.finish_at(cp, crate::parser::element_type::RubyElementType::WhileStatement);
        Ok(())
    }

    fn parse_return_stmt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // return
        // `return if cond` / `return unless cond`
        if let Some(is_unless) = self.peek_same_line_modifier(state) {
            state.bump();
            self.skip_expr_gap(state);
            PrattParser::parse(state, 0, self);
            state.eat(Semicolon);
            state.eat(Newline);
            state.finish_at(
                cp,
                if is_unless {
                    crate::parser::element_type::RubyElementType::UnlessStatement
                } else {
                    crate::parser::element_type::RubyElementType::IfStatement
                },
            );
            return Ok(());
        }
        if !matches!(state.peek_kind(), Some(End | Else | Elsif | Newline | Semicolon) | None) {
            PrattParser::parse(state, 0, self);
        }
        state.eat(Semicolon);
        state.eat(Newline);
        state.finish_at(cp, crate::parser::element_type::RubyElementType::ReturnStatement);
        Ok(())
    }
}

fn looks_like_parenless_arg<S: Source + ?Sized>(state: &State<'_, S>) -> bool {
    use crate::lexer::token_type::RubyTokenType::*;
    // 语句边界 / 修饰符关键字不能当无括号实参。
    if matches!(
        state.peek_kind(),
        Some(
            If | Unless
                | While
                | Until
                | For
                | Do
                | End
                | Else
                | Elsif
                | When
                | Then
                | Rescue
                | Ensure
                | Newline
                | Semicolon
                | RightParen
                | RightBracket
                | RightBrace
                | Comma
        ) | None
    ) {
        return false;
    }
    matches!(
        state.peek_kind(),
        Some(
            IntegerLiteral
                | FloatLiteral
                | StringLiteral
                | Symbol
                | True
                | False
                | Nil
                | Identifier
                | Constant
                | GlobalVariable
                | InstanceVariable
                | ClassVariable
                | Self_
                | LeftBracket
                | LeftParen
                | LeftBrace
        )
    )
}

impl<'config> Pratt<RubyLanguage> for RubyParser<'config> {
    fn primary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, RubyLanguage> {
        use crate::lexer::token_type::RubyTokenType::*;
        let cp = state.checkpoint();
        match state.peek_kind() {
            Some(Identifier) | Some(Constant) | Some(GlobalVariable) | Some(InstanceVariable) | Some(ClassVariable) => {
                // 不要用 bump()：它会 skip_trivia 把换行吞掉，导致
                // `unless base\n@skill=...` 把下一行当成无括号实参。
                if let Some(token) = state.current() {
                    let kind = token.kind;
                    let len = token.length();
                    state.sink.push_leaf(kind, len);
                    state.tokens.advance();
                }
                while matches!(state.peek_kind(), Some(Whitespace | Comment)) {
                    state.bump();
                }
                if state.eat(LeftParen) {
                    while state.not_at_end() && !state.at(RightParen) {
                        self.skip_expr_gap(state);
                        if state.at(RightParen) {
                            break;
                        }
                        PrattParser::parse(state, 0, self);
                        state.eat(Comma);
                    }
                    let _ = state.expect(RightParen);
                    self.parse_optional_block(state);
                    state.finish_at(cp, crate::parser::element_type::RubyElementType::CallExpression)
                } else {
                    while matches!(state.peek_kind(), Some(Whitespace | Comment)) {
                        state.bump();
                    }
                    // 换行 / 语句边界：绝不当无括号调用。
                    if matches!(
                        state.peek_kind(),
                        Some(
                            Newline
                                | Semicolon
                                | End
                                | Else
                                | Elsif
                                | When
                                | Rescue
                                | Ensure
                                | If
                                | Unless
                                | While
                                | Until
                                | Do
                                | RightParen
                                | RightBracket
                                | RightBrace
                                | Comma
                        ) | None
                    ) || !looks_like_parenless_arg(state)
                    {
                        state.skip_trivia();
                        state.finish_at(cp, crate::parser::element_type::RubyElementType::Identifier)
                    } else {
                        loop {
                            while matches!(state.peek_kind(), Some(Whitespace | Comment)) {
                                state.bump();
                            }
                            if matches!(
                                state.peek_kind(),
                                Some(Newline | Semicolon | End | Else | Elsif | When | Rescue | Ensure) | None
                            ) {
                                break;
                            }
                            if !looks_like_parenless_arg(state) {
                                break;
                            }
                            PrattParser::parse(state, 0, self);
                            while matches!(state.peek_kind(), Some(Whitespace | Comment)) {
                                state.bump();
                            }
                            if !state.eat(Comma) {
                                break;
                            }
                        }
                        self.parse_optional_block(state);
                        state.skip_trivia();
                        state.finish_at(cp, crate::parser::element_type::RubyElementType::CallExpression)
                    }
                }
            }
            Some(IntegerLiteral) | Some(FloatLiteral) | Some(StringLiteral) | Some(True) | Some(False) | Some(Nil) | Some(Self_) | Some(Symbol) => {
                state.bump();
                state.finish_at(cp, crate::parser::element_type::RubyElementType::LiteralExpression)
            }
            Some(LeftBracket) => {
                state.bump();
                while state.not_at_end() && !state.at(RightBracket) {
                    self.skip_expr_gap(state);
                    if state.at(RightBracket) {
                        break;
                    }
                    PrattParser::parse(state, 0, self);
                    state.eat(Comma);
                }
                let _ = state.expect(RightBracket);
                state.finish_at(cp, crate::parser::element_type::RubyElementType::ArrayExpression)
            }
            Some(LeftBrace) => {
                // `{|a| ...}` 当块；`{}` / `{k => v}` 当哈希。
                state.bump(); // {
                self.skip_expr_gap(state);
                if state.at(BitOr) {
                    // 回退到块：已 bump `{`，继续吃块体。
                    if state.eat(BitOr) {
                        while state.not_at_end() && !state.at(BitOr) {
                            self.skip_expr_gap(state);
                            if matches!(state.peek_kind(), Some(Identifier | Constant)) {
                                state.bump();
                            }
                            state.eat(Comma);
                        }
                        state.eat(BitOr);
                    }
                    while state.not_at_end() && !state.at(RightBrace) {
                        self.skip_expr_gap(state);
                        if state.at(RightBrace) {
                            break;
                        }
                        let _ = self.parse_statement(state);
                    }
                    state.eat(RightBrace);
                    state.finish_at(cp, crate::parser::element_type::RubyElementType::BlockExpression)
                } else {
                    while state.not_at_end() && !state.at(RightBrace) {
                        self.skip_expr_gap(state);
                        if state.at(RightBrace) {
                            break;
                        }
                        PrattParser::parse(state, 0, self); // key
                        if state.eat(EqualGreater) || state.eat(Colon) {
                            PrattParser::parse(state, 0, self); // value
                        }
                        state.eat(Comma);
                    }
                    let _ = state.expect(RightBrace);
                    state.finish_at(cp, crate::parser::element_type::RubyElementType::HashExpression)
                }
            }
            Some(LeftParen) => {
                state.bump();
                PrattParser::parse(state, 0, self);
                state.expect(RightParen).ok();
                state.finish_at(cp, crate::parser::element_type::RubyElementType::ParenExpression)
            }
            _ => {
                state.bump();
                state.finish_at(cp, crate::parser::element_type::RubyElementType::Error)
            }
        }
    }

    fn prefix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, RubyLanguage> {
        use crate::lexer::token_type::RubyTokenType::*;
        match state.peek_kind() {
            Some(kind @ (Plus | Minus | Not | Tilde)) => {
                let cp = state.checkpoint();
                state.expect(kind).ok();
                self.skip_expr_gap(state);
                let _right = PrattParser::parse(state, 13, self);
                state.finish_at(cp, crate::parser::element_type::RubyElementType::UnaryExpression)
            }
            _ => self.primary(state),
        }
    }

    fn infix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, RubyLanguage>, min_precedence: u8) -> Option<&'a GreenNode<'a, RubyLanguage>> {
        use crate::lexer::token_type::RubyTokenType::*;
        let kind = state.peek_kind()?;

        let (prec, assoc) = match kind {
            Dot => {
                if 25 < min_precedence {
                    return None;
                }
                let cp = state.checkpoint_before(left);
                state.bump(); // .
                self.skip_expr_gap(state);
                // method name
                if matches!(state.peek_kind(), Some(Identifier | Constant)) {
                    state.bump();
                }
                // `obj.foo = bar` → setter 调用
                while matches!(state.peek_kind(), Some(Whitespace | Comment)) {
                    state.bump();
                }
                if state.eat(Assign) {
                    self.skip_expr_gap(state);
                    PrattParser::parse(state, 0, self);
                    state.eat(Semicolon);
                    state.eat(Newline);
                    return Some(state.finish_at(cp, crate::parser::element_type::RubyElementType::CallExpression));
                }
                if state.eat(LeftParen) {
                    while state.not_at_end() && !state.at(RightParen) {
                        self.skip_expr_gap(state);
                        if state.at(RightParen) {
                            break;
                        }
                        PrattParser::parse(state, 0, self);
                        state.eat(Comma);
                    }
                    let _ = state.expect(RightParen);
                } else if looks_like_parenless_arg(state) {
                    // `obj.foo a, b` 无括号实参
                    loop {
                        while matches!(state.peek_kind(), Some(Whitespace | Comment)) {
                            state.bump();
                        }
                        if matches!(
                            state.peek_kind(),
                            Some(Newline | Semicolon | End | Else | Elsif | When | Rescue | Ensure) | None
                        ) {
                            break;
                        }
                        if !looks_like_parenless_arg(state) {
                            break;
                        }
                        PrattParser::parse(state, 0, self);
                        while matches!(state.peek_kind(), Some(Whitespace | Comment)) {
                            state.bump();
                        }
                        if !state.eat(Comma) {
                            break;
                        }
                    }
                }
                // `obj.foo { ... }` / `obj.foo do ... end`
                self.parse_optional_block(state);
                return Some(state.finish_at(cp, crate::parser::element_type::RubyElementType::CallExpression));
            }
            DoubleColon => {
                if 25 < min_precedence {
                    return None;
                }
                let cp = state.checkpoint_before(left);
                state.bump(); // ::
                self.skip_expr_gap(state);
                if matches!(state.peek_kind(), Some(Identifier | Constant)) {
                    state.bump();
                }
                if state.eat(LeftParen) {
                    while state.not_at_end() && !state.at(RightParen) {
                        self.skip_expr_gap(state);
                        if state.at(RightParen) {
                            break;
                        }
                        PrattParser::parse(state, 0, self);
                        state.eat(Comma);
                    }
                    let _ = state.expect(RightParen);
                }
                self.parse_optional_block(state);
                return Some(state.finish_at(cp, crate::parser::element_type::RubyElementType::CallExpression));
            }
            DotDot | DotDotDot => (4, Associativity::Left),
            Power => (30, Associativity::Right),
            Multiply | Divide | Modulo => (20, Associativity::Left),
            Plus | Minus => (10, Associativity::Left),
            EqualEqual | NotEqual | Less | Greater | LessEqual | GreaterEqual => (5, Associativity::Left),
            AndAnd | And => (2, Associativity::Left),
            OrOr | Or => (1, Associativity::Left),
            LeftBracket => {
                if 28 < min_precedence {
                    return None;
                }
                let cp = state.checkpoint_before(left);
                state.bump(); // [
                while state.not_at_end() && !state.at(RightBracket) {
                    self.skip_expr_gap(state);
                    if state.at(RightBracket) {
                        break;
                    }
                    PrattParser::parse(state, 0, self);
                    state.eat(Comma);
                }
                let _ = state.expect(RightBracket);
                return Some(state.finish_at(cp, crate::parser::element_type::RubyElementType::CallExpression));
            }
            LeftBrace | Do => {
                // 无点调用后的块：`loop do` / `foo { }`（left 已是 receiver/call）
                if 0 < min_precedence {
                    return None;
                }
                let cp = state.checkpoint_before(left);
                self.parse_optional_block(state);
                return Some(state.finish_at(cp, crate::parser::element_type::RubyElementType::CallExpression));
            }
            _ => return None,
        };

        if prec < min_precedence {
            return None;
        }

        // 运算符后允许换行续行（不用 oak-core binary，因其不跳 Newline）。
        let cp = state.checkpoint_before(left);
        state.expect(kind).ok();
        self.skip_expr_gap(state);
        let next_prec = match assoc {
            Associativity::Left | Associativity::None => prec.saturating_add(1),
            Associativity::Right => prec,
        };
        let _right = PrattParser::parse(state, next_prec, self);
        Some(state.finish_at(cp, crate::parser::element_type::RubyElementType::BinaryExpression))
    }
}

impl<'config> Parser<RubyLanguage> for RubyParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<RubyLanguage>) -> ParseOutput<'a, RubyLanguage> {
        let lexer = RubyLexer::new(&self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let cp = state.checkpoint();
            while state.not_at_end() {
                self.skip_stmt_gap(state);
                if !state.not_at_end() {
                    break;
                }
                let _ = self.parse_statement(state);
            }
            Ok(state.finish_at(cp, crate::parser::element_type::RubyElementType::Root))
        })
    }
}
