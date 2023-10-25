//! `ImportDeclaration` RedTree formatting rule.

use oak_core::tree::{RedLeaf, RedNode};
use oak_pretty_print::{Document, FormatContext, FormatResult, FormatRule};

use crate::{
    cst_format::{CstFormatOptions, trivia_guard},
    language::TypeScriptLanguage,
    parser::element_type::TypeScriptElementType,
    print::{FormatOptions, format_source as ast_print_source},
};

/// Formats `import` declarations via AST print when trivia guards pass.
pub struct ImportDeclarationRule;

impl FormatRule<TypeScriptLanguage, CstFormatOptions> for ImportDeclarationRule {
    fn name(&self) -> &str {
        "typescript.import_declaration"
    }

    fn priority(&self) -> u8 {
        10
    }

    fn applies_to_node(&self, node: &RedNode<TypeScriptLanguage>) -> bool {
        node.element_type() == TypeScriptElementType::ImportDeclaration
    }

    fn apply_node<'a>(
        &self,
        node: &RedNode<TypeScriptLanguage>,
        _context: &FormatContext<TypeScriptLanguage, CstFormatOptions>,
        source: &'a str,
        _format_children: &dyn Fn(&RedNode<TypeScriptLanguage>) -> FormatResult<Document<'a>>,
    ) -> FormatResult<Option<Document<'a>>> {
        let span = node.span();
        let snippet = source.get(span.start..span.end).unwrap_or("");
        if trivia_guard::should_preserve_verbatim(snippet) {
            return Ok(Some(Document::text(snippet)));
        }
        let formatted = ast_print_source(snippet, &FormatOptions::default())
            .map_err(oak_core::errors::OakError::format_error)?;
        Ok(Some(Document::text(formatted)))
    }

    fn apply_token<'a>(
        &self,
        _token: &RedLeaf<TypeScriptLanguage>,
        _context: &FormatContext<TypeScriptLanguage, CstFormatOptions>,
        _source: &'a str,
    ) -> FormatResult<Option<Document<'a>>> {
        Ok(None)
    }
}
