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

/// Error-recovery loops must exit when no token is consumed, or the green tree grows exponentially.
#[inline]
fn stalled<S: oak_core::Source + ?Sized>(state: &State<'_, S>, before: usize) -> bool {
    token_index(state) == before
}

/// Type-name start tokens: `Self` / `micro` / `mezzo` are keywords; raw ids and ordinary identifiers are `Identifier`.
fn is_type_name_token(kind: &ValkyrieTokenType) -> bool {
    matches!(kind, ValkyrieTokenType::Identifier | ValkyrieTokenType::Keyword(ValkyrieKeywords::SelfType) | ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro) | ValkyrieTokenType::Keyword(ValkyrieKeywords::Mezzo))
}

/// 解析类型名路径：`std.text.utf16` / `core::primitive::usize` / `Self`。
///
/// 必须产出 `NamePath` 节点；分隔符同时接受 `.` 与 `::`。
/// 遇到 turbofish `::<` 时停在 `::` 之前。禁止只 bump 首段后把后续段摊成 Type 叶子。
fn parse_type_name_path<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    if let Some(token) = state.current() {
        if is_type_name_token(&token.kind) {
            state.bump();
        }
    }
    loop {
        let at_colon_colon = state.at(ValkyrieTokenType::ColonColon);
        let at_dot = state.at(ValkyrieTokenType::Dot);
        if !at_colon_colon && !at_dot {
            break;
        }
        if at_colon_colon && state.peek_non_trivia_kind_at(1) == Some(ValkyrieTokenType::LessThan) {
            break;
        }
        state.bump();
        if let Some(token) = state.current() {
            if is_type_name_token(&token.kind) {
                state.bump();
                continue;
            }
        }
        break;
    }
    state.sink.finish_node(cp, ValkyrieElementType::NamePath);
    Ok(())
}

/// Parse a type
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
    if state.at(ValkyrieTokenType::Ampersand) {
        state.bump();
    }
    // `micro`/`mezzo` 函数类型保持为 Type 下的关键词叶子（builder 靠它认函数形状）。
    // 普通标识符 / `Self` 走 NamePath，必须吃完 `.` / `::` 链。
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)) || state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Mezzo)) {
        state.bump();
    }
    else if let Some(token) = state.current() {
        if is_type_name_token(&token.kind) {
            parse_type_name_path(state)?;
        }
    }
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_argument_list(state)?;
    }
    if state.at(ValkyrieTokenType::ParenthesisL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::ParenthesisR) {
            let before = token_index(state);
            if state.at(ValkyrieTokenType::Identifier) && state.peek_non_trivia_kind_at(1) == Some(ValkyrieTokenType::Colon) {
                state.bump();
                state.bump();
                parse_type(state)?;
                if state.at(ValkyrieTokenType::Comma) {
                    state.bump();
                }
                continue;
            }
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

/// Parse micro/mezzo parameter generics: prefer `micro wrap<T>(...)`, also accept `micro wrap::<T>(...)`.
///
/// Distinct from type application `Envelope<T>` and call-site turbofish `foo::<T>()`.
pub(crate) fn parse_micro_generic_parameter_clause<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    if state.at(ValkyrieTokenType::ColonColon) && state.peek_non_trivia_kind_at(1) == Some(ValkyrieTokenType::LessThan) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_parameter_list(state)?;
    }
    Ok(())
}

/// Legacy name alias.
pub(crate) fn parse_term_generic_parameter_clause<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_micro_generic_parameter_clause(state)
}

/// Parse type-level generic parameter lists for type/unite/trait: `<T, U>`.
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

/// Parse a generic parameter list
pub(crate) fn parse_generic_argument_list<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    while state.not_at_end() && !state.at(ValkyrieTokenType::GreaterThan) {
        let before = token_index(state);
        if state.at(ValkyrieTokenType::Identifier) && state.peek_non_trivia_kind_at(1) == Some(ValkyrieTokenType::Eq) {
            state.bump();
            state.bump();
            parse_type(state)?;
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
            continue;
        }
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

/// Parse a parameter list
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

// Functions below are defined in other modules
pub(crate) fn parse_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_expressions::parse_expression(state)
}

/// 约束使用独立节点，不能把 bound 的 Type 节点挂到返回类型位置。
pub(crate) fn parse_where_clause<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    if !state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Where)) {
        return Ok(());
    }
    let cp = state.sink.checkpoint();
    state.bump();
    loop {
        let ccp = state.sink.checkpoint();
        let before = token_index(state);
        parse_type(state)?;
        if stalled(state, before) || !state.at(ValkyrieTokenType::Colon) {
            return Err(state.source.syntax_error("where 约束缺少目标类型或冒号".to_string(), state.current().map(|t| t.span.start).unwrap_or(0)));
        }
        state.bump();
        loop {
            let bcp = state.sink.checkpoint();
            if !state.at(ValkyrieTokenType::Identifier) {
                return Err(state.source.syntax_error("where 约束缺少 trait 路径".to_string(), state.current().map(|t| t.span.start).unwrap_or(0)));
            }
            parse_type_name_path(state)?;
            if state.at(ValkyrieTokenType::LessThan) {
                state.bump();
                loop {
                    let before = token_index(state);
                    if state.at(ValkyrieTokenType::Identifier) && state.peek_non_trivia_kind_at(1) == Some(ValkyrieTokenType::Eq) {
                        let ecp = state.sink.checkpoint();
                        state.bump();
                        state.bump();
                        let rhs = token_index(state);
                        parse_type(state)?;
                        if stalled(state, rhs) {
                            return Err(state.source.syntax_error("关联类型等式缺少右侧类型".to_string(), state.current().map(|t| t.span.start).unwrap_or(0)));
                        }
                        state.sink.finish_node(ecp, ValkyrieElementType::AssociatedTypeEquation);
                    }
                    else {
                        parse_type(state)?;
                    }
                    if stalled(state, before) {
                        return Err(state.source.syntax_error("trait bound 缺少类型实参".to_string(), state.current().map(|t| t.span.start).unwrap_or(0)));
                    }
                    if !state.at(ValkyrieTokenType::Comma) {
                        break;
                    }
                    state.bump();
                }
                if !state.at(ValkyrieTokenType::GreaterThan) {
                    return Err(state.source.syntax_error("trait bound 缺少闭合尖括号".to_string(), state.current().map(|t| t.span.start).unwrap_or(0)));
                }
                state.bump();
            }
            state.sink.finish_node(bcp, ValkyrieElementType::TraitBound);
            if !state.at(ValkyrieTokenType::Plus) && !state.at(ValkyrieTokenType::Ampersand) {
                break;
            }
            state.bump();
        }
        state.sink.finish_node(ccp, ValkyrieElementType::WhereConstraint);
        if !state.at(ValkyrieTokenType::Comma) {
            break;
        }
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::WhereClause);
    Ok(())
}
