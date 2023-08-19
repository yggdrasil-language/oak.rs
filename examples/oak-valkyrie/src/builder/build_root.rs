use crate::{
    ValkyrieLanguage,
    ast::{Attribute, Identifier, ShaderDeclaration, StatementNode, ValkyrieRoot},
    builder::ValkyrieBuilder,
    lexer::token_type::ValkyrieTokenType,
    parser::element_type::ValkyrieElementType,
};
use oak_core::{GreenNode, OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    /// Builds a strongly-typed AST from a green tree.
    pub fn build_root<S: Source + ?Sized>(&self, green_tree: &GreenNode<ValkyrieLanguage>, source: &S) -> Result<ValkyrieRoot, OakError> {
        let red_root = RedNode::<ValkyrieLanguage>::new(green_tree, 0);
        let mut items = Vec::new();
        for child in red_root.children() {
            match child {
                RedTree::Node(n) => {
                    // 仅含 Eof 的空 ExprStatement 不进 AST。
                    if n.green.kind == ValkyrieElementType::ExprStatement && is_eof_only_expr_stmt(&n) {
                        continue;
                    }
                    match self.build_item(n, source) {
                        Ok(item) => items.push(item),
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Eof => continue,
                    _ => {
                        return Err(source.syntax_error(format!("Unexpected token in root: {:?}", t.kind), t.span.start));
                    }
                },
            }
        }
        Ok(ValkyrieRoot { items })
    }

    pub(crate) fn build_shader<S: Source + ?Sized>(&self, n: RedNode<ValkyrieLanguage>, source: &S) -> Result<ShaderDeclaration, OakError> {
        let mut name = Identifier::default();
        let mut kind = Identifier::default();
        let mut items = Vec::new();
        let mut annotations = Vec::new();
        let mut pending_annotations: Vec<Attribute> = Vec::new();
        let span = n.span();

        for child in n.children() {
            match child {
                RedTree::Node(child_node) => match child_node.green.kind {
                    ValkyrieElementType::Attribute => {
                        let attr = self.build_attribute(child_node, source)?;
                        pending_annotations.push(attr);
                    }
                    ValkyrieElementType::NamePath => {
                        let name_path = self.build_name_path(child_node, source)?;
                        if name.name.is_empty() {
                            name = name_path.parts.first().unwrap_or(&Identifier::default()).clone();
                        }
                        else if kind.name.is_empty() {
                            kind = name_path.parts.first().unwrap_or(&Identifier::default()).clone();
                        }
                    }
                    _ => {
                        let mut item = self.build_item(child_node, source)?;
                        let pending = std::mem::take(&mut pending_annotations);
                        let item_annotations = Self::take_annotations_for_item(&mut item, pending);
                        if !item_annotations.is_empty() && items.is_empty() {
                            annotations.extend(item_annotations);
                        }
                        items.push(item);
                    }
                },
                _ => {}
            }
        }

        Ok(ShaderDeclaration { name, kind, items, annotations, span })
    }

    fn take_annotations_for_item(item: &mut StatementNode, annotations: Vec<Attribute>) -> Vec<Attribute> {
        if annotations.is_empty() {
            return Vec::new();
        }
        match item {
            StatementNode::Micro(micro) => {
                micro.annotations = annotations;
                Vec::new()
            }
            StatementNode::Let(let_stmt) => {
                let_stmt.annotations = annotations;
                Vec::new()
            }
            StatementNode::Namespace(ns) => {
                ns.annotations = annotations;
                Vec::new()
            }
            StatementNode::Class(class) => {
                class.annotations = annotations;
                Vec::new()
            }
            StatementNode::Structure(structure) => {
                structure.annotations = annotations;
                Vec::new()
            }
            _ => annotations,
        }
    }

    pub(crate) fn build_item<S: Source + ?Sized>(&self, n: RedNode<ValkyrieLanguage>, source: &S) -> Result<StatementNode, OakError> {
        match n.green.kind {
            ValkyrieElementType::Namespace => {
                let ns = self.build_namespace(n, source)?;
                Ok(StatementNode::Namespace(Box::new(ns)))
            }
            ValkyrieElementType::Class => {
                let class = self.build_class(n, source)?;
                Ok(StatementNode::Class(Box::new(class)))
            }
            ValkyrieElementType::Struct => {
                let structure = self.build_struct(n, source)?;
                Ok(StatementNode::Structure(Box::new(structure)))
            }
            ValkyrieElementType::Flags => {
                let flags = self.build_flags(n, source)?;
                Ok(StatementNode::Flags(Box::new(flags)))
            }
            ValkyrieElementType::Enums => {
                let enums = self.build_enums(n, source)?;
                Ok(StatementNode::Enums(Box::new(enums)))
            }
            ValkyrieElementType::Trait => {
                let trait_node = self.build_trait(n, source)?;
                Ok(StatementNode::Trait(Box::new(trait_node)))
            }
            ValkyrieElementType::Widget => {
                let widget = self.build_widget(n, source)?;
                Ok(StatementNode::Widget(Box::new(widget)))
            }
            ValkyrieElementType::Singleton => {
                let singleton = self.build_singleton(n, source)?;
                Ok(StatementNode::Singleton(Box::new(singleton)))
            }
            ValkyrieElementType::UsingStatement => {
                let us = self.build_using(n, source)?;
                Ok(StatementNode::Using(Box::new(us)))
            }
            ValkyrieElementType::Micro => {
                let micro = self.build_micro(n, source)?;
                Ok(StatementNode::Micro(Box::new(micro)))
            }
            ValkyrieElementType::Mezzo => {
                let mezzo = self.build_mezzo(n, source)?;
                Ok(StatementNode::TypeFunction(Box::new(mezzo)))
            }
            ValkyrieElementType::LetStatement => {
                let stmt = self.build_let(n, source)?;
                Ok(StatementNode::Let(Box::new(stmt)))
            }
            ValkyrieElementType::ExprStatement => {
                let stmt = self.build_expr_stmt(n, source)?;
                Ok(StatementNode::ExprStmt(Box::new(stmt)))
            }
            ValkyrieElementType::Variant => {
                let variant = self.build_variant_decl(n, source)?;
                Ok(StatementNode::Variant(Box::new(variant)))
            }
            ValkyrieElementType::EffectDefinition => {
                let effect = self.build_effect(n, source)?;
                Ok(StatementNode::Effect(Box::new(effect)))
            }
            ValkyrieElementType::Shader => {
                let shader = self.build_shader(n, source)?;
                Ok(StatementNode::Shader(Box::new(shader)))
            }
            _ => Err(source.syntax_error(format!("Unexpected item: {:?}", n.green.kind), n.span().start)),
        }
    }
}

fn is_eof_only_expr_stmt(node: &RedNode<ValkyrieLanguage>) -> bool {
    let mut saw_eof = false;
    for child in node.children() {
        match child {
            RedTree::Leaf(t) => match t.kind {
                ValkyrieTokenType::Whitespace
                | ValkyrieTokenType::Newline
                | ValkyrieTokenType::LineComment
                | ValkyrieTokenType::BlockComment => {}
                ValkyrieTokenType::Eof => saw_eof = true,
                _ => return false,
            },
            RedTree::Node(_) => return false,
        }
    }
    saw_eof
}
