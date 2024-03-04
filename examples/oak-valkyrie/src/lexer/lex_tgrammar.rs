//! TGrammar lexing: `support_t_grammar` controls whether `<% ... %>` enters the token stream.
//!
//! - `<%` / `%>` → `TemplateL` / `TemplateR`
//! - Between `<%` and `%>` → normal Valkyrie tokens (keywords, identifiers, etc.)
//! - Between `%>` and the next `<%` in template regions → always `TemplateText` (includes `aaa` and `micro fx() { ... }`, etc.)

use crate::lexer::{State, token_type::ValkyrieTokenType};
use oak_core::{TokenType, source::Source};

const TG_CONTROL_L: &str = "<%";
const TG_CONTROL_R: &str = "%>";

/// Entered the interleaved template region (seen the first `<% ... %>`).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TemplateLexMode {
    pub region: bool,
}

impl crate::lexer::ValkyrieLexer<'_> {
    /// Try to consume a TGrammar fragment; returns `true` if the current position was handled.
    pub(crate) fn lex_tgrammar_fragment<S: Source + ?Sized>(&self, state: &mut State<'_, S>, mode: &mut TemplateLexMode) -> bool {
        if state.starts_with(TG_CONTROL_L) {
            if !self.config.support_t_grammar {
                lex_tgrammar_disabled_error(state);
                return true;
            }
            lex_tgrammar_control(self, state, mode);
            return true;
        }

        if !mode.region {
            return false;
        }

        lex_tgrammar_gap_text(state);
        true
    }
}

fn lex_tgrammar_disabled_error<S: Source + ?Sized>(state: &mut State<'_, S>) {
    let start = state.get_position();
    state.advance(TG_CONTROL_L.len());
    while state.not_at_end() {
        if state.starts_with(TG_CONTROL_R) {
            state.advance(TG_CONTROL_R.len());
            break;
        }
        let step = state.current().map(|ch| ch.len_utf8()).unwrap_or(1);
        state.advance(step);
    }
    state.add_token(ValkyrieTokenType::Error, start, state.get_position());
}

fn lex_tgrammar_control<S: Source + ?Sized>(lexer: &crate::lexer::ValkyrieLexer<'_>, state: &mut State<'_, S>, mode: &mut TemplateLexMode) {
    let block_start = state.get_position();
    state.advance(TG_CONTROL_L.len());
    state.add_token(ValkyrieTokenType::TemplateL, block_start, state.get_position());

    let content_start = state.get_position();
    let content_end = scan_control_end(state);
    let mut closes_template_region = false;
    if content_start < content_end {
        let mut sub_state = state.sub_state(content_start, content_end);
        let _ = lexer.run_template_directive_programming(&mut sub_state);
        let directive_tokens = sub_state.get_tokens();
        closes_template_region = directive_tokens.iter().find(|token| !token.kind.is_ignored()).is_some_and(|token| token.kind == ValkyrieTokenType::Keyword(crate::lexer::ValkyrieKeywords::End));
        for token in directive_tokens {
            state.add_token(token.kind, token.span.start, token.span.end);
        }
        state.set_position(content_end);
    }

    if state.starts_with(TG_CONTROL_R) {
        let end_start = state.get_position();
        state.advance(TG_CONTROL_R.len());
        state.add_token(ValkyrieTokenType::TemplateR, end_start, state.get_position());
        // `<% end %>` closes the interleaved template region.  The following
        // source must be lexed as ordinary Valkyrie again; otherwise the whole
        // remainder of the file becomes TemplateText and cannot form items.
        mode.region = !closes_template_region;
    }
    else {
        state.add_token(ValkyrieTokenType::Error, block_start, state.get_position());
    }
}

fn scan_control_end<S: Source + ?Sized>(state: &mut State<'_, S>) -> usize {
    let end = state.get_length();
    while state.get_position() < end {
        if state.starts_with(TG_CONTROL_R) {
            return state.get_position();
        }
        if let Some(ch) = state.current() {
            state.advance(ch.len_utf8());
        }
        else {
            break;
        }
    }
    state.get_position()
}

fn lex_tgrammar_gap_text<S: Source + ?Sized>(state: &mut State<'_, S>) {
    let start = state.get_position();
    let end = state.get_length();
    while state.get_position() < end {
        if state.starts_with(TG_CONTROL_L) {
            break;
        }
        if let Some(ch) = state.current() {
            state.advance(ch.len_utf8());
        }
        else {
            break;
        }
    }
    if start < state.get_position() {
        state.add_token(ValkyrieTokenType::TemplateText, start, state.get_position());
    }
}
