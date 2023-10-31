#![doc = include_str!("readme.md")]
/// Token type module for HTML.
pub mod token_type;

use crate::{language::HtmlLanguage, lexer::token_type::HtmlTokenType};
use oak_core::{
    Lexer, LexerCache, LexerState, OakError,
    lexer::{LexOutput, StringConfig},
    source::{Source, TextEdit},
};
use std::{simd::prelude::*, sync::LazyLock};

pub(crate) type State<'a, S> = LexerState<'a, S, HtmlLanguage>;

// HTML static configuration

static HTML_STRING: LazyLock<StringConfig> = LazyLock::new(|| StringConfig { quotes: &['"', '\''], escape: None });

/// Lexer for the HTML language.
///
/// This lexer converts a raw string into a stream of HTML syntax tokens.
#[derive(Clone, Debug)]
pub struct HtmlLexer<'config> {
    config: &'config HtmlLanguage,
}

impl<'config> Lexer<HtmlLanguage> for HtmlLexer<'config> {
    /// Tokenizes the input source text using the provided cache.
    fn lex<'a, S: Source + ?Sized>(&self, source: &'a S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<HtmlLanguage>) -> LexOutput<HtmlLanguage> {
        let mut state = State::new_with_cache(source, 0, cache);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> HtmlLexer<'config> {
    /// Creates a new `HtmlLexer` with the given configuration.
    pub fn new(config: &'config HtmlLanguage) -> Self {
        Self { config }
    }

    /// The main lexing loop that iterates through the source text.
    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let mut in_markup = false;
        while state.not_at_end() {
            let safe_point = state.get_position();

            if let Some(ch) = state.peek() {
                match ch {
                    ' ' | '\t' | '\n' | '\r' => {
                        self.skip_whitespace(state);
                    }
                    '<' => {
                        if let Some(next) = state.peek_next_n(1) {
                            if next == '!' {
                                if state.starts_with("<!--") {
                                    self.lex_comment(state);
                                }
                                else if state.starts_with("<![CDATA[") {
                                    self.lex_cdata(state);
                                }
                                else if !self.lex_doctype(state) {
                                    self.apply_tag_operator(state, &mut in_markup);
                                }
                            }
                            else if next == '?' {
                                self.lex_processing_instruction(state);
                            }
                            else {
                                self.apply_tag_operator(state, &mut in_markup);
                            }
                        }
                        else {
                            self.apply_tag_operator(state, &mut in_markup);
                        }
                    }
                    '/' | '>' if in_markup => {
                        self.apply_tag_operator(state, &mut in_markup);
                    }
                    '&' => {
                        self.lex_entity_reference(state);
                    }
                    '"' | '\'' if in_markup => {
                        self.lex_string_literal(state);
                    }
                    'a'..='z' | 'A'..='Z' | '_' | ':' if in_markup => {
                        self.lex_identifier(state);
                    }
                    '=' if in_markup => {
                        self.lex_single_char_tokens(state);
                    }
                    _ => {
                        if self.lex_text(state, in_markup) {
                            continue;
                        }

                        // Safety check to prevent infinite loop
                        state.advance(ch.len_utf8());
                        state.add_token(HtmlTokenType::Error, safe_point, state.get_position());
                    }
                }
            }

            state.advance_if_dead_lock(safe_point)
        }

        Ok(())
    }

    fn apply_tag_operator<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, in_markup: &mut bool) {
        if let Some(kind) = self.lex_tag_operators(state) {
            match kind {
                HtmlTokenType::TagOpen | HtmlTokenType::TagSlashOpen => *in_markup = true,
                HtmlTokenType::TagClose | HtmlTokenType::TagSelfClose => *in_markup = false,
                _ => {}
            }
        }
    }

    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let bytes = state.rest_bytes();
        let mut i = 0;
        let len = bytes.len();
        const LANES: usize = 32;

        while i + LANES <= len {
            let chunk = Simd::<u8, LANES>::from_slice(unsafe { bytes.get_unchecked(i..i + LANES) });
            let is_le_space = chunk.simd_le(Simd::splat(32));

            if !is_le_space.all() {
                let not_space = !is_le_space;
                let idx = not_space.first_set().unwrap();
                i += idx;
                state.advance(i);
                state.add_token(HtmlTokenType::Whitespace, start, state.get_position());
                return true;
            }
            i += LANES;
        }

        while i < len {
            if !unsafe { *bytes.get_unchecked(i) }.is_ascii_whitespace() {
                break;
            }
            i += 1;
        }

        if i > 0 {
            state.advance(i);
            state.add_token(HtmlTokenType::Whitespace, start, state.get_position());
            true
        }
        else {
            false
        }
    }

    fn lex_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        if !state.starts_with("<!--") {
            return false;
        }

        let start = state.get_position();
        let len = {
            let rest = state.rest();
            match rest.find("-->") {
                Some(end_at) => end_at + "-->".len(),
                None => rest.len(),
            }
        };
        state.advance(len);
        state.add_token(HtmlTokenType::Comment, start, state.get_position());
        true
    }

    fn lex_doctype<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('<') = state.peek() {
            if let Some('!') = state.peek_next_n(1) {
                if let Some('D') = state.peek_next_n(2) {
                    let doctype_start = "DOCTYPE";
                    let mut matches = true;

                    for (i, expected_ch) in doctype_start.chars().enumerate() {
                        if let Some(actual_ch) = state.peek_next_n(2 + i) {
                            if actual_ch.to_ascii_uppercase() != expected_ch {
                                matches = false;
                                break;
                            }
                        }
                        else {
                            matches = false;
                            break;
                        }
                    }

                    if matches {
                        state.advance(2 + doctype_start.len()); // Skip <!DOCTYPE

                        // Find doctype end >
                        while state.not_at_end() {
                            if let Some('>') = state.peek() {
                                state.advance(1); // Skip >
                                state.add_token(HtmlTokenType::Doctype, start_pos, state.get_position());
                                return true;
                            }
                            if let Some(ch) = state.peek() {
                                state.advance(ch.len_utf8());
                            }
                            else {
                                break;
                            }
                        }

                        // Unclosed doctype
                        state.add_token(HtmlTokenType::Error, start_pos, state.get_position());
                        return true;
                    }
                }
            }
        }

        false
    }

    fn lex_cdata<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('<') = state.peek() {
            if let Some('!') = state.peek_next_n(1) {
                if let Some('[') = state.peek_next_n(2) {
                    let cdata_start = "CDATA[";
                    let mut matches = true;

                    for (i, expected_ch) in cdata_start.chars().enumerate() {
                        if let Some(actual_ch) = state.peek_next_n(3 + i) {
                            if actual_ch != expected_ch {
                                matches = false;
                                break;
                            }
                        }
                        else {
                            matches = false;
                            break;
                        }
                    }

                    if matches {
                        state.advance(3 + cdata_start.len()); // Skip <![CDATA[

                        // Find CDATA end ]]>
                        while state.not_at_end() {
                            if let Some(']') = state.peek() {
                                if let Some(']') = state.peek_next_n(1) {
                                    if let Some('>') = state.peek_next_n(2) {
                                        state.advance(3); // Skip ]]>
                                        state.add_token(HtmlTokenType::CData, start_pos, state.get_position());
                                        return true;
                                    }
                                }
                            }
                            if let Some(ch) = state.peek() {
                                state.advance(ch.len_utf8());
                            }
                            else {
                                break;
                            }
                        }

                        // Unclosed CDATA
                        state.add_token(HtmlTokenType::Error, start_pos, state.get_position());
                        return true;
                    }
                }
            }
        }

        false
    }

    fn lex_processing_instruction<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('<') = state.peek() {
            if let Some('?') = state.peek_next_n(1) {
                state.advance(2); // Skip <?

                // Find processing instruction end ?>
                while state.not_at_end() {
                    if let Some('?') = state.peek() {
                        if let Some('>') = state.peek_next_n(1) {
                            state.advance(2); // Skip ?>
                            state.add_token(HtmlTokenType::ProcessingInstruction, start_pos, state.get_position());
                            return true;
                        }
                    }
                    if let Some(ch) = state.peek() {
                        state.advance(ch.len_utf8());
                    }
                    else {
                        break;
                    }
                }

                // Unclosed processing instruction
                state.add_token(HtmlTokenType::Error, start_pos, state.get_position());
                return true;
            }
        }

        false
    }

    fn lex_tag_operators<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Option<HtmlTokenType> {
        let start_pos = state.get_position();

        let kind = match state.peek() {
            Some('<') => {
                if let Some('/') = state.peek_next_n(1) {
                    state.advance(2);
                    HtmlTokenType::TagSlashOpen
                }
                else {
                    state.advance(1);
                    HtmlTokenType::TagOpen
                }
            }
            Some('/') => {
                if let Some('>') = state.peek_next_n(1) {
                    state.advance(2);
                    HtmlTokenType::TagSelfClose
                }
                else {
                    return None;
                }
            }
            Some('>') => {
                state.advance(1);
                HtmlTokenType::TagClose
            }
            _ => return None,
        };
        state.add_token(kind, start_pos, state.get_position());
        Some(kind)
    }

    fn lex_entity_reference<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('&') = state.peek() {
            state.advance(1);

            if let Some('#') = state.peek() {
                state.advance(1);

                // Character reference &#123; or &#x1A;
                if let Some('x') = state.peek() {
                    state.advance(1);
                    // Hexadecimal character reference
                    let mut has_digits = false;
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_hexdigit() {
                            state.advance(1);
                            has_digits = true;
                        }
                        else {
                            break;
                        }
                    }

                    if has_digits && state.peek() == Some(';') {
                        state.advance(1);
                        state.add_token(HtmlTokenType::CharRef, start_pos, state.get_position());
                        return true;
                    }
                }
                else {
                    // Decimal character reference
                    let mut has_digits = false;
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_digit() {
                            state.advance(1);
                            has_digits = true;
                        }
                        else {
                            break;
                        }
                    }

                    if has_digits && state.peek() == Some(';') {
                        state.advance(1);
                        state.add_token(HtmlTokenType::CharRef, start_pos, state.get_position());
                        return true;
                    }
                }
            }
            else {
                // Named entity reference &name;
                let mut has_name = false;
                while let Some(ch) = state.peek() {
                    if ch.is_ascii_alphanumeric() {
                        state.advance(1);
                        has_name = true;
                    }
                    else {
                        break;
                    }
                }

                if has_name && state.peek() == Some(';') {
                    state.advance(1);
                    state.add_token(HtmlTokenType::EntityRef, start_pos, state.get_position());
                    return true;
                }
            }

            // Invalid entity reference
            state.add_token(HtmlTokenType::Error, start_pos, state.get_position());
            return true;
        }

        false
    }

    fn lex_string_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        HTML_STRING.scan(state, HtmlTokenType::AttributeValue)
    }

    fn lex_identifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            if ch.is_ascii_alphabetic() || ch == '_' || ch == ':' {
                state.advance(ch.len_utf8());

                while let Some(ch) = state.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == ':' {
                        state.advance(ch.len_utf8());
                    }
                    else {
                        break;
                    }
                }

                state.add_token(HtmlTokenType::TagName, start_pos, state.get_position());
                return true;
            }
        }

        false
    }

    fn lex_single_char_tokens<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        let kind = match state.peek() {
            Some('=') => HtmlTokenType::Equal,
            Some('"') => HtmlTokenType::Quote,
            Some('\'') => HtmlTokenType::Quote,
            Some('!') => return false, // Already handled elsewhere
            Some('?') => return false, // Already handled elsewhere
            Some('&') => return false, // Already handled elsewhere
            Some(';') => return false, // Already handled elsewhere
            _ => return false,
        };

        if let Some(ch) = state.peek() {
            state.advance(ch.len_utf8());
            state.add_token(kind, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    fn lex_text<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, in_markup: bool) -> bool {
        let start_pos = state.get_position();
        let bytes = state.rest_bytes();
        let mut i = 0;
        let len = bytes.len();
        const LANES: usize = 32;

        while i + LANES <= len {
            let chunk = Simd::<u8, LANES>::from_slice(unsafe { bytes.get_unchecked(i..i + LANES) });

            let is_lt = chunk.simd_eq(Simd::splat(b'<'));
            let is_amp = chunk.simd_eq(Simd::splat(b'&'));
            let stop = if in_markup {
                let is_le_space = chunk.simd_le(Simd::splat(32));
                is_lt | is_amp | is_le_space
            }
            else {
                is_lt | is_amp
            };

            if stop.any() {
                let idx = stop.first_set().unwrap();
                i += idx;
                state.advance(i);
                state.add_token(HtmlTokenType::Text, start_pos, state.get_position());
                return true;
            }
            i += LANES
        }
        while i < len {
            let ch = unsafe { *bytes.get_unchecked(i) };
            if ch == b'<' || ch == b'&' || (in_markup && ch <= 32) {
                break;
            }
            i += 1
        }

        if i > 0 {
            state.advance(i);
            state.add_token(HtmlTokenType::Text, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }
}
