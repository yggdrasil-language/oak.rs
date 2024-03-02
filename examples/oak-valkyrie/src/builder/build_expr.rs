use crate::{ValkyrieLanguage, ast::*, builder::ValkyrieBuilder};
use oak_core::{OakError, RedNode, Source};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_expr<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        match node.green.kind {
            crate::parser::element_type::ValkyrieElementType::LiteralExpression => self.build_literal(node, source),
            crate::parser::element_type::ValkyrieElementType::BooleanLiteral => self.build_bool_literal(node, source),
            crate::parser::element_type::ValkyrieElementType::BinaryExpression => self.build_binary(node, source),
            crate::parser::element_type::ValkyrieElementType::UnaryExpression => self.build_unary(node, source),
            crate::parser::element_type::ValkyrieElementType::CallExpression => self.build_call(node, source),
            crate::parser::element_type::ValkyrieElementType::FieldExpression => self.build_field_expr(node, source),
            crate::parser::element_type::ValkyrieElementType::TurbofishExpression => self.build_turbofish(node, source),
            crate::parser::element_type::ValkyrieElementType::IndexExpression => self.build_index(node, source),
            crate::parser::element_type::ValkyrieElementType::OffsetExpression => self.build_offset(node, source),
            crate::parser::element_type::ValkyrieElementType::ParenthesizedExpression => self.build_paren(node, source),
            crate::parser::element_type::ValkyrieElementType::CastExpression => self.build_cast(node, source),
            crate::parser::element_type::ValkyrieElementType::BlockExpression => Ok(TermExpression::Block(self.build_block(node, source)?)),
            crate::parser::element_type::ValkyrieElementType::LambdaExpression => Ok(TermExpression::Micro(self.build_lambda_expr(node, source)?)),
            crate::parser::element_type::ValkyrieElementType::ObjectExpression => self.build_object(node, source),
            crate::parser::element_type::ValkyrieElementType::IfExpression => self.build_if(node, source),
            crate::parser::element_type::ValkyrieElementType::MatchExpression => self.build_match(node, source),
            crate::parser::element_type::ValkyrieElementType::LoopExpression => self.build_loop(node, source),
            crate::parser::element_type::ValkyrieElementType::ReturnExpression => self.build_return(node, source),
            crate::parser::element_type::ValkyrieElementType::BreakExpression => self.build_break(node, source),
            crate::parser::element_type::ValkyrieElementType::ContinueExpression => self.build_continue(node, source),
            crate::parser::element_type::ValkyrieElementType::YieldExpression => self.build_yield(node, source),
            crate::parser::element_type::ValkyrieElementType::RaiseExpression => self.build_raise(node, source),
            crate::parser::element_type::ValkyrieElementType::ResumeExpression => self.build_resume(node, source),
            crate::parser::element_type::ValkyrieElementType::CatchExpression => self.build_catch(node, source),
            crate::parser::element_type::ValkyrieElementType::IdentifierExpression => self.build_identifier_expr(node, source),
            crate::parser::element_type::ValkyrieElementType::PathExpression => self.build_path_expr(node, source),
            crate::parser::element_type::ValkyrieElementType::AnonymousClass => self.build_anonymous_class(node, source),
            crate::parser::element_type::ValkyrieElementType::SuperCallExpression => self.build_super_call(node, source),
            _ => Err(source.syntax_error(format!("Unexpected expression kind: {:?}", node.green.kind), span.start)),
        }
    }
}
