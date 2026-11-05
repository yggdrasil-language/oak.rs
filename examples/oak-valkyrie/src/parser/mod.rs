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

#[cfg(test)]
mod array_pattern_tests {
    use super::*;
    use crate::{
        ValkyrieBuilder, ValkyrieRoot,
        ast::{Pattern, Statement, StatementNode, TermExpression},
    };
    use oak_core::{Builder, OakDiagnostics, SourceText, parser::ParseSession};

    fn build(source: &str) -> OakDiagnostics<ValkyrieRoot> {
        let language = ValkyrieLanguage::default();
        let builder = ValkyrieBuilder::new(&language);
        let text = SourceText::new(source);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        builder.build(&text, &[], &mut session)
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

    fn single_arm_pattern(root: &ValkyrieRoot) -> &Pattern {
        let micro = root
            .items
            .iter()
            .find_map(|item| match item {
                StatementNode::Micro(micro) => Some(micro),
                _ => None,
            })
            .expect("expected a micro item");
        let statement = micro.body.statements.first().expect("expected one body statement");
        let expression = match statement {
            Statement::Expression(expr) => &expr.expr,
            other => panic!("expected expression statement, got {other:?}"),
        };
        match expression {
            TermExpression::Match { arms, .. } => &arms.first().expect("expected one match arm").pattern,
            other => panic!("expected match expression, got {other:?}"),
        }
    }

    /// 正例：`case [head, ...tail]` 解析为 prefix + rest 的数组模式。
    #[test]
    fn array_pattern_with_rest_builds_array_node() {
        let output = build("micro f() { match data { case [head, ...tail]: head } }\n");
        assert!(output.diagnostics.is_empty(), "diagnostics: {:?}", output.diagnostics);
        let root = output.result.unwrap_or_else(|error| panic!("array pattern must build, got {error:?}"));
        match single_arm_pattern(&root) {
            Pattern::Array(array) => {
                assert_eq!(array.prefix.len(), 1, "prefix: {:?}", array.prefix);
                assert!(matches!(&array.prefix[0], Pattern::Variable(variable) if variable.name.name == "head"));
                let rest = array.rest.as_ref().expect("expected a rest binding");
                assert!(matches!(rest.as_ref(), Pattern::Variable(variable) if variable.name.name == "tail"));
                assert!(array.suffix.is_empty(), "suffix: {:?}", array.suffix);
            }
            other => panic!("expected array pattern, got {other:?}"),
        }
    }

    /// 正例：`[a, b, ...rest, c]` 完整 prefix/rest/suffix 形态。
    #[test]
    fn array_pattern_prefix_rest_suffix_builds() {
        let output = build("micro f() { match data { [a, b, ...rest, c]: 0 } }\n");
        assert!(output.diagnostics.is_empty(), "diagnostics: {:?}", output.diagnostics);
        let root = output.result.unwrap_or_else(|error| panic!("array pattern must build, got {error:?}"));
        match single_arm_pattern(&root) {
            Pattern::Array(array) => {
                let names: Vec<&str> = array
                    .prefix
                    .iter()
                    .map(|pattern| match pattern {
                        Pattern::Variable(variable) => variable.name.name.as_str(),
                        other => panic!("expected variable prefix pattern, got {other:?}"),
                    })
                    .collect();
                assert_eq!(names, ["a", "b"]);
                let rest = array.rest.as_ref().expect("expected a rest binding");
                assert!(matches!(rest.as_ref(), Pattern::Variable(variable) if variable.name.name == "rest"));
                assert_eq!(array.suffix.len(), 1, "suffix: {:?}", array.suffix);
                assert!(matches!(&array.suffix[0], Pattern::Variable(variable) if variable.name.name == "c"));
            }
            other => panic!("expected array pattern, got {other:?}"),
        }
    }

    /// 反例：双 `...` 记一条诊断并由根循环恢复，兄弟 item 不受影响。
    #[test]
    fn double_rest_marker_reports_and_recovers() {
        let output = build("micro f() { match data { [...a, ...b]: 0 } }\nmicro g() {}\n");
        let root = output.result.unwrap_or_else(|error| panic!("recovery must keep the build alive, got {error:?}"));
        assert_eq!(micro_names(&root), ["g"]);
        assert_eq!(output.diagnostics.len(), 1, "diagnostics: {:?}", output.diagnostics);
        assert!(output.diagnostics[0].to_string().contains("只允许一个"), "diagnostics: {:?}", output.diagnostics);
    }

    /// 反例：未闭合 `[` 记一条诊断并由根循环恢复，兄弟 item 不受影响。
    #[test]
    fn unclosed_array_pattern_reports_and_recovers() {
        let output = build("micro f() { match data { [head, ...tail: head } }\nmicro g() {}\n");
        let root = output.result.unwrap_or_else(|error| panic!("recovery must keep the build alive, got {error:?}"));
        assert_eq!(micro_names(&root), ["g"]);
        assert_eq!(output.diagnostics.len(), 1, "diagnostics: {:?}", output.diagnostics);
        assert!(output.diagnostics[0].to_string().contains("缺少闭合方括号"), "diagnostics: {:?}", output.diagnostics);
    }

    /// 反例：`...` 后缺绑定记一条诊断并由根循环恢复。
    #[test]
    fn rest_marker_without_binding_reports_and_recovers() {
        let output = build("micro f() { match data { [a, ...]: 0 } }\nmicro g() {}\n");
        let root = output.result.unwrap_or_else(|error| panic!("recovery must keep the build alive, got {error:?}"));
        assert_eq!(micro_names(&root), ["g"]);
        assert_eq!(output.diagnostics.len(), 1, "diagnostics: {:?}", output.diagnostics);
        assert!(output.diagnostics[0].to_string().contains("必须跟绑定模式"), "diagnostics: {:?}", output.diagnostics);
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

#[cfg(test)]
mod trait_member_tests {
    use super::*;
    use crate::{
        ValkyrieBuilder, ValkyrieRoot,
        ast::{ImplyDeclaration, MethodDeclaration, StatementNode, Trait},
    };
    use oak_core::{Builder, OakDiagnostics, SourceText, parser::ParseSession};

    fn build(source: &str) -> OakDiagnostics<ValkyrieRoot> {
        let language = ValkyrieLanguage::default();
        let builder = ValkyrieBuilder::new(&language);
        let text = SourceText::new(source);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        builder.build(&text, &[], &mut session)
    }

    fn root(source: &str) -> ValkyrieRoot {
        let output = build(source);
        assert!(output.diagnostics.is_empty(), "diagnostics: {:?}", output.diagnostics);
        output.result.unwrap_or_else(|error| panic!("trait members must build, got {error:?}"))
    }

    fn trait_of<'root>(root: &'root ValkyrieRoot, name: &str) -> &'root Trait {
        root.items
            .iter()
            .find_map(|item| match item {
                StatementNode::Trait(trait_decl) if trait_decl.name.name == name => Some(trait_decl.as_ref()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("expected trait {name}"))
    }

    fn imply_of<'root>(root: &'root ValkyrieRoot) -> &'root ImplyDeclaration {
        root.items
            .iter()
            .find_map(|item| match item {
                StatementNode::Imply(imply) => Some(imply.as_ref()),
                _ => None,
            })
            .expect("expected an imply block")
    }

    fn method<'a>(trait_decl: &'a Trait, name: &str) -> &'a MethodDeclaration {
        trait_decl
            .body
            .methods
            .iter()
            .find(|method| method.name.name == name)
            .unwrap_or_else(|| panic!("expected method {name}, got {:?}", trait_decl.body.methods.iter().map(|method| method.name.name.as_str()).collect::<Vec<_>>()))
    }

    fn annotations(method: &MethodDeclaration) -> Vec<&str> {
        method.annotations.iter().map(|attribute| attribute.name.name.as_str()).collect()
    }

    /// `infix`/`suffix` 算子成员：修饰符进注解，raw id 是成员名，无体即纯声明。
    #[test]
    fn operator_member_without_body_stays_a_declaration() {
        let root = root("trait Equal {\n    infix `==`(self, other: Self): bool\n}\n");
        let equal = trait_of(&root, "Equal");
        assert_eq!(equal.body.methods.len(), 1, "`infix` must not become a method name: {:?}", equal.body.methods.iter().map(|method| method.name.name.as_str()).collect::<Vec<_>>());
        let equals = method(equal, "==");
        assert_eq!(annotations(equals), ["infix"]);
        assert_eq!(equals.parameters.len(), 2);
        assert!(equals.return_type.is_some(), "`==` must keep its return type");
        assert!(equals.body.is_none(), "bodyless operator member must stay `body: None`");
    }

    /// `micro` 成员：无体是纯声明，带体是默认体。
    #[test]
    fn micro_member_distinguishes_signature_from_default_body() {
        let root = root("trait Counter { micro value(self) -> i32 micro bump(self) -> i32 { return 1 } }\n");
        let counter = trait_of(&root, "Counter");
        let value = method(counter, "value");
        assert!(value.body.is_none(), "semicolon-less `micro` signature must stay `body: None`");
        let bump = method(counter, "bump");
        let body = bump.body.as_ref().expect("default body must reach the AST");
        assert!(!body.statements.is_empty(), "default body must keep its statements");
    }

    /// 判据主用例：普通方法成员与算子成员都能带 `{ ... }` 默认体，且体内容完整进 AST。
    #[test]
    fn trait_members_accept_default_bodies() {
        let root = root(
            "trait Comparable: Equal {\n    micro compare(self, other: Self): Ordering\n    infix `<`(self, other: Self): bool { return compare(self, other) == Ordering.Less }\n}\n",
        );
        let comparable = trait_of(&root, "Comparable");
        let names: Vec<&str> = comparable.body.methods.iter().map(|method| method.name.name.as_str()).collect();
        assert_eq!(names, ["compare", "<"], "member names must not leak modifiers or body identifiers");

        let compare = method(comparable, "compare");
        assert!(compare.body.is_none(), "`compare` is a pure signature");
        assert_eq!(compare.parameters.len(), 2);
        assert!(compare.return_type.is_some());

        let less = method(comparable, "<");
        assert_eq!(annotations(less), ["infix"]);
        assert_eq!(less.parameters.len(), 2);
        assert!(less.return_type.is_some());
        let body = less.body.as_ref().expect("`<` default body must reach the AST");
        assert!(!body.statements.is_empty(), "`<` default body must keep its statements");
    }

    /// 普通 Identifier 方法成员：带体与不带体都要正确分类。
    #[test]
    fn plain_method_member_body_is_optional() {
        let with_body = root("trait T { area(self): f64 { return 0.0 } }\n");
        assert!(method(trait_of(&with_body, "T"), "area").body.is_some(), "plain member body must reach the AST");

        let without_body = root("trait T { area(self): f64 }\n");
        assert!(method(trait_of(&without_body, "T"), "area").body.is_none(), "plain bodyless member must stay `body: None`");
    }

    /// imply 侧同样带体（算子与 `micro` 两种写法）。
    #[test]
    fn imply_members_keep_bodies() {
        let root = root(
            "imply Utf8Text: Equal {\n    infix `<`(self, other: Self): bool { return self.length() == 0 }\n    micro length(self) -> i32 { return 1 }\n}\n",
        );
        let imply = imply_of(&root);
        let names: Vec<&str> = imply.methods.iter().map(|method| method.name.name.as_str()).collect();
        assert_eq!(names, ["<", "length"]);
        let less = imply.methods.iter().find(|method| method.name.name == "<").expect("`<` implementation");
        assert_eq!(annotations(less), ["infix"]);
        assert!(!less.body.as_ref().expect("implementation body").statements.is_empty(), "implementation body must keep its statements");
        let length = imply.methods.iter().find(|method| method.name.name == "length").expect("`length` implementation");
        assert!(!length.body.as_ref().expect("implementation body").statements.is_empty(), "micro implementation body must keep its statements");
    }

    /// `&` 连接的多 supertrait：每个 bound 都是独立 `Type` 节点，按源序进 AST。
    #[test]
    fn super_traits_accept_ampersand_list() {
        let root = root("trait A {}\ntrait C {}\ntrait B: A & C {}\n");
        assert!(trait_of(&root, "A").super_traits.is_empty(), "no header bound must stay an empty list");
        let b = trait_of(&root, "B");
        assert_eq!(b.super_traits.len(), 2, "both supertraits must reach the AST: {:?}", b.super_traits);
        for (super_trait, expected) in b.super_traits.iter().zip(["A", "C"]) {
            let crate::ast::TypeExpression::Namepath(path) = super_trait
            else {
                panic!("supertrait must stay a plain name path, got {super_trait:?}");
            };
            assert_eq!(path.parts.len(), 1, "supertrait must not absorb the `&` separator: {path:?}");
            assert_eq!(path.parts[0].name, expected, "supertraits must keep source order: {:?}", b.super_traits);
        }
    }
}

#[cfg(test)]
mod spaceship_operator_tests {
    use super::*;
    use crate::{
        ValkyrieBuilder, ValkyrieRoot,
        ast::{Statement, StatementNode, TermExpression},
    };
    use oak_core::{Builder, GreenTree, OakDiagnostics, SourceText, parser::ParseSession};

    fn build(source: &str) -> OakDiagnostics<ValkyrieRoot> {
        let language = ValkyrieLanguage::default();
        let builder = ValkyrieBuilder::new(&language);
        let text = SourceText::new(source);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        builder.build(&text, &[], &mut session)
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

    /// 收集 CST 叶子 token 及其字节长度，用于验证词法切分。
    fn leaves(source: &str) -> Vec<(ValkyrieTokenType, u32)> {
        let language = ValkyrieLanguage::default();
        let parser = ValkyrieParser::new(&language);
        let text = SourceText::new(source);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        let output = parser.parse(&text, &[], &mut session);
        assert!(output.diagnostics.is_empty(), "diagnostics: {:?}", output.diagnostics);
        let green = output.result.unwrap_or_else(|error| panic!("lexing must succeed, got {error:?}"));

        fn walk(tree: &GreenTree<ValkyrieLanguage>, out: &mut Vec<(ValkyrieTokenType, u32)>) {
            match tree {
                GreenTree::Leaf(leaf) => out.push((leaf.kind, leaf.length)),
                GreenTree::Node(node) => {
                    for child in node.children {
                        walk(child, out);
                    }
                }
            }
        }

        let mut kinds = Vec::new();
        for child in green.children {
            walk(child, &mut kinds);
        }
        kinds
    }

    fn single_micro_body_expr(source: &str) -> TermExpression {
        let output = build(source);
        assert!(output.diagnostics.is_empty(), "diagnostics: {:?}", output.diagnostics);
        let root = output.result.unwrap_or_else(|error| panic!("expression must build, got {error:?}"));
        let micro = root
            .items
            .iter()
            .find_map(|item| match item {
                StatementNode::Micro(micro) => Some(micro),
                _ => None,
            })
            .expect("expected a micro item");
        let statement = micro.body.statements.first().expect("expected one body statement");
        match statement {
            Statement::Expression(expr) => expr.expr.clone(),
            other => panic!("expected expression statement, got {other:?}"),
        }
    }

    fn name_of(expr: &TermExpression) -> Option<&str> {
        match expr {
            TermExpression::NamePath(path) if path.parts.len() == 1 => Some(path.parts[0].name.as_str()),
            _ => None,
        }
    }

    fn operator_of(expr: &TermExpression) -> Option<ValkyrieTokenType> {
        match expr {
            TermExpression::Binary(node) => Some(node.operator),
            _ => None,
        }
    }

    /// 正例：`<=>` 整体成三字节 token，不被切成 `<=` + `>`。
    #[test]
    fn spaceship_lexes_as_a_single_three_byte_token() {
        let kinds = leaves("micro f() { a <=> b }\n");
        let spaceship_lengths: Vec<u32> = kinds.iter().filter(|(kind, _)| *kind == ValkyrieTokenType::LessEqGreater).map(|(_, length)| *length).collect();
        assert_eq!(spaceship_lengths, [3], "spaceship must be one three-byte token, leaves: {kinds:?}");
        assert!(!kinds.iter().any(|(kind, _)| matches!(kind, ValkyrieTokenType::LessEq | ValkyrieTokenType::GreaterThan)), "spaceship must not leave comparison remnants: {kinds:?}");
    }

    /// 反例：`a <= b > c` 仍是两个比较算子，贪婪匹配不得吞成 `<=>`。
    #[test]
    fn adjacent_comparisons_are_not_greedy() {
        let kinds = leaves("micro f() { a <= b > c }\n");
        assert!(!kinds.iter().any(|(kind, _)| *kind == ValkyrieTokenType::LessEqGreater), "`<=` followed by `>` must not merge: {kinds:?}");
        let less_eq_lengths: Vec<u32> = kinds.iter().filter(|(kind, _)| *kind == ValkyrieTokenType::LessEq).map(|(_, length)| *length).collect();
        assert_eq!(less_eq_lengths, [2], "`<=` must stay a two-byte token, leaves: {kinds:?}");
        let greater_lengths: Vec<u32> = kinds.iter().filter(|(kind, _)| *kind == ValkyrieTokenType::GreaterThan).map(|(_, length)| *length).collect();
        assert_eq!(greater_lengths, [1], "`>` must stay a one-byte token, leaves: {kinds:?}");
    }

    /// 既有最长匹配不变：`<<` 仍是单个左移 token。
    #[test]
    fn left_shift_stays_a_single_token() {
        let kinds = leaves("micro f() { a << b }\n");
        let shift_lengths: Vec<u32> = kinds.iter().filter(|(kind, _)| *kind == ValkyrieTokenType::LeftShift).map(|(_, length)| *length).collect();
        assert_eq!(shift_lengths, [2], "`<<` must stay a two-byte token, leaves: {kinds:?}");
        assert!(!kinds.iter().any(|(kind, _)| *kind == ValkyrieTokenType::LessEqGreater), "`<<` must not merge into spaceship: {kinds:?}");
    }

    /// 正例：`<=>` 与算术优先级正确 —— 算术先结合，三向比较取比较层。
    #[test]
    fn spaceship_binds_below_arithmetic() {
        let expr = single_micro_body_expr("micro f() { a <=> b + c * d }\n");
        assert_eq!(operator_of(&expr), Some(ValkyrieTokenType::LessEqGreater), "expression: {expr:?}");
        let TermExpression::Binary(node) = &expr
        else {
            panic!("expected binary expression, got {expr:?}")
        };
        assert_eq!(name_of(&node.lhs), Some("a"), "lhs: {:?}", node.lhs);
        assert_eq!(operator_of(&node.rhs), Some(ValkyrieTokenType::Plus), "rhs must be the arithmetic addition: {:?}", node.rhs);
        let TermExpression::Binary(plus) = &node.rhs
        else {
            panic!("expected addition on the rhs, got {:?}", node.rhs)
        };
        assert_eq!(name_of(&plus.lhs), Some("b"), "addition lhs: {:?}", plus.lhs);
        assert_eq!(operator_of(&plus.rhs), Some(ValkyrieTokenType::Star), "multiplication must nest under addition: {:?}", plus.rhs);
    }

    /// 同族比较左结合：`a <=> b <=> c` 解析为 `(a <=> b) <=> c`。
    #[test]
    fn spaceship_is_left_associative_within_comparisons() {
        let expr = single_micro_body_expr("micro f() { a <=> b <=> c }\n");
        assert_eq!(operator_of(&expr), Some(ValkyrieTokenType::LessEqGreater), "expression: {expr:?}");
        let TermExpression::Binary(node) = &expr
        else {
            panic!("expected binary expression, got {expr:?}")
        };
        assert_eq!(name_of(&node.rhs), Some("c"), "outer rhs must be the last operand: {:?}", node.rhs);
        assert_eq!(operator_of(&node.lhs), Some(ValkyrieTokenType::LessEqGreater), "left side must nest first: {:?}", node.lhs);
    }

    /// 反例：`a < => b` 不完整形态报结构化语法错误，兄弟 item 仍可恢复。
    #[test]
    fn incomplete_spaceship_reports_structured_error() {
        let output = build("micro f() { a < => b }\nmicro g() {}\n");
        let root = output.result.unwrap_or_else(|error| panic!("recovery must keep the build alive, got {error:?}"));
        assert_eq!(micro_names(&root), ["g"]);
        assert_eq!(output.diagnostics.len(), 1, "diagnostics: {:?}", output.diagnostics);
        assert!(output.diagnostics[0].to_string().contains("需要表达式"), "diagnostics: {:?}", output.diagnostics);
    }

    /// 格式化往返：CST token-gap 格式化保留 `<=>`，重解析仍是三向比较算子。
    #[test]
    fn formatter_roundtrip_preserves_spaceship() {
        let formatted = crate::formatter::format_source("micro f(){a<=>b}\n", &crate::formatter::FormatOptions::default()).expect("formatting must succeed");
        assert!(formatted.contains("<=>"), "formatted output must keep the operator: {formatted:?}");
        let expr = single_micro_body_expr(&formatted);
        assert_eq!(operator_of(&expr), Some(ValkyrieTokenType::LessEqGreater), "reparsed expression: {expr:?}");
    }
}

#[cfg(test)]
mod inclusive_range_and_member_attribute_tests {
    use super::*;
    use crate::{
        ValkyrieBuilder, ValkyrieRoot,
        ast::{ClassDeclaration, Statement, StatementNode, TermExpression},
    };
    use oak_core::{Builder, GreenTree, OakDiagnostics, SourceText, parser::ParseSession};

    fn build(source: &str) -> OakDiagnostics<ValkyrieRoot> {
        let language = ValkyrieLanguage::default();
        let builder = ValkyrieBuilder::new(&language);
        let text = SourceText::new(source);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        builder.build(&text, &[], &mut session)
    }

    fn build_root(source: &str) -> ValkyrieRoot {
        let output = build(source);
        assert!(output.diagnostics.is_empty(), "diagnostics: {:?}", output.diagnostics);
        output.result.unwrap_or_else(|error| panic!("source must build, got {error:?}"))
    }

    fn leaves(source: &str) -> Vec<(ValkyrieTokenType, u32)> {
        let language = ValkyrieLanguage::default();
        let parser = ValkyrieParser::new(&language);
        let text = SourceText::new(source);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        let output = parser.parse(&text, &[], &mut session);
        assert!(output.diagnostics.is_empty(), "diagnostics: {:?}", output.diagnostics);
        let green = output.result.unwrap_or_else(|error| panic!("lexing must succeed, got {error:?}"));

        fn walk(tree: &GreenTree<ValkyrieLanguage>, out: &mut Vec<(ValkyrieTokenType, u32)>) {
            match tree {
                GreenTree::Leaf(leaf) => out.push((leaf.kind, leaf.length)),
                GreenTree::Node(node) => {
                    for child in node.children {
                        walk(child, out);
                    }
                }
            }
        }

        let mut kinds = Vec::new();
        for child in green.children {
            walk(child, &mut kinds);
        }
        kinds
    }

    fn single_micro_body_expr(source: &str) -> TermExpression {
        let root = build_root(source);
        let micro = root
            .items
            .iter()
            .find_map(|item| match item {
                StatementNode::Micro(micro) => Some(micro),
                _ => None,
            })
            .expect("expected a micro item");
        let statement = micro.body.statements.first().expect("expected one body statement");
        match statement {
            Statement::Expression(expr) => expr.expr.clone(),
            other => panic!("expected expression statement, got {other:?}"),
        }
    }

    fn class_of<'root>(root: &'root ValkyrieRoot, name: &str) -> &'root ClassDeclaration {
        root.items
            .iter()
            .find_map(|item| match item {
                StatementNode::Class(class) if class.name.name == name => Some(class.as_ref()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("expected class {name}"))
    }

    /// 正例：`..=` 紧邻合并为单个三字节 token，不再拆成 `..` + `=`。
    #[test]
    fn inclusive_range_lexes_as_one_token() {
        let kinds = leaves("micro f() { 0..=5 }\n");
        let lengths: Vec<u32> = kinds.iter().filter(|(kind, _)| *kind == ValkyrieTokenType::DotDotEq).map(|(_, length)| *length).collect();
        assert_eq!(lengths, [3], "`..=` must be one three-byte token, leaves: {kinds:?}");
        assert!(!kinds.iter().any(|(kind, _)| *kind == ValkyrieTokenType::DotDot), "`..=` must not leave a `DotDot` remnant: {kinds:?}");
    }

    /// 反例：`..` 与 `=` 之间有空白时不合并，仍是 `DotDot` + `Eq`。
    #[test]
    fn spaced_inclusive_range_stays_two_tokens() {
        let kinds = leaves("micro f() { a .. b = c }\n");
        assert!(!kinds.iter().any(|(kind, _)| *kind == ValkyrieTokenType::DotDotEq), "spaced `.. =` must not merge: {kinds:?}");
        assert_eq!(kinds.iter().filter(|(kind, _)| *kind == ValkyrieTokenType::DotDot).count(), 1, "leaves: {kinds:?}");
        assert_eq!(kinds.iter().filter(|(kind, _)| *kind == ValkyrieTokenType::Eq).count(), 1, "leaves: {kinds:?}");
    }

    /// 正例：`..=` 在表达式头里产生同级 BinaryExpression，且优先级低于相等。
    #[test]
    fn inclusive_range_parses_as_binary_operator() {
        let expr = single_micro_body_expr("micro f() { a..=b }\n");
        let TermExpression::Binary(node) = &expr
        else {
            panic!("expected binary expression, got {expr:?}")
        };
        assert_eq!(node.operator, ValkyrieTokenType::DotDotEq, "operator: {expr:?}");
    }

    /// 反例：区间运算符非结合，`a..b..c` 必须报结构化语法错误而不是静默套叠。
    #[test]
    fn chained_range_reports_structured_error() {
        let output = build("micro f() { a..b..c }\n");
        assert!(!output.diagnostics.is_empty(), "chained range must fail, root: {:?}", output.result);
    }

    /// 正例：类成员前的 `[export(case: "...")]` 落在字段注解上，不并进类注解。
    #[test]
    fn class_member_bracket_attribute_attaches_to_member() {
        let root = build_root("class Sample {\n    [export(case: \"camelCase\")]\n    value: i32\n}\n");
        let class = class_of(&root, "Sample");
        assert!(class.annotations.iter().all(|attribute| attribute.name.name != "export"), "class annotations: {:?}", class.annotations);
        assert_eq!(class.body.fields.len(), 1, "fields: {:?}", class.body.fields);
        let field = &class.body.fields[0];
        assert_eq!(field.annotations.len(), 1, "field annotations: {:?}", field.annotations);
        assert_eq!(field.annotations[0].name.name, "export", "field annotations: {:?}", field.annotations);
        assert_eq!(field.annotations[0].args.len(), 1, "attribute args: {:?}", field.annotations[0].args);
        assert_eq!(field.annotations[0].args[0].key.as_ref().map(|key| key.name.as_str()), Some("case"), "attribute args: {:?}", field.annotations[0].args);
    }

    /// 正例：`@export` 成员属性同样只落到成员，不并进类注解。
    #[test]
    fn class_member_at_attribute_attaches_to_member() {
        let root = build_root("class Sample {\n    @export\n    value: i32\n}\n");
        let class = class_of(&root, "Sample");
        assert!(class.annotations.iter().all(|attribute| attribute.name.name != "export"), "class annotations: {:?}", class.annotations);
        assert_eq!(class.body.fields.len(), 1, "fields: {:?}", class.body.fields);
        assert_eq!(class.body.fields[0].annotations[0].name.name, "export", "field annotations: {:?}", class.body.fields[0].annotations);
    }
}
