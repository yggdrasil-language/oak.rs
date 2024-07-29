use crate::{ast::*, builder::TypeScriptBuilder, language::TypeScriptLanguage, lexer::token_type::TypeScriptTokenType, parser::element_type::TypeScriptElementType};
use oak_core::{OakError, RedNode, RedTree, Source, SourceText};

impl<'config> TypeScriptBuilder<'config> {
    pub(crate) fn build_parameter(&self, node: &RedNode<TypeScriptLanguage>, source: &SourceText) -> Result<Option<FunctionParam>, OakError> {
        let mut name = String::new();
        let mut binding_names = Vec::new();
        let mut pattern_text = None;
        let mut default = None;
        let mut is_rest = false;
        let mut ty = None;
        let mut optional_marker = None;
        let mut decorators = Vec::new();
        let mut after_equal = false;

        for child in node.children() {
            match child {
                RedTree::Node(child_node) => {
                    let child_span = child_node.span();
                    if after_equal {
                        if default.is_none() {
                            default = self.build_expression(&child_node, source)?;
                        }
                        continue;
                    }
                    match child_node.green.kind {
                        TypeScriptElementType::IdentifierName => {
                            let text = crate::builder::build_binding::binding_identifier_text(&child_node, source);
                            if !text.is_empty() {
                                name = text.clone();
                                binding_names.push(text);
                            }
                        }
                        TypeScriptElementType::BindingPattern => {
                            pattern_text = Some(source.get_text_in(child_span.into()).trim().to_string());
                            binding_names.extend(crate::builder::build_binding::binding_pattern_names(&child_node, source));
                        }
                        TypeScriptElementType::TypeAnnotation => {
                            if !self.erase_types {
                                ty = self.build_type_annotation(&child_node, source)?
                            }
                        }
                        TypeScriptElementType::Decorator => {
                            if let Some(d) = self.build_decorator(&child_node, source)? {
                                decorators.push(d)
                            }
                        }
                        _ => {}
                    }
                }
                RedTree::Leaf(leaf) => match leaf.kind {
                    TypeScriptTokenType::Question => {
                        if !after_equal {
                            optional_marker = Some(OptionalMarker::new(leaf.span))
                        }
                    }
                    TypeScriptTokenType::DotDotDot => is_rest = true,
                    TypeScriptTokenType::Equal => after_equal = true,
                    TypeScriptTokenType::IdentifierName => {
                        if !after_equal && name.is_empty() {
                            name = source.get_text_in(leaf.span.into()).trim().to_string();
                            binding_names.push(name.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
        let span = node.span();
        Ok(Some(FunctionParam { name, binding_names, pattern_text, default, is_rest, ty, optional_marker, decorators, span: span.into() }))
    }

    pub(crate) fn build_enum_member(&self, node: &RedNode<TypeScriptLanguage>, source: &SourceText) -> Result<Option<EnumMember>, OakError> {
        let mut name = String::new();
        let mut initializer = None;
        let span = node.span();

        for child in node.children() {
            if let RedTree::Node(child_node) = child {
                match child_node.green.kind {
                    TypeScriptElementType::IdentifierName => name = source.get_text_in(child_node.span().into()).to_string(),
                    _ => {
                        if let Some(expr) = self.build_expression(&child_node, source)? {
                            initializer = Some(expr)
                        }
                    }
                }
            }
        }
        Ok(Some(EnumMember { name, initializer, span: span.into() }))
    }

    pub(crate) fn build_decorator(&self, node: &RedNode<TypeScriptLanguage>, source: &SourceText) -> Result<Option<Decorator>, OakError> {
        let mut expression = None;
        let span = node.span();

        for child in node.children() {
            if let RedTree::Node(child_node) = child {
                expression = self.build_expression(&child_node, source)?
            }
        }

        if let Some(expr) = expression { Ok(Some(Decorator { expression: expr, span: span.into() })) } else { Ok(None) }
    }
}
