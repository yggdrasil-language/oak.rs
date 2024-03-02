use crate::{
    ValkyrieLanguage,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::element_type::ValkyrieElementType,
};
use oak_core::parser::ParserState;

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

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
        parse_expression(state)?;
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
            state.skip_trivia();
            if state.at(ValkyrieTokenType::BraceR) {
                break;
            }
            parse_match_arm(state)?;
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
    else if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Else)) {
        state.bump();
    }
    else {
        parse_match_pattern(state)?;
    }
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::If)) {
        state.bump();
        parse_expression(state)?;
    }
    if state.at(ValkyrieTokenType::Colon) || state.at(ValkyrieTokenType::Arrow) {
        state.bump();
        if state.at(ValkyrieTokenType::BraceL) {
            parse_block_expression(state)?;
        }
        else {
            parse_expression(state)?;
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
    if state.at(ValkyrieTokenType::ParenthesisL) {
        state.bump();
        parse_expression(state)?;
        if state.at(ValkyrieTokenType::ParenthesisR) {
            state.bump();
        }
    }
    parse_block_expression(state)?;
    state.sink.finish_node(cp, ValkyrieElementType::LoopExpression);
    Ok(())
}

/// 解析 while 表达式
pub(crate) fn parse_while_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    parse_expression(state)?;
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

/// 解析 return 表达式
pub(crate) fn parse_return_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if !state.at(ValkyrieTokenType::Semicolon) && !state.at(ValkyrieTokenType::BraceR) && !state.at(ValkyrieTokenType::ParenthesisR) && state.not_at_end() {
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

// 以下函数在其他模块中定义
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
