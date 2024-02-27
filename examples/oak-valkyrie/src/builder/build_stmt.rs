use crate::{
    ValkyrieLanguage,
    ast::*,
    builder::{ValkyrieBuilder, text},
    lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::element_type::ValkyrieElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_system<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<SystemDeclaration, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut annotations = Vec::new();
        let mut methods = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier if name.name.is_empty() => {
                        name = Identifier { name: text(source, t.span), span: t.span };
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::Micro | ValkyrieElementType::Method => {
                        let mut method = self.build_function(n, source)?;
                        method.annotations.splice(0..0, std::mem::take(&mut annotations));
                        methods.push(method);
                    }
                    _ => {}
                },
            }
        }

        Ok(SystemDeclaration { annotations, name, methods, span })
    }

    pub(crate) fn build_let<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Let, OakError> {
        let span = node.span();
        let mut pattern = None;
        let mut expr = None;
        let mut ty = None;
        let mut annotations = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Let) => continue,
                    ValkyrieTokenType::Identifier => {
                        if pattern.is_none() {
                            let name = text(source, t.span);
                            pattern = Some(Pattern::Variable(Box::new(VariablePattern { name: Identifier { name, span: t.span }, span: t.span })));
                        }
                    }
                    ValkyrieTokenType::Underscore => {
                        if pattern.is_none() {
                            pattern = Some(Pattern::Wildcard(Box::new(WildcardPattern { span: t.span })));
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::Modifier => {
                        let modifier = self.build_modifier(n, source)?;
                        annotations.push(ValkyrieBuilder::modifier_to_attribute(modifier));
                    }
                    ValkyrieElementType::Attribute => {
                        annotations.push(self.build_attribute(n, source)?);
                    }
                    ValkyrieElementType::Pattern => {
                        pattern = Some(self.build_pattern(n, source)?);
                    }
                    ValkyrieElementType::Type => {
                        ty = Some(self.build_type(n, source)?);
                    }
                    _ => {
                        if expr.is_none() {
                            expr = Some(self.build_expr(n, source)?);
                        }
                    }
                },
            }
        }

        let pattern = pattern.ok_or_else(|| source.syntax_error("Missing pattern in let statement".to_string(), span.start))?;
        let expr = expr.ok_or_else(|| source.syntax_error("Missing expression in let statement".to_string(), span.start))?;

        Ok(Let { annotations, pattern, expr, ty, span })
    }

    pub(crate) fn build_expr_stmt<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<ExprStmt, OakError> {
        let span = node.span();
        let mut expr = None;
        let mut semi = false;
        let mut annotations = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Semicolon => {
                        semi = true;
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::Attribute => {
                        annotations.push(self.build_attribute(n, source)?);
                    }
                    _ => {
                        if expr.is_none() {
                            expr = Some(self.build_expr(n, source)?);
                        }
                    }
                },
            }
        }

        let expr = expr.ok_or_else(|| source.syntax_error("Missing expression in expression statement".to_string(), span.start))?;

        Ok(ExprStmt { annotations, expr, semi, span })
    }

    pub(crate) fn build_using<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<UsingDeclaration, OakError> {
        let span = node.span();
        let mut path = NamePath { parts: Vec::new(), span: Default::default() };
        let mut alias = None;
        let mut imports = Vec::new();
        let mut in_import_list = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::BraceL => {
                        in_import_list = true;
                    }
                    ValkyrieTokenType::BraceR => {
                        in_import_list = false;
                    }
                    ValkyrieTokenType::Identifier => {
                        if in_import_list {
                            imports.push(Identifier { name: text(source, t.span), span: t.span });
                        }
                        else if !path.parts.is_empty() && alias.is_none() {
                            alias = Some(Identifier { name: text(source, t.span), span: t.span });
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => {
                    if n.green.kind == ValkyrieElementType::NamePath {
                        path = self.build_name_path(n, source)?;
                    }
                }
            }
        }
        Ok(UsingDeclaration { path, alias, imports, span })
    }

    pub(crate) fn build_effect<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Effect, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut annotations = Vec::new();
        let mut operations = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => name = Identifier { name: text(source, t.span), span: t.span },
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::BlockExpression => {
                        for inner_child in n.children() {
                            if let RedTree::Node(inner_n) = inner_child {
                                if inner_n.green.kind == ValkyrieElementType::Micro {
                                    if let Ok(func) = self.build_function(inner_n, source) {
                                        operations.push(func);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(Effect { name, operations, annotations, span })
    }

    pub(crate) fn build_function<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<MethodDeclaration, OakError> {
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
                        annotations.push(Self::modifier_to_attribute(modifier));
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

        Ok(MethodDeclaration { name, generics, params, return_type, body, annotations, span })
    }

    pub(crate) fn build_modifier<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Modifier, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                if matches!(t.kind, ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment) {
                    continue;
                }
                if matches!(t.kind, ValkyrieTokenType::Identifier) {
                    name.name = text(source, t.span);
                    name.span = t.span;
                }
            }
        }
        Ok(Modifier { name, span })
    }

    pub(crate) fn modifier_to_attribute(modifier: Modifier) -> Attribute {
        Attribute { name: modifier.name, args: Vec::new(), span: modifier.span }
    }

    pub(crate) fn build_attribute<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Attribute, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut args = Vec::new();

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
                    ValkyrieTokenType::At => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::NamePath => {
                        if name.name.is_empty() {
                            let path = self.build_name_path(n, source)?;
                            name.name = path.parts.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join("::");
                            name.span = path.span;
                        }
                    }
                    ValkyrieElementType::AttributeArgument => {
                        let argument = self.build_attribute_argument(n, source)?;
                        args.push(argument);
                    }
                    _ => {
                        let value = self.build_expr(n, source)?;
                        args.push(AttributeArgument { key: None, value, span: n.span() });
                    }
                },
            }
        }

        Ok(Attribute { name, args, span })
    }

    pub(crate) fn build_attribute_argument<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<AttributeArgument, OakError> {
        let span = node.span();
        let mut key = None;
        let mut value = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier if key.is_none() && value.is_none() => {
                        key = Some(Identifier { name: text(source, t.span), span: t.span });
                    }
                    _ => {}
                },
                RedTree::Node(n) => {
                    if value.is_none() {
                        value = Some(self.build_expr(n, source)?);
                    }
                }
            }
        }

        let value = value.ok_or_else(|| source.syntax_error("Missing attribute argument value".to_string(), span.start))?;
        Ok(AttributeArgument { key, value, span })
    }
}
