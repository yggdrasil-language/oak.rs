use crate::{
    ValkyrieLanguage,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::{
        element_type::ValkyrieElementType,
        parse_modifiers::{dispatch_prefixed_declaration, is_declaration_keyword, is_member_name_token, is_raw_id_follower, parse_field_modifiers, parse_modifiers, parse_modifiers_followed_by},
    },
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

/// Parse a top-level item
pub(crate) fn parse_item<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    state.skip_trivia();
    while state.at(ValkyrieTokenType::BracketL) {
        parse_bracket_attribute_list(state)?;
        state.skip_trivia();
    }
    while state.at(ValkyrieTokenType::At) {
        parse_attribute(state)?;
        state.skip_trivia();
    }
    // After `<% end %>`, skip whitespace-only `TemplateText` (Resolver-appended newlines) instead of emitting an empty ExprStatement.
    while state.at(ValkyrieTokenType::TemplateText) {
        let token = *state.current().expect("template text");
        let text = state.source.get_text_in(token.span);
        if !text.trim().is_empty() {
            break;
        }
        state.bump();
        state.skip_trivia();
    }
    // Resolver concatenation often leaves trailing newlines; after skipping trivia at Eof, do not emit an empty ExprStatement.
    if state.at(ValkyrieTokenType::Eof) {
        return Ok(());
    }
    if state.at(ValkyrieTokenType::Identifier) {
        let next = state.peek_non_trivia_kind_at(1);
        if next.is_some_and(|kind| is_declaration_keyword(&kind)) {
            return dispatch_prefixed_declaration(state);
        }
    }
    if let Some(token) = state.current() {
        match &token.kind {
            ValkyrieTokenType::TemplateL => super::parse_template::parse_template_item(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro) => parse_micro(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Mezzo) => parse_mezzo(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Namespace) => parse_namespace(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Class) => parse_class(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Struct) | ValkyrieTokenType::Keyword(ValkyrieKeywords::Structure) => parse_struct(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Enums) | ValkyrieTokenType::Keyword(ValkyrieKeywords::Unity) => parse_enums(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Enum) => parse_enum(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Flags) => parse_flags(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Trait) => parse_trait(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Imply) => parse_imply(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Using) => parse_using(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Type) => parse_type_alias(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Let) => {
                let cp = state.sink.checkpoint();
                parse_let_statement(state)?;
                state.sink.finish_node(cp, ValkyrieElementType::LetStatement);
                Ok(())
            }
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Widget) => parse_widget(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Union) => parse_union(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Singleton) => parse_singleton(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Shader) => parse_shader(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Component) => parse_component(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::System) => parse_system(state),
            ValkyrieTokenType::At => parse_attribute_item(state),
            ValkyrieTokenType::Error => {
                let offset = token.span.start;
                Err(state.source.syntax_error("unexpected lexical error token while parsing item".to_string(), offset))
            }
            _ => {
                let cp = state.sink.checkpoint();
                parse_expr_statement(state)?;
                state.sink.finish_node(cp, ValkyrieElementType::ExprStatement);
                Ok(())
            }
        }
    }
    else {
        Ok(())
    }
}

/// Parse a module-level type alias; type aliases carry no runtime members.
fn parse_type_alias<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::Eq) {
        state.bump();
        parse_type(state)?;
    }
    if state.at(ValkyrieTokenType::Semicolon) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::TypeAlias);
    Ok(())
}

/// Declarations like `micro yield()` use soft keywords as function names; consume before reading the signature.
fn bump_micro_name<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) {
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
        return;
    }
    if matches!(state.current().map(|token| &token.kind), Some(ValkyrieTokenType::Keyword(_))) {
        let next = state.peek_non_trivia_kind_at(1);
        if matches!(next, Some(ValkyrieTokenType::ParenthesisL) | Some(ValkyrieTokenType::LessThan) | Some(ValkyrieTokenType::ColonColon)) {
            state.bump();
        }
    }
}

/// Parse a micro function definition
pub(crate) fn parse_micro<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_micro_member(state, true)
}

/// Parse a micro member.
///
/// `implicit_empty_body` 保留历史上的恢复体：顶层与类成员的宿主声明（`micro write(message: utf8): unit`）
/// 依赖一个空 `BlockExpression` 表示“已声明但无实现”。声明容器（trait）传 `false`，
/// 让无体成员保持 `body: None`，与 `;` 结尾的纯声明形态一致。
fn parse_micro_member<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, implicit_empty_body: bool) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    bump_micro_name(state);
    parse_micro_generic_parameter_clause(state)?;
    if state.at(ValkyrieTokenType::ParenthesisL) {
        parse_parameter_list(state)?;
    }
    if state.at(ValkyrieTokenType::Arrow) || state.at(ValkyrieTokenType::Colon) {
        state.bump();
        parse_type(state)?;
    }
    crate::parser::parse_types::parse_where_clause(state)?;
    if state.at(ValkyrieTokenType::BraceL) {
        parse_block(state)?;
    }
    else if state.at(ValkyrieTokenType::Semicolon) {
        // Abstract / signature-only declaration: `micro poll(self): bool;`
        state.bump();
        // Do NOT create a BlockExpression — let the builder detect body.is_none() → is_abstract = true.
    }
    else if implicit_empty_body {
        // Implicit empty body for error recovery.
        let bcp = state.sink.checkpoint();
        state.sink.finish_node(bcp, ValkyrieElementType::BlockExpression);
    }
    state.sink.finish_node(cp, ValkyrieElementType::Micro);
    Ok(())
}

/// Parse a mezzo function definition
pub(crate) fn parse_mezzo<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    parse_micro_generic_parameter_clause(state)?;
    if state.at(ValkyrieTokenType::ParenthesisL) {
        parse_parameter_list(state)?;
    }
    if state.at(ValkyrieTokenType::Arrow) || state.at(ValkyrieTokenType::Colon) {
        state.bump();
        parse_type(state)?;
    }
    crate::parser::parse_types::parse_where_clause(state)?;
    if state.at(ValkyrieTokenType::BraceL) {
        parse_block(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::Mezzo);
    Ok(())
}

/// Parse a namespace definition
pub(crate) fn parse_namespace<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    // File-level namespace declaration: `namespace! core::types;`.
    // `!` 是主空间身份，保留在 CST；AST 由 builder 记录 `primary`。
    if state.at(ValkyrieTokenType::Bang) {
        state.bump();
    }
    if !matches!(state.current().map(|token| &token.kind), Some(ValkyrieTokenType::Identifier | ValkyrieTokenType::Keyword(_))) {
        let offset = state.current().map(|token| token.span.start).unwrap_or(0);
        return Err(state.source.syntax_error("namespace declaration requires a name path".to_string(), offset));
    }
    parse_name_path(state)?;
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            parse_item(state)?;
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    else if state.at(ValkyrieTokenType::Semicolon) {
        // Empty namespace block `namespace marker;` is standard in core/std source.
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::Namespace);
    Ok(())
}

/// Parse a class definition
pub(crate) fn parse_class<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_parameter_list(state)?;
    }
    if state.at(ValkyrieTokenType::Colon) {
        state.bump();
        parse_type(state)?;
        while state.at(ValkyrieTokenType::Plus) {
            state.bump();
            parse_type(state)?;
        }
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            parse_class_member(state)?;
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Class);
    Ok(())
}

/// Parse a class-like member.
///
/// 成员产生式只有一条：`修饰符列表 + 名字`，然后按名字之后的 token 决定类别：
///
/// - `(` → method（参数列表、可选返回类型、方法体）
/// - `:` → field（类型、可选默认值）
///
/// 修饰符只是位置上的 `Identifier` 序列，语法层不读名字文本。
pub(crate) fn parse_class_member<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    // 成员属性与条目属性同形：`[export(...)]` 或 `@export(...)`，都留在成员节点之前。
    while state.at(ValkyrieTokenType::BracketL) {
        parse_bracket_attribute_list(state)?;
        state.skip_trivia();
    }
    while state.at(ValkyrieTokenType::At) {
        parse_attribute(state)?;
        state.skip_trivia();
    }
    if state.at(ValkyrieTokenType::Identifier) {
        let next = state.peek_non_trivia_kind_at(1);
        if next.is_some_and(|kind| is_declaration_keyword(&kind)) {
            return dispatch_prefixed_declaration(state);
        }
        if let Some(after_name) = member_shape_after_modifiers(state) {
            let cp = state.sink.checkpoint();
            parse_modifiers_followed_by(state, is_member_name_token)?;
            state.bump();
            if after_name == ValkyrieTokenType::ParenthesisL {
                parse_parameter_list(state)?;
                if state.at(ValkyrieTokenType::Arrow) || state.at(ValkyrieTokenType::Colon) {
                    state.bump();
                    parse_type(state)?;
                }
                crate::parser::parse_types::parse_where_clause(state)?;
                if state.at(ValkyrieTokenType::BraceL) {
                    parse_block(state)?;
                }
                state.sink.finish_node(cp, ValkyrieElementType::Method);
            }
            else if state.at(ValkyrieTokenType::Colon) {
                state.bump();
                parse_type(state)?;
                if state.at(ValkyrieTokenType::Eq) {
                    state.bump();
                    parse_expression(state)?;
                }
                state.sink.finish_node(cp, ValkyrieElementType::Field);
                if state.at(ValkyrieTokenType::Comma) {
                    state.bump();
                }
            }
            else {
                debug_assert!(false, "member shape must end with `(` or `:`");
            }
            return Ok(());
        }
    }
    if let Some(token) = state.current() {
        match &token.kind {
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro) => parse_micro(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Let) => {
                let cp = state.sink.checkpoint();
                parse_let_statement(state)?;
                state.sink.finish_node(cp, ValkyrieElementType::Field);
                Ok(())
            }
            _ => {
                let cp = state.sink.checkpoint();
                parse_expr_statement(state)?;
                state.sink.finish_node(cp, ValkyrieElementType::ExprStatement);
                Ok(())
            }
        }
    }
    else {
        Ok(())
    }
}

/// 判断 `Identifier` 成员是否成立：跳过修饰符后，最后一个名字后面必须是 `(` 或 `:`。
///
/// 只看 token 类别与位置，不读名字文本。
fn member_shape_after_modifiers<S: oak_core::Source + ?Sized>(state: &State<'_, S>) -> Option<ValkyrieTokenType> {
    let mut offset = 0usize;
    while state.peek_non_trivia_kind_at(offset).is_some_and(|kind| is_member_name_token(&kind)) {
        offset += 1;
    }
    if offset == 0 {
        return None;
    }
    match state.peek_non_trivia_kind_at(offset) {
        Some(kind @ (ValkyrieTokenType::ParenthesisL | ValkyrieTokenType::Colon)) => Some(kind),
        _ => None,
    }
}

/// Parse a struct definition
pub(crate) fn parse_struct<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_parameter_list(state)?;
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            state.skip_trivia();
            while state.at(ValkyrieTokenType::BracketL) {
                parse_bracket_attribute_list(state)?;
                state.skip_trivia();
            }
            while state.at(ValkyrieTokenType::At) {
                parse_attribute(state)?;
                state.skip_trivia();
            }
            if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)) {
                parse_micro(state)?;
                continue;
            }
            let fcp = state.sink.checkpoint();
            let before = state.tokens.index();
            parse_field_modifiers(state)?;
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
            state.sink.finish_node(fcp, ValkyrieElementType::Field);
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
            if state.tokens.index() == before {
                let offset = state.current().map(|token| token.span.start).unwrap_or(0);
                return Err(state.source.syntax_error("structure member parsing stalled on unrecognized token".to_string(), offset));
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Struct);
    Ok(())
}

/// Parse an enums definition (with variant block)
pub(crate) fn parse_enums<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_parameter_list(state)?;
    }
    if state.at(ValkyrieTokenType::Colon) {
        state.bump();
        parse_type(state)?;
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            state.skip_trivia();
            let before = token_index(state);
            parse_variant(state)?;
            if stalled(state, before) {
                break;
            }
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Enums);
    Ok(())
}

/// Parse an enum definition (simple enumeration)
pub(crate) fn parse_enum<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            state.skip_trivia();
            let before = token_index(state);
            let vcp = state.sink.checkpoint();
            if state.at(ValkyrieTokenType::Identifier) {
                state.bump();
            }
            if state.at(ValkyrieTokenType::Eq) {
                state.bump();
                parse_expression(state)?;
            }
            state.sink.finish_node(vcp, ValkyrieElementType::Variant);
            if stalled(state, before) {
                break;
            }
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Enum);
    Ok(())
}

/// Parse a variant definition
pub(crate) fn parse_variant<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.skip_trivia();
    while state.at(ValkyrieTokenType::BracketL) {
        parse_bracket_attribute_list(state)?;
        state.skip_trivia();
    }
    while state.at(ValkyrieTokenType::At) {
        parse_attribute(state)?;
        state.skip_trivia();
    }
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::Eq) {
        state.bump();
        parse_expression(state)?;
    }
    if state.at(ValkyrieTokenType::ParenthesisL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::ParenthesisR) {
            let before = token_index(state);
            parse_type(state)?;
            if stalled(state, before) {
                break;
            }
            if state.at(ValkyrieTokenType::Identifier) {
                state.bump();
            }
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::ParenthesisR) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            state.skip_trivia();
            let before = token_index(state);
            while state.at(ValkyrieTokenType::At) {
                parse_attribute(state)?;
            }
            let fcp = state.sink.checkpoint();
            parse_field_modifiers(state)?;
            if state.at(ValkyrieTokenType::Identifier) {
                state.bump();
            }
            if state.at(ValkyrieTokenType::Colon) {
                state.bump();
                parse_type(state)?;
            }
            state.sink.finish_node(fcp, ValkyrieElementType::Field);
            if stalled(state, before) {
                break;
            }
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Variant);
    Ok(())
}

/// Parse a flags definition
pub(crate) fn parse_flags<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            state.skip_trivia();
            let before = token_index(state);
            if state.at(ValkyrieTokenType::Identifier) {
                state.bump();
            }
            if state.at(ValkyrieTokenType::Eq) {
                state.bump();
                parse_expression(state)?;
            }
            if stalled(state, before) {
                break;
            }
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Flags);
    Ok(())
}

/// Parse an associated type item: `type Item;`, `type Iter: Iterator<Item=Self::Item>;`, `type Output = T`.
fn parse_associated_type_item<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
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
    if state.at(ValkyrieTokenType::Semicolon) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::AssociatedType);
    Ok(())
}

/// 方法成员（trait / imply 共用）：`prefix`/`infix`/`postfix`/`suffix` 修饰符 + raw id（`` `==` ``），
/// 或普通 Identifier 方法名；签名后可带 `where` 子句与 `{ ... }` 方法体。
/// 无体成员不产出 `BlockExpression`，保持 `body: None` 的纯声明形态。
fn parse_method_member<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers_followed_by(state, is_raw_id_follower)?;
    if !state.at(ValkyrieTokenType::Identifier) {
        let offset = state.current().map(|token| token.span.start).unwrap_or(0);
        return Err(state.source.syntax_error("imply member missing method name (raw id or identifier)".to_string(), offset));
    }
    state.bump();
    parse_micro_generic_parameter_clause(state)?;
    if state.at(ValkyrieTokenType::ParenthesisL) {
        parse_parameter_list(state)?;
    }
    if state.at(ValkyrieTokenType::Arrow) || state.at(ValkyrieTokenType::Colon) {
        state.bump();
        parse_type(state)?;
    }
    crate::parser::parse_types::parse_where_clause(state)?;
    if state.at(ValkyrieTokenType::BraceL) {
        parse_block(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::Micro);
    Ok(())
}

/// Parse an `imply` implementation block.
pub(crate) fn parse_imply<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_parameter_list(state)?;
    }
    parse_type(state)?;
    if state.at(ValkyrieTokenType::Colon) {
        state.bump();
        parse_type(state)?;
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            state.skip_trivia();
            while state.at(ValkyrieTokenType::At) {
                parse_attribute(state)?;
                state.skip_trivia();
            }
            let before = token_index(state);
            if state.at(ValkyrieTokenType::BracketL) {
                parse_bracket_attribute_list(state)?;
                continue;
            }
            if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)) {
                parse_micro(state)?;
            }
            else if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Type)) {
                parse_associated_type_item(state)?;
            }
            else if state.at(ValkyrieTokenType::Identifier) {
                parse_method_member(state)?;
            }
            else if stalled(state, before) {
                break;
            }
            else {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Imply);
    Ok(())
}

/// Parse a trait definition
pub(crate) fn parse_trait<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_parameter_list(state)?;
    }
    if state.at(ValkyrieTokenType::Colon) {
        state.bump();
        // supertrait 列表以 `&` 连接（`trait X: Y & Z`）：每个 bound 都是独立的
        // `Type` 节点，`&` 作为分隔符留在 Trait 节点下，不进任何 bound。
        parse_type(state)?;
        while state.at(ValkyrieTokenType::Ampersand) {
            state.bump();
            parse_type(state)?;
        }
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            state.skip_trivia();
            while state.at(ValkyrieTokenType::At) {
                parse_attribute(state)?;
                state.skip_trivia();
            }
            if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)) {
                // Trait 成员里的 `micro` 是签名/默认体，不产出宿主声明用的隐式空体。
                parse_micro_member(state, false)?;
            }
            else if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Type)) {
                parse_associated_type_item(state)?;
            }
            else if state.at(ValkyrieTokenType::Identifier) {
                parse_method_member(state)?;
            }
            else {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Trait);
    Ok(())
}

/// Parse a using statement
pub(crate) fn parse_using<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    // `using!` 是主空间专属导入构造，不是 `using` 的别名：必须消费 `Bang`，
    // 让身份留在 CST（builder 记录到 `UsingDeclaration.primary`）。
    if state.at(ValkyrieTokenType::Bang) {
        state.bump();
    }
    if !matches!(state.current().map(|token| &token.kind), Some(ValkyrieTokenType::Identifier | ValkyrieTokenType::Keyword(_))) {
        let offset = state.current().map(|token| token.span.start).unwrap_or(0);
        return Err(state.source.syntax_error("using declaration requires a name path".to_string(), offset));
    }
    parse_using_name_path(state)?;
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            state.skip_trivia();
            if state.at(ValkyrieTokenType::Identifier) {
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
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::As)) {
        state.bump();
        if !state.at(ValkyrieTokenType::Identifier) {
            let offset = state.current().map(|token| token.span.start).unwrap_or(0);
            return Err(state.source.syntax_error("using alias requires an identifier".to_string(), offset));
        }
        state.bump();
    }
    if state.at(ValkyrieTokenType::Semicolon) {
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::UsingStatement);
    Ok(())
}

/// 解析导入路径：空路径、尾随 `::`/`.` 与尾随分隔符后接非名字，都是畸形导入。
fn parse_using_name_path<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    while state.at(ValkyrieTokenType::ColonColon) || state.at(ValkyrieTokenType::Dot) {
        let separator_offset = state.current().map(|token| token.span.start).unwrap_or(0);
        state.bump();
        // `using path::{A, B}` 的分隔符后直接接选择列表，是合法形态。
        if state.at(ValkyrieTokenType::BraceL) {
            break;
        }
        if !matches!(state.current().map(|token| &token.kind), Some(ValkyrieTokenType::Identifier | ValkyrieTokenType::Keyword(_))) {
            return Err(state.source.syntax_error("using path must not end with a path separator".to_string(), separator_offset));
        }
        state.bump();
    }
    state.sink.finish_node(cp, ValkyrieElementType::NamePath);
    Ok(())
}

/// Parse a union definition
pub(crate) fn parse_union<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::LessThan) {
        parse_generic_parameter_list(state)?;
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            state.skip_trivia();
            while state.at(ValkyrieTokenType::At) {
                parse_attribute(state)?;
                state.skip_trivia();
            }
            let fcp = state.sink.checkpoint();
            let before = state.tokens.index();
            parse_field_modifiers(state)?;
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
            state.sink.finish_node(fcp, ValkyrieElementType::Field);
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
            if state.tokens.index() == before {
                let offset = state.current().map(|token| token.span.start).unwrap_or(0);
                return Err(state.source.syntax_error("union member parsing stalled on unrecognized token".to_string(), offset));
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Union);
    Ok(())
}

/// Parse a widget definition
pub(crate) fn parse_widget<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            parse_item(state)?;
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Widget);
    Ok(())
}

/// Parse a singleton definition
pub(crate) fn parse_singleton<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::Colon) {
        state.bump();
        parse_type(state)?;
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            parse_class_member(state)?;
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Singleton);
    Ok(())
}

/// Parse an attribute item (top-level `@` attribute plus item)
pub(crate) fn parse_attribute_item<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    while state.at(ValkyrieTokenType::At) {
        parse_attribute(state)?;
    }
    parse_item(state)
}

/// Parse a bracket attribute list like `[export(name: "answer"), main]`.
pub(crate) fn parse_bracket_attribute_list<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    state.bump();
    state.skip_trivia();
    while state.not_at_end() && !state.at(ValkyrieTokenType::BracketR) {
        let before = state.checkpoint().0;
        parse_bracket_attribute(state)?;
        if state.checkpoint().0 == before {
            break;
        }
        state.skip_trivia();
        if state.at(ValkyrieTokenType::Comma) {
            state.bump();
            state.skip_trivia();
        }
    }
    if state.at(ValkyrieTokenType::BracketR) {
        state.bump();
    }
    Ok(())
}

/// Parse one attribute entry in a bracket attribute list.
pub(crate) fn parse_bracket_attribute<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_name_path(state)?;
    if state.at(ValkyrieTokenType::ParenthesisL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::ParenthesisR) {
            parse_attribute_argument(state)?;
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::ParenthesisR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Attribute);
    Ok(())
}

/// Soft keywords like `case` are named-argument keys in `[export(case: "...")]`; do not use the expression main path.
fn is_attribute_argument_name<S: oak_core::Source + ?Sized>(state: &State<'_, S>) -> bool {
    let next = state.peek_non_trivia_kind_at(1);
    let named = next == Some(ValkyrieTokenType::Colon) || next == Some(ValkyrieTokenType::Eq);
    if state.at(ValkyrieTokenType::Identifier) {
        return true;
    }
    named && matches!(state.current().map(|token| &token.kind), Some(ValkyrieTokenType::Keyword(_)))
}

/// Parse an attribute argument: `name: "answer"` or a positional argument.
pub(crate) fn parse_attribute_argument<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    if is_attribute_argument_name(state) {
        let next = state.peek_non_trivia_kind_at(1);
        if next == Some(ValkyrieTokenType::Colon) || next == Some(ValkyrieTokenType::Eq) {
            state.bump();
            state.bump();
            crate::parser::parse_expressions::parse_expression(state)?;
        }
        else {
            crate::parser::parse_expressions::parse_expression(state)?;
        }
    }
    else {
        crate::parser::parse_expressions::parse_expression(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::AttributeArgument);
    Ok(())
}

/// Parse an attribute
pub(crate) fn parse_attribute<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::ParenthesisL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::ParenthesisR) {
            parse_attribute_argument(state)?;
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::ParenthesisR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Attribute);
    Ok(())
}

/// Parse a name path
pub(crate) fn parse_name_path<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    if matches!(state.current().map(|token| &token.kind), Some(ValkyrieTokenType::Identifier | ValkyrieTokenType::Keyword(_))) {
        state.bump();
    }
    while state.at(ValkyrieTokenType::ColonColon) || state.at(ValkyrieTokenType::Dot) {
        state.bump();
        if matches!(state.current().map(|token| &token.kind), Some(ValkyrieTokenType::Identifier | ValkyrieTokenType::Keyword(_))) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::NamePath);
    Ok(())
}

// Functions below are defined in other modules
pub(crate) fn parse_micro_generic_parameter_clause<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_types::parse_micro_generic_parameter_clause(state)
}

pub(crate) fn parse_term_generic_parameter_clause<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_types::parse_term_generic_parameter_clause(state)
}

pub(crate) fn parse_generic_parameter_list<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_types::parse_generic_parameter_list(state)
}

pub(crate) fn parse_parameter_list<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_types::parse_parameter_list(state)
}

pub(crate) fn parse_type<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_types::parse_type(state)
}

pub(crate) fn parse_block<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_blocks::parse_block(state)
}

pub(crate) fn parse_let_statement<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_statements::parse_let_statement(state)
}

pub(crate) fn parse_expr_statement<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_statements::parse_expr_statement(state)
}

pub(crate) fn parse_expression<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    crate::parser::parse_expressions::parse_expression(state)
}

/// Parse a shader definition
pub(crate) fn parse_shader<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::By)) {
        state.bump();
        if state.at(ValkyrieTokenType::Identifier) {
            state.bump();
        }
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            parse_item(state)?;
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Shader);
    Ok(())
}

/// Parse a component definition
pub(crate) fn parse_component<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            parse_component_member(state)?;
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Component);
    Ok(())
}

/// Parse a component member
pub(crate) fn parse_component_member<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    while state.at(ValkyrieTokenType::At) {
        parse_attribute(state)?;
    }
    if let Some(token) = state.current() {
        match &token.kind {
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Events) => parse_event(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Let) => {
                let cp = state.sink.checkpoint();
                parse_let_statement(state)?;
                state.sink.finish_node(cp, ValkyrieElementType::Field);
                Ok(())
            }
            _ => {
                let cp = state.sink.checkpoint();
                parse_expr_statement(state)?;
                state.sink.finish_node(cp, ValkyrieElementType::ExprStatement);
                Ok(())
            }
        }
    }
    else {
        Ok(())
    }
}

/// Parse an event definition
pub(crate) fn parse_event<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::Colon) {
        state.bump();
        if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)) {
            parse_micro(state)?;
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Event);
    Ok(())
}

/// Parse a system definition
pub(crate) fn parse_system<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers(state)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            parse_class_member(state)?;
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::System);
    Ok(())
}
