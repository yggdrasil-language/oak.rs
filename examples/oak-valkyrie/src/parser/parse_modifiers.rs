use crate::{
    ValkyrieLanguage,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::element_type::ValkyrieElementType,
};
use oak_core::parser::ParserState;

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

/// getter/setter 前的修饰符锚点（如 `virtual get area`）。
pub(crate) fn is_member_accessor_keyword(kind: &ValkyrieTokenType) -> bool {
    matches!(kind, ValkyrieTokenType::Keyword(ValkyrieKeywords::Get) | ValkyrieTokenType::Keyword(ValkyrieKeywords::Set))
}

/// 字段名前的修饰符锚点（如 `readonly x: f64`、`mut total: i64`）。
pub(crate) fn is_field_name_follower(kind: &ValkyrieTokenType) -> bool {
    matches!(kind, ValkyrieTokenType::Identifier)
}

/// `let` 绑定模式前的修饰符锚点（如 `let mut x = 1`）。
pub(crate) fn is_pattern_follower(kind: &ValkyrieTokenType) -> bool {
    matches!(kind, ValkyrieTokenType::Identifier | ValkyrieTokenType::Underscore)
}

/// 形参名前的修饰符锚点（如 `mut self`）。
pub(crate) fn is_parameter_name_follower(kind: &ValkyrieTokenType) -> bool {
    matches!(kind, ValkyrieTokenType::Identifier)
}

/// 声明项起始关键字；其前的 `Identifier` 序列解析为 [`ValkyrieElementType::Modifier`]。
pub(crate) fn is_declaration_keyword(kind: &ValkyrieTokenType) -> bool {
    matches!(
        kind,
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Mezzo)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Namespace)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Class)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Struct)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Structure)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Enums)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Unity)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Enum)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Flags)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Trait)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Imply)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Widget)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Singleton)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Shader)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::Component)
            | ValkyrieTokenType::Keyword(ValkyrieKeywords::System)
    )
}

/// 解析锚点关键字前的 `Identifier` 修饰符序列。
pub(crate) fn parse_modifiers_followed_by<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, is_follower: impl Fn(&ValkyrieTokenType) -> bool) -> Result<(), oak_core::OakError> {
    while state.at(ValkyrieTokenType::Identifier) {
        let next = state.peek_non_trivia_kind_at(1);
        if !next.is_some_and(|kind| is_follower(&kind)) {
            break;
        }
        let cp = state.sink.checkpoint();
        state.bump();
        state.sink.finish_node(cp, ValkyrieElementType::Modifier);
    }
    Ok(())
}

/// 解析声明前的修饰符：`abstract` / `sealed` / `override` / `virtual` 等在词法上都是 `Identifier`。
pub(crate) fn parse_modifiers<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_modifiers_followed_by(state, is_declaration_keyword)
}

/// 解析字段名前的修饰符：`readonly` / `mut` 等在词法上都是 `Identifier`。
pub(crate) fn parse_field_modifiers<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_modifiers_followed_by(state, is_field_name_follower)
}

/// 解析 `let` 绑定前的修饰符（如 `let mut x`）。
pub(crate) fn parse_binding_modifiers<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_modifiers_followed_by(state, is_pattern_follower)
}

/// 解析形参名前的修饰符（如 `mut self`）。
pub(crate) fn parse_parameter_modifiers<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_modifiers_followed_by(state, is_parameter_name_follower)
}

/// `sealed class Foo` 等以 Identifier 开头的声明项。
pub(crate) fn dispatch_prefixed_declaration<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let next = state.peek_non_trivia_kind_at(1).ok_or_else(|| oak_core::OakError::custom_error("expected declaration keyword after modifier"))?;
    match next {
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro) => super::parse_items::parse_micro(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Mezzo) => super::parse_items::parse_mezzo(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Namespace) => super::parse_items::parse_namespace(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Class) => super::parse_items::parse_class(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Struct) | ValkyrieTokenType::Keyword(ValkyrieKeywords::Structure) => super::parse_items::parse_struct(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Enums) | ValkyrieTokenType::Keyword(ValkyrieKeywords::Unity) => super::parse_items::parse_enums(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Enum) => super::parse_items::parse_enum(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Flags) => super::parse_items::parse_flags(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Trait) => super::parse_items::parse_trait(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Imply) => super::parse_items::parse_imply(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Widget) => super::parse_items::parse_widget(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Singleton) => super::parse_items::parse_singleton(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Shader) => super::parse_items::parse_shader(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::Component) => super::parse_items::parse_component(state),
        ValkyrieTokenType::Keyword(ValkyrieKeywords::System) => super::parse_items::parse_system(state),
        _ => Ok(()),
    }
}
