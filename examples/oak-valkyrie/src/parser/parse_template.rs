use crate::{
    ValkyrieLanguage,
    lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::{
        element_type::ValkyrieElementType,
        template_directive::{ParsedTemplateDirective, directive_first_keyword, is_else_if_directive, parse_template_directive},
    },
};
use oak_core::{OakError, Token, TokenType, parser::ParserState};

type State<'a, S> = ParserState<'a, ValkyrieLanguage, S>;

/// 解析顶层 TGrammar 项。
pub(crate) fn parse_template_item<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_template_node(state)?;
    Ok(())
}

/// 解析块内 TGrammar 语句。
pub(crate) fn parse_template_statement<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    parse_template_node(state)?;
    Ok(())
}

fn parse_template_node<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), oak_core::OakError> {
    let cp = state.checkpoint();
    let open = read_and_bump_directive(state)?;
    if is_unexpected_boundary_directive(&open.tokens) {
        return Err(oak_core::OakError::custom_error("unexpected template boundary directive"));
    }
    match directive_first_keyword(&open.tokens) {
        Some(ValkyrieKeywords::If) => parse_if_block(state, cp)?,
        Some(ValkyrieKeywords::Loop) => parse_loop_block(state, cp)?,
        Some(ValkyrieKeywords::Match) => parse_match_block(state, cp)?,
        _ => {
            state.finish_at(cp, ValkyrieElementType::TemplateStatement);
        }
    }
    Ok(())
}

fn is_unexpected_boundary_directive(tokens: &[Token<ValkyrieTokenType>]) -> bool {
    is_else_if_directive(tokens) || matches!(directive_first_keyword(tokens), Some(ValkyrieKeywords::Case) | Some(ValkyrieKeywords::Else) | Some(ValkyrieKeywords::End))
}

fn is_if_arm_boundary(tokens: &[Token<ValkyrieTokenType>]) -> bool {
    is_else_if_directive(tokens) || matches!(directive_first_keyword(tokens), Some(ValkyrieKeywords::Else) | Some(ValkyrieKeywords::End))
}

fn is_match_arm_boundary(tokens: &[Token<ValkyrieTokenType>]) -> bool {
    matches!(directive_first_keyword(tokens), Some(ValkyrieKeywords::Case) | Some(ValkyrieKeywords::Else) | Some(ValkyrieKeywords::End))
}

fn parse_if_block<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, if_cp: (usize, usize)) -> Result<(), oak_core::OakError> {
    parse_template_body(state, is_if_arm_boundary)?;
    state.finish_at(if_cp, ValkyrieElementType::TemplateIfFragment);

    while state.at(ValkyrieTokenType::TemplateL) {
        let boundary_cp = state.checkpoint();
        let directive = read_and_bump_directive(state)?;
        if directive_first_keyword(&directive.tokens) == Some(ValkyrieKeywords::End) {
            state.finish_at(boundary_cp, ValkyrieElementType::TemplateEndFragment);
            state.finish_at(if_cp, ValkyrieElementType::TemplateIfStatement);
            return Ok(());
        }
        if is_else_if_directive(&directive.tokens) {
            parse_template_body(state, is_if_arm_boundary)?;
            state.finish_at(boundary_cp, ValkyrieElementType::TemplateIfFragment);
            continue;
        }
        if directive_first_keyword(&directive.tokens) == Some(ValkyrieKeywords::Else) {
            parse_template_body(state, is_if_arm_boundary)?;
            state.finish_at(boundary_cp, ValkyrieElementType::TemplateIfFragment);
            if !state.at(ValkyrieTokenType::TemplateL) {
                return Err(oak_core::OakError::custom_error("expected `<% end %>` after `<% else %>`"));
            }
            let end_cp = state.checkpoint();
            let close = read_and_bump_directive(state)?;
            if directive_first_keyword(&close.tokens) != Some(ValkyrieKeywords::End) {
                return Err(oak_core::OakError::custom_error("expected `<% end %>` after `<% else %>`"));
            }
            state.finish_at(end_cp, ValkyrieElementType::TemplateEndFragment);
            state.finish_at(if_cp, ValkyrieElementType::TemplateIfStatement);
            return Ok(());
        }
        return Err(oak_core::OakError::custom_error("expected `<% else %>`, `<% else if %>` or `<% end %>`"));
    }

    Err(oak_core::OakError::custom_error("unclosed `<% if %>` block"))
}

fn parse_loop_block<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, loop_cp: (usize, usize)) -> Result<(), oak_core::OakError> {
    state.finish_at(loop_cp, ValkyrieElementType::TemplateLoopFragment);
    parse_template_body(state, |tokens| directive_first_keyword(tokens) == Some(ValkyrieKeywords::End))?;
    if !state.at(ValkyrieTokenType::TemplateL) {
        return Err(oak_core::OakError::custom_error("expected `<% end %>` to close `<% loop %>`"));
    }
    let end_cp = state.checkpoint();
    let close = read_and_bump_directive(state)?;
    if directive_first_keyword(&close.tokens) != Some(ValkyrieKeywords::End) {
        return Err(oak_core::OakError::custom_error("expected `<% end %>` to close `<% loop %>`"));
    }
    state.finish_at(end_cp, ValkyrieElementType::TemplateEndFragment);
    state.finish_at(loop_cp, ValkyrieElementType::TemplateLoop);
    Ok(())
}

fn parse_match_block<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, match_cp: (usize, usize)) -> Result<(), oak_core::OakError> {
    state.finish_at(match_cp, ValkyrieElementType::TemplateMatchFragment);
    lint_match_prelude_gap(state)?;

    while state.at(ValkyrieTokenType::TemplateL) {
        let boundary_cp = state.checkpoint();
        let directive = read_and_bump_directive(state)?;
        match directive_first_keyword(&directive.tokens) {
            Some(ValkyrieKeywords::Case) => {
                parse_template_body(state, is_match_arm_boundary)?;
                state.finish_at(boundary_cp, ValkyrieElementType::TemplateCaseFragment);
            }
            Some(ValkyrieKeywords::Else) => {
                parse_template_body(state, is_match_arm_boundary)?;
                state.finish_at(boundary_cp, ValkyrieElementType::TemplateElseFragment);
                if !state.at(ValkyrieTokenType::TemplateL) {
                    return Err(oak_core::OakError::custom_error("expected `<% end %>` after `<% else %>`"));
                }
                let end_cp = state.checkpoint();
                let close = read_and_bump_directive(state)?;
                if directive_first_keyword(&close.tokens) != Some(ValkyrieKeywords::End) {
                    return Err(oak_core::OakError::custom_error("expected `<% end %>` after `<% else %>`"));
                }
                state.finish_at(end_cp, ValkyrieElementType::TemplateEndFragment);
                state.finish_at(match_cp, ValkyrieElementType::TemplateMatch);
                return Ok(());
            }
            Some(ValkyrieKeywords::End) => {
                state.finish_at(boundary_cp, ValkyrieElementType::TemplateEndFragment);
                state.finish_at(match_cp, ValkyrieElementType::TemplateMatch);
                return Ok(());
            }
            _ => return Err(oak_core::OakError::custom_error("expected `<% case %>`, `<% else %>` or `<% end %>`")),
        }
    }

    Err(oak_core::OakError::custom_error("unclosed `<% match %>` block"))
}

/// `<% match %>` 与首个 `<% case %>` 之间无语义；若出现非空 `TemplateText` 仅记 lint。
fn lint_match_prelude_gap<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<(), OakError> {
    loop {
        state.skip_trivia();
        if state.at(ValkyrieTokenType::TemplateL) || state.at(ValkyrieTokenType::BraceR) {
            break;
        }
        if state.at(ValkyrieTokenType::TemplateText) {
            let token = *state.current().expect("template text");
            let text = state.source.get_text_in(token.span);
            let cp = state.checkpoint();
            state.bump();
            state.finish_at(cp, ValkyrieElementType::TemplateText);
            if !text.trim().is_empty() {
                state.errors.push(OakError::semantic_error("unexpected template text between `<% match %>` and `<% case %>`"));
            }
            continue;
        }
        break;
    }
    Ok(())
}

fn parse_template_body<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>, stop: impl Fn(&[Token<ValkyrieTokenType>]) -> bool) -> Result<(), oak_core::OakError> {
    while state.not_at_end() {
        state.skip_trivia();
        if state.at(ValkyrieTokenType::BraceR) {
            break;
        }
        if state.at(ValkyrieTokenType::TemplateL) {
            if stop(&peek_template_directive(state)?.tokens) {
                break;
            }
            parse_template_node(state)?;
            continue;
        }
        if state.at(ValkyrieTokenType::TemplateText) {
            let cp = state.checkpoint();
            state.bump();
            state.finish_at(cp, ValkyrieElementType::TemplateText);
            continue;
        }
        break;
    }
    Ok(())
}

fn read_and_bump_directive<S: oak_core::Source + ?Sized>(state: &mut State<'_, S>) -> Result<ParsedTemplateDirective, oak_core::OakError> {
    let parsed = peek_template_directive(state)?;
    if !state.at(ValkyrieTokenType::TemplateL) {
        return Err(oak_core::OakError::custom_error("expected `<%`"));
    }
    state.bump();
    while state.not_at_end() && !state.at(ValkyrieTokenType::TemplateR) {
        state.bump();
    }
    if !state.at(ValkyrieTokenType::TemplateR) {
        return Err(oak_core::OakError::custom_error("expected `%>`"));
    }
    state.bump();
    Ok(parsed)
}

fn peek_template_directive<S: oak_core::Source + ?Sized>(state: &State<'_, S>) -> Result<ParsedTemplateDirective, oak_core::OakError> {
    if !state.at(ValkyrieTokenType::TemplateL) {
        return Err(oak_core::OakError::custom_error("expected `<%`"));
    }
    Ok(parse_template_directive(&inner_directive_tokens(state)))
}

fn inner_directive_tokens<S: oak_core::Source + ?Sized>(state: &State<'_, S>) -> Vec<Token<ValkyrieTokenType>> {
    let mut offset = 1;
    let mut tokens = Vec::new();
    while let Some(token) = state.peek_at(offset) {
        if token.kind == ValkyrieTokenType::TemplateR {
            break;
        }
        if !token.kind.is_ignored() {
            tokens.push(*token);
        }
        offset += 1;
    }
    tokens
}
