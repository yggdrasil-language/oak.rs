use crate::{MatlabLanguage, lexer::token_type::MatlabTokenType, parser::element_type::MatlabElementType};
use oak_core::{RedTree, TokenType};

/// Skip whitespace / comments.
pub(crate) fn is_trivia(node: &RedTree<'_, MatlabLanguage>) -> bool {
    match node {
        RedTree::Leaf(t) => t.kind().is_ignored(),
        RedTree::Node(_) => false,
    }
}

pub(crate) fn is_expr_kind(kind: MatlabElementType) -> bool {
    matches!(
        kind,
        MatlabElementType::Expression
            | MatlabElementType::Symbol
            | MatlabElementType::Literal
            | MatlabElementType::Array
            | MatlabElementType::Call
            | MatlabElementType::BinaryExpr
            | MatlabElementType::PrefixExpr
            | MatlabElementType::PostfixExpr
            | MatlabElementType::AnonymousFunction
            | MatlabElementType::FunctionHandle
    )
}
