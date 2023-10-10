use crate::{
    ast::*, builder::TypeScriptBuilder, language::TypeScriptLanguage, lexer::token_type::TypeScriptTokenType,
    parser::element_type::TypeScriptElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source, SourceText};

impl<'config> TypeScriptBuilder<'config> {
    pub(crate) fn build_class_member(
        &self,
        node: &RedNode<TypeScriptLanguage>,
        source: &SourceText,
    ) -> Result<Option<ClassMember>, OakError> {
        let kind = node.green.kind;
        let span = node.span();

        match kind {
            TypeScriptElementType::PropertyDeclaration | TypeScriptElementType::PropertySignature => {
                let mut name = String::new();
                let mut ty = None;
                let mut value = None;
                let mut visibility = None;
                let mut is_static = false;
                let mut is_readonly = false;
                let mut is_optional = false;
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
                            TypeScriptTokenType::Static => is_static = true,
                            TypeScriptTokenType::Readonly => is_readonly = true,
                            TypeScriptTokenType::Question => is_optional = true,
                            _ => {}
                        },
                    }
                }
                Ok(Some(ClassMember::Property {
                    decorators,
                    name,
                    ty,
                    initializer: value,
                    visibility,
                    is_static,
                    is_readonly,
                    is_optional,
                    span: span.into(),
                    is_abstract: false,
                }))
            }
            TypeScriptElementType::MethodDeclaration
            | TypeScriptElementType::MethodSignature
            | TypeScriptElementType::Constructor
            | TypeScriptElementType::ConstructorDeclaration => {
                let mut name = if matches!(
                    kind,
                    TypeScriptElementType::Constructor | TypeScriptElementType::ConstructorDeclaration
                ) {
                    "constructor".to_string()
                }
                else {
                    String::new()
                };
                let mut type_params = Vec::new();
                let mut params = Vec::new();
                let mut return_type = None;
                let mut body = Vec::new();
                let mut visibility = None;
                let mut is_static = false;
                let mut is_abstract = false;
                let mut is_async = false;
                let mut is_getter = false;
                let mut is_setter = false;
                let mut is_optional = false;
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
                                    if let Some(Statement::BlockStatement(block)) =
                                        self.build_statement(&child_node, source)?
                                    {
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
                            TypeScriptTokenType::Static => is_static = true,
                            TypeScriptTokenType::Abstract => is_abstract = true,
                            TypeScriptTokenType::Async => is_async = true,
                            TypeScriptTokenType::Get => is_getter = true,
                            TypeScriptTokenType::Set => is_setter = true,
                            TypeScriptTokenType::Question => is_optional = true,
                            _ => {}
                        },
                    }
                }

                Ok(Some(ClassMember::Method {
                    decorators,
                    name,
                    type_params,
                    params,
                    return_type,
                    body,
                    visibility,
                    is_static,
                    is_abstract,
                    is_async,
                    is_getter,
                    is_setter,
                    is_optional,
                    span: span.into(),
                }))
            }
            _ => Ok(None),
        }
    }
}
