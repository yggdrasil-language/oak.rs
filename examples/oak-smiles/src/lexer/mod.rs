//! Lexer for SMILES and reaction SMILES.

pub mod token_type;

use oak_core::{
    lexer::{LexOutput, Lexer, LexerCache, LexerState},
    source::{Source, TextEdit},
};

use crate::{language::SmilesLanguage, lexer::token_type::SmilesTokenType};

/// Two-letter organic-subset symbols, matched before single letters.
const TWO_LETTER_ATOMS: &[&str] = &["Cl", "Br"];

/// One-letter organic-subset symbols, including aromatic lowercase forms.
const ONE_LETTER_ATOMS: &[char] = &[
    'B', 'C', 'N', 'O', 'P', 'S', 'F', 'I', 'b', 'c', 'n', 'o', 'p', 's', 'H', 'h', '*',
];

/// Lexer for SMILES.
pub struct SmilesLexer;

impl Lexer<SmilesLanguage> for SmilesLexer {
    fn lex<'a, S: Source + ?Sized>(
        &self,
        text: &S,
        _edits: &[TextEdit],
        cache: &'a mut impl LexerCache<SmilesLanguage>,
    ) -> LexOutput<SmilesLanguage> {
        let mut state = LexerState::new_with_cache(text, text.length(), cache);

        while state.not_at_end() {
            let safe_point = state.get_position();
            let start = state.get_position();
            let Some(current) = state.current()
            else {
                break;
            };

            match current {
                ' ' | '\t' | '\r' | '\n' => {
                    let _ = state.bump();
                }
                '[' => {
                    let _ = state.bump();
                    while state.not_at_end() && state.current() != Some(']') {
                        let _ = state.bump();
                    }
                    if state.current() == Some(']') {
                        let _ = state.bump();
                    }
                    state.add_token(SmilesTokenType::BracketAtom, start, state.get_position());
                }
                '(' => {
                    let _ = state.bump();
                    state.add_token(SmilesTokenType::BranchOpen, start, state.get_position());
                }
                ')' => {
                    let _ = state.bump();
                    state.add_token(SmilesTokenType::BranchClose, start, state.get_position());
                }
                '.' => {
                    let _ = state.bump();
                    state.add_token(SmilesTokenType::Dot, start, state.get_position());
                }
                '-' | '=' | '#' | ':' | '/' | '\\' | '~' => {
                    let _ = state.bump();
                    state.add_token(SmilesTokenType::Bond, start, state.get_position());
                }
                '>' => {
                    let first = state.bump();
                    if state.current() == Some('>') {
                        let _ = state.bump();
                    }
                    let _ = first;
                    state.add_token(SmilesTokenType::Arrow, start, state.get_position());
                }
                '%' => {
                    let _ = state.bump();
                    if state.current() == Some('(') {
                        // `%(...)` ring names stay refused; the span covers the
                        // whole label so the error points at the construct.
                        while state.not_at_end() && state.current() != Some(')') {
                            let _ = state.bump();
                        }
                        if state.current() == Some(')') {
                            let _ = state.bump();
                        }
                    }
                    else {
                        let mut digits = 0;
                        while digits < 2 && state.current().is_some_and(|character| character.is_ascii_digit()) {
                            let _ = state.bump();
                            digits += 1;
                        }
                    }
                    state.add_token(SmilesTokenType::RingLabel, start, state.get_position());
                }
                digit if digit.is_ascii_digit() => {
                    let _ = state.bump();
                    state.add_token(SmilesTokenType::RingLabel, start, state.get_position());
                }
                _ => {
                    let rest_start = state.get_position();
                    let mut matched = 0usize;
                    if let Some(symbol) = TWO_LETTER_ATOMS
                        .iter()
                        .find(|symbol| peek_starts_with(text, rest_start, symbol))
                    {
                        matched = symbol.len();
                    }
                    else if ONE_LETTER_ATOMS.contains(&current) {
                        matched = current.len_utf8();
                    }
                    if matched == 0 {
                        // An unrecognised character is folded into an empty
                        // organic atom so the builder can refuse it with a span.
                        let _ = state.bump();
                        state.add_token(SmilesTokenType::OrganicAtom, start, state.get_position());
                    }
                    else {
                        for _ in 0..matched {
                            let _ = state.bump();
                        }
                        state.add_token(SmilesTokenType::OrganicAtom, start, state.get_position());
                    }
                }
            }

            state.advance_if_dead_lock(safe_point);
        }

        state.add_eof();
        state.finish_with_cache(Ok(()), cache)
    }
}

impl Default for SmilesLexer {
    fn default() -> Self {
        Self
    }
}

/// Whether the source at `offset` starts with `symbol`.
fn peek_starts_with<S: Source + ?Sized>(text: &S, offset: usize, symbol: &str) -> bool {
    let end = offset + symbol.len();
    if end > text.length() {
        return false;
    }
    text.get_text_in((offset..end).into()) == symbol
}
