//! Parser turning SMILES tokens into a green tree.

pub mod element_type;

use oak_core::{
    ParseOutput,
    parser::{ParseCache, Parser, parse_with_lexer},
    source::{Source, TextEdit},
};

use crate::{
    language::SmilesLanguage,
    lexer::{SmilesLexer, token_type::SmilesTokenType},
};

pub use element_type::SmilesElementType;

/// Parser for SMILES.
pub struct SmilesParser;

impl Parser<SmilesLanguage> for SmilesParser {
    fn parse<'a, S: Source + ?Sized>(
        &self,
        text: &'a S,
        edits: &[TextEdit],
        cache: &'a mut impl ParseCache<SmilesLanguage>,
    ) -> ParseOutput<'a, SmilesLanguage> {
        let lexer = SmilesLexer::default();
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let root = state.checkpoint();
            while state.not_at_end() && !state.at(SmilesTokenType::Eof) {
                let checkpoint = state.checkpoint();
                let element = match state.current().map(|token| token.kind) {
                    Some(SmilesTokenType::OrganicAtom) => SmilesElementType::OrganicAtom,
                    Some(SmilesTokenType::BracketAtom) => SmilesElementType::BracketAtom,
                    Some(SmilesTokenType::Bond) => SmilesElementType::Bond,
                    Some(SmilesTokenType::BranchOpen) => SmilesElementType::BranchOpen,
                    Some(SmilesTokenType::BranchClose) => SmilesElementType::BranchClose,
                    Some(SmilesTokenType::Dot) => SmilesElementType::Dot,
                    Some(SmilesTokenType::RingLabel) => SmilesElementType::RingLabel,
                    Some(SmilesTokenType::Arrow) => SmilesElementType::Arrow,
                    _ => SmilesElementType::Dot,
                };
                state.bump();
                state.finish_at(checkpoint, element);
            }
            Ok(state.finish_at(root, SmilesElementType::SourceFile))
        })
    }
}

impl Default for SmilesParser {
    fn default() -> Self {
        Self
    }
}
