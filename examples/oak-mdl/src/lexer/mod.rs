//! Line-oriented lexer for MDL connection tables.

pub mod token_type;

use oak_core::{
    lexer::{LexOutput, Lexer, LexerCache, LexerState},
    source::{Source, TextEdit},
};

use crate::{language::MdlLanguage, lexer::token_type::MdlTokenType};

/// Lexer for MDL molfiles and SDF members.
pub struct MdlLexer;

impl Lexer<MdlLanguage> for MdlLexer {
    fn lex<'a, S: Source + ?Sized>(
        &self,
        text: &S,
        _edits: &[TextEdit],
        cache: &'a mut impl LexerCache<MdlLanguage>,
    ) -> LexOutput<MdlLanguage> {
        let mut state = LexerState::new_with_cache(text, text.length(), cache);

        while state.not_at_end() {
            let safe_point = state.get_position();
            let start = state.get_position();
            let mut content = String::new();
            while state.not_at_end() {
                match state.current() {
                    Some('\n') => {
                        let _ = state.bump();
                        break;
                    }
                    Some(character) => {
                        content.push(character);
                        let _ = state.bump();
                    }
                    None => break,
                }
            }
            let end = state.get_position();
            let kind = classify(&content);
            state.add_token(kind, start, end);
            state.advance_if_dead_lock(safe_point);
        }

        state.add_eof();
        state.finish_with_cache(Ok(()), cache)
    }
}

impl Default for MdlLexer {
    fn default() -> Self {
        Self
    }
}

/// Classify one line by its leading bytes.
///
/// Blank lines stay [`MdlTokenType::Line`]: the framework drops whitespace-role
/// tokens before the parser sees them, and a blank line still occupies a
/// position in the 3-line MDL header.
fn classify(line: &str) -> MdlTokenType {
    let trimmed = line.trim_end_matches('\r');
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        return MdlTokenType::Line;
    }
    if trimmed.starts_with("$$$$") {
        return MdlTokenType::RecordSeparator;
    }
    if trimmed.starts_with("$RXN") || trimmed.starts_with("$RDFILE") || trimmed.starts_with("$MOL") {
        return MdlTokenType::ReactionMarker;
    }
    if trimmed.starts_with('>') {
        return MdlTokenType::DataHeader;
    }
    // `M  CHG`, `M  ISO`, `M  RAD`, and the `A  ` alias block are all property
    // lines: they start with a capital letter followed by padding.
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'M' && (bytes[1] == b' ' || bytes[1] == b'\t') {
        return MdlTokenType::Property;
    }
    if bytes.len() >= 2 && bytes[0] == b'A' && (bytes[1] == b' ' || bytes[1] == b'\t') {
        return MdlTokenType::Property;
    }
    MdlTokenType::Line
}
