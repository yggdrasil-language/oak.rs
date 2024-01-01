use crate::{
    ValkyrieLanguage,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::element_type::ValkyrieElementType,
};
use oak_core::parser::ParserState;

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

/// 解析表达式（入口，处理赋值和低优先级运算符）
pub(crate) fn parse_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_or_expression(state)
}

/// 解析 || 表达式
pub(crate) fn parse_or_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_and_expression(state)?;
    while state.at(ValkyrieTokenType::OrOr) {
        let cp = state.sink.checkpoint() - 1;
        state.bump();
        parse_and_expression(state)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 && 表达式
pub(crate) fn parse_and_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_equality_expression(state)?;
    while state.at(ValkyrieTokenType::AndAnd) {
        let cp = state.sink.checkpoint() - 1;
        state.bump();
        parse_equality_expression(state)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 == != 表达式
pub(crate) fn parse_equality_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_comparison_expression(state)?;
    while state.at(ValkyrieTokenType::EqEq) || state.at(ValkyrieTokenType::NotEq) {
        let cp = state.sink.checkpoint() - 1;
        state.bump();
        parse_comparison_expression(state)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 < <= > >= 表达式
pub(crate) fn parse_comparison_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_additive_expression(state)?;
    while state.at(ValkyrieTokenType::LessThan) || state.at(ValkyrieTokenType::LessEq) || state.at(ValkyrieTokenType::GreaterThan) || state.at(ValkyrieTokenType::GreaterEq) {
        let cp = state.sink.checkpoint() - 1;
        state.bump();
        parse_additive_expression(state)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 + - 表达式
pub(crate) fn parse_additive_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_multiplicative_expression(state)?;
    while state.at(ValkyrieTokenType::Plus) || state.at(ValkyrieTokenType::Minus) {
        let cp = state.sink.checkpoint() - 1;
        state.bump();
        parse_multiplicative_expression(state)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 * / % 表达式
pub(crate) fn parse_multiplicative_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_unary_expression(state)?;
    while state.at(ValkyrieTokenType::Star) || state.at(ValkyrieTokenType::Slash) || state.at(ValkyrieTokenType::Percent) {
        let cp = state.sink.checkpoint() - 1;
        state.bump();
        parse_unary_expression(state)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析一元表达式
pub(crate) fn parse_unary_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    if state.at(ValkyrieTokenType::Minus) || state.at(ValkyrieTokenType::Bang) {
        let cp = state.sink.checkpoint();
        state.bump();
        parse_unary_expression(state)?;
        state.sink.finish_node(cp, ValkyrieElementType::UnaryExpression);
        return Ok(());
    }
    parse_postfix_expression(state)
}

/// 解析后缀表达式（字段访问、索引、调用）
pub(crate) fn parse_postfix_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_primary_expression(state)?;
    loop {
        if state.at(ValkyrieTokenType::Dot) {
            let cp = state.sink.checkpoint() - 1;
            state.bump();
            if state.at(ValkyrieTokenType::Identifier) {
                state.bump();
            }
            state.sink.finish_node(cp, ValkyrieElementType::FieldExpression);
        }
        else if state.at(ValkyrieTokenType::LeftBracket) {
            let cp = state.sink.checkpoint() - 1;
            state.bump();
            parse_expression(state)?;
            if state.at(ValkyrieTokenType::RightBracket) {
                state.bump();
            }
            state.sink.finish_node(cp, ValkyrieElementType::IndexExpression);
        }
        else if state.at(ValkyrieTokenType::LeftParen) {
            let cp = state.sink.checkpoint() - 1;
            parse_argument_list(state)?;
            state.sink.finish_node(cp, ValkyrieElementType::CallExpression);
        }
        else {
            break;
        }
    }
    Ok(())
}

/// 解析参数列表
pub(crate) fn parse_argument_list<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    while state.not_at_end() && !state.at(ValkyrieTokenType::RightParen) {
        parse_expression(state)?;
        if state.at(ValkyrieTokenType::Comma) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::RightParen) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::ArgList);
    Ok(())
}

/// 解析主表达式
pub(crate) fn parse_primary_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    if let Some(token) = state.current() {
        match &token.kind {
            ValkyrieTokenType::Identifier => {
                let cp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::ColonColon) {
                    while state.at(ValkyrieTokenType::ColonColon) {
                        state.bump();
                        if state.at(ValkyrieTokenType::Identifier) {
                            state.bump();
                        }
                    }
                    state.sink.finish_node(cp, ValkyrieElementType::PathExpression);
                }
                else {
                    state.sink.finish_node(cp, ValkyrieElementType::IdentifierExpression);
                }
                Ok(())
            }
            ValkyrieTokenType::IntegerLiteral | ValkyrieTokenType::FloatLiteral => {
                let cp = state.sink.checkpoint();
                state.bump();
                state.sink.finish_node(cp, ValkyrieElementType::LiteralExpression);
                Ok(())
            }
            ValkyrieTokenType::StringLiteral => {
                let cp = state.sink.checkpoint();
                state.bump();
                state.sink.finish_node(cp, ValkyrieElementType::LiteralExpression);
                Ok(())
            }
            ValkyrieTokenType::CharLiteral => {
                let cp = state.sink.checkpoint();
                state.bump();
                state.sink.finish_node(cp, ValkyrieElementType::LiteralExpression);
                Ok(())
            }
            ValkyrieTokenType::BoolLiteral | ValkyrieTokenType::Keyword(ValkyrieKeywords::True) | ValkyrieTokenType::Keyword(ValkyrieKeywords::False) => {
                let cp = state.sink.checkpoint();
                state.bump();
                state.sink.finish_node(cp, ValkyrieElementType::BooleanLiteral);
                Ok(())
            }
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Null) => {
                let cp = state.sink.checkpoint();
                state.bump();
                state.sink.finish_node(cp, ValkyrieElementType::LiteralExpression);
                Ok(())
            }
            ValkyrieTokenType::LeftParen => {
                let cp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::RightParen) {
                    state.bump();
                    state.sink.finish_node(cp, ValkyrieElementType::LiteralExpression);
                }
                else {
                    parse_expression(state)?;
                    if state.at(ValkyrieTokenType::RightParen) {
                        state.bump();
                    }
                    state.sink.finish_node(cp, ValkyrieElementType::ParenthesizedExpression);
                }
                Ok(())
            }
            ValkyrieTokenType::LeftBrace => parse_block_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::If) => parse_if_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Match) => parse_match_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Loop) => parse_loop_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::While) => parse_while_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::For) => parse_for_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Return) => parse_return_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Break) => {
                let cp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::Identifier) {
                    state.bump();
                }
                state.sink.finish_node(cp, ValkyrieElementType::BreakExpression);
                Ok(())
            }
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Continue) => {
                let cp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::Identifier) {
                    state.bump();
                }
                state.sink.finish_node(cp, ValkyrieElementType::ContinueExpression);
                Ok(())
            }
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Yield) => {
                let cp = state.sink.checkpoint();
                state.bump();
                parse_expression(state)?;
                state.sink.finish_node(cp, ValkyrieElementType::YieldExpression);
                Ok(())
            }
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Raise) => {
                let cp = state.sink.checkpoint();
                state.bump();
                parse_expression(state)?;
                state.sink.finish_node(cp, ValkyrieElementType::RaiseExpression);
                Ok(())
            }
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Lambda) => parse_lambda_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::SelfType) => {
                let cp = state.sink.checkpoint();
                state.bump();
                state.sink.finish_node(cp, ValkyrieElementType::IdentifierExpression);
                Ok(())
            }
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Super) => {
                let cp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::Dot) {
                    state.bump();
                    if state.at(ValkyrieTokenType::Identifier) {
                        state.bump();
                    }
                }
                if state.at(ValkyrieTokenType::LeftParen) {
                    parse_argument_list(state)?;
                }
                state.sink.finish_node(cp, ValkyrieElementType::SuperCallExpression);
                Ok(())
            }
            _ => {
                state.bump();
                Ok(())
            }
        }
    }
    else {
        Ok(())
    }
}

// 以下函数在其他模块中定义
pub(crate) fn parse_block_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_blocks::parse_block_expression(state)
}

pub(crate) fn parse_if_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_control::parse_if_expression(state)
}

pub(crate) fn parse_match_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_control::parse_match_expression(state)
}

pub(crate) fn parse_loop_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_control::parse_loop_expression(state)
}

pub(crate) fn parse_while_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_control::parse_while_expression(state)
}

pub(crate) fn parse_for_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_control::parse_for_expression(state)
}

pub(crate) fn parse_return_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_control::parse_return_expression(state)
}

pub(crate) fn parse_lambda_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_control::parse_lambda_expression(state)
}
