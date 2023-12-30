//! `VariableDeclaration` RedTree formatting rule (first `FormatRule` scaffold).

use oak_core::tree::{RedLeaf, RedNode};
use oak_pretty_print::{Document, FormatContext, FormatResult, FormatRule};

use crate::{
    formatter::{options::FormatOptions, trivia_guard},
    language::TypeScriptLanguage,
    parser::element_type::TypeScriptElementType,
    printer::{FormatOptions as PrintOptions, format_source as ast_print_source},
};

/// Formats `const`/`let`/`var` declarations via AST print when trivia guards pass.
pub struct VariableDeclarationRule;

impl FormatRule<TypeScriptLanguage, FormatOptions> for VariableDeclarationRule {
    fn name(&self) -> &str {
        "typescript.variable_declaration"
    }

    fn priority(&self) -> u8 {
        10
    }

    fn applies_to_node(&self, node: &RedNode<TypeScriptLanguage>) -> bool {
        node.element_type() == TypeScriptElementType::VariableDeclaration
    }

    fn apply_node<'a>(
        &self,
        node: &RedNode<TypeScriptLanguage>,
        _context: &FormatContext<TypeScriptLanguage, FormatOptions>,
        source: &'a str,
        _format_children: &dyn Fn(&RedNode<TypeScriptLanguage>) -> FormatResult<Document<'a>>,
    ) -> FormatResult<Option<Document<'a>>> {
        let span = node.span();
        let snippet = source.get(span.start..span.end).unwrap_or("");
        if trivia_guard::should_preserve_verbatim(snippet) {
            return Ok(Some(Document::text(snippet)));
        }
        let formatted = ast_print_source(snippet, &PrintOptions::default()).map_err(oak_core::errors::OakError::format_error)?;
        Ok(Some(Document::text(formatted)))
    }

    fn apply_token<'a>(&self, _token: &RedLeaf<TypeScriptLanguage>, _context: &FormatContext<TypeScriptLanguage, FormatOptions>, _source: &'a str) -> FormatResult<Option<Document<'a>>> {
        Ok(None)
    }
}
