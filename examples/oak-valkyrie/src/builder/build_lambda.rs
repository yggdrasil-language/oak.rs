use crate::{ValkyrieLanguage, ast::*, builder::ValkyrieBuilder, lexer::token_type::ValkyrieTokenType};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    /// 构建 lambda 表达式
    pub(crate) fn build_lambda_expr<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<AnonymousMicro, OakError> {
        let span = node.span();
        let mut params = Vec::new();
        let mut return_type = None;
        let mut body = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::ParameterList => {
                        params = self.build_params(n, source)?;
                    }
                    crate::parser::element_type::ValkyrieElementType::Type => {
                        return_type = Some(self.build_type(n, source)?);
                    }
                    crate::parser::element_type::ValkyrieElementType::BlockExpression => {
                        body = Some(self.build_block(n, source)?);
                    }
                    _ => {
                        if body.is_none() {
                            // 如果不是 BlockExpression，我们需要将表达式包装在一个 Block 中
                            let expr = self.build_expr(n, source)?;
                            body = Some(Block { statements: vec![Statement::Expression(Box::new(ExprStmt { expr, semi: false, annotations: Vec::new(), span: n.span() }))], span: n.span() });
                        }
                    }
                },
            }
        }

        let body = body.ok_or_else(|| source.syntax_error("Missing lambda body".to_string(), span.start))?;

        Ok(AnonymousMicro { params, return_type, body, span })
    }
}
