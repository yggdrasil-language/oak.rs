mod from_red;

use crate::{HtmlParser, ast::HtmlDocument, language::HtmlLanguage};
use oak_core::{Builder, BuilderCache, GreenNode, OakDiagnostics, Parser, SourceText, TextEdit, builder::BuildOutput, source::Source};

/// A builder for HTML documents that coordinates parsing and AST construction.
#[derive(Clone)]
pub struct HtmlBuilder {
    config: HtmlLanguage,
}

impl HtmlBuilder {
    /// Creates a new `HtmlBuilder` with the specified language configuration.
    pub fn new(config: HtmlLanguage) -> Self {
        Self { config }
    }
}

impl Builder<HtmlLanguage> for HtmlBuilder {
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<HtmlLanguage>) -> BuildOutput<HtmlLanguage> {
        let parser = HtmlParser::new(&self.config);

        let mut parse_cache = oak_core::parser::session::ParseSession::<HtmlLanguage>::default();
        let parse_result = parser.parse(source, edits, &mut parse_cache);

        match parse_result.result {
            Ok(green_tree) => {
                let source_text = SourceText::new(source.get_text_in((0..source.length()).into()).into_owned());
                let ast_root = self.build_root(green_tree, &source_text);
                OakDiagnostics { result: Ok(ast_root), diagnostics: parse_result.diagnostics }
            }
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics: parse_result.diagnostics },
        }
    }
}

impl HtmlBuilder {
    pub(crate) fn build_root<'a>(&self, green_tree: &'a GreenNode<'a, HtmlLanguage>, source: &SourceText) -> HtmlDocument {
        from_red::build_document(green_tree, source)
    }
}
