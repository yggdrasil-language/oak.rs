use crate::{
    ValkyrieLanguage,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::{element_type::ValkyrieElementType, parse_modifiers::parse_binding_modifiers},
};
use oak_core::parser::ParserState;

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

/// 解析语句
pub(crate) fn parse_statement<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    state.skip_trivia();
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Let)) {
        let cp = state.sink.checkpoint();
        parse_let_statement(state)?;
        state.sink.finish_node(cp, ValkyrieElementType::LetStatement);
        Ok(())
    }
    else {
        let cp = state.sink.checkpoint();
        parse_expression(state)?;
        if state.at(ValkyrieTokenType::Semicolon) {
            state.bump();
        }
        state.sink.finish_node(cp, ValkyrieElementType::ExprStatement);
        Ok(())
    }
}

/// 解析 let 语句
pub(crate) fn parse_let_statement<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    state.bump();
    parse_binding_modifiers(state)?;
    parse_pattern(state)?;
    if state.at(ValkyrieTokenType::Colon) {
        state.bump();
        parse_type(state)?;
    }
    if state.at(ValkyrieTokenType::Eq) {
        state.bump();
        parse_expression(state)?;
    }
    if state.at(ValkyrieTokenType::Semicolon) {
        state.bump();
    }
    Ok(())
}

/// 解析表达式语句
pub(crate) fn parse_expr_statement<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_expression(state)?;
    if state.at(ValkyrieTokenType::Semicolon) {
        state.bump();
    }
    Ok(())
}

/// 解析模式
pub(crate) fn parse_pattern<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    if state.at(ValkyrieTokenType::Underscore) {
        state.bump();
    }
    else if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::Pattern);
    Ok(())
}

// 以下函数在其他模块中定义
pub(crate) fn parse_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_expressions::parse_expression(state)
}

pub(crate) fn parse_type<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_types::parse_type(state)
}
