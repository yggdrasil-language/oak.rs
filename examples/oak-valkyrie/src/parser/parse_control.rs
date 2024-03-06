use crate::{
    ValkyrieLanguage,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::{element_type::ValkyrieElementType, parse_blocks::parse_statement},
};
use oak_core::parser::ParserState;

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

fn at_default_arm_keyword<S: oak_core::Source + ?Sized>(state: &State<'_, S>) -> bool {
    state.at(ValkyrieTokenType::Identifier) && state.peek_text().as_deref() == Some("default")
}

fn brace_balance_between<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, from: usize, to: usize) -> i32 {
    let restore = state.checkpoint();
    state.tokens.set_index(from);
    let mut balance = 0i32;
    while state.tokens.index() < to {
        if state.at(ValkyrieTokenType::BraceL) {
            balance += 1;
        }
        else if state.at(ValkyrieTokenType::BraceR) {
            balance -= 1;
        }
        state.bump();
    }
    state.restore(restore);
    balance
}

/// `case Pat:` 后允许多条语句，直到同级 `case` / `else` / `}`。
fn parse_implicit_match_arm_body<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    let mut depth = 0i32;
    while state.not_at_end() {
        state.skip_trivia();
        if depth == 0 {
            if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Case)) || state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Else)) || at_default_arm_keyword(state) {
                break;
            }
            if state.at(ValkyrieTokenType::BraceR) {
                break;
            }
        }
        let before = state.checkpoint().0;
        parse_statement(state)?;
        if state.checkpoint().0 == before {
            break;
        }
        depth += brace_balance_between(state, before, state.checkpoint().0);
    }
    state.sink.finish_node(cp, ValkyrieElementType::BlockExpression);
    Ok(())
}

/// 解析 if 表达式
pub(crate) fn parse_if_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if state.at(ValkyrieTokenType::ParenthesisL) {
        state.bump();
        parse_expression(state)?;
        if state.at(ValkyrieTokenType::ParenthesisR) {
            state.bump();
        }
    }
    else {
        crate::parser::parse_expressions::parse_condition_expression(state)?;
    }
    parse_block_expression(state)?;
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Else)) {
        state.bump();
        if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::If)) {
            parse_if_expression(state)?;
        }
        else {
            parse_block_expression(state)?;
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::IfExpression);
    Ok(())
}

/// 解析 match 表达式
pub(crate) fn parse_match_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    crate::parser::parse_expressions::parse_scrutinee_expression(state)?;
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            let before = state.checkpoint().0;
            state.skip_trivia();
            if state.at(ValkyrieTokenType::BraceR) {
                break;
            }
            parse_match_arm(state)?;
            if state.checkpoint().0 == before {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::MatchExpression);
    Ok(())
}

/// 解析 match 模式：`Some(value)` / `None` / `_`。
fn parse_match_pattern<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let pcp = state.sink.checkpoint();
    if state.at(ValkyrieTokenType::Underscore) {
        state.bump();
    }
    else if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
        if state.at(ValkyrieTokenType::ParenthesisL) {
            state.bump();
            while state.not_at_end() && !state.at(ValkyrieTokenType::ParenthesisR) {
                state.skip_trivia();
                if state.at(ValkyrieTokenType::ParenthesisR) {
                    break;
                }
                parse_match_pattern(state)?;
                state.skip_trivia();
                if state.at(ValkyrieTokenType::Comma) {
                    state.bump();
                }
            }
            if state.at(ValkyrieTokenType::ParenthesisR) {
                state.bump();
            }
        }
        else if state.at(ValkyrieTokenType::BraceL) {
            state.bump();
            while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
                state.skip_trivia();
                if state.at(ValkyrieTokenType::BraceR) {
                    break;
                }
                let before = state.checkpoint().0;
                if state.at(ValkyrieTokenType::Identifier) || state.at(ValkyrieTokenType::Underscore) {
                    state.bump();
                    if state.at(ValkyrieTokenType::Colon) {
                        state.bump();
                        parse_match_pattern(state)?;
                    }
                }
                else {
                    parse_match_pattern(state)?;
                }
                if state.checkpoint().0 == before {
                    state.bump();
                }
                state.skip_trivia();
                if state.at(ValkyrieTokenType::Comma) {
                    state.bump();
                }
            }
            if state.at(ValkyrieTokenType::BraceR) {
                state.bump();
            }
        }
    }
    state.sink.finish_node(pcp, ValkyrieElementType::Pattern);
    Ok(())
}

/// 解析 match arm
pub(crate) fn parse_match_arm<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.skip_trivia();
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Case)) {
        state.bump();
        state.skip_trivia();
        parse_match_pattern(state)?;
    }
    else if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Else)) || at_default_arm_keyword(state) {
        state.bump();
    }
    else {
        parse_match_pattern(state)?;
    }
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::If)) {
        state.bump();
        crate::parser::parse_expressions::parse_condition_expression(state)?;
    }
    if state.at(ValkyrieTokenType::Colon) || state.at(ValkyrieTokenType::Arrow) {
        state.bump();
        if state.at(ValkyrieTokenType::BraceL) {
            crate::parser::parse_blocks::parse_block_expression(state)?;
        }
        else {
            parse_implicit_match_arm_body(state)?;
        }
    }
    if state.at(ValkyrieTokenType::Comma) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::MatchArm);
    Ok(())
}

/// 解析 loop 表达式
pub(crate) fn parse_loop_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if (state.at(ValkyrieTokenType::Identifier) || state.at(ValkyrieTokenType::Underscore)) && state.peek_non_trivia_kind_at(1) == Some(ValkyrieTokenType::Keyword(ValkyrieKeywords::In)) {
        parse_pattern(state)?;
        state.bump();
        crate::parser::parse_expressions::parse_scrutinee_expression(state)?;
    }
    else if state.at(ValkyrieTokenType::ParenthesisL) {
        state.bump();
        parse_expression(state)?;
        if state.at(ValkyrieTokenType::ParenthesisR) {
            state.bump();
        }
    }
    if !state.at(ValkyrieTokenType::BraceL) {
        return Err(state.source.syntax_error("循环缺少主体".into(), state.current_offset()));
    }
    parse_block_expression(state)?;
    state.sink.finish_node(cp, ValkyrieElementType::LoopExpression);
    Ok(())
}

/// 解析 while 表达式
pub(crate) fn parse_while_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    crate::parser::parse_expressions::parse_condition_expression(state)?;
    parse_block_expression(state)?;
    state.sink.finish_node(cp, ValkyrieElementType::LoopExpression);
    Ok(())
}

/// 解析 for 表达式
pub(crate) fn parse_for_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    parse_pattern(state)?;
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::In)) {
        state.bump();
    }
    parse_expression(state)?;
    parse_block_expression(state)?;
    state.sink.finish_node(cp, ValkyrieElementType::LoopExpression);
    Ok(())
}

fn can_start_return_value<S: oak_core::Source + ?Sized>(state: &State<'_, S>) -> bool {
    if state.at(ValkyrieTokenType::Semicolon) || state.at(ValkyrieTokenType::BraceR) || state.at(ValkyrieTokenType::ParenthesisR) || state.at(ValkyrieTokenType::Eof) {
        return false;
    }
    if matches!(state.current().map(|token| &token.kind), Some(ValkyrieTokenType::Keyword(ValkyrieKeywords::Case | ValkyrieKeywords::Else | ValkyrieKeywords::Let | ValkyrieKeywords::Return | ValkyrieKeywords::Break | ValkyrieKeywords::Continue))) {
        return false;
    }
    !at_default_arm_keyword(state)
}

/// 解析 return 表达式
pub(crate) fn parse_return_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if can_start_return_value(state) {
        parse_expression(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::ReturnExpression);
    Ok(())
}

/// 解析 lambda 表达式
pub(crate) fn parse_lambda_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if state.at(ValkyrieTokenType::ParenthesisL) {
        parse_parameter_list(state)?;
    }
    if state.at(ValkyrieTokenType::Arrow) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::BraceL) {
        parse_block_expression(state)?;
    }
    else {
        parse_expression(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::LambdaExpression);
    Ok(())
}

// Functions below are defined in other modules
/// 匿名 micro 的参数、返回类型与主体均由结构节点保存。
pub(crate) fn parse_anonymous_micro<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if !state.at(ValkyrieTokenType::ParenthesisL) {
        return Err(state.source.syntax_error("匿名 micro 缺少参数列表".into(), state.current_offset()));
    }
    parse_parameter_list(state)?;
    if state.at(ValkyrieTokenType::Arrow) || state.at(ValkyrieTokenType::Colon) {
        state.bump();
        super::parse_types::parse_type(state)?;
    }
    if !state.at(ValkyrieTokenType::BraceL) {
        return Err(state.source.syntax_error("匿名 micro 缺少主体".into(), state.current_offset()));
    }
    parse_block_expression(state)?;
    state.sink.finish_node(cp, ValkyrieElementType::LambdaExpression);
    Ok(())
}

pub(crate) fn parse_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_expressions::parse_expression(state)
}

pub(crate) fn parse_block_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_blocks::parse_block_expression(state)
}

pub(crate) fn parse_pattern<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_statements::parse_pattern(state)
}

pub(crate) fn parse_parameter_list<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_types::parse_parameter_list(state)
}
