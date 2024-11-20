//! Parser turning MDL tokens into a green tree.

pub mod element_type;

use oak_core::{
    ParseOutput,
    parser::{ParseCache, Parser, ParserState, parse_with_lexer},
    source::{Source, TextEdit},
};

use crate::{
    language::MdlLanguage,
    lexer::{MdlLexer, token_type::MdlTokenType},
};

pub use element_type::MdlElementType;

/// Parser state alias for the MDL language.
pub(crate) type State<'a, S> = ParserState<'a, MdlLanguage, S>;

/// Parser for MDL connection tables.
pub struct MdlParser;

impl Parser<MdlLanguage> for MdlParser {
    fn parse<'a, S: Source + ?Sized>(
        &self,
        text: &'a S,
        edits: &[TextEdit],
        cache: &'a mut impl ParseCache<MdlLanguage>,
    ) -> ParseOutput<'a, MdlLanguage> {
        let lexer = MdlLexer::default();
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let root = state.checkpoint();
            while state.not_at_end() && !state.at(MdlTokenType::Eof) {
                let checkpoint = state.checkpoint();
                let element = match state.current().map(|token| token.kind) {
                    Some(MdlTokenType::Line) => MdlElementType::HeaderLine,
                    Some(MdlTokenType::Property) => MdlElementType::PropertyLine,
                    Some(MdlTokenType::DataHeader) => MdlElementType::DataFieldName,
                    Some(MdlTokenType::RecordSeparator) => MdlElementType::RecordSeparator,
                    Some(MdlTokenType::ReactionMarker) => MdlElementType::ReactionMarker,
                    _ => MdlElementType::HeaderLine,
                };
                state.bump();
                state.finish_at(checkpoint, element);
            }
            Ok(state.finish_at(root, MdlElementType::SourceFile))
        })
    }
}

impl Default for MdlParser {
    fn default() -> Self {
        Self
    }
}
