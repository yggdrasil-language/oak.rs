use crate::{
    ValkyrieLanguage,
    ast::{NamePath, NamespaceDeclaration, Span, StatementNode, root_nodes::ProgramBody},
    builder::ValkyrieBuilder,
    lexer::token_type::ValkyrieTokenType,
    parser::element_type::ValkyrieElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_namespace<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<NamespaceDeclaration, OakError> {
        let span = node.span();
        let mut name = NamePath { parts: Vec::new(), span: Default::default() };
        let mut annotations = Vec::new();
        let mut items = Vec::new();
        let mut has_body = false;
        let mut body_span: Option<Span> = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::BraceL => {
                        has_body = true;
                        body_span = Some(t.span);
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    ValkyrieElementType::Attribute => {
                        annotations.push(self.build_attribute(n, source)?);
                    }
                    ValkyrieElementType::NamePath => {
                        name = self.build_name_path(n, source)?;
                    }
                    ValkyrieElementType::Namespace => {
                        let ns = self.build_namespace(n, source)?;
                        items.push(StatementNode::Namespace(Box::new(ns)));
                    }
                    ValkyrieElementType::Class => {
                        let class = self.build_class(n, source)?;
                        items.push(StatementNode::Class(Box::new(class)));
                    }
                    ValkyrieElementType::Flags => {
                        let flags = self.build_flags(n, source)?;
                        items.push(StatementNode::Flags(Box::new(flags)));
                    }
                    ValkyrieElementType::Enums => {
                        let enums = self.build_enums(n, source)?;
                        items.push(StatementNode::Enums(Box::new(enums)));
                    }
                    ValkyrieElementType::Trait => {
                        let trait_node = self.build_trait(n, source)?;
                        items.push(StatementNode::Trait(Box::new(trait_node)));
                    }
                    ValkyrieElementType::Widget => {
                        let widget = self.build_widget(n, source)?;
                        items.push(StatementNode::Widget(Box::new(widget)));
                    }
                    ValkyrieElementType::UsingStatement => {
                        let us = self.build_using(n, source)?;
                        items.push(StatementNode::Using(Box::new(us)));
                    }
                    ValkyrieElementType::Micro => {
                        let micro = self.build_micro(n, source)?;
                        items.push(StatementNode::Micro(Box::new(micro)));
                    }
                    ValkyrieElementType::Mezzo => {
                        let mezzo = self.build_mezzo(n, source)?;
                        items.push(StatementNode::TypeFunction(Box::new(mezzo)));
                    }
                    ValkyrieElementType::LetStatement => {
                        let stmt = self.build_let(n, source)?;
                        items.push(StatementNode::Let(Box::new(stmt)));
                    }
                    ValkyrieElementType::ExprStatement => {
                        let stmt = self.build_expr_stmt(n, source)?;
                        items.push(StatementNode::ExprStmt(Box::new(stmt)));
                    }
                    ValkyrieElementType::BlockExpression => {
                        has_body = true;
                        // Body of `namespace X { ... }`: item build errors must propagate.
                        // Swallowing them made a declaration exist in the grammar but vanish from
                        // the AST, so the missing declaration could only surface far downstream.
                        for inner_child in n.children() {
                            if let RedTree::Node(inner_n) = inner_child {
                                items.push(self.build_item(inner_n, source)?);
                            }
                        }
                    }
                    _ => items.push(self.build_item(n, source)?),
                },
            }
        }
        let body = if has_body { Some(ProgramBody { items, span: body_span.unwrap_or_else(|| span.clone()) }) } else { None };
        Ok(NamespaceDeclaration { name, annotations, body, span })
    }
}
