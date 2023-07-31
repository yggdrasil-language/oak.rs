use crate::{language::AthenaLanguage, parser::AthenaParser};
use oak_core::{Builder, BuilderCache, Lexer, OakDiagnostics, Parser, TextEdit, source::Source};

/// AST builder for athena.
#[derive(Clone)]
pub struct AthenaBuilder<'config> {
    /// Language configuration.
    config: &'config AthenaLanguage,
}

impl<'config> AthenaBuilder<'config> {
    /// Creates a new builder.
    pub fn new(config: &'config AthenaLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<AthenaLanguage> for AthenaBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(
        &self,
        source: &S,
        edits: &[TextEdit],
        cache: &'a mut impl BuilderCache<AthenaLanguage>,
    ) -> OakDiagnostics<()> {
        let parser = AthenaParser::new(self.config);
        let lexer = crate::lexer::AthenaLexer::new(self.config);
        lexer.lex(source, edits, cache);
        let parse_result = parser.parse(source, edits, cache);
        match parse_result.result {
            Ok(_) => OakDiagnostics {
                result: Ok(()),
                diagnostics: parse_result.diagnostics,
            },
            Err(e) => OakDiagnostics {
                result: Err(e),
                diagnostics: parse_result.diagnostics,
            },
        }
    }
}
