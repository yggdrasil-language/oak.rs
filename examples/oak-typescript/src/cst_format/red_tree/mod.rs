//! RedTree formatter scaffold via `oak-pretty-print` `FormatRule` + `RuleSet`.
//!
//! This is the path toward `oak-formatter` on `RedTree` with companion source text.
//! `VariableDeclaration` and `ImportDeclaration` have dedicated rules today. Other
//! supported statements still fall back to transitional AST print until more rules land.

use oak_core::tree::RedNode;
use oak_pretty_print::{Document, FormatContext, FormatResult, Printer, RuleSet};

use crate::{
    language::TypeScriptLanguage,
    print::{FormatOptions, format_source as ast_print_source},
};

use super::{CstFormatOptions, trivia_guard};

mod rules;

/// TypeScript CST formatter driven by RedTree formatting rules.
pub struct TypeScriptRedTreeFormatter {
    rules: RuleSet<TypeScriptLanguage, CstFormatOptions>,
}

impl Default for TypeScriptRedTreeFormatter {
    fn default() -> Self {
        Self {
            rules: rules::default_rule_set(),
        }
    }
}

impl TypeScriptRedTreeFormatter {
    /// Create a formatter with the default rule set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Format one top-level statement node.
    pub fn format_statement(
        &self,
        source: &str,
        node: &RedNode<TypeScriptLanguage>,
        options: &CstFormatOptions,
    ) -> Result<String, String> {
        let context = FormatContext::new(options.clone(), options.printer_config());
        let format_children = |child: &RedNode<TypeScriptLanguage>| -> FormatResult<Document<'_>> {
            let span = child.span();
            let text = source.get(span.start..span.end).unwrap_or("");
            Ok(Document::text(text))
        };

        if let Some(doc) = self
            .rules
            .apply_node_rules(node, &context, source, &format_children)
            .map_err(|err| err.to_string())?
        {
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
    ast_print_source(snippet, &FormatOptions::default())
}

#[cfg(test)]
mod tests {
    use oak_core::{ParseSession, Parser, RedNode, SourceText};

    use crate::{
        cst_format::CstFormatOptions,
        language::TypeScriptLanguage,
        parser::{TypeScriptParser, element_type::TypeScriptElementType},
    };

    use super::*;

    #[test]
    fn variable_declaration_rule_normalizes_spacing() {
        let input = "const  x=1";
        let text = SourceText::new(input);
        let language = TypeScriptLanguage::default();
        let parser = TypeScriptParser::new(&language);
        let mut session = ParseSession::default();
        let parsed = parser.parse(&text, &[], &mut session).result.expect("parse");
        let file = RedNode::new(parsed, 0);
        let stmt = file
            .children()
            .find_map(|child| match child {
                oak_core::RedTree::Node(node) => Some(node),
                oak_core::RedTree::Leaf(_) => None,
            })
            .expect("statement");
        assert_eq!(
            stmt.element_type(),
            TypeScriptElementType::VariableDeclaration
        );
        let out = TypeScriptRedTreeFormatter::new()
            .format_statement(input, &stmt, &CstFormatOptions::default())
            .expect("format");
        assert_eq!(out, "const x = 1");
    }

    #[test]
    fn import_declaration_rule_normalizes_spacing() {
        let input = "import  {  foo }  from 'pkg'";
        let text = SourceText::new(input);
        let language = TypeScriptLanguage::default();
        let parser = TypeScriptParser::new(&language);
        let mut session = ParseSession::default();
        let parsed = parser.parse(&text, &[], &mut session).result.expect("parse");
        let file = RedNode::new(parsed, 0);
        let stmt = file
            .children()
            .find_map(|child| match child {
                oak_core::RedTree::Node(node) => Some(node),
                oak_core::RedTree::Leaf(_) => None,
            })
            .expect("statement");
        assert_eq!(
            stmt.element_type(),
            TypeScriptElementType::ImportDeclaration
        );
        let out = TypeScriptRedTreeFormatter::new()
            .format_statement(input, &stmt, &CstFormatOptions::default())
            .expect("format");
        assert_eq!(out, "import { foo } from 'pkg';");
    }
}
