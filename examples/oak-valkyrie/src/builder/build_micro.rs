use crate::{
    ValkyrieLanguage,
    ast::{type_nodes::{ApplyType, AssociatedType, FunctionType, OptionalType, TypeUnaryNode}, *},
    builder::{ValkyrieBuilder, text},
    lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::element_type::ValkyrieElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source};

fn wrap_optional_type(inner: TypeExpression, nullable: bool, span: Span) -> TypeExpression {
    if nullable {
        TypeExpression::Optional(Box::new(OptionalType { inner: Box::new(inner), span }))
    }
    else {
        inner
    }
}

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_mezzo<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TypeFunction, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut generics = Vec::new();
        let mut annotations = Vec::new();
        let mut params = Vec::new();
        let mut return_type = None;
        let mut body = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        name.name = crate::builder::identifier_text(source, t.span);
                        name.span = t.span;
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => {
                        annotations.push(self.build_attribute(n, source)?);
                    }
                    ValkyrieElementType::GenericParameterList => {
                        generics = self.build_generic_params(n, source)?;
                    }
                    ValkyrieElementType::ParameterList => {
                        params = self.build_params(n, source)?;
                    }
                    ValkyrieElementType::Type => {
                        return_type = Some(self.build_type(n, source)?);
                    }
                    ValkyrieElementType::BlockExpression => {
                        body = Some(self.build_block(n, source)?);
                    }
                    _ => {}
                },
            }
        }

        let body = body.ok_or_else(|| source.syntax_error(format!("Missing mezzo body at {:?}", span), span.start))?;

        Ok(TypeFunction { name, generics, annotations, params, return_type, body, span })
    }

    pub(crate) fn build_micro<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<MicroDeclaration, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut generics = Vec::new();
        let mut annotations = Vec::new();
        let mut params = Vec::new();
        let mut return_type = None;
        let mut body = None;
        let mut is_abstract = false;
        let mut is_final = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        if name.name.is_empty() {
                            name.name = crate::builder::identifier_text(source, t.span);
                            name.span = t.span;
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::Modifier => {
                        let modifier = self.build_modifier(n, source)?;
                        if modifier.name.name == "abstract" {
                            is_abstract = true;
                        }
                        if modifier.name.name == "final" {
                            is_final = true;
                        }
                        annotations.push(ValkyrieBuilder::modifier_to_attribute(modifier));
                    }
                    ValkyrieElementType::Attribute => {
                        annotations.push(self.build_attribute(n, source)?);
                    }
                    ValkyrieElementType::GenericParameterList => {
                        generics = self.build_generic_params(n, source)?;
                    }
                    ValkyrieElementType::ParameterList => {
                        params = self.build_params(n, source)?;
                    }
                    ValkyrieElementType::Type => {
                        return_type = Some(self.build_type(n, source)?);
                    }
                    ValkyrieElementType::BlockExpression => {
                        body = Some(self.build_block(n, source)?);
                    }
                    _ => {}
                },
            }
        }

        let body = if is_abstract { Block { statements: Vec::new(), span: span.clone() } } else { body.ok_or_else(|| source.syntax_error(format!("Missing micro body at {:?}", span), span.start))? };

        Ok(MicroDeclaration { name, generics, annotations, params, return_type, body, span, is_abstract, is_final })
    }

    pub(crate) fn build_generic_params<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Vec<GenericParam>, OakError> {
        let mut params = Vec::new();
        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::GenericParameter => params.push(self.build_generic_param(n, source)?),
                    _ => {}
                },
            }
        }
        Ok(params)
    }

    pub(crate) fn build_generic_param<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<GenericParam, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut constraints = Vec::new();
        let mut default = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        if name.name.is_empty() {
                            name.name = crate::builder::identifier_text(source, t.span);
                            name.span = t.span;
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::Type => {
                        if default.is_none() {
                            constraints.push(self.build_type(n, source)?);
                        }
                    }
                    _ => {}
                },
            }
        }

        Ok(GenericParam { name, constraints, default, span })
    }

    pub(crate) fn build_type<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TypeExpression, OakError> {
        let span = node.span();
        if node.children().any(|child| matches!(child, RedTree::Leaf(t) if t.kind == ValkyrieTokenType::BracketL)) {
            return self.build_bracket_type(node, source);
        }
        let mut base_ident: Option<Identifier> = None;
        let mut base_namepath: Option<NamePath> = None;
        let mut has_double_colon = false;
        let mut associated_name: Option<Identifier> = None;
        let mut nullable = false;
        let mut has_arrow = false;
        let mut is_function_kind = false;
        let mut nested_types = Vec::new();
        let mut generic_arguments = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Question => nullable = true,
                    ValkyrieTokenType::Arrow => has_arrow = true,
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro) | ValkyrieTokenType::Keyword(ValkyrieKeywords::Mezzo) => {
                        is_function_kind = true;
                    }
                    ValkyrieTokenType::Identifier => {
                        if base_ident.is_none() {
                            base_ident = Some(Identifier { name: text(source, t.span), span: t.span });
                        }
                        else if has_double_colon && associated_name.is_none() {
                            associated_name = Some(Identifier { name: text(source, t.span), span: t.span });
                        }
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::SelfType) => {
                        if base_ident.is_none() {
                            base_ident = Some(Identifier { name: "Self".to_string(), span: t.span });
                        }
                    }
                    ValkyrieTokenType::ColonColon => {
                        has_double_colon = true;
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::NamePath => {
                        if base_ident.is_none() && base_namepath.is_none() && !has_arrow {
                            base_namepath = Some(self.build_name_path(n, source)?);
                        }
                    }
                    ValkyrieElementType::GenericArgumentList => {
                        for arg_child in n.children() {
                            if let RedTree::Node(type_node) = arg_child {
                                if type_node.green.kind == ValkyrieElementType::Type {
                                    generic_arguments.push(self.build_type(type_node, source)?);
                                }
                            }
                        }
                    }
                    ValkyrieElementType::Type => nested_types.push(self.build_type(n, source)?),
                    _ => {}
                },
            }
        }

        if has_arrow && (is_function_kind || nested_types.len() >= 2) {
            let return_type = nested_types.pop().ok_or_else(|| source.syntax_error("Missing function return type".to_string(), span.start))?;
            let function = TypeExpression::Function(Box::new(FunctionType { params: nested_types, return_type: Box::new(return_type), span }));
            return Ok(wrap_optional_type(function, nullable, span));
        }

        if let (Some(base), true, Some(name)) = (base_ident.clone(), has_double_colon, associated_name) {
            return Ok(wrap_optional_type(TypeExpression::AssociatedType(Box::new(AssociatedType { base, name, span })), nullable, span));
        }
        if let Some(path) = base_namepath {
            let inner = if generic_arguments.is_empty() {
                TypeExpression::Namepath(Box::new(path))
            }
            else {
                TypeExpression::Apply(Box::new(ApplyType {
                    base: TypeExpression::Namepath(Box::new(path)),
                    arguments: generic_arguments,
                    span,
                }))
            };
            return Ok(wrap_optional_type(inner, nullable, span));
        }
        if let Some(base) = base_ident {
            let inner = if generic_arguments.is_empty() {
                TypeExpression::Namepath(Box::new(NamePath { parts: vec![base], span }))
            }
            else {
                TypeExpression::Apply(Box::new(ApplyType {
                    base: TypeExpression::Namepath(Box::new(NamePath { parts: vec![base], span })),
                    arguments: generic_arguments,
                    span,
                }))
            };
            return Ok(wrap_optional_type(inner, nullable, span));
        }
        Ok(wrap_optional_type(TypeExpression::Namepath(Box::new(NamePath { parts: Vec::new(), span })), nullable, span))
    }

    fn build_bracket_type<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TypeExpression, OakError> {
        let span = node.span();
        let mut element_type = None;
        let mut length = None;
        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::Type => element_type = Some(self.build_type(n, source)?),
                    ValkyrieElementType::LiteralExpression
                    | ValkyrieElementType::IdentifierExpression
                    | ValkyrieElementType::BinaryExpression
                    | ValkyrieElementType::UnaryExpression => {
                        if let Ok(expr) = self.build_expr(n, source) {
                            length = Some(expr);
                        }
                    }
                    _ => {}
                },
            }
        }
        let base = element_type.unwrap_or_else(|| TypeExpression::Namepath(Box::new(NamePath { parts: Vec::new(), span })));
        Ok(TypeExpression::Unary(Box::new(TypeUnaryNode {
            operator: ValkyrieTokenType::BracketL,
            base,
            length,
            span,
        })))
    }

    pub(crate) fn build_params<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Vec<Param>, OakError> {
        let mut params = Vec::new();
        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::Parameter => params.push(self.build_param(n, source)?),
                    _ => {}
                },
            }
        }
        Ok(params)
    }

    pub(crate) fn build_param<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Param, OakError> {
        let span = node.span();
        let mut name: Option<Identifier> = None;
        let mut ty = None;
        let mut default = None;
        let mut annotations = Vec::new();
        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        if name.is_none() {
                            name = Some(Identifier { name: text(source, t.span), span: t.span });
                        }
                    }
                    ValkyrieTokenType::Colon => continue,
                    ValkyrieTokenType::Eq => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::Modifier => {
                        let modifier = self.build_modifier(n, source)?;
                        annotations.push(ValkyrieBuilder::modifier_to_attribute(modifier));
                    }
                    ValkyrieElementType::Type => ty = Some(self.build_type(n, source)?),
                    _ => {
                        if default.is_none() {
                            default = Some(self.build_expr(n, source)?);
                        }
                    }
                },
            }
        }
        if let Some(name) = name { Ok(Param { name, ty, default, annotations, span }) } else { Err(source.syntax_error(format!("Missing name in parameter at {:?}", span), span.start)) }
    }
}
