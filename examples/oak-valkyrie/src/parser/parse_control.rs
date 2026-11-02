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
    // `if (a) == (b) { … }` 里的括号只是分组，不是条件括号；
    // 条件统一按“禁止对象字面量后缀”的完整表达式解析，`{` 留给 then 分支。
    crate::parser::parse_expressions::parse_condition_expression(state)?;
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

/// 解析 match 模式：`Some(value)` / `None` / `_` / `[head, ...tail]`。
fn parse_match_pattern<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let pcp = state.sink.checkpoint();
    // 字面量原子：完整模式或构造/数组子模式的单个 token，直接消费进当前 Pattern 节点；
    // builder 按叶节点映射 `Pattern::Literal`，不经零消费的递归调用产生空节点。
    if state.at(ValkyrieTokenType::IntegerLiteral)
        || state.at(ValkyrieTokenType::FloatLiteral)
        || state.at(ValkyrieTokenType::BoolLiteral)
        || state.at(ValkyrieTokenType::StringLiteral)
        || state.at(ValkyrieTokenType::CharLiteral)
    {
        state.bump();
    }
    else if state.at(ValkyrieTokenType::Underscore) {
        state.bump();
    }
    else if state.at(ValkyrieTokenType::BracketL) {
        state.bump();
        let mut rest_seen = false;
        while state.not_at_end() && !state.at(ValkyrieTokenType::BracketR) {
            state.skip_trivia();
            if state.at(ValkyrieTokenType::BracketR) {
                break;
            }
            // 未闭合保护：撞上 arm / 块结构边界立即按未闭合处理，不吞并后续结构。
            if state.at(ValkyrieTokenType::BraceR) || state.at(ValkyrieTokenType::Arrow) || state.at(ValkyrieTokenType::Colon) || state.at(ValkyrieTokenType::Eq) {
                break;
            }
            let before = state.checkpoint().0;
            if state.at(ValkyrieTokenType::DotDot) {
                if rest_seen {
                    return Err(state.source.syntax_error("数组模式只允许一个 ... 剩余绑定".into(), state.current_offset()));
                }
                rest_seen = true;
                state.bump();
                if state.at(ValkyrieTokenType::Dot) {
                    state.bump();
                }
                state.skip_trivia();
                let rest_before = state.checkpoint().0;
                parse_match_pattern(state)?;
                if state.checkpoint().0 == rest_before {
                    return Err(state.source.syntax_error("数组模式的 ... 后必须跟绑定模式".into(), state.current_offset()));
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
        if state.at(ValkyrieTokenType::BracketR) {
            state.bump();
        }
        else {
            return Err(state.source.syntax_error("数组模式缺少闭合方括号".into(), state.current_offset()));
        }
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
                // 与数组/字段模式同构的进展保护：字面量等未覆盖原子在此层不展开
                // （由 rusty-valkyrie 的源码恢复修补），但必须消费一个 token，
                // 否则 `case Some(60)` / `case Some('<')` 会原地死循环。
                let before = state.checkpoint().0;
                parse_match_pattern(state)?;
                if state.checkpoint().0 == before {
                    state.bump();
                }
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

/// 解析带标签的循环：`'label: loop/while/for …`。
pub(crate) fn parse_labeled_loop_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if state.at(ValkyrieTokenType::Colon) {
        state.bump();
    }
    match state.current().map(|token| &token.kind) {
        Some(ValkyrieTokenType::Keyword(ValkyrieKeywords::Loop)) => parse_loop_expression_with(state, cp),
        Some(ValkyrieTokenType::Keyword(ValkyrieKeywords::While)) => parse_while_expression_with(state, cp),
        Some(ValkyrieTokenType::Keyword(ValkyrieKeywords::For)) => parse_for_expression_with(state, cp),
        _ => Err(state.source.syntax_error("标签只能标注 loop / while / for".into(), state.current_offset())),
    }
}

/// 解析 loop 表达式
pub(crate) fn parse_loop_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_loop_expression_with(state, cp)
}

fn parse_loop_expression_with<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, cp: usize) -> Result<(), oak_core::OakError> {
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
    parse_while_expression_with(state, cp)
}

fn parse_while_expression_with<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, cp: usize) -> Result<(), oak_core::OakError> {
    state.bump();
    crate::parser::parse_expressions::parse_condition_expression(state)?;
    parse_block_expression(state)?;
    state.sink.finish_node(cp, ValkyrieElementType::LoopExpression);
    Ok(())
}

/// 解析 for 表达式
pub(crate) fn parse_for_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_for_expression_with(state, cp)
}

fn parse_for_expression_with<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, cp: usize) -> Result<(), oak_core::OakError> {
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
