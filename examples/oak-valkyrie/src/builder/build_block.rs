use crate::{
    ValkyrieLanguage,
    ast::*,
    builder::{ValkyrieBuilder, utils},
    lexer::token_type::ValkyrieTokenType,
};
use oak_core::{OakError, RedNode, Source};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_block<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Block, OakError> {
        let span = node.span();
        let statements =
            utils::build_block_expr(&node, source, |n, s| Ok(Statement::Let(self.build_let(*n, s)?)), |n, s| Ok(Statement::Expression(Box::new(self.build_expr_stmt(*n, s)?))), |n, s| self.build_template_statement(*n, s), |n, s| self.build_expr(*n, s))?;

        Ok(Block { statements, span })
    }
}
