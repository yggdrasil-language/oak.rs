#![doc = include_str!("readme.md")]

/// XML element types.
pub mod element_type;
pub use element_type::XmlElementType;

use crate::{
    language::XmlLanguage,
    lexer::{XmlLexer, token_type::XmlTokenType},
};
use oak_core::{
    OakError, TextEdit,
    parser::{ParseCache, ParseOutput, Parser, ParserState, parse_with_lexer},
    source::Source,
};

pub(crate) type State<'a, S> = ParserState<'a, XmlLanguage, S>;

/// XML parser.
pub struct XmlParser<'config> {
    pub(crate) config: &'config XmlLanguage,
}

impl<'config> Parser<XmlLanguage> for XmlParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<XmlLanguage>) -> ParseOutput<'a, XmlLanguage> {
        let lexer = XmlLexer::new(&self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();

            self.skip_trivia(state);
            if state.at(XmlTokenType::LeftAngle) && state.peek_kind_at(1) == Some(XmlTokenType::Question) {
                self.parse_prolog(state)?;
            }

            while state.not_at_end() {
                self.skip_trivia(state);
                if state.at(XmlTokenType::LeftAngle) {
                    self.parse_element(state)?;
                }
                else if state.not_at_end() {
                    state.advance();
                }
            }

            Ok(state.finish_at(checkpoint, element_type::XmlElementType::Root))
        })
    }
}

impl<'config> XmlParser<'config> {
    /// Creates a new `XmlParser`.
    pub fn new(config: &'config XmlLanguage) -> Self {
        Self { config }
    }

    /// Consumes the current token into the CST without running post-token trivia skip.
    ///
    /// Used at the `>` boundary so element text whitespace stays in content, not inside
    /// the start tag node.
    fn bump_without_skipping_trivia<'a, S: Source + ?Sized>(state: &mut State<'a, S>) {
        if let Some(token) = state.current() {
            state.sink.push_leaf(token.kind, token.length());
            state.tokens.advance();
        }
    }

    fn close_start_tag<'a, S: Source + ?Sized>(
        state: &mut State<'a, S>,
        start_tag_checkpoint: (usize, usize),
    ) -> Result<(), OakError> {
        if state.at(XmlTokenType::RightAngle) {
            Self::bump_without_skipping_trivia(state);
        } else {
            state.expect(XmlTokenType::RightAngle)?;
        }
        state.finish_at(start_tag_checkpoint, element_type::XmlElementType::StartTag);
        Ok(())
    }

    pub(crate) fn parse_prolog<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        state.expect(XmlTokenType::LeftAngle)?;
        state.expect(XmlTokenType::Question)?;
        state.expect(XmlTokenType::Identifier)?; // xml

        while state.not_at_end() && !state.at(XmlTokenType::Question) {
            self.skip_trivia(state);
            if state.at(XmlTokenType::Identifier) {
                self.parse_attribute(state)?
            }
            else {
                break;
            }
        }

        self.skip_trivia(state);
        state.expect(XmlTokenType::Question)?;
        state.expect(XmlTokenType::RightAngle)?;
        state.finish_at(checkpoint, element_type::XmlElementType::Prolog);
        Ok(())
    }

    pub(crate) fn parse_element<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();

        // Start Tag
        let start_tag_checkpoint = state.checkpoint();
        state.expect(XmlTokenType::LeftAngle)?;
        self.skip_trivia(state);
        state.expect(XmlTokenType::Identifier)?;

        while state.not_at_end() && !state.at(XmlTokenType::RightAngle) && !state.at(XmlTokenType::SlashRightAngle) {
            self.skip_trivia(state);
            if state.at(XmlTokenType::Identifier) {
                self.parse_attribute(state)?;
            }
            else {
                break;
            }
        }

        self.skip_trivia(state);
        let is_self_closing = state.at(XmlTokenType::SlashRightAngle);
        if is_self_closing {
            state.expect(XmlTokenType::SlashRightAngle)?;
            state.finish_at(start_tag_checkpoint, element_type::XmlElementType::SelfClosingTag);
            state.finish_at(checkpoint, XmlElementType::Element);
            return Ok(());
        }

        Self::close_start_tag(state, start_tag_checkpoint)?;

        // Content — `bump()` attaches meaningful text plus following trivia into the CST.
        while state.not_at_end() {
            if state.at(XmlTokenType::LeftAngleSlash) {
                break;
            }
            if state.at(XmlTokenType::LeftAngle) {
                self.skip_comments_and_pi(state);
                if state.at(XmlTokenType::LeftAngleSlash) {
                    break;
                }
                if state.at(XmlTokenType::LeftAngle) {
                    self.parse_element(state)?;
                }
                continue;
            }
            if state.not_at_end() {
                state.bump();
            }
        }

        // End Tag
        let end_tag_checkpoint = state.checkpoint();
        state.expect(XmlTokenType::LeftAngleSlash)?;
        self.skip_trivia(state);
        state.expect(XmlTokenType::Identifier)?;
        self.skip_trivia(state);
        state.expect(XmlTokenType::RightAngle)?;
        state.finish_at(end_tag_checkpoint, element_type::XmlElementType::EndTag);

        state.finish_at(checkpoint, element_type::XmlElementType::Element);
        Ok(())
    }

    fn parse_attribute<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        state.expect(XmlTokenType::Identifier)?;
        self.skip_trivia(state);
        state.expect(XmlTokenType::Equals)?;
        self.skip_trivia(state);
        state.expect(XmlTokenType::StringLiteral)?;
        state.finish_at(checkpoint, element_type::XmlElementType::Attribute);
        Ok(())
    }

    fn skip_trivia<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use oak_core::TokenType;
        while let Some(token) = state.current() {
            if token.kind.is_ignored() {
                state.bump();
            }
            else {
                break;
            }
        }
    }

    fn skip_comments_and_pi<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while let Some(token) = state.current() {
            match token.kind {
                XmlTokenType::Comment | XmlTokenType::ProcessingInstruction => state.bump(),
                _ => break,
            }
        }
    }
}
