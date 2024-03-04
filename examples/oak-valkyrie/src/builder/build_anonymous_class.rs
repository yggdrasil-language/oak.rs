use crate::{
    ValkyrieLanguage,
    ast::*,
    builder::{ValkyrieBuilder, text},
    lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType},
};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    /// Builds an anonymous `class { ... }` / `structure { ... }` expression.
    ///
    /// Supported shapes:
    /// - `class { ... }`
    /// - `structure { ... }`
    /// - `class: Trait { ... }`
    /// - `class(Parent)` / `class(alias: Parent)` / `class()`
    pub(crate) fn build_anonymous_class<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut is_structure = false;
        let mut annotations = Vec::new();
        let mut generics = Vec::new();
        let mut parents = Vec::new();
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut captures = Vec::new();
        let mut saw_colon = false;
        let mut parent_nodes = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Structure) => {
                        is_structure = true;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Class) => {
                        is_structure = false;
                    }
                    ValkyrieTokenType::Colon => {
                        saw_colon = true;
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::Modifier => {
                        let modifier = self.build_modifier(n, source)?;
                        annotations.push(ValkyrieBuilder::modifier_to_attribute(modifier));
                    }
                    crate::parser::element_type::ValkyrieElementType::Attribute => {
                        annotations.push(self.build_attribute(n, source)?);
                    }
                    crate::parser::element_type::ValkyrieElementType::GenericParameterList => {
                        generics = self.build_generic_params(n, source)?;
                    }
                    crate::parser::element_type::ValkyrieElementType::NamePath => {
                        if saw_colon {
                            let path = self.build_name_path(n, source)?;
                            parents.push(Parent { alias: None, name: path, span: n.span() });
                        }
                    }
                    crate::parser::element_type::ValkyrieElementType::ParameterList | crate::parser::element_type::ValkyrieElementType::ArgList => {
                        parent_nodes.push(n);
                    }
                    crate::parser::element_type::ValkyrieElementType::Field => {
                        if let Ok(field) = self.build_field(n, source) {
                            fields.push(field);
                        }
                    }
                    crate::parser::element_type::ValkyrieElementType::Micro => {
                        if let Ok(method) = self.build_function(n, source) {
                            methods.push(method);
                        }
                    }
                    _ => {}
                },
            }
        }

        if !parent_nodes.is_empty() {
            for parent_node in parent_nodes {
                for child in parent_node.children() {
                    match child {
                        RedTree::Leaf(t) => match t.kind {
                            ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                            ValkyrieTokenType::Identifier => {
                                let name = Identifier { name: text(source, t.span), span: t.span };
                                parents.push(Parent { alias: None, name: NamePath { parts: vec![name], span: t.span }, span: t.span });
                            }
                            _ => {}
                        },
                        RedTree::Node(n) => {
                            if n.green.kind == crate::parser::element_type::ValkyrieElementType::NamePath {
                                parents.push(Parent { alias: None, name: self.build_name_path(n, source)?, span: n.span() });
                            }
                        }
                    }
                }
            }
        }

        let body = StructureBody { fields, methods, associated_types: Vec::new(), span };
        if is_structure {
            Ok(TermExpression::AnonymousStructure(Box::new(AnonymousStructure { annotations, generics, parents, body, captures, span })))
        }
        else {
            Ok(TermExpression::AnonymousClass(Box::new(AnonymousClass { annotations, generics, parents, body, captures, span })))
        }
    }

    /// Builds a super call expression for constructor chaining.
    ///
    /// Syntax: `super.initiate(args)` or `super.alias.initiate(args)`
    ///
    /// ```v
    /// class Derived(Base) {
    ///     initiate(mut self, x: i32) {
    ///         super.initiate(x)  // Call parent constructor
    ///     }
    /// }
    ///
    /// class Child(primary: ParentA, secondary: ParentB) {
    ///     initiate(mut self) {
    ///         super.primary.initiate()  // Call specific parent
    ///         super.secondary.initiate()
    ///     }
    /// }
    /// ```
    pub(crate) fn build_super_call<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut parent_alias = None;
        let mut method = None;
        let mut args = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Identifier => {
                        if method.is_none() {
                            method = Some(Identifier { name: text(source, t.span), span: t.span });
                        }
                        else if parent_alias.is_none() {
                            parent_alias = method.take();
                            method = Some(Identifier { name: text(source, t.span), span: t.span });
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::ArgList => {
                        for arg_child in n.children() {
                            if let RedTree::Node(arg_n) = arg_child {
                                if let Ok(arg) = self.build_expr(arg_n, source) {
                                    args.push(arg);
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }

        let method = method.ok_or_else(|| source.syntax_error("Missing method name in super call".to_string(), span.start))?;

        Ok(TermExpression::SuperCall { parent_alias, method, args, span })
    }
}
