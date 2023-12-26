//! RedTree formatter scaffold via `oak-pretty-print` `FormatRule` + `RuleSet`.
//!
//! This is the path toward `oak-formatter` on `RedTree` with companion source text.
//! `VariableDeclaration` and `ImportDeclaration` have dedicated rules today. Other
//! supported statements still fall back to transitional AST print until more rules land.

use oak_core::tree::RedNode;
use oak_pretty_print::{Document, FormatContext, FormatResult, Printer, RuleSet};

use crate::{
    language::TypeScriptLanguage,
    print::{FormatOptions as PrintOptions, format_source as ast_print_source},
};

use super::{options::FormatOptions, trivia_guard};

mod rules;

/// TypeScript CST formatter driven by RedTree formatting rules.
pub struct TypeScriptRedTreeFormatter {
    rules: RuleSet<TypeScriptLanguage, FormatOptions>,
}

impl Default for TypeScriptRedTreeFormatter {
    fn default() -> Self {
        Self { rules: rules::default_rule_set() }
    }
}

impl TypeScriptRedTreeFormatter {
    /// Create a formatter with the default rule set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Format one top-level statement node.
    pub fn format_statement(&self, source: &str, node: &RedNode<TypeScriptLanguage>, options: &FormatOptions) -> Result<String, String> {
        let context = FormatContext::new(options.clone(), options.printer_config());
        let format_children = |child: &RedNode<TypeScriptLanguage>| -> FormatResult<Document<'_>> {
            let span = child.span();
            let text = source.get(span.start..span.end).unwrap_or("");
            Ok(Document::text(text))
        };

        if let Some(doc) = self.rules.apply_node_rules(node, &context, source, &format_children).map_err(|err| err.to_string())? {
            let mut printer_config = options.printer_config();
            printer_config.insert_final_newline = false;
            return Ok(Printer::new(printer_config).print(&doc));
        }

        let span = node.span();
        let snippet = source.get(span.start..span.end).unwrap_or("");
        format_statement_fallback(snippet)
    }
}

fn format_statement_fallback(snippet: &str) -> Result<String, String> {
    if trivia_guard::should_preserve_verbatim(snippet) {
        return Ok(snippet.to_string());
    }
    ast_print_source(snippet, &PrintOptions::default())
}
