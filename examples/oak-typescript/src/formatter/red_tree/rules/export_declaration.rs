//! CST layout for exported classes, with member bodies retained losslessly.

use oak_core::{
    errors::OakError,
    tree::{RedLeaf, RedNode, RedTree},
};
use oak_pretty_print::{Document, FormatContext, FormatResult, FormatRule};

use crate::{formatter::CstFormatOptions, language::TypeScriptLanguage, lexer::token_type::TypeScriptTokenType, parser::element_type::TypeScriptElementType};

/// Format exported class boundaries without regenerating their member bodies.
pub struct ExportDeclarationRule;

fn class_body<'tree>(node: &RedNode<'tree, TypeScriptLanguage>) -> Option<RedNode<'tree, TypeScriptLanguage>> {
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if child.element_type() == TypeScriptElementType::ClassBody {
                return Some(child);
            }
            if child.element_type() == TypeScriptElementType::ClassDeclaration {
                return class_body(&child);
            }
        }
    }
    None
}

impl FormatRule<TypeScriptLanguage, CstFormatOptions> for ExportDeclarationRule {
    fn name(&self) -> &str {
        "typescript.export_declaration"
    }

    fn priority(&self) -> u8 {
        5
    }

    fn applies_to_node(&self, node: &RedNode<TypeScriptLanguage>) -> bool {
        node.element_type() == TypeScriptElementType::ExportDeclaration
    }

    fn apply_node<'a>(
        &self,
        node: &RedNode<TypeScriptLanguage>,
        context: &FormatContext<TypeScriptLanguage, CstFormatOptions>,
        source: &'a str,
        _format_children: &dyn Fn(&RedNode<TypeScriptLanguage>) -> FormatResult<Document<'a>>,
    ) -> FormatResult<Option<Document<'a>>> {
        let Some(body) = class_body(node)
        else {
            return Ok(None);
        };
        let span = node.span();
        let open = body
            .children()
            .find_map(|child| match child {
                RedTree::Leaf(leaf) if leaf.kind() == TypeScriptTokenType::LeftBrace => Some(leaf.span().start),
                _ => None,
            })
            .ok_or_else(|| OakError::format_error("class CST missing opening brace"))?;
        let close = body
            .children()
            .filter_map(|child| match child {
                RedTree::Leaf(leaf) if leaf.kind() == TypeScriptTokenType::RightBrace => Some(leaf.span().start),
                _ => None,
            })
            .last()
            .ok_or_else(|| OakError::format_error("class CST missing closing brace"))?;
        if open >= close {
            return Err(OakError::format_error("invalid class CST delimiters"));
        }
        let mut output = format!("{} {{", source[span.start..open].trim_end());
        let unit = " ".repeat(usize::from(context.config.indent_width));
        let mut cursor = open + 1;
        for child in body.children() {
            let RedTree::Node(member) = child
            else {
                continue;
            };
            let member_span = member.span();
            if member_span.start < cursor || member_span.end > close {
                return Err(OakError::format_error("overlapping class CST spans"));
            }
            let gap = source[cursor..member_span.start].trim();
            if !gap.is_empty() {
                output.push('\n');
                output.push_str(&unit);
                output.push_str(gap);
            }
            let member_text = source[member_span.start..member_span.end].trim();
            if !member_text.is_empty() {
                output.push('\n');
                output.push_str(&unit);
                output.push_str(member_text);
            }
            cursor = member_span.end;
        }
        let trailing = source[cursor..close].trim();
        if !trailing.is_empty() {
            output.push('\n');
            output.push_str(&unit);
            output.push_str(trailing);
        }
        if output.ends_with('{') {
            output.push('}');
        }
        else {
            output.push_str("\n}");
        }
        output.push_str(&source[close + 1..span.end]);
        Ok(Some(Document::text(output)))
    }

    fn apply_token<'a>(&self, _token: &RedLeaf<TypeScriptLanguage>, _context: &FormatContext<TypeScriptLanguage, CstFormatOptions>, _source: &'a str) -> FormatResult<Option<Document<'a>>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::formatter::{FormatOptions, format_source};

    #[test]
    fn exported_class_members_are_separated_and_idempotent() {
        let options = FormatOptions { indent_width: 4, line_width: 120, ..FormatOptions::default() };
        let input = "export default class Foo{count=0;inc(){this.count++}}";
        let output = format_source(input, &options).unwrap();
        assert_eq!(output, "export default class Foo {\n    count=0;\n    inc(){this.count++}\n}");
        assert_eq!(format_source(&output, &options).unwrap(), output);
    }

    #[test]
    fn exported_class_keeps_literal_and_comment_contents() {
        let input = "export default class Foo {\n// keep\ntext = ` a\n b `;\nmethod() { return /a{2}/; }\n}";
        let options = FormatOptions::default();
        let output = format_source(input, &options).unwrap();
        assert!(output.contains("// keep"));
        assert!(output.contains("` a\n b `"));
        assert!(output.contains("/a{2}/"));
        assert_eq!(format_source(&output, &options).unwrap(), output);
    }

    #[test]
    fn exported_empty_class_and_invalid_class() {
        let options = FormatOptions::default();
        assert_eq!(format_source("export default class Foo {}", &options).unwrap(), "export default class Foo {}");
        assert!(format_source("export default class Foo {", &options).is_err());
    }

    #[test]
    fn class_text_in_exported_string_is_not_a_class() {
        let options = FormatOptions::default();
        assert_eq!(format_source("export const text='class Foo'", &options).unwrap(), "export const text = 'class Foo'");
    }
}
