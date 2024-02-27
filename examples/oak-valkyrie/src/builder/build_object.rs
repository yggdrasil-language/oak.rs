use crate::{ValkyrieLanguage, ast::*, builder::ValkyrieBuilder, lexer::token_type::ValkyrieTokenType};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_object<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut callee = None;
        let mut fields = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::BlockExpression => {
                        for block_child in n.children() {
                            if let RedTree::Node(stmt_n) = block_child {
                                match stmt_n.green.kind {
                                    crate::parser::element_type::ValkyrieElementType::Whitespace
                                    | crate::parser::element_type::ValkyrieElementType::Newline
                                    | crate::parser::element_type::ValkyrieElementType::LineComment
                                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                                    crate::parser::element_type::ValkyrieElementType::ExprStatement => {
                                        for expr_child in stmt_n.children() {
                                            if let RedTree::Node(expr_n) = expr_child {
                                                match expr_n.green.kind {
                                                    crate::parser::element_type::ValkyrieElementType::Whitespace
                                                    | crate::parser::element_type::ValkyrieElementType::Newline
                                                    | crate::parser::element_type::ValkyrieElementType::LineComment
                                                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                                                    crate::parser::element_type::ValkyrieElementType::BinaryExpression => {
                                                        if let Some((name, value)) = self.extract_object_field(&expr_n, source)? {
                                                            fields.push((name, Some(value)));
                                                        }
                                                    }
                                                    crate::parser::element_type::ValkyrieElementType::IdentifierExpression => {
                                                        if let Ok(expr) = self.build_identifier_expr(expr_n.clone(), source) {
                                                            if let TermExpression::NamePath(path) = expr {
                                                                if let Some(ident) = path.parts.first().cloned() {
                                                                    fields.push((ident, None));
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {
                        if callee.is_none() {
                            callee = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
            }
        }

        let callee = callee.ok_or_else(|| source.syntax_error("Missing object callee".to_string(), span.start))?;

        Ok(TermExpression::Object { callee, fields, span })
    }

    /// Extracts a field name and value from an object field expression.
    ///
    /// Supports both new syntax (`:` separator) and deprecated syntax (`=` separator).
    /// When the deprecated `=` syntax is detected, a warning is logged.
    fn extract_object_field<S: Source + ?Sized>(&self, node: &RedNode<ValkyrieLanguage>, source: &S) -> Result<Option<(Identifier, TermExpression)>, OakError> {
        let mut field_name = None;
        let mut value = None;
        let mut separator_found = false;
        let mut uses_deprecated_syntax = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Eq => {
                        if !separator_found {
                            separator_found = true;
                            uses_deprecated_syntax = true;
                        }
                    }
                    ValkyrieTokenType::Colon => {
                        if !separator_found {
                            separator_found = true;
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::IdentifierExpression => {
                        if field_name.is_none() {
                            if let Ok(expr) = self.build_identifier_expr(n.clone(), source) {
                                if let TermExpression::NamePath(path) = expr {
                                    if let Some(ident) = path.parts.first().cloned() {
                                        field_name = Some(ident);
                                    }
                                }
                            }
                        }
                        else if value.is_none() {
                            value = Some(self.build_identifier_expr(n.clone(), source)?);
                        }
                    }
                    _ => {
                        if field_name.is_some() && value.is_none() {
                            value = Some(self.build_expr(n, source)?);
                        }
                    }
                },
            }
        }

        if uses_deprecated_syntax {
            if let Some(ref name) = field_name {
                eprintln!("Warning: Use of deprecated '=' syntax in object field at offset {}. Use ':' instead. Field: '{}'", name.span.start, name.name);
            }
        }

        if let (Some(name), Some(val)) = (field_name, value) { Ok(Some((name, val))) } else { Ok(None) }
    }
}
