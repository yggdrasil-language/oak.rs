use crate::{
    ValkyrieLanguage,
    lexer::token_type::ValkyrieTokenType,
    parser::{element_type::ValkyrieElementType, parse_statements},
};
use oak_core::parser::ParserState;

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

/// 解析块表达式
pub(crate) fn parse_block<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    while state.not_at_end() && !state.at(ValkyrieTokenType::RightBrace) {
        state.skip_trivia();
        if state.at(ValkyrieTokenType::RightBrace) {
            break;
        }
        parse_statement(state)?;
    }
    if state.at(ValkyrieTokenType::RightBrace) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::BlockExpression);
    Ok(())
}

/// 解析块表达式
pub(crate) fn parse_block_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    if state.at(ValkyrieTokenType::LeftBrace) { parse_block(state) } else { Ok(()) }
}

/// 解析 `Point { x: 1, y }` 对象初始化块（也容纳 trailing closure 表达式体）。
pub(crate) fn parse_object_initializer<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    while state.not_at_end() && !state.at(ValkyrieTokenType::RightBrace) {
        state.skip_trivia();
        if state.at(ValkyrieTokenType::RightBrace) {
            break;
        }
        let stmt_cp = state.sink.checkpoint();
        if state.at(ValkyrieTokenType::Identifier) {
            let name_cp = state.sink.checkpoint();
            state.bump();
            state.sink.finish_node(name_cp, ValkyrieElementType::IdentifierExpression);
            if state.at(ValkyrieTokenType::Colon) || state.at(ValkyrieTokenType::Eq) {
                let field_cp = state.sink.checkpoint() - 1;
                state.bump();
                crate::parser::parse_expressions::parse_expression(state)?;
                state.sink.finish_node(field_cp, ValkyrieElementType::BinaryExpression);
            }
        }
        else {
            crate::parser::parse_expressions::parse_expression(state)?;
        }
        state.sink.finish_node(stmt_cp, ValkyrieElementType::ExprStatement);
        state.skip_trivia();
        if state.at(ValkyrieTokenType::Comma) {
            state.bump();
        }
        if state.at(ValkyrieTokenType::Semicolon) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::RightBrace) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::BlockExpression);
    Ok(())
}

// 以下函数在其他模块中定义
pub(crate) fn parse_statement<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_statements::parse_statement(state)
}
