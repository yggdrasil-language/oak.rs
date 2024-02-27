//! 从 `<%` 与 `%>` 之间的 Valkyrie token 流识别 TGrammar 指令。

use crate::lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType};
use oak_core::{Token, TokenType};

/// `<% ... %>` 指令内部 token 流（不含 `TemplateL` / `TemplateR`）。
pub(crate) struct ParsedTemplateDirective {
    pub tokens: Vec<Token<ValkyrieTokenType>>,
}

/// 解析完整指令 token 流。
pub(crate) fn parse_template_directive(inner_tokens: &[Token<ValkyrieTokenType>]) -> ParsedTemplateDirective {
    ParsedTemplateDirective { tokens: inner_tokens.iter().filter(|token| !token.kind.is_ignored()).copied().collect() }
}

/// 指令首 token 若为关键词则返回。
pub(crate) fn directive_first_keyword(tokens: &[Token<ValkyrieTokenType>]) -> Option<ValkyrieKeywords> {
    match tokens.first().map(|token| token.kind) {
        Some(ValkyrieTokenType::Keyword(keyword)) => Some(keyword),
        _ => None,
    }
}

/// `<% else if %>`。
pub(crate) fn is_else_if_directive(tokens: &[Token<ValkyrieTokenType>]) -> bool {
    tokens.len() >= 2 && tokens[0].kind == ValkyrieTokenType::Keyword(ValkyrieKeywords::Else) && tokens[1].kind == ValkyrieTokenType::Keyword(ValkyrieKeywords::If)
}

/// 关键词之后的 payload token（仍为词法 token 流，不是源码子串）。
pub(crate) fn directive_payload_tokens(directive: &ParsedTemplateDirective) -> Vec<Token<ValkyrieTokenType>> {
    let skip = if is_else_if_directive(&directive.tokens) {
        2
    }
    else {
        match directive_first_keyword(&directive.tokens) {
            Some(ValkyrieKeywords::If) | Some(ValkyrieKeywords::Loop) | Some(ValkyrieKeywords::Match) | Some(ValkyrieKeywords::Case) | Some(ValkyrieKeywords::End) => 1,
            Some(ValkyrieKeywords::Else) => directive.tokens.len(),
            _ => 0,
        }
    };
    directive.tokens.iter().skip(skip).copied().collect()
}
