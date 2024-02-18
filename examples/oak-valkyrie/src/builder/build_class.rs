use crate::{
    ValkyrieLanguage,
    ast::{
        ClassDeclaration, EnumVariant, Enums, EnumsKind, FieldDeclaration, Flags, Identifier, ImplyDeclaration, Parent, SingletonDeclaration, StatementNode,
        StructureDeclaration, Trait, TypeExpression, Variant, VariantCase, WidgetDeclaration,
    },
    builder::{ValkyrieBuilder, text},
    lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::element_type::ValkyrieElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_class<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<ClassDeclaration, OakError> {
        let span = node.span();
        let mut kind = 0;
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut generics = Vec::new();
        let mut annotations = Vec::new();
        let mut parents = Vec::new();
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut is_abstract = false;
        let mut is_sealed = false;
        let mut is_final = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        let t_text = text(source, t.span);
                        name = Identifier { name: t_text, span: t.span };
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Class) => {
                        kind = 1;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Abstract) => {
                        is_abstract = true;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Sealed) => {
                        is_sealed = true;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Final) => {
                        is_final = true;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Struct) => {
                        kind = 2;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Structure) => {
                        kind = 3;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Widget) => {
                        kind = 4;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Trait) => {
                        kind = 5;
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
                    ValkyrieElementType::NamePath => {
                        let parent = Parent { alias: None, name: self.build_name_path(n, source)?, span: n.span() };
                        parents.push(parent);
                    }
                    ValkyrieElementType::Type => {
                        for child in n.children() {
                            if let RedTree::Node(inner) = child {
                                if inner.green.kind == ValkyrieElementType::NamePath {
                                    let parent = Parent { alias: None, name: self.build_name_path(inner, source)?, span: inner.span() };
                                    parents.push(parent);
                                }
                            }
                        }
                    }
                    ValkyrieElementType::Field => {
                        if let Ok(field) = self.build_field(n, source) {
                            fields.push(field);
                        }
                    }
                    ValkyrieElementType::Micro => {
                        if let Ok(method) = self.build_function(n, source) {
                            methods.push(method);
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(ClassDeclaration { name, generics, annotations, parents, fields, methods, span })
    }

    pub(crate) fn build_flags<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Flags, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut annotations = Vec::new();
        let mut variants = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        let t_text = text(source, t.span);
                        name = Identifier { name: t_text, span: t.span };
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::Variant => {
                        let variant = self.build_enum_variant(n, source)?;
                        variants.push(variant);
                    }
                    ValkyrieElementType::BlockExpression => {
                        for inner_child in n.children() {
                            if let RedTree::Node(inner_n) = inner_child {
                                if inner_n.green.kind == ValkyrieElementType::Variant {
                                    if let Ok(v) = self.build_enum_variant(inner_n, source) {
                                        variants.push(v);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(Flags { name, variants, annotations, span })
    }

    pub(crate) fn build_enums<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Enums, OakError> {
        let span = node.span();
        let mut kind = EnumsKind::default();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut generics = Vec::new();
        let mut annotations = Vec::new();
        let mut variants = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        let t_text = text(source, t.span);
                        name = Identifier { name: t_text, span: t.span }
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Enums) => {
                        kind = EnumsKind::Enums;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Enum) => {
                        kind = EnumsKind::Enum;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Unity) => {
                        kind = EnumsKind::Unity;
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::GenericParameterList => {
                        generics = self.build_generic_params(n, source)?;
                    }
                    ValkyrieElementType::Variant => {
                        let variant = self.build_enum_variant(n, source)?;
                        variants.push(variant)
                    }
                    ValkyrieElementType::BlockExpression => {
                        for inner_child in n.children() {
                            if let RedTree::Node(inner_n) = inner_child {
                                if inner_n.green.kind == ValkyrieElementType::Variant {
                                    if let Ok(v) = self.build_enum_variant(inner_n, source) {
                                        variants.push(v)
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(Enums { kind, name, generics, variants, annotations, span })
    }

    pub(crate) fn build_enum_variant<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<EnumVariant, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut fields = Vec::new();
        let mut annotations = Vec::new();
        let mut value = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        let t_text = text(source, t.span);
                        name = Identifier { name: t_text, span: t.span };
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::Field => {
                        if let Ok(field) = self.build_field(n, source) {
                            fields.push(field);
                        }
                    }
                    _ => {
                        if value.is_none() {
                            if let Ok(expr) = self.build_expr(n, source) {
                                value = Some(expr);
                            }
                        }
                    }
                },
            }
        }
        Ok(EnumVariant { name, fields, annotations, span, value })
    }

    pub(crate) fn build_field<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<FieldDeclaration, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut ty = None;
        let mut default = None;
        let mut annotations = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        if name.name.is_empty() {
                            name = Identifier { name: text(source, t.span), span: t.span };
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::Type => ty = Some(self.build_type(n, source)?),
                    _ => {
                        if default.is_none() {
                            default = Some(self.build_expr(n, source)?);
                        }
                    }
                },
            }
        }

        let ty = ty.ok_or_else(|| source.syntax_error("Missing type for field".to_string(), span.start))?;
        Ok(FieldDeclaration { name, ty, default, annotations, span })
    }

    pub(crate) fn build_variant_decl<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Variant, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut generics = Vec::new();
        let mut annotations = Vec::new();
        let mut cases = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        let t_text = text(source, t.span);
                        name = Identifier { name: t_text, span: t.span };
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::GenericParameterList => {
                        generics = self.build_generic_params(n, source)?;
                    }
                    ValkyrieElementType::MatchArm => {
                        if let Ok(case) = self.build_variant_case(n, source) {
                            cases.push(case);
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(Variant { name, generics, cases, annotations, span })
    }

    pub(crate) fn build_variant_case<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<VariantCase, OakError> {
        let span = node.span();
        let mut pattern = None;
        let mut body = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Whitespace | ValkyrieElementType::Newline | ValkyrieElementType::LineComment | ValkyrieElementType::BlockComment => continue,
                    ValkyrieElementType::Pattern => {
                        pattern = Some(self.build_pattern(n, source)?);
                    }
                    _ => {
                        if body.is_none() {
                            body = Some(self.build_expr(n, source)?);
                        }
                    }
                },
            }
        }

        let pattern = pattern.ok_or_else(|| source.syntax_error("Missing pattern in variant case".to_string(), span.start))?;
        let body = body.ok_or_else(|| source.syntax_error("Missing body in variant case".to_string(), span.start))?;
        Ok(VariantCase { pattern, body, span })
    }

    pub(crate) fn build_imply<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<ImplyDeclaration, OakError> {
        let span = node.span();
        let mut annotations = Vec::new();
        let mut generics = Vec::new();
        let mut target_type = TypeExpression::Namepath(Box::new(crate::ast::NamePath { parts: Vec::new(), span: Default::default() }));
        let mut trait_type = None;
        let mut methods = Vec::new();
        let mut saw_target = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::GenericParameterList => generics = self.build_generic_params(n, source)?,
                    ValkyrieElementType::Type => {
                        let ty = self.build_type(n, source)?;
                        if !saw_target {
                            target_type = ty;
                            saw_target = true;
                        }
                        else {
                            trait_type = Some(ty);
                        }
                    }
                    ValkyrieElementType::Micro => {
                        if let Ok(func) = self.build_function(n, source) {
                            methods.push(func);
                        }
                    }
                    ValkyrieElementType::Method => {
                        if let Ok(func) = self.build_function(n, source) {
                            methods.push(func);
                        }
                    }
                    ValkyrieElementType::BlockExpression => {
                        for inner_child in n.children() {
                            if let RedTree::Node(inner_n) = inner_child {
                                match inner_n.green.kind {
                                    ValkyrieElementType::Micro | ValkyrieElementType::Method => {
                                        if let Ok(func) = self.build_function(inner_n, source) {
                                            methods.push(func);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }

        Ok(ImplyDeclaration { annotations, generics, target_type, trait_type, methods, span })
    }

    pub(crate) fn build_trait<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Trait, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut generics = Vec::new();
        let mut annotations = Vec::new();
        let mut methods = Vec::new();
        let mut associated_types = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        let t_text = text(source, t.span);
                        name = Identifier { name: t_text, span: t.span };
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::GenericParameterList => {
                        generics = self.build_generic_params(n, source)?;
                    }
                    ValkyrieElementType::NamePath => {}
                    ValkyrieElementType::Type => {}
                    ValkyrieElementType::Micro => {
                        if let Ok(func) = self.build_function(n, source) {
                            methods.push(func);
                        }
                    }
                    ValkyrieElementType::BlockExpression => {
                        for inner_child in n.children() {
                            if let RedTree::Node(inner_n) = inner_child {
                                if inner_n.green.kind == ValkyrieElementType::Micro {
                                    if let Ok(func) = self.build_function(inner_n, source) {
                                        methods.push(func);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(Trait { name, generics, methods, associated_types, annotations, span })
    }

    pub(crate) fn build_widget<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<WidgetDeclaration, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut generics = Vec::new();
        let mut annotations = Vec::new();
        let mut items = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        name = Identifier { name: text(source, t.span), span: t.span };
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::GenericParameterList => {
                        generics = self.build_generic_params(n, source)?;
                    }
                    ValkyrieElementType::Namespace => {
                        let ns = self.build_namespace(n, source)?;
                        items.push(StatementNode::Namespace(Box::new(ns)))
                    }
                    ValkyrieElementType::Class => {
                        let class = self.build_class(n, source)?;
                        items.push(StatementNode::Class(Box::new(class)))
                    }
                    ValkyrieElementType::Flags => {
                        let flags = self.build_flags(n, source)?;
                        items.push(StatementNode::Flags(Box::new(flags)))
                    }
                    ValkyrieElementType::Trait => {
                        let trait_node = self.build_trait(n, source)?;
                        items.push(StatementNode::Trait(Box::new(trait_node)))
                    }
                    ValkyrieElementType::Widget => {
                        let widget = self.build_widget(n, source)?;
                        items.push(StatementNode::Widget(Box::new(widget)))
                    }
                    ValkyrieElementType::UsingStatement => {
                        let us = self.build_using(n, source)?;
                        items.push(StatementNode::Using(Box::new(us)))
                    }
                    ValkyrieElementType::Micro => {
                        let micro = self.build_micro(n, source)?;
                        items.push(StatementNode::Micro(Box::new(micro)))
                    }
                    ValkyrieElementType::LetStatement => {
                        let stmt = self.build_let(n, source)?;
                        items.push(StatementNode::Let(Box::new(stmt)))
                    }
                    ValkyrieElementType::ExprStatement => {
                        let stmt = self.build_expr_stmt(n, source)?;
                        items.push(StatementNode::ExprStmt(Box::new(stmt)))
                    }
                    _ => {}
                },
            }
        }
        Ok(WidgetDeclaration { name, generics, items, annotations, span })
    }

    pub(crate) fn build_singleton<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<SingletonDeclaration, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut generics = Vec::new();
        let mut parents = Vec::new();
        let mut annotations = Vec::new();
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        name = Identifier { name: text(source, t.span), span: t.span };
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => annotations.push(self.build_attribute(n, source)?),
                    ValkyrieElementType::GenericParameterList => {
                        generics = self.build_generic_params(n, source)?;
                    }
                    ValkyrieElementType::NamePath => {
                        let parent = Parent { alias: None, name: self.build_name_path(n, source)?, span: n.span() };
                        parents.push(parent);
                    }
                    ValkyrieElementType::Type => {
                        for child in n.children() {
                            if let RedTree::Node(inner) = child {
                                if inner.green.kind == ValkyrieElementType::NamePath {
                                    let parent = Parent { alias: None, name: self.build_name_path(inner, source)?, span: inner.span() };
                                    parents.push(parent);
                                }
                            }
                        }
                    }
                    ValkyrieElementType::Field => {
                        if let Ok(field) = self.build_field(n, source) {
                            fields.push(field);
                        }
                    }
                    ValkyrieElementType::Micro => {
                        if let Ok(method) = self.build_function(n, source) {
                            methods.push(method);
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(SingletonDeclaration { name, generics, parents, fields, methods, annotations, span })
    }

    pub(crate) fn build_struct<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<StructureDeclaration, OakError> {
        let span = node.span();
        let mut name = Identifier { name: String::new(), span: Default::default() };
        let mut generics = Vec::new();
        let mut annotations = Vec::new();
        let mut parents = Vec::new();
        let mut fields = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        if name.name.is_empty() {
                            name = Identifier { name: text(source, t.span), span: t.span };
                        }
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
                    ValkyrieElementType::NamePath => {
                        let parent = Parent { alias: None, name: self.build_name_path(n, source)?, span: n.span() };
                        parents.push(parent);
                    }
                    ValkyrieElementType::Type => {
                        for child in n.children() {
                            if let RedTree::Node(inner) = child {
                                if inner.green.kind == ValkyrieElementType::NamePath {
                                    let parent = Parent { alias: None, name: self.build_name_path(inner, source)?, span: inner.span() };
                                    parents.push(parent);
                                }
                            }
                        }
                    }
                    ValkyrieElementType::Field => {
                        if let Ok(field) = self.build_field(n, source) {
                            fields.push(field);
                        }
                    }
                    ValkyrieElementType::BlockExpression => {
                        for inner_child in n.children() {
                            if let RedTree::Node(inner_n) = inner_child {
                                if inner_n.green.kind == ValkyrieElementType::Field {
                                    if let Ok(field) = self.build_field(inner_n, source) {
                                        fields.push(field);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(StructureDeclaration { name, generics, parents, fields, annotations, span })
    }
}
