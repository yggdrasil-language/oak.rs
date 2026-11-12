//! Line-oriented lexer for ABC notation.

pub mod token_type;

use oak_core::{
    lexer::{LexOutput, Lexer, LexerCache, LexerState},
    source::{Source, TextEdit},
};

use crate::{language::AbcLanguage, lexer::token_type::AbcTokenType};

/// Lexer for ABC notation.
pub struct AbcLexer;

impl Lexer<AbcLanguage> for AbcLexer {
    fn lex<'a, S: Source + ?Sized>(
        &self,
        text: &S,
        _edits: &[TextEdit],
        cache: &'a mut impl LexerCache<AbcLanguage>,
    ) -> LexOutput<AbcLanguage> {
        let mut state = LexerState::new_with_cache(text, text.length(), cache);

        while state.not_at_end() {
            let safe_point = state.get_position();
            let start = state.get_position();
            while state.not_at_end() {
                match state.current() {
                    Some('\n') => {
                        let _ = state.bump();
                        break;
                    }
                    Some(_) => {
                        let _ = state.bump();
                    }
                    None => break,
                }
            }
            state.add_token(AbcTokenType::Line, start, state.get_position());
            state.advance_if_dead_lock(safe_point);
        }

        state.add_eof();
        state.finish_with_cache(Ok(()), cache)
    }
}

impl Default for AbcLexer {
    fn default() -> Self {
        Self
    }
}
