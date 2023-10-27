/// Element type module for HTML.
pub mod element_type;

use crate::{
    language::HtmlLanguage,
    lexer::{HtmlLexer, token_type::HtmlTokenType},
    parser::element_type::HtmlElementType,
};
use oak_core::{
    GreenNode, OakError,
    parser::{ParseCache, ParseOutput, Parser, ParserState, parse_with_lexer},
    source::{Source, TextEdit},
};

pub(crate) type State<'a, S> = ParserState<'a, HtmlLanguage, S>;

/// Parser for the HTML language.
///
/// This parser transforms a stream of tokens into a green tree of HTML syntax nodes.
pub struct HtmlParser<'config> {
    pub(crate) config: &'config HtmlLanguage,
}

impl<'config> HtmlParser<'config> {
    /// Creates a new `HtmlParser` with the given configuration.
    pub fn new(config: &'config HtmlLanguage) -> Self {
        Self { config }
    }

    /// Parses an HTML tag, including its attributes and potentially its children.
    ///
    /// This method handles both self-closing tags (e.g., `<br/>`) and tags with
    /// separate closing tags (e.g., `<div>...</div>`).
    fn parse_tag<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::HtmlTokenType::*;
        let cp = state.checkpoint();
        state.expect(TagOpen).ok();
        state.expect(TagName).ok();

        while state.not_at_end() && !matches!(state.peek_kind(), Some(TagClose) | Some(TagSelfClose)) {
            if state.at(AttributeName) || state.at(TagName) {
                let attr_cp = state.checkpoint();
                state.bump(); // attribute name token
                if state.eat(Equal) {
                    state.eat(Quote);
                    state.eat(AttributeValue);
                    state.eat(Quote);
                }
                state.finish_at(attr_cp, HtmlElementType::Attribute);
            }
            else {
                state.advance();
            }
        }

        if state.eat(TagSelfClose) {
            // Self-closing tag
        }
        else if state.eat(TagClose) {
            // Recurse to parse children until the matching closing tag is found
            // Simplified handling: skip until closing tag
            while state.not_at_end() && !state.at(TagSlashOpen) {
                if state.at(TagOpen) {
                    self.parse_tag(state)?
                }
                else {
                    state.advance();
                }
            }
            if state.eat(TagSlashOpen) {
                state.eat(TagName);
                state.expect(TagClose).ok();
            }
        }

        state.finish_at(cp, HtmlElementType::Element);
        Ok(())
    }
}

impl<'config> Parser<HtmlLanguage> for HtmlParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<HtmlLanguage>) -> ParseOutput<'a, HtmlLanguage> {
        let lexer = HtmlLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();

            while state.not_at_end() {
                match state.peek_kind() {
                    Some(HtmlTokenType::TagOpen) => self.parse_tag(state)?,
                    Some(HtmlTokenType::Doctype) => {
                        state.bump();
                    }
                    Some(HtmlTokenType::Comment) => {
                        state.bump();
                    }
                    _ => {
                        state.bump();
                    }
                }
            }

            Ok(state.finish_at(checkpoint, crate::parser::element_type::HtmlElementType::Document))
        })
    }
}
