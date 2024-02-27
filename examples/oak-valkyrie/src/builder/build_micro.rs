use crate::{
    ValkyrieLanguage,
    ast::{type_nodes::AssociatedType, *},
    builder::{ValkyrieBuilder, text},
    lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::element_type::ValkyrieElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source};

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
                        name.name = text(source, t.span);
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
                            name.name = text(source, t.span);
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
                            name.name = text(source, t.span);
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
        let mut base_ident: Option<Identifier> = None;
        let mut has_double_colon = false;
        let mut associated_name: Option<Identifier> = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
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
                        if base_ident.is_none() {
                            let path = self.build_name_path(n, source)?;
                            return Ok(TypeExpression::Namepath(Box::new(path)));
                        }
                    }
                    _ => {}
                },
            }
        }

        if let (Some(base), true, Some(name)) = (base_ident.clone(), has_double_colon, associated_name) {
            Ok(TypeExpression::AssociatedType(Box::new(AssociatedType { base, name, span })))
        }
        else if let Some(base) = base_ident {
            Ok(TypeExpression::Namepath(Box::new(NamePath { parts: vec![base], span })))
        }
        else {
            Ok(TypeExpression::Namepath(Box::new(NamePath { parts: Vec::new(), span })))
        }
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
