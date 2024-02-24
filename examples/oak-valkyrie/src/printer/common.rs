//! 共享打印辅助。

use crate::ast::{Identifier, NamePath};
use crate::lexer::token_type::ValkyrieTokenType;

pub(crate) fn print_identifier(id: &Identifier) -> String {
    id.name.clone()
}

pub(crate) fn print_name_path(path: &NamePath) -> String {
    path.parts.iter().map(|part| part.name.as_str()).collect::<Vec<_>>().join("::")
}

pub(crate) fn print_operator(operator: ValkyrieTokenType) -> Option<&'static str> {
    match operator {
        ValkyrieTokenType::Plus => Some("+"),
        ValkyrieTokenType::Minus => Some("-"),
        ValkyrieTokenType::Star => Some("*"),
        ValkyrieTokenType::Slash => Some("/"),
        ValkyrieTokenType::Percent => Some("%"),
        ValkyrieTokenType::EqEq => Some("=="),
        ValkyrieTokenType::NotEq => Some("!="),
        ValkyrieTokenType::LessThan => Some("<"),
        ValkyrieTokenType::GreaterThan => Some(">"),
        ValkyrieTokenType::LessEq => Some("<="),
        ValkyrieTokenType::GreaterEq => Some(">="),
        ValkyrieTokenType::AndAnd => Some("&&"),
        ValkyrieTokenType::OrOr => Some("||"),
        ValkyrieTokenType::Eq => Some("="),
        _ => None,
    }
}
