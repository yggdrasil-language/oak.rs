use crate::{ast::*, builder::TypeScriptBuilder, language::TypeScriptLanguage, lexer::token_type::TypeScriptTokenType, parser::element_type::TypeScriptElementType};
use oak_core::{OakError, RedNode, RedTree, Source, SourceText};

impl<'config> TypeScriptBuilder<'config> {
    pub(crate) fn build_class_member(&self, node: &RedNode<TypeScriptLanguage>, source: &SourceText) -> Result<Option<ClassMember>, OakError> {
        let kind = node.green.kind;
        let span = node.span();

        match kind {
            TypeScriptElementType::PropertyDeclaration | TypeScriptElementType::PropertySignature => {
                let mut name = String::new();
                let mut ty = None;
                let mut value = None;
                let mut visibility = None;
                let mut modifiers: Vec<Modifier> = Vec::new();
                let mut optional_marker = None;
                let mut decorators = Vec::new();

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::IdentifierName => {
                                    name = source.get_text_in(child_node.span().into()).trim().to_string();
                                }
                                TypeScriptElementType::TypeAnnotation => {
                                    if !self.erase_types {
                                        ty = self.build_type_annotation(&child_node, source)?;
                                    }
                                }
                                TypeScriptElementType::Decorator => {
                                    if let Some(d) = self.build_decorator(&child_node, source)? {
                                        decorators.push(d);
                                    }
                                }
                                _ => {
                                    if value.is_none() {
                                        value = self.build_expression(&child_node, source)?;
                                    }
                                }
                            }
                        }
                        RedTree::Leaf(leaf) => match leaf.kind {
                            TypeScriptTokenType::IdentifierName if name.is_empty() => {
                                name = source.get_text_in(leaf.span().into()).to_string();
                            }
                            TypeScriptTokenType::Public => visibility = Some(Visibility::Public),
                            TypeScriptTokenType::Private => visibility = Some(Visibility::Private),
                            TypeScriptTokenType::Protected => visibility = Some(Visibility::Protected),
                            TypeScriptTokenType::Static => modifiers.push(Modifier::new(ModifierKind::Static, leaf.span)),
                            TypeScriptTokenType::Readonly => modifiers.push(Modifier::new(ModifierKind::Readonly, leaf.span)),
                            TypeScriptTokenType::Question => optional_marker = Some(OptionalMarker::new(leaf.span)),
                            _ => {}
                        },
                    }
                }
                Ok(Some(ClassMember::Property { decorators, name, ty, initializer: value, visibility, modifiers, optional_marker, span: span.into() }))
            }
            TypeScriptElementType::MethodDeclaration | TypeScriptElementType::MethodSignature | TypeScriptElementType::Constructor | TypeScriptElementType::ConstructorDeclaration => {
                let mut name = if matches!(kind, TypeScriptElementType::Constructor | TypeScriptElementType::ConstructorDeclaration) { "constructor".to_string() } else { String::new() };
                let mut type_params = Vec::new();
                let mut params = Vec::new();
                let mut return_type = None;
                let mut body = Vec::new();
                let mut visibility = None;
                let mut modifiers: Vec<Modifier> = Vec::new();
                let mut optional_marker = None;
                let mut decorators = Vec::new();

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::IdentifierName => {
                                    if name.is_empty() {
                                        name = source.get_text_in(child_node.span().into()).trim().to_string();
                                    }
                                }
                                TypeScriptElementType::TypeParameter => {
                                    if !self.erase_types {
                                        if let Some(tp) = self.build_type_parameter(&child_node, source)? {
                                            type_params.push(tp);
                                        }
                                    }
                                }
                                TypeScriptElementType::Parameter => {
                                    if let Some(p) = self.build_parameter(&child_node, source)? {
                                        params.push(p);
                                    }
                                }
                                TypeScriptElementType::TypeAnnotation => {
                                    if !self.erase_types {
                                        return_type = self.build_type_annotation(&child_node, source)?;
                                    }
                                }
                                TypeScriptElementType::BlockStatement => {
                                    if let Some(Statement::BlockStatement(block)) = self.build_statement(&child_node, source)? {
                                        body = block.statements;
                                    }
                                }
                                TypeScriptElementType::Decorator => {
                                    if let Some(d) = self.build_decorator(&child_node, source)? {
                                        decorators.push(d);
                                    }
                                }
                                _ => {}
                            }
                        }
                        RedTree::Leaf(leaf) => match leaf.kind {
                            TypeScriptTokenType::IdentifierName if name.is_empty() => {
                                name = source.get_text_in(leaf.span().into()).to_string();
                            }
                            TypeScriptTokenType::Public => visibility = Some(Visibility::Public),
                            TypeScriptTokenType::Private => visibility = Some(Visibility::Private),
                            TypeScriptTokenType::Protected => visibility = Some(Visibility::Protected),
                            TypeScriptTokenType::Static => modifiers.push(Modifier::new(ModifierKind::Static, leaf.span)),
                            TypeScriptTokenType::Abstract => modifiers.push(Modifier::new(ModifierKind::Abstract, leaf.span)),
                            TypeScriptTokenType::Async => modifiers.push(Modifier::new(ModifierKind::Async, leaf.span)),
                            TypeScriptTokenType::Get => modifiers.push(Modifier::new(ModifierKind::Get, leaf.span)),
                            TypeScriptTokenType::Set => modifiers.push(Modifier::new(ModifierKind::Set, leaf.span)),
                            TypeScriptTokenType::Question => optional_marker = Some(OptionalMarker::new(leaf.span)),
                            _ => {}
                        },
                    }
                }

                Ok(Some(ClassMember::Method { decorators, name, type_params, params, return_type, body, visibility, modifiers, optional_marker, span: span.into() }))
            }
            _ => Ok(None),
        }
    }
}
