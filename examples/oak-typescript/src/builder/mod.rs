use crate::{ast::*, language::TypeScriptLanguage, lexer::TypeScriptLexer, parser::TypeScriptParser};
use oak_core::{Builder, BuilderCache, Lexer, OakDiagnostics, Parser, SourceText, TextEdit, source::Source};

mod build_class;
mod build_expression;
pub use build_expression::decode_string_literal_text;
mod build_jsx;
mod build_misc;
mod build_root;
mod build_statement;
mod build_type;

/// AST builder for the TypeScript language.
#[derive(Clone)]
pub struct TypeScriptBuilder<'config> {
    config: &'config TypeScriptLanguage,
    /// Whether to erase types and generate a pure JS AST.
    pub erase_types: bool,
}

impl<'config> TypeScriptBuilder<'config> {
    /// Creates a new `TypeScriptBuilder`.
    pub fn new(config: &'config TypeScriptLanguage) -> Self {
        Self { config, erase_types: false }
    }

    /// Sets whether to erase types.
    pub fn with_type_erasure(mut self, erase_types: bool) -> Self {
        self.erase_types = erase_types;
        self
    }
}

impl<'config> Builder<TypeScriptLanguage> for TypeScriptBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(&self, source: &'a S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<TypeScriptLanguage>) -> OakDiagnostics<TypeScriptRoot> {
        let parser = TypeScriptParser::new(self.config);
        let lexer = TypeScriptLexer::new(&self.config);

        let mut session = oak_core::parser::session::ParseSession::<TypeScriptLanguage>::default();
        lexer.lex(source, edits, &mut session);
        let parse_result = parser.parse(source, edits, &mut session);

        match parse_result.result {
            Ok(green_tree) => {
                let source_text = SourceText::new(source.get_text_in((0..source.length()).into()).into_owned());
                match self.build_root(green_tree, &source_text) {
                    Ok(ast_root) => OakDiagnostics { result: Ok(ast_root), diagnostics: parse_result.diagnostics },
                    Err(build_error) => {
                        let mut diagnostics = parse_result.diagnostics;
                        diagnostics.push(build_error.clone());
                        OakDiagnostics { result: Err(build_error), diagnostics }
                    }
                }
            }
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics: parse_result.diagnostics },
        }
    }
}
