use oak_core::{
    OakError, Source, TextEdit, TokenType,
    parser::{ParseCache, Parser, ParserState},
};

/// Element types for the VON parser.
pub mod element_type;
use crate::{
    language::VonLanguage,
    lexer::{VonLexer, VonTokenType},
};
pub use element_type::VonElementType;

pub(crate) type State<'a, S> = ParserState<'a, VonLanguage, S>;

/// Parser for VON (Value-Oriented Notation).
pub struct VonParser<'config> {
    pub(crate) config: &'config VonLanguage,
}

impl<'config> VonParser<'config> {
    /// Creates a new VON parser.
    pub fn new(config: &'config VonLanguage) -> Self {
        Self { config }
    }

    /// Parses a value in VON.
    ///
    /// Values can be objects, arrays, enums, strings, numbers, booleans, or null.
    pub(crate) fn parse_value<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        self.skip_trivia(state);
        state.incremental_node(VonElementType::Value, |state| {
            let token = if let Some(t) = state.current() {
                if t.kind == VonTokenType::Eof {
                    return Err(state.unexpected_eof());
                }
                t
            }
            else {
                return Err(state.unexpected_eof());
            };

            match token.kind {
                VonTokenType::LeftBrace => self.parse_object(state),
                VonTokenType::LeftBracket => self.parse_array(state),
                VonTokenType::Identifier => self.parse_enum(state),
                VonTokenType::StringLiteral | VonTokenType::NumberLiteral | VonTokenType::BoolLiteral | VonTokenType::NullLiteral => {
                    state.bump();
                    Ok(())
                }
                _ => {
                    state.record_unexpected_token(format!("{:?}", token.kind));
                    Err(state.errors.last().unwrap().clone())
                }
            }
        })
    }

    /// Parses an enum variant in VON.
    fn parse_enum<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        state.incremental_node(VonElementType::Enum, |state| {
            if !state.eat(VonTokenType::Identifier) {
                state.record_expected("variant");
                return Err(state.errors.last().cloned().expect("Error should have been recorded"));
            }

            self.skip_trivia(state);
            if state.at(VonTokenType::LeftBrace) || state.at(VonTokenType::LeftBracket) || state.at(VonTokenType::LeftParen) {
                if state.at(VonTokenType::LeftParen) {
                    state.bump();
                    self.parse_value(state)?;
                    self.skip_trivia(state);
                    if !state.eat(VonTokenType::RightParen) {
                        state.record_expected(")");
                    }
                }
                else {
                    self.parse_value(state)?;
                }
            }
            Ok(())
        })
    }

    /// Parses the content of an object in VON.
    pub(crate) fn parse_object_inner<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let mut first = true;
        while state.not_at_end() {
            self.skip_trivia(state);
            if state.at(VonTokenType::RightBrace) {
                break;
            }

            if state.at(VonTokenType::Eof) {
                break;
            }

            if !first {
                // Comma is optional, but consume it if it exists
                if state.eat(VonTokenType::Comma) {
                    self.skip_trivia(state);
                    if state.at(VonTokenType::RightBrace) || state.at(VonTokenType::Eof) {
                        break;
                    }
                }
            }
            first = false;

            // Check if it's actually an ObjectEntry (starts with Identifier or StringLiteral)
            if !state.at(VonTokenType::Identifier) && !state.at(VonTokenType::StringLiteral) {
                break;
            }

            self.parse_object_entry(state)?;
            self.skip_trivia(state);
        }
        Ok(())
    }

    /// Parses an object in VON.
    fn parse_object<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        state.incremental_node(VonElementType::Object, |state| {
            if !state.eat(VonTokenType::LeftBrace) {
                state.record_expected("{");
                return Err(state.errors.last().cloned().expect("Error should have been recorded"));
            }

            self.parse_object_inner(state)?;

            if !state.eat(VonTokenType::RightBrace) {
                if state.at(VonTokenType::Eof) || !state.not_at_end() {
                    return Err(state.unexpected_eof());
                }
                state.record_expected("}");
            }
            Ok(())
        })
    }

    /// Parses an entry in an object in VON.
    fn parse_object_entry<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        state.incremental_node(VonElementType::ObjectEntry, |state| {
            if state.at(VonTokenType::Identifier) || state.at(VonTokenType::StringLiteral) {
                state.bump();
            }
            else {
                state.record_expected("key");
                return Err(state.errors.last().cloned().expect("Error should have been recorded"));
            }

            self.skip_trivia(state);
            if state.at(VonTokenType::Eq) || state.at(VonTokenType::Colon) {
                state.bump();
            }
            else {
                state.record_expected("= or :");
            }
            self.skip_trivia(state);
            // Ensure we haven't reached EOF or } before trying to parse a value
            if state.at(VonTokenType::RightBrace) || state.at(VonTokenType::Eof) {
                state.record_expected("value");
                return Err(state.errors.last().cloned().expect("Error should have been recorded"));
            }
            self.parse_value(state)?;
            Ok(())
        })
    }

    /// Parses an array in VON.
    fn parse_array<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        state.incremental_node(VonElementType::Array, |state| {
            if !state.eat(VonTokenType::LeftBracket) {
                state.record_expected("[");
                return Err(state.errors.last().cloned().expect("Error should have been recorded"));
            }

            let mut first = true;
            while state.not_at_end() {
                self.skip_trivia(state);
                if state.at(VonTokenType::RightBracket) {
                    break;
                }

                if state.at(VonTokenType::Eof) {
                    return Err(state.unexpected_eof());
                }

                if !first {
                    // Comma is optional
                    if state.eat(VonTokenType::Comma) {
                        self.skip_trivia(state);
                        if state.at(VonTokenType::RightBracket) {
                            break;
                        }
                    }
                }
                first = false;

                state.incremental_node(VonElementType::ArrayElement, |state| self.parse_value(state))?;
                self.skip_trivia(state);
            }

            if !state.eat(VonTokenType::RightBracket) {
                if state.at(VonTokenType::Eof) || !state.not_at_end() {
                    return Err(state.unexpected_eof());
                }
                state.record_expected("]");
            }
            Ok(())
        })
    }

    /// Skips whitespace and comments.
    fn skip_trivia<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while let Some(token) = state.current() {
            if token.kind.is_ignored() {
                state.bump();
            }
            else {
                break;
            }
        }
    }
}

impl<'config> Parser<VonLanguage> for VonParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, source: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<VonLanguage>) -> oak_core::ParseOutput<'a, VonLanguage> {
        let lexer = VonLexer::new(&self.config);
        oak_core::parser::parse_with_lexer(&lexer, source, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            self.skip_trivia(state);

            // Check if it is an implicit root object (does not start with { or [)
            if state.at(VonTokenType::LeftBrace) || state.at(VonTokenType::LeftBracket) {
                let _ = self.parse_value(state);
            }
            else if state.at(VonTokenType::Identifier) {
                // If it starts with an identifier, it might be an implicit object or Enum
                // We first try to parse as an implicit object, if it fails then backtrack (here simply handled as an implicit object)
                let _ = state.incremental_node(VonElementType::Object, |state| self.parse_object_inner(state));
            }
            else if state.at(VonTokenType::Eof) {
                // Empty file
            }
            else {
                // Otherwise, try to parse as a normal value
                let _ = self.parse_value(state);
            }

            while state.not_at_end() {
                if let Some(token) = state.current() {
                    if token.kind.is_ignored() {
                        state.bump();
                        continue;
                    }
                }
                break;
            }

            Ok(state.finish_at(checkpoint, crate::parser::element_type::VonElementType::Root))
        })
    }
}
