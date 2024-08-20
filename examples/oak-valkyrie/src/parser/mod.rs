pub use element_type::ValkyrieElementType;

use crate::{
    ValkyrieLanguage, ValkyrieLexer,
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::{parse_items::parse_item, parse_modifiers::is_declaration_keyword},
};
use oak_core::{
    OakError, Parser, Source, TextEdit,
    parser::{ParseCache, ParserState},
};

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
                if state.at(ValkyrieTokenType::Eof) {
                    break;
                }
                let before = state.checkpoint();
                if let Err(error) = parse_item(state) {
                    recover_failed_item(state, before, error);
                }
                // 容错解析若未前进必须停止，否则会无限生成 ExprStatement 节点并撑爆内存。
                if state.checkpoint().0 == before.0 {
                    break;
                }
                state.skip_trivia();
            }
            let root = state.sink.finish_node(cp, ValkyrieElementType::Root);
            Ok(root)
        })
    }
}

/// 根循环 item 级恢复：记录错误、跳到下一 item 边界，并把坏 item 的已解析
/// 片段连同垃圾 token 收进一个 `Error` 节点，让后续兄弟 item 继续解析。
///
/// 只在根循环启用；namespace 等嵌套体内部的错误仍向上传播，由根循环按整个
/// item 恢复，嵌套结构不产出半成品节点。返回后由调用方做零前进检查。
fn recover_failed_item<S: Source + ?Sized>(state: &mut ParserState<ValkyrieLanguage, S>, item_start: (usize, usize), error: OakError) {
    // `expect`/`ParserState::syntax_error` 在出错时已记录同一错误；由
    // `Source::syntax_error` 直接构造的错误尚未记录，按 (offset, message) 去重后补记。
    let already_recorded = state.errors.last().is_some_and(|recorded| recorded.source_offset() == error.source_offset() && recorded.to_string() == error.to_string());
    if !already_recorded {
        state.errors.push(error);
    }
    while state.not_at_end() && !state.at(ValkyrieTokenType::Eof) && !at_item_boundary(state) {
        state.bump();
    }
    state.finish_at(item_start, ValkyrieElementType::Error);
}

/// 恢复同步点：当前位置是否能作为下一个 item 的起始边界。
fn at_item_boundary<S: Source + ?Sized>(state: &ParserState<ValkyrieLanguage, S>) -> bool {
    use ValkyrieKeywords as K;
    match state.peek_kind() {
        Some(ValkyrieTokenType::Keyword(
            K::Micro
            | K::Mezzo
            | K::Namespace
            | K::Class
            | K::Struct
            | K::Structure
            | K::Enums
            | K::Unity
            | K::Enum
            | K::Flags
            | K::Trait
            | K::Imply
            | K::Using
            | K::Type
            | K::Let
            | K::Widget
            | K::Union
            | K::Singleton
            | K::Shader
            | K::Component
            | K::System,
        ))
        | Some(ValkyrieTokenType::TemplateL)
        | Some(ValkyrieTokenType::BracketL)
        | Some(ValkyrieTokenType::At) => true,
        // 修饰前缀声明（如 `sealed class`）：修饰词本身也是 item 边界。
        Some(ValkyrieTokenType::Identifier) => state.peek_non_trivia_kind_at(1).is_some_and(|kind| is_declaration_keyword(&kind)),
        _ => false,
    }
}

impl<'config> ValkyrieParser<'config> {
    /// 创建新的 Valkyrie 解析器
    pub fn new(config: &'config ValkyrieLanguage) -> Self {
        Self { config }
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    use crate::{ValkyrieBuilder, ValkyrieRoot, ast::StatementNode};
    use oak_core::{Builder, OakDiagnostics, SourceText, parser::ParseSession};

    fn build(source: &str) -> OakDiagnostics<ValkyrieRoot> {
        let language = ValkyrieLanguage::default();
        let builder = ValkyrieBuilder::new(&language);
        let text = SourceText::new(source);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        builder.build(&text, &[], &mut session)
    }

    fn offsets(diagnostics: &[oak_core::OakError]) -> Vec<Option<usize>> {
        diagnostics.iter().map(|error| error.source_offset()).collect()
    }

    fn micro_names(root: &ValkyrieRoot) -> Vec<&str> {
        root.items
            .iter()
            .filter_map(|item| match item {
                StatementNode::Micro(micro) => Some(micro.name.name.as_str()),
                _ => None,
            })
            .collect()
    }

    /// 坏表达式：错误记入 diagnostics，坏 item 收进 Error 节点，后续兄弟 item 不受影响。
    #[test]
    fn broken_expression_recovers_sibling_item() {
        let source = "micro a() { + }\nmicro b() {}\n";
        let output = build(source);
        let root = output.result.unwrap_or_else(|error| panic!("recovery must keep the build alive, got {error:?}"));
        assert_eq!(micro_names(&root), ["b"]);
        assert_eq!(output.diagnostics.len(), 1, "diagnostics: {:?}", output.diagnostics);
        assert_eq!(offsets(&output.diagnostics), [Some(12)]);
    }

    /// 多坏 item：每个坏 item 各记一条诊断，好 item 照常进入 AST。
    #[test]
    fn multiple_broken_items_report_multiple_diagnostics() {
        let source = "micro a() { + }\nclass B {\n  micro c() { [1, 2 }\nmicro d() {}\n";
        let output = build(source);
        let root = output.result.unwrap_or_else(|error| panic!("recovery must keep the build alive, got {error:?}"));
        assert_eq!(micro_names(&root), ["d"]);
        assert_eq!(output.diagnostics.len(), 2, "diagnostics: {:?}", output.diagnostics);
        assert_eq!(offsets(&output.diagnostics), [Some(12), Some(46)]);
    }

    /// 未闭合构造 + 垃圾 token：未闭合的数组字面量在垃圾处报错并恢复，后续 item 不受影响。
    #[test]
    fn unterminated_item_with_garbage_recovers() {
        let source = "micro a() { [1, 2\n$$$\nmicro b() {}\n";
        let output = build(source);
        let root = output.result.unwrap_or_else(|error| panic!("recovery must keep the build alive, got {error:?}"));
        assert_eq!(micro_names(&root), ["b"]);
        assert_eq!(output.diagnostics.len(), 1, "diagnostics: {:?}", output.diagnostics);
        assert_eq!(offsets(&output.diagnostics), [Some(18)]);
    }

    /// fatal 语义不变：解析成功但 AST 合同不满足（type alias 缺目标）仍是致命错误。
    #[test]
    fn fatal_build_error_still_fails() {
        let output = build("type X;\n");
        assert!(output.result.is_err(), "missing type alias target must stay fatal");
        assert!(output.diagnostics.iter().any(|error| error.to_string().contains("Missing type alias target")));
    }

    /// fatal 语义不变：悬空注解仍是致命错误。
    #[test]
    fn dangling_attribute_still_fails() {
        let output = build("@foo\n");
        assert!(output.result.is_err(), "dangling attribute must stay fatal");
    }

    /// green tree 层：坏 item 以 `Error` 节点落树，兄弟 item 节点照常产出。
    #[test]
    fn green_tree_contains_error_node_and_sibling_item() {
        let language = ValkyrieLanguage::default();
        let parser = ValkyrieParser::new(&language);
        let text = SourceText::new("micro a() { + }\nmicro b() {}\n");
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        let output = parser.parse(&text, &[], &mut session);
        assert_eq!(output.diagnostics.len(), 1, "diagnostics: {:?}", output.diagnostics);
        let green = output.result.expect("recovery must produce a green tree");
        let kinds: Vec<ValkyrieElementType> = green
            .children
            .iter()
            .filter_map(|child| match child {
                oak_core::GreenTree::Node(node) => Some(node.kind),
                _ => None,
            })
            .collect();
        assert!(kinds.contains(&ValkyrieElementType::Error), "kinds: {kinds:?}");
        assert!(kinds.contains(&ValkyrieElementType::Micro), "kinds: {kinds:?}");
    }
}

/// 元素类型定义
pub mod element_type;

pub(crate) mod parse_blocks;
pub(crate) mod parse_control;
pub(crate) mod parse_expressions;
pub(crate) mod parse_items;
pub(crate) mod parse_modifiers;
pub(crate) mod parse_statements;
/// 字符串段解析器（插值和转义处理）
pub(crate) mod parse_string_segments;
pub(crate) mod parse_template;
pub(crate) mod parse_types;
pub(crate) mod template_directive;
