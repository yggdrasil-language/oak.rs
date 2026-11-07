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

/// Elements whose content is raw text rather than markup.
const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style", "textarea", "title"];

/// Reads the tag name at the current position when it names a raw text element.
fn raw_text_element_name<'a, S: Source + ?Sized>(state: &mut State<'a, S>) -> Option<String> {
    let rest = state.rest();
    let end = rest.find(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':'))).unwrap_or(rest.len());
    let name = rest[..end].to_ascii_lowercase();
    RAW_TEXT_ELEMENTS.contains(&name.as_str()).then_some(name)
}

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
        // HTML allows an attribute value without quotes, and that value may
        // start with any character except whitespace (`/x`, `2`). Without this
        // state the `/` in `<a href=/x>` lexes as a self-closing slash and the
        // remainder of the tag turns into text.
        let mut expecting_attribute_value = false;
        // `script`, `style`, `textarea` and `title` hold raw text: a `<` inside
        // them is data, not the start of a tag. The flag records the element of
        // the open start tag so the raw text state begins after its `>`.
        let mut raw_text_tag: Option<String> = None;
        let mut tag_name_seen = false;
        let mut closing_tag = false;
        while state.not_at_end() {
            let safe_point = state.get_position();
            let mut handled = false;

            if let Some(ch) = state.peek() {
                if expecting_attribute_value && !matches!(ch, ' ' | '\t' | '\n' | '\r') {
                    handled = match ch {
                        '"' | '\'' => {
                            self.lex_string_literal(state);
                            true
                        }
                        '>' => false,
                        '/' if state.peek_next_n(1) == Some('>') => false,
                        _ => self.lex_unquoted_attribute_value(state),
                    };
                }

                if handled {
                    expecting_attribute_value = false;
                }
                else {
                    if ch == '<' {
                        expecting_attribute_value = false;
                    }
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
                                        self.apply_tag_operator(state, &mut in_markup, &mut raw_text_tag, &mut tag_name_seen, &mut closing_tag);
                                    }
                                }
                                else if next == '?' {
                                    self.lex_processing_instruction(state);
                                }
                                else {
                                    self.apply_tag_operator(state, &mut in_markup, &mut raw_text_tag, &mut tag_name_seen, &mut closing_tag);
                                }
                            }
                            else {
                                self.apply_tag_operator(state, &mut in_markup, &mut raw_text_tag, &mut tag_name_seen, &mut closing_tag);
                            }
                        }
                        '/' | '>' if in_markup => {
                            self.apply_tag_operator(state, &mut in_markup, &mut raw_text_tag, &mut tag_name_seen, &mut closing_tag);
                        }
                        '&' => {
                            self.lex_entity_reference(state);
                        }
                        '"' | '\'' if in_markup => {
                            self.lex_string_literal(state);
                        }
                        'a'..='z' | 'A'..='Z' | '_' | ':' if in_markup => {
                            if !tag_name_seen && !closing_tag {
                                tag_name_seen = true;
                                raw_text_tag = raw_text_element_name(state);
                            }
                            self.lex_identifier(state);
                        }
                        '=' if in_markup => {
                            self.lex_single_char_tokens(state);
                            expecting_attribute_value = true;
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

                    if !in_markup {
                        expecting_attribute_value = false;
                    }
                }
            }

            state.advance_if_dead_lock(safe_point)
        }

        Ok(())
    }

    fn apply_tag_operator<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, in_markup: &mut bool, raw_text_tag: &mut Option<String>, tag_name_seen: &mut bool, closing_tag: &mut bool) {
        let Some(kind) = self.lex_tag_operators(state)
        else {
            return;
        };
        match kind {
            HtmlTokenType::TagOpen | HtmlTokenType::TagSlashOpen => {
                *in_markup = true;
                *closing_tag = matches!(kind, HtmlTokenType::TagSlashOpen);
                *tag_name_seen = false;
                *raw_text_tag = None;
            }
            HtmlTokenType::TagSelfClose => {
                *in_markup = false;
                *closing_tag = false;
                *tag_name_seen = false;
                *raw_text_tag = None;
            }
            HtmlTokenType::TagClose => {
                *in_markup = false;
                let raw = if *closing_tag { None } else { raw_text_tag.take() };
                *closing_tag = false;
                *tag_name_seen = false;
                if let Some(name) = raw {
                    self.lex_raw_text(state, &name);
                }
            }
            _ => {}
        }
    }

    /// Consumes the raw text of `name` up to its closing tag as a single text
    /// token, so markup-looking content stays data.
    fn lex_raw_text<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, name: &str) {
        let start = state.get_position();
        let rest = state.rest();
        let lower = rest.to_ascii_lowercase();
        let needle = format!("</{name}");
        let len = lower.find(&needle).unwrap_or(rest.len());
        if len == 0 {
            return;
        }
        state.advance(len);
        state.add_token(HtmlTokenType::Text, start, state.get_position());
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

    /// Lex an attribute value the author wrote without quotes.
    ///
    /// The value runs until whitespace, the tag close bracket, or the next tag
    /// open, so characters such as `/` and digits stay part of the value
    /// instead of turning into tag operators.
    fn lex_unquoted_attribute_value<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();
        let mut advanced = false;
        while let Some(ch) = state.peek() {
            if matches!(ch, ' ' | '\t' | '\n' | '\r' | '>' | '<') {
                break;
            }
            advanced = true;
            state.advance(ch.len_utf8());
        }

        if !advanced {
            return false;
        }
        state.add_token(HtmlTokenType::AttributeValue, start_pos, state.get_position());
        true
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
