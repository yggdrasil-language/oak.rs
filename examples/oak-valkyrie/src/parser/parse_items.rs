use crate::{
    ValkyrieLanguage,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::{
        element_type::ValkyrieElementType,
        parse_modifiers::{dispatch_prefixed_declaration, is_declaration_keyword, is_member_accessor_keyword, parse_field_modifiers, parse_modifiers, parse_modifiers_followed_by},
    },
};
use oak_core::parser::ParserState;

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

/// 解析顶层项
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
    // `<% end %>` 之后若仍有仅空白 `TemplateText`（Resolver 拼接换行），跳过而不生成空 ExprStatement。
    while state.at(ValkyrieTokenType::TemplateText) {
        let token = *state.current().expect("template text");
        let text = state.source.get_text_in(token.span);
        if !text.trim().is_empty() {
            break;
        }
        state.bump();
        state.skip_trivia();
    }
    // Resolver 拼接源码后常在文件末尾留下换行；跳过 trivia 后若已到 Eof 则不再生成空 ExprStatement。
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
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Let) => {
                let cp = state.sink.checkpoint();
                parse_let_statement(state)?;
                state.sink.finish_node(cp, ValkyrieElementType::LetStatement);
                Ok(())
            }
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Widget) => parse_widget(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Singleton) => parse_singleton(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Shader) => parse_shader(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Component) => parse_component(state),
            ValkyrieTokenType::Keyword(ValkyrieKeywords::System) => parse_system(state),
            ValkyrieTokenType::At => parse_attribute_item(state),
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

/// 解析 micro 函数定义
pub(crate) fn parse_micro<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
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
    if state.at(ValkyrieTokenType::Arrow) {
        state.bump();
        parse_type(state)?;
    }
    if state.at(ValkyrieTokenType::BraceL) {
        parse_block(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::Micro);
    Ok(())
}

/// 解析 mezzo 函数定义
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
    if state.at(ValkyrieTokenType::Arrow) {
        state.bump();
        parse_type(state)?;
    }
    if state.at(ValkyrieTokenType::BraceL) {
        parse_block(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::Mezzo);
    Ok(())
}

/// 解析 namespace 定义
pub(crate) fn parse_namespace<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
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
    state.sink.finish_node(cp, ValkyrieElementType::Namespace);
    Ok(())
}

/// 解析 class 定义
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

/// 解析 class 成员
pub(crate) fn parse_class_member<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    while state.at(ValkyrieTokenType::At) {
        parse_attribute(state)?;
    }
    if state.at(ValkyrieTokenType::Identifier) {
        let next = state.peek_non_trivia_kind_at(1);
        if next.is_some_and(|kind| is_declaration_keyword(&kind)) {
            return dispatch_prefixed_declaration(state);
        }
        if next.is_some_and(|kind| is_member_accessor_keyword(&kind)) {
            return parse_property(state);
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
            ValkyrieTokenType::Keyword(ValkyrieKeywords::Get) | ValkyrieTokenType::Keyword(ValkyrieKeywords::Set) => parse_property(state),
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

/// 解析 property 定义
pub(crate) fn parse_property<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    parse_modifiers_followed_by(state, is_member_accessor_keyword)?;
    state.bump();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::Colon) {
        state.bump();
        parse_type(state)?;
    }
    if state.at(ValkyrieTokenType::BraceL) {
        parse_block(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::Property);
    Ok(())
}

/// 解析 struct 定义
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
            if state.at(ValkyrieTokenType::Eq) {
                state.bump();
                parse_expression(state)?;
            }
            state.sink.finish_node(fcp, ValkyrieElementType::Field);
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Struct);
    Ok(())
}

/// 解析 enums 定义（带变体块）
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
            parse_variant(state)?;
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

/// 解析 enum 定义（简单枚举）
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
            let vcp = state.sink.checkpoint();
            if state.at(ValkyrieTokenType::Identifier) {
                state.bump();
            }
            state.sink.finish_node(vcp, ValkyrieElementType::Variant);
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

/// 解析 variant 定义
pub(crate) fn parse_variant<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    while state.at(ValkyrieTokenType::At) {
        parse_attribute(state)?;
    }
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    if state.at(ValkyrieTokenType::ParenthesisL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::ParenthesisR) {
            parse_type(state)?;
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

/// 解析 flags 定义
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
            if state.at(ValkyrieTokenType::Identifier) {
                state.bump();
            }
            if state.at(ValkyrieTokenType::Comma) {
                state.bump();
            }
            else {
                break;
            }
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Flags);
    Ok(())
}

/// 解析 `imply` 实现块。
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
            while state.at(ValkyrieTokenType::At) {
                parse_attribute(state)?;
            }
            if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)) {
                parse_micro(state)?;
            }
            else if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Type)) {
                let acp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::Identifier) {
                    state.bump();
                }
                state.sink.finish_node(acp, ValkyrieElementType::AssociatedType);
            }
            else if state.at(ValkyrieTokenType::Identifier) {
                let mcp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::ParenthesisL) {
                    parse_parameter_list(state)?;
                }
                if state.at(ValkyrieTokenType::Colon) {
                    state.bump();
                    parse_type(state)?;
                }
                state.sink.finish_node(mcp, ValkyrieElementType::Method);
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

/// 解析 trait 定义
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
    if state.at(ValkyrieTokenType::BraceL) {
        state.bump();
        while state.not_at_end() && !state.at(ValkyrieTokenType::BraceR) {
            while state.at(ValkyrieTokenType::At) {
                parse_attribute(state)?;
            }
            if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)) {
                parse_micro(state)?;
            }
            else if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::Type)) {
                let acp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::Identifier) {
                    state.bump();
                }
                state.sink.finish_node(acp, ValkyrieElementType::AssociatedType);
            }
            else if state.at(ValkyrieTokenType::Identifier) {
                let mcp = state.sink.checkpoint();
                state.bump();
                if state.at(ValkyrieTokenType::ParenthesisL) {
                    parse_parameter_list(state)?;
                }
                if state.at(ValkyrieTokenType::Colon) {
                    state.bump();
                    parse_type(state)?;
                }
                state.sink.finish_node(mcp, ValkyrieElementType::Method);
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

/// 解析 using 语句
pub(crate) fn parse_using<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    state.bump();
    parse_name_path(state)?;
    if state.at(ValkyrieTokenType::Keyword(ValkyrieKeywords::As)) {
        state.bump();
        if state.at(ValkyrieTokenType::Identifier) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::UsingStatement);
    Ok(())
}

/// 解析 widget 定义
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

/// 解析 singleton 定义
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
            parse_item(state)?;
        }
        if state.at(ValkyrieTokenType::BraceR) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::Singleton);
    Ok(())
}

/// 解析属性项（以 @ 开头的顶层属性+项）
pub(crate) fn parse_attribute_item<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    while state.at(ValkyrieTokenType::At) {
        parse_attribute(state)?;
    }
    parse_item(state)
}

/// 解析 `[export(name: "answer"), main]` 形式的方括号属性列表。
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

/// 解析方括号属性列表中的一个属性项。
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

/// 解析属性参数：`name: "answer"` 或位置参数。
pub(crate) fn parse_attribute_argument<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    if state.at(ValkyrieTokenType::Identifier) {
        let next = state.peek_non_trivia_kind_at(1);
        if next == Some(ValkyrieTokenType::Colon) || next == Some(ValkyrieTokenType::Eq) {
            state.bump();
            state.bump();
            parse_expression(state)?;
        }
        else {
            parse_expression(state)?;
        }
    }
    else {
        parse_expression(state)?;
    }
    state.sink.finish_node(cp, ValkyrieElementType::AttributeArgument);
    Ok(())
}

/// 解析属性
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

/// 解析名称路径
pub(crate) fn parse_name_path<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.sink.checkpoint();
    if state.at(ValkyrieTokenType::Identifier) {
        state.bump();
    }
    while state.at(ValkyrieTokenType::ColonColon) {
        state.bump();
        if state.at(ValkyrieTokenType::Identifier) {
            state.bump();
        }
    }
    state.sink.finish_node(cp, ValkyrieElementType::NamePath);
    Ok(())
}

// 以下函数在其他模块中定义
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

/// 解析 shader 定义
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

/// 解析 component 定义
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

/// 解析 component 成员
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

/// 解析 event 定义
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

/// 解析 system 定义
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
