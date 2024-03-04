use crate::{
    ValkyrieLanguage,
    ast::type_nodes::{AssociatedTypeEquation, TraitBound, WhereConstraint},
    builder::ValkyrieBuilder,
    lexer::token_type::ValkyrieTokenType,
    parser::element_type::ValkyrieElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    /// 只消费 CST 的约束节点，不重新扫描源码语法。
    pub(crate) fn build_where_clause<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Vec<WhereConstraint>, OakError> {
        let mut constraints = Vec::new();
        for child in node.children() {
            let RedTree::Node(n) = child
            else {
                continue;
            };
            if n.green.kind != ValkyrieElementType::WhereConstraint {
                continue;
            }
            let span = n.span();
            let mut target = None;
            let mut bounds = Vec::new();
            for child in n.children() {
                let RedTree::Node(n) = child
                else {
                    continue;
                };
                match n.green.kind {
                    ValkyrieElementType::Type => target = Some(self.build_type(n, source)?),
                    ValkyrieElementType::TraitBound => bounds.push(self.build_trait_bound(n, source)?),
                    _ => {}
                }
            }
            let target = target.ok_or_else(|| source.syntax_error("where 约束缺少目标类型".to_string(), span.start))?;
            if bounds.is_empty() {
                return Err(source.syntax_error("where 约束缺少 bound".to_string(), span.start));
            }
            constraints.push(WhereConstraint { target, bounds, span });
        }
        Ok(constraints)
    }

    fn build_trait_bound<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TraitBound, OakError> {
        let span = node.span();
        let mut trait_path = None;
        let mut type_arguments = Vec::new();
        let mut associated_types = Vec::new();
        for child in node.children() {
            let RedTree::Node(n) = child
            else {
                continue;
            };
            match n.green.kind {
                ValkyrieElementType::NamePath => trait_path = Some(self.build_name_path(n, source)?),
                ValkyrieElementType::Type => type_arguments.push(self.build_type(n, source)?),
                ValkyrieElementType::AssociatedTypeEquation => {
                    let equation_span = n.span();
                    let mut name = None;
                    let mut ty = None;
                    for child in n.children() {
                        match child {
                            RedTree::Leaf(t) if t.kind == ValkyrieTokenType::Identifier => name = Some(crate::ast::Identifier { name: crate::builder::identifier_text(source, t.span), span: t.span }),
                            RedTree::Node(n) if n.green.kind == ValkyrieElementType::Type => ty = Some(self.build_type(n, source)?),
                            _ => {}
                        }
                    }
                    associated_types.push(AssociatedTypeEquation {
                        name: name.ok_or_else(|| source.syntax_error("关联等式缺少成员名".to_string(), equation_span.start))?,
                        ty: ty.ok_or_else(|| source.syntax_error("关联等式缺少类型".to_string(), equation_span.start))?,
                    });
                }
                _ => {}
            }
        }
        Ok(TraitBound { trait_path: trait_path.ok_or_else(|| source.syntax_error("bound 缺少 trait 路径".to_string(), span.start))?, type_arguments, associated_types })
    }
}
