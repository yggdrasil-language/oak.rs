use crate::{
    ValkyrieLanguage,
    ast::{term_nodes::TermBinaryNode as Binary, *},
    builder::{ValkyrieBuilder, utils},
    lexer::token_type::ValkyrieTokenType,
};
use oak_core::{OakError, RedNode, Source};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_binary<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let (left, op, right) = utils::build_binary_expr(&node, source, |n, s| self.build_expr(*n, s), crate::lexer::token_type::ValkyrieTokenType::Plus)?;

        if op == ValkyrieTokenType::Eq && !matches!(left.as_ref(), TermExpression::NamePath(path) if path.parts.len() == 1) && !matches!(left.as_ref(), TermExpression::DotCall { .. } | TermExpression::Index { .. } | TermExpression::Offset { .. }) {
            return Err(source.syntax_error("赋值左侧必须为可写位置".into(), span.start));
        }

        Ok(TermExpression::Binary(Box::new(Binary { lhs: *left, operator: op, rhs: *right, span })))
    }
}
