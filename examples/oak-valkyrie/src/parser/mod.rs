pub use element_type::ValkyrieElementType;

use crate::{ValkyrieLanguage, ValkyrieLexer, parser::parse_items::parse_item};
use oak_core::{Parser, Source, TextEdit, parser::ParseCache};

/// Valkyrie 语言解析器
///
/// 将 Valkyrie 源码解析为结构化的 green tree 节点。
/// 支持完整的 Valkyrie 语言特性：micro/mezzo 函数、namespace、class、struct、
/// enums、flags、trait、表达式、控制流等。
pub struct ValkyrieParser<'config> {
    config: &'config ValkyrieLanguage,
}

impl<'config> Parser<ValkyrieLanguage> for ValkyrieParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<ValkyrieLanguage>) -> oak_core::parser::ParseOutput<'a, ValkyrieLanguage> {
        oak_core::parser::parse_with_lexer(&ValkyrieLexer::new(self.config), text, edits, cache, |state| {
            let cp = state.sink.checkpoint();
            while state.not_at_end() {
                // Eof 是真实 token，不能再 parse_item，否则会生成空 ExprStatement。
                if state.at(crate::ValkyrieTokenType::Eof) {
                    break;
                }
                parse_item(state)?;
                state.skip_trivia();
            }
            let root = state.sink.finish_node(cp, ValkyrieElementType::Root);
            Ok(root)
        })
    }
}

impl<'config> ValkyrieParser<'config> {
    /// 创建新的 Valkyrie 解析器
    pub fn new(config: &'config ValkyrieLanguage) -> Self {
        Self { config }
    }
}

/// 元素类型定义
pub mod element_type;

pub(crate) mod parse_blocks;
pub(crate) mod parse_control;
pub(crate) mod parse_expressions;
pub(crate) mod parse_items;
pub(crate) mod parse_statements;
/// 字符串段解析器（插值和转义处理）
pub(crate) mod parse_string_segments;
pub(crate) mod parse_types;
