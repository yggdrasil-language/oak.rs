use crate::{
    ValkyrieLanguage,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::{element_type::ValkyrieElementType, parse_modifiers::at_identifier_like},
};
use oak_core::parser::ParserState;

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

/// 解析表达式（入口，处理赋值和低优先级运算符）
pub(crate) fn parse_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_expression_with_options(state, true)
}

/// 解析 if / while 等紧随 block 的条件表达式，禁止把 `{` 当作对象字面量后缀。
pub(crate) fn parse_condition_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_expression_with_options(state, false)
}

fn parse_expression_with_options<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_or_expression_with_options(state, allow_object_literal)?;
    // 赋值低于逻辑运算，右侧递归保证右结合；保留两侧完整 CST。
    if state.at(ValkyrieTokenType::Eq) {
        state.bump();
        parse_expression_with_options(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 || 表达式
pub(crate) fn parse_or_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_or_expression_with_options(state, true)
}

fn parse_or_expression_with_options<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_and_expression_with_options(state, allow_object_literal)?;
    while state.at(ValkyrieTokenType::OrOr) {
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        parse_and_expression_with_options(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 && 表达式
pub(crate) fn parse_and_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_and_expression_with_options(state, true)
}

fn parse_and_expression_with_options<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_range_expression(state, allow_object_literal)?;
    while state.at(ValkyrieTokenType::AndAnd) {
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        parse_range_expression(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 范围保留普通运算符节点，不在 parser 中展开迭代行为。
fn parse_range_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_equality_expression_with_options(state, allow_object_literal)?;
    while state.at(ValkyrieTokenType::DotDot) {
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        parse_equality_expression_with_options(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 == != 表达式
pub(crate) fn parse_equality_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_equality_expression_with_options(state, true)
}

fn parse_equality_expression_with_options<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_comparison_expression_with_options(state, allow_object_literal)?;
    while state.at(ValkyrieTokenType::EqEq) || state.at(ValkyrieTokenType::NotEq) {
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        parse_comparison_expression_with_options(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 < <= > >= 表达式
pub(crate) fn parse_comparison_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_comparison_expression_with_options(state, true)
}

fn parse_comparison_expression_with_options<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_bit_or_expression(state, allow_object_literal)?;
    while state.at(ValkyrieTokenType::LessThan) || state.at(ValkyrieTokenType::LessEq) || state.at(ValkyrieTokenType::GreaterThan) || state.at(ValkyrieTokenType::GreaterEq) {
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        parse_bit_or_expression(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 位或低于位与，两者均高于比较；仅产生普通二元 CST。
fn parse_bit_or_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_bit_and_expression(state, allow_object_literal)?;
    while state.at(ValkyrieTokenType::Pipe) {
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        parse_bit_and_expression(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 移位与位与消费完整算术表达式，禁止把运算符作为无意义 token 丢弃。
fn parse_bit_and_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_additive_expression_with_options(state, allow_object_literal)?;
    loop {
        // 泛型闭合仍是两个 `>` token；表达式位置只把相邻的两枚识别为移位。
        let split_right_shift = state.current().zip(state.peek_at(1)).is_some_and(|(left, right)| left.kind == ValkyrieTokenType::GreaterThan && right.kind == ValkyrieTokenType::GreaterThan && left.span.end == right.span.start);
        if !(split_right_shift || state.at(ValkyrieTokenType::Ampersand) || state.at(ValkyrieTokenType::LeftShift) || state.at(ValkyrieTokenType::RightShift)) {
            break;
        }
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        if split_right_shift {
            state.bump();
        }
        parse_additive_expression_with_options(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 + - 表达式
pub(crate) fn parse_additive_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_additive_expression_with_options(state, true)
}

fn parse_additive_expression_with_options<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_multiplicative_expression_with_options(state, allow_object_literal)?;
    while state.at(ValkyrieTokenType::Plus) || state.at(ValkyrieTokenType::Minus) {
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        parse_multiplicative_expression_with_options(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析 * / % 表达式
pub(crate) fn parse_multiplicative_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_multiplicative_expression_with_options(state, true)
}

fn parse_multiplicative_expression_with_options<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_caret_expression(state, allow_object_literal)?;
    while state.at(ValkyrieTokenType::Star) || state.at(ValkyrieTokenType::Slash) || state.at(ValkyrieTokenType::Percent) {
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        parse_caret_expression(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 保留现有运算符合同中 caret 的独立结合层级。
fn parse_caret_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_unary_expression_with_options(state, allow_object_literal)?;
    while state.at(ValkyrieTokenType::Caret) {
        let cp = state.sink.last_node_checkpoint();
        state.bump();
        parse_unary_expression_with_options(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::BinaryExpression);
    }
    Ok(())
}

/// 解析一元表达式
pub(crate) fn parse_unary_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_unary_expression_with_options(state, true)
}

fn parse_unary_expression_with_options<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    if state.at(ValkyrieTokenType::Minus) || state.at(ValkyrieTokenType::Bang) {
        let cp = state.sink.checkpoint();
        state.bump();
        parse_unary_expression_with_options(state, allow_object_literal)?;
        state.sink.finish_node(cp, ValkyrieElementType::UnaryExpression);
        return Ok(());
    }
    parse_postfix_expression_with_options(state, allow_object_literal)
}

/// 解析后缀表达式（字段访问、索引、调用）
pub(crate) fn parse_postfix_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_postfix_expression_with_options(state, true)
}

/// 解析 match / catch 等结构的 scrutinee，禁止把紧随其后的 `{` 当作对象字面量。
pub(crate) fn parse_scrutinee_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_condition_expression(state)
}

fn parse_postfix_expression_with_options<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, allow_object_literal: bool) -> Result<(), oak_core::OakError> {
    parse_primary_expression(state)?;
    loop {
        if state.at(ValkyrieTokenType::Dot) || (state.at(ValkyrieTokenType::ColonColon) && state.peek_non_trivia_kind_at(1) != Some(ValkyrieTokenType::LessThan) && state.peek_non_trivia_kind_at(1) != Some(ValkyrieTokenType::BracketL)) {
            let cp = state.sink.last_node_checkpoint();
            state.bump();
            if at_identifier_like(state) {
                state.bump();
            }
            else {
                return Err(state.source.syntax_error("成员路径缺少名称".into(), state.current_offset()));
            }
            state.sink.finish_node(cp, ValkyrieElementType::FieldExpression);
        }
        else if state.at(ValkyrieTokenType::OffsetL) || (state.at(ValkyrieTokenType::ColonColon) && state.peek_non_trivia_kind_at(1) == Some(ValkyrieTokenType::BracketL)) {
            let cp = state.sink.last_node_checkpoint();
            let closing = if state.at(ValkyrieTokenType::ColonColon) {
                state.bump();
                ValkyrieTokenType::BracketR
            }
            else {
                ValkyrieTokenType::OffsetR
            };
            state.bump();
            parse_expression(state)?;
            if !state.at(closing) {
                return Err(state.source.syntax_error("基数下标缺少闭合括号".into(), state.current_offset()));
            }
            state.bump();
            state.sink.finish_node(cp, ValkyrieElementType::OffsetExpression);
        }
        else if state.at(ValkyrieTokenType::BracketL) {
            let cp = state.sink.last_node_checkpoint();
            state.bump();
            parse_expression(state)?;
            if state.at(ValkyrieTokenType::BracketR) {
                state.bump();
            }
            state.sink.finish_node(cp, ValkyrieElementType::IndexExpression);
        }
        else if state.at(ValkyrieTokenType::ColonColon) && state.peek_non_trivia_kind_at(1) == Some(ValkyrieTokenType::LessThan) {
            let cp = state.sink.last_node_checkpoint();
            state.bump();
            crate::parser::parse_types::parse_generic_argument_list(state)?;
            state.sink.finish_node(cp, ValkyrieElementType::TurbofishExpression);
        }
        else if state.at(ValkyrieTokenType::ParenthesisL) {
            let cp = state.sink.last_node_checkpoint();
            parse_argument_list(state)?;
            state.sink.finish_node(cp, ValkyrieElementType::CallExpression);
        }
        else if allow_object_literal && state.at(ValkyrieTokenType::BraceL) {
            let cp = state.sink.last_node_checkpoint();
            crate::parser::parse_blocks::parse_object_initializer(state)?;
            state.sink.finish_node(cp, ValkyrieElementType::ObjectExpression);
        }
        else if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::As)) {
            let cp = state.sink.last_node_checkpoint();
            state.bump();
            crate::parser::parse_types::parse_type(state)?;
            state.sink.finish_node(cp, ValkyrieElementType::CastExpression);
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
    while state.not_at_end() && !state.at(ValkyrieTokenType::ParenthesisR) {
        let before = state.checkpoint().0;
        parse_expression(state)?;
        if state.checkpoint().0 == before {
            break;
        }
        if state.at(ValkyrieTokenType::Comma) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::ParenthesisR) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::ArgList);
    Ok(())
}

/// 解析主表达式
pub(crate) fn parse_primary_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    if let Some(token) = state.current() {
        match &token.kind {
            _ if token.kind.is_soft_identifier() => {
                let cp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::ColonColon) && state.peek_non_trivia_kind_at(1) != Some(ValkyrieTokenType::LessThan) && state.peek_non_trivia_kind_at(1) != Some(ValkyrieTokenType::BracketL) {
                    while state.at(ValkyrieTokenType::ColonColon) {
                        state.bump();
                        if at_identifier_like(state) {
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
            ValkyrieTokenType::ParenthesisL => {
                let cp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::ParenthesisR) {
                    state.bump();
                    state.sink.finish_node(cp, ValkyrieElementType::LiteralExpression);
                }
                else {
                    parse_expression(state)?;
                    if state.at(ValkyrieTokenType::Comma) {
                        while state.at(ValkyrieTokenType::Comma) {
                            state.bump();
                            parse_expression(state)?;
                        }
                        if state.at(ValkyrieTokenType::ParenthesisR) {
                            state.bump();
                        }
                        state.sink.finish_node(cp, ValkyrieElementType::TupleExpression);
                    }
                    else {
                        if state.at(ValkyrieTokenType::ParenthesisR) {
                            state.bump();
                        }
                        state.sink.finish_node(cp, ValkyrieElementType::ParenthesizedExpression);
                    }
                }
                Ok(())
            }
            ValkyrieTokenType::BracketL => {
                let cp = state.sink.checkpoint();
                state.bump();
                while !state.at(ValkyrieTokenType::BracketR) && state.not_at_end() {
                    parse_expression(state)?;
                    if !state.at(ValkyrieTokenType::Comma) {
                        break;
                    }
                    state.bump();
                }
                if !state.at(ValkyrieTokenType::BracketR) {
                    return Err(state.source.syntax_error("数组字面量缺少闭合括号".into(), state.current_offset()));
                }
                state.bump();
                state.sink.finish_node(cp, ValkyrieElementType::ArrayExpression);
                Ok(())
            }
            ValkyrieTokenType::BraceL => parse_block_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::If) => parse_if_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Match) => parse_match_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Loop) => parse_loop_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::While) => parse_while_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::For) => parse_for_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Return) => parse_return_expression(state),
            ValkyrieTokenType::Label => parse_labeled_loop_expression(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Break) => {
                let cp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::Label) || state.at(ValkyrieTokenType::Identifier) {
                    state.bump();
                }
                state.sink.finish_node(cp, ValkyrieElementType::BreakExpression);
                Ok(())
            }
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Continue) => {
                let cp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::Label) || state.at(ValkyrieTokenType::Identifier) {
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
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro) => super::parse_control::parse_anonymous_micro(state),
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
                if state.at(ValkyrieTokenType::ParenthesisL) {
                    parse_argument_list(state)?;
                }
                state.sink.finish_node(cp, ValkyrieElementType::SuperCallExpression);
                Ok(())
            }
            _ => Err(state.source.syntax_error("此处需要表达式".into(), state.current_offset())),
        }
    }
    else {
        Err(state.source.syntax_error("缺少表达式".into(), state.current_offset()))
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

pub(crate) fn parse_labeled_loop_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_control::parse_labeled_loop_expression(state)
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
