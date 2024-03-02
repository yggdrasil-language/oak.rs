use crate::{
    ValkyrieLanguage,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::{element_type::ValkyrieElementType, parse_modifiers::parse_parameter_modifiers},
};
use oak_core::parser::ParserState;

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

#[inline]
fn token_index<S: oak_core::Source + ?Sized>(state: &State<'_, S>) -> usize {
    state.checkpoint().0
}

/// 容错循环若未消耗 token 则必须退出，否则 green tree 会指数膨胀并撑爆内存。
#[inline]
fn stalled<S: oak_core::Source + ?Sized>(state: &State<'_, S>, before: usize) -> bool {
    token_index(state) == before
}

/// 类型名起始 token：`Self` / `micro` / `mezzo` 为关键词，raw id 与普通标识符同为 `Identifier`。
fn is_type_name_token(kind: &ValkyrieTokenType) -> bool {
    matches!(
        kind,
        ValkyrieTokenType::Identifier
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::SelfType)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Mezzo)
    )
}

/// 解析类型
pub(crate) fn parse_type<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    if state.at(ValkyrieTokenType::BracketL) {
        state.bump();
        parse_type(state)?;
        if state.at(ValkyrieTokenType::Semicolon) {
            state.bump();
            parse_expression(state)?;
        }
        if state.at(ValkyrieTokenType::BracketR) {
            state.bump();
        }
        state.sink.finish_node(cp, ValkyrieElementType::Type);
        return Ok(());
    }
    if state.at(ValkyrieTokenType::Question) {
        state.bump();
    }
    if let Some(token) = state.current() {
        if is_type_name_token(&token.kind) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_argument_list(state)?;
    }
    if state.at(ValkyrieTokenType::ParenthesisL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::ParenthesisR) {
            let before = token_index(state);
            parse_type(state)?;
            if stalled(state, before) {
                break;
            }
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::ParenthesisR) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::Arrow) {
        state.bump();
        parse_type(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::Type);
    Ok(())
}

/// 解析 micro/mezzo 形参泛型：首选 `micro wrap<T>(...)`，亦接受 `micro wrap::<T>(...)`。
///
/// 与类型应用 `Envelope<T>`、调用点 turbofish `foo::<T>()` 分属不同语法位置。
pub(crate) fn parse_micro_generic_parameter_clause<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    if state.at(ValkyrieTokenType::ColonColon) && state.peek_non_trivia_kind_at(1) == Some(ValkyrieTokenType::LessThan) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_parameter_list(state)?;
    }
    Ok(())
}

/// 兼容旧名。
pub(crate) fn parse_term_generic_parameter_clause<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_micro_generic_parameter_clause(state)
}

/// 解析 type / unite / trait 等类型层泛型形参列表：`<T, U>`。
pub(crate) fn parse_generic_parameter_list<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    while state.not_at_end() && !state.at(ValkyrieTokenType::GreaterThan) {
        let before = token_index(state);
        let gcp = state.sink.checkpoint();
        if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Type)) {
            state.bump();
        }
        if state.at(ValkyrieTokenType::Identifier) {
            state.bump();
        }
        if state.at(ValkyrieTokenType::Colon) {
            state.bump();
            parse_type(state)?;
        }
        if state.at(ValkyrieTokenType::Eq) {
            state.bump();
            parse_type(state)?;
        }
        if stalled(state, before) {
            break;
        }
        state.sink.finish_node(gcp, ValkyrieElementType::GenericParameter);
        if state.at(ValkyrieTokenType::Comma) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::GreaterThan) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::GenericParameterList);
    Ok(())
}

/// 解析泛型参数列表
pub(crate) fn parse_generic_argument_list<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    while state.not_at_end() && !state.at(ValkyrieTokenType::GreaterThan) {
        let before = token_index(state);
        parse_type(state)?;
        if stalled(state, before) {
            break;
        }
        if state.at(ValkyrieTokenType::Comma) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::GreaterThan) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::GenericArgumentList);
    Ok(())
}

/// 解析参数列表
pub(crate) fn parse_parameter_list<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    while state.not_at_end() && !state.at(ValkyrieTokenType::ParenthesisR) {
        let pcp = state.sink.checkpoint();
        let progressed = state.at(ValkyrieTokenType::Identifier) || state.at(ValkyrieTokenType::Colon) || state.at(ValkyrieTokenType::Eq);
        parse_parameter_modifiers(state)?;
        if state.at(ValkyrieTokenType::Identifier) {
            state.bump();
        }
        if state.at(ValkyrieTokenType::Colon) {
            state.bump();
            parse_type(state)?;
        }
        if state.at(ValkyrieTokenType::Eq) {
            state.bump();
            parse_expression(state)?;
        }
        if !progressed {
            break;
        }
        state.sink.finish_node(pcp, ValkyrieElementType::Parameter);
        if state.at(ValkyrieTokenType::Comma) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::ParenthesisR) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::ParameterList);
    Ok(())
}

// 以下函数在其他模块中定义
pub(crate) fn parse_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_expressions::parse_expression(state)
}
