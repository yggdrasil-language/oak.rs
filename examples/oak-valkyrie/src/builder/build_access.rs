use crate::{
    ValkyrieLanguage,
    ast::*,
    builder::{ValkyrieBuilder, utils},
    lexer::token_type::ValkyrieTokenType,
    parser::element_type::ValkyrieElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_field_expr<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let (receiver, field) = utils::build_field_expr(&node, source, |n, s| self.build_expr(*n, s))?;

        Ok(TermExpression::DotCall { receiver, field, span })
    }

    pub(crate) fn build_index<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let (receiver, index) = utils::build_index_expr(&node, source, |n, s| self.build_expr(*n, s), "Missing index")?;

        Ok(TermExpression::Index { receiver, index, span })
    }

    /// 构建基数索引表达式。
    ///
    /// 基数索引使用 `⁅ ⁆` 括号，表示从 0 开始的偏移量访问。
    /// 与普通索引 `[ ]`（从 1 开始）不同，基数索引更接近底层指针算术风格。
    pub(crate) fn build_offset<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let (receiver, offset) = utils::build_index_expr(&node, source, |n, s| self.build_expr(*n, s), "Missing offset")?;

        Ok(TermExpression::Offset { receiver, offset, span })
    }

    pub(crate) fn build_turbofish<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut expr = None;
        let mut arguments = Vec::new();
        for child in node.children() {
            if utils::should_skip_node(&child) {
                continue;
            }
            if let RedTree::Node(n) = child {
                match n.green.kind {
                    ValkyrieElementType::GenericArgumentList => {
                        for arg_child in n.children() {
                            if utils::should_skip_node(&arg_child) {
                                continue;
                            }
                            if let RedTree::Node(type_node) = arg_child {
                                if type_node.green.kind == ValkyrieElementType::Type {
                                    arguments.push(self.build_type(type_node, source)?);
                                }
                            }
                        }
                    }
                    _ => {
                        if expr.is_none() {
                            expr = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                }
            }
        }
        let expr = expr.ok_or_else(|| source.syntax_error("Missing turbofish expression".to_string(), span.start))?;
        Ok(TermExpression::Turbofish { expr, arguments, span })
    }

    pub(crate) fn build_paren<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let expr = utils::get_required_expr(&node, source, |n, s| self.build_expr(*n, s), "Missing parenthesized expression", span.start)?;
        Ok(TermExpression::Paren { expr, span })
    }

    pub(crate) fn build_cast<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut expr = None;
        let mut ty = None;
        for child in node.children() {
            if utils::should_skip_node(&child) {
                continue;
            }
            if let RedTree::Node(n) = child {
                match n.green.kind {
                    ValkyrieElementType::Type => ty = Some(self.build_type(n, source)?),
                    _ => {
                        if expr.is_none() {
                            expr = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                }
            }
        }
        let expr = expr.ok_or_else(|| source.syntax_error("Missing cast expression".to_string(), span.start))?;
        let ty = ty.ok_or_else(|| source.syntax_error("Missing cast target type".to_string(), span.start))?;
        Ok(TermExpression::Cast { expr, ty, span })
    }
}
