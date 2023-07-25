/// Token type definitions.
pub mod token_type;

use oak_core::{
    lexer::{LexOutput, Lexer, LexerCache, LexerState, Token},
    source::{Source, TextEdit},
};

use crate::{language::MetisLanguage, lexer::token_type::MetisTokenType};

/// Lexer for Metis island language.
pub struct MetisLexer;

impl Default for MetisLexer {
    fn default() -> Self {
        Self
    }
}

impl Lexer<MetisLanguage> for MetisLexer {
    fn lex<'a, S: Source + ?Sized>(&self, text: &S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<MetisLanguage>) -> LexOutput<MetisLanguage> {
        let mut state = LexerState::new_with_cache(text, text.length(), cache);

        while state.not_at_end() {
            let safe_point = state.get_position();
            let start = state.get_position();

            match state.current() {
                Some(' ') | Some('\t') | Some('\r') | Some('\n') => {
                    while matches!(state.current(), Some(' ' | '\t' | '\r' | '\n')) {
                        let _ = state.bump();
                    }
                    state.add_token(MetisTokenType::Whitespace, start, state.get_position());
                }
                Some('/') if state.peek_next() == Some('/') => {
                    while state.not_at_end() && state.current() != Some('\n') {
                        let _ = state.bump();
                    }
                    state.add_token(MetisTokenType::Comment, start, state.get_position());
                }
                Some('"') => {
                    let _ = state.bump();
                    while state.not_at_end() {
                        match state.current() {
                            Some('"') => {
                                let _ = state.bump();
                                break;
                            }
                            Some('\\') => {
                                let _ = state.bump();
                                let _ = state.bump();
                            }
                            Some(_) => {
                                let _ = state.bump();
                            }
                            None => break,
                        }
                    }
                    state.add_token(MetisTokenType::String, start, state.get_position());
                }
                Some('{') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::LBrace, start, state.get_position());
                }
                Some('}') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::RBrace, start, state.get_position());
                }
                Some('(') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::LParen, start, state.get_position());
                }
                Some(')') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::RParen, start, state.get_position());
                }
                Some('[') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::LBracket, start, state.get_position());
                }
                Some(']') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::RBracket, start, state.get_position());
                }
                Some(',') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::Comma, start, state.get_position());
                }
                Some(';') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::Semi, start, state.get_position());
                }
                Some('.') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::Dot, start, state.get_position());
                }
                Some('|') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::Pipe, start, state.get_position());
                }
                Some(':') if state.peek_next() == Some(':') => {
                    let _ = state.bump();
                    let _ = state.bump();
                    state.add_token(MetisTokenType::PathSep, start, state.get_position());
                }
                Some(':') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::Colon, start, state.get_position());
                }
                Some('<') if state.peek_next() == Some('-') => {
                    // could be <- or <->
                    let _ = state.bump(); // <
                    let _ = state.bump(); // -
                    if state.current() == Some('>') {
                        let _ = state.bump();
                        state.add_token(MetisTokenType::Iff, start, state.get_position());
                    }
                    else {
                        state.add_token(MetisTokenType::Error, start, state.get_position());
                    }
                }
                Some('<') if state.peek_next() == Some('=') => {
                    let _ = state.bump();
                    let _ = state.bump();
                    state.add_token(MetisTokenType::OpLe, start, state.get_position());
                }
                Some('-') if state.peek_next() == Some('>') => {
                    let _ = state.bump();
                    let _ = state.bump();
                    state.add_token(MetisTokenType::Arrow, start, state.get_position());
                }
                Some('=') if state.peek_next() == Some('=') => {
                    let _ = state.bump();
                    let _ = state.bump();
                    state.add_token(MetisTokenType::EqEq, start, state.get_position());
                }
                Some('=') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::Eq, start, state.get_position());
                }
                Some('·') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::OpMul, start, state.get_position());
                }
                Some('+') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::OpPlus, start, state.get_position());
                }
                Some('⊆') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::OpSubseteq, start, state.get_position());
                }
                Some('⊇') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::OpSupseteq, start, state.get_position());
                }
                Some('≅') => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::OpIso, start, state.get_position());
                }
                Some('⁻') => {
                    let _ = state.bump();
                    if state.current() == Some('¹') {
                        let _ = state.bump();
                    }
                    state.add_token(MetisTokenType::OpInv, start, state.get_position());
                }
                Some(c) if is_ident_start(c) => {
                    let _ = state.bump();
                    while matches!(state.current(), Some(ch) if is_ident_continue(ch)) {
                        let _ = state.bump();
                    }
                    let end = state.get_position();
                    let text = state.get_text_in((start..end).into());
                    let kind = keyword_or_ident(text.as_ref());
                    state.add_token(kind, start, end);
                }
                Some(_) => {
                    let _ = state.bump();
                    state.add_token(MetisTokenType::Error, start, state.get_position());
                }
                None => break,
            }

            state.advance_if_dead_lock(safe_point);
        }

        state.add_eof();
        state.finish_with_cache(Ok(()), cache)
    }
}

fn keyword_or_ident(text: &str) -> MetisTokenType {
    match text {
        "island" => MetisTokenType::KwIsland,
        "namespace" => MetisTokenType::KwNamespace,
        "use" => MetisTokenType::KwUse,
        "using" => MetisTokenType::KwUsing,
        "node" => MetisTokenType::KwNode,
        "relation" => MetisTokenType::KwRelation,
        "axiom" => MetisTokenType::KwAxiom,
        "theorem" => MetisTokenType::KwTheorem,
        "action" => MetisTokenType::KwAction,
        "rewrites" => MetisTokenType::KwRewrites,
        "connection" => MetisTokenType::KwConnection,
        "forall" => MetisTokenType::KwForall,
        "exists" => MetisTokenType::KwExists,
        "and" => MetisTokenType::KwAnd,
        "or" => MetisTokenType::KwOr,
        "not" => MetisTokenType::KwNot,
        "let" => MetisTokenType::KwLet,
        "if" => MetisTokenType::KwIf,
        "in" => MetisTokenType::KwIn,
        _ => MetisTokenType::Ident,
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Lex into concrete tokens with source text.
pub fn lex_tokens(source: &str) -> Result<Vec<(MetisTokenType, String)>, String> {
    use oak_core::{lexer::Lexer, parser::session::ParseSession, source::SourceText};

    let lexer = MetisLexer;
    let text = SourceText::new(source.to_string());
    let mut cache = ParseSession::<MetisLanguage>::default();
    let out = lexer.lex(&text, &[], &mut cache);
    let tokens = out.result.map_err(|e| format!("{e:?}"))?;
    Ok(tokens
        .iter()
        .map(|Token { kind, span }| {
            let slice = source.get(span.start..span.end).unwrap_or("").to_string();
            (*kind, slice)
        })
        .collect())
}
