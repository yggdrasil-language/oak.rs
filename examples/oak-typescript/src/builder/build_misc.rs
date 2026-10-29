use crate::{ast::*, builder::TypeScriptBuilder, language::TypeScriptLanguage, lexer::token_type::TypeScriptTokenType, parser::element_type::TypeScriptElementType};
use oak_core::{OakError, RedNode, RedTree, Source, SourceText};

impl<'config> TypeScriptBuilder<'config> {
    pub(crate) fn build_parameter(&self, node: &RedNode<TypeScriptLanguage>, source: &SourceText) -> Result<Option<FunctionParam>, OakError> {
        let mut name = String::new();
        let mut ty = None;
        let mut optional_marker = None;
        let mut decorators = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Node(child_node) => match child_node.green.kind {
                    TypeScriptElementType::IdentifierName => name = source.get_text_in(child_node.span().into()).to_string(),
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
                },
                RedTree::Leaf(leaf) => {
                    if leaf.kind == TypeScriptTokenType::Question {
                        optional_marker = Some(OptionalMarker::new(leaf.span))
                    }
                    else if leaf.kind == TypeScriptTokenType::IdentifierName {
                        name = source.get_text_in(leaf.span.into()).to_string();
                    }
                }
            }
        }
        let span = node.span();
        Ok(Some(FunctionParam { name, ty, optional_marker, decorators, span: span.into() }))
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
