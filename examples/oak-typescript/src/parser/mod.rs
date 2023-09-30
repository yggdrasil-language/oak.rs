/// Element types for TypeScript AST.
pub mod element_type;
mod parse_class;
mod parse_expression;
mod parse_jsx;
mod parse_misc;
mod parse_statement;

use crate::{language::TypeScriptLanguage, lexer::TypeScriptLexer};

use oak_core::{
    GreenNode, TextEdit,
    parser::{ParseCache, ParseOutput, Parser, ParserState, parse_with_lexer, pratt::Pratt},
    source::Source,
};

pub(crate) type State<'a, S> = ParserState<'a, TypeScriptLanguage, S>;

/// Parser for the TypeScript language.
pub struct TypeScriptParser<'config> {
    pub(crate) config: &'config TypeScriptLanguage,
}

impl<'config> TypeScriptParser<'config> {
    /// Creates a new `TypeScriptParser` with the given language configuration.
    pub fn new(config: &'config TypeScriptLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Pratt<TypeScriptLanguage> for TypeScriptParser<'config> {
    fn primary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, TypeScriptLanguage> {
        self.primary(state)
    }

    fn prefix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, TypeScriptLanguage> {
        self.prefix(state)
    }

    fn infix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, TypeScriptLanguage>, min_precedence: u8) -> Option<&'a GreenNode<'a, TypeScriptLanguage>> {
        self.infix(state, left, min_precedence)
    }
}

impl<'config> Parser<TypeScriptLanguage> for TypeScriptParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<TypeScriptLanguage>) -> ParseOutput<'a, TypeScriptLanguage> {
        let lexer = TypeScriptLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            while state.not_at_end() {
                self.skip_trivia(state);
                if state.not_at_end() {
                    let before = state.checkpoint();
                    self.parse_statement(state).ok();
                    if state.checkpoint().0 == before.0 && state.not_at_end() {
                        state.bump();
                    }
                }
            }
            Ok(state.finish_at(checkpoint, crate::parser::element_type::TypeScriptElementType::SourceFile))
        })
    }
}
