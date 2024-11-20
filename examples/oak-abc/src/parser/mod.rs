//! Parser turning ABC lines into a green tree.

pub mod element_type;

use oak_core::{
    ParseOutput,
    parser::{ParseCache, Parser, parse_with_lexer},
    source::{Source, TextEdit},
};

use crate::{language::AbcLanguage, lexer::AbcLexer};

pub use element_type::AbcElementType;

/// Parser for ABC notation.
pub struct AbcParser;

impl Parser<AbcLanguage> for AbcParser {
    fn parse<'a, S: Source + ?Sized>(
        &self,
        text: &'a S,
        edits: &[TextEdit],
        cache: &'a mut impl ParseCache<AbcLanguage>,
    ) -> ParseOutput<'a, AbcLanguage> {
        let lexer = AbcLexer::default();
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let root = state.checkpoint();
            while state.not_at_end() && !state.at(crate::lexer::token_type::AbcTokenType::Eof) {
                let checkpoint = state.checkpoint();
                state.bump();
                state.finish_at(checkpoint, AbcElementType::Line);
            }
            Ok(state.finish_at(root, AbcElementType::SourceFile))
        })
    }
}

impl Default for AbcParser {
    fn default() -> Self {
        Self
    }
}
