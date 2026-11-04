use oak_core::{Builder, Lexer, OakError, Source, parser::ParseSession, source::SourceText};
use oak_testing::{building::BuilderTester, source_from_path};
use oak_typst::{
    TypstBuilder, TypstLanguage,
    ast::{TypstItem, TypstRoot},
    lexer::{TypstLexer, token_type::TypstTokenType},
};
use std::{path::Path, time::Duration};

fn fixture_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/builder")
}

/// 递归收集一个 AST 子树中的文本内容，用于断言结构边界。
fn collect_text(items: &[TypstItem]) -> String {
    let mut text = String::new();
    for item in items {
        match item {
            TypstItem::Text(content) | TypstItem::Raw(content) => text.push_str(content),
            TypstItem::Space | TypstItem::Parbreak => {}
            TypstItem::Heading(heading) => text.push_str(&collect_text(&heading.content.items)),
            TypstItem::Strong(root) | TypstItem::Emphasis(root) | TypstItem::Math(root) | TypstItem::Quote(root) | TypstItem::ListItem(root) | TypstItem::EnumItem(root) | TypstItem::Block(root) => text.push_str(&collect_text(&root.items)),
            TypstItem::Link(link) => {
                if let Some(content) = &link.content {
                    text.push_str(&collect_text(&content.items))
                }
            }
        }
    }
    text
}

#[test]
fn test_typst_builder_fixture() -> Result<(), OakError> {
    let language = TypstLanguage::default();
    let builder = TypstBuilder::new(&language);
    let tester = BuilderTester::new(fixture_dir()).with_extension("typ").with_timeout(Duration::from_secs(5));
    tester.run_tests(&builder)
}

#[test]
fn test_typst_markup_line_boundaries() -> Result<(), OakError> {
    let language = TypstLanguage::default();
    let builder = TypstBuilder::new(&language);
    let source = source_from_path(&fixture_dir().join("basic.typ"))?;
    let mut cache = ParseSession::<TypstLanguage>::new(1024);
    let diagnostics = builder.build(&source, &[], &mut cache);
    let root = diagnostics.result.expect("fixture 应当构建成功");

    // 标题必须在行尾结束，不能吞掉后续正文与列表。
    let headings: Vec<_> = root
        .items
        .iter()
        .filter_map(|item| match item {
            TypstItem::Heading(heading) => Some(heading),
            _ => None,
        })
        .collect();
    assert_eq!(headings.len(), 1, "fixture 应只产生一个 Heading");
    assert_eq!(headings[0].level, 1, "`= 标题` 应为一级标题");
    assert_eq!(collect_text(&headings[0].content.items), "标题", "Heading 内容不应包含后续行");

    // 根层正文文本独立于标题存在。
    let body_text: String = root
        .items
        .iter()
        .filter_map(|item| match item {
            TypstItem::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert!(body_text.contains("标题之后的正文。"), "标题后的正文应成为根层文本，实际为：{body_text}");
    assert!(body_text.contains("末尾段落文本。"), "末尾段落文本应成为根层文本，实际为：{body_text}");
    assert!(!body_text.contains("列表项"), "列表项不应作为根层纯文本出现");

    // 两个 `-` 项与一个 `+` 项各自成为独立的 ListItem，行尾即结束。
    let list_items: Vec<&TypstRoot> = root
        .items
        .iter()
        .filter_map(|item| match item {
            TypstItem::ListItem(item) => Some(item),
            _ => None,
        })
        .collect();
    let list_texts: Vec<String> = list_items.iter().map(|item| collect_text(&item.items)).collect();
    assert_eq!(list_texts, vec!["-列表项一", "-列表项二", "+加号列表项"], "每个列表项应只包含自身内容");

    // EnumItem 同样在行尾结束。
    let enum_items: Vec<&TypstRoot> = root
        .items
        .iter()
        .filter_map(|item| match item {
            TypstItem::EnumItem(item) => Some(item),
            _ => None,
        })
        .collect();
    let enum_texts: Vec<String> = enum_items.iter().map(|item| collect_text(&item.items)).collect();
    assert_eq!(enum_texts, vec!["1.枚举项一"], "枚举项应只包含自身内容");

    Ok(())
}

/// 校验转义文本条目与其原文字节区间。
///
/// `expected` 是词法层的 `(token 类型, 起始字节, 结束字节)`，`expected_texts` 是
/// AST 中按顺序出现的非空文本条目。token 必须逐字节连续覆盖原文，因为 `GreenTree`
/// 用 token 长度累加还原 span，任何缺口都会让后续文本坐标左移。
fn assert_escape_case(source_text: &str, expected: &[(TypstTokenType, usize, usize)], expected_texts: &[&str]) {
    let language = TypstLanguage::default();
    let source = SourceText::new(source_text);

    let lexer = TypstLexer::new(&language);
    let mut cache = ParseSession::<TypstLanguage>::new(1024);
    let lexed = lexer.lex(&source, &[], &mut cache);
    let tokens = lexed.result.expect("lexing 应当成功");
    let actual: Vec<_> = tokens.iter().filter(|token| token.kind != TypstTokenType::Eof).collect();

    assert_eq!(actual.len(), expected.len(), "token 数量不符：{source_text:?}");
    let mut cursor = 0;
    for (token, (kind, start, end)) in actual.iter().zip(expected) {
        assert_eq!(token.kind, *kind, "token 类型不符：{source_text:?}");
        assert_eq!((token.span.start, token.span.end), (*start, *end), "token 坐标不符：{source_text:?}");
        assert_eq!(token.span.start, cursor, "token 之间存在未覆盖字节：{source_text:?}");
        cursor = token.span.end;
    }
    assert_eq!(cursor, source.length(), "token 覆盖未到达源末尾：{source_text:?}");

    let builder = TypstBuilder::new(&language);
    let mut cache = ParseSession::<TypstLanguage>::new(1024);
    let built = builder.build(&source, &[], &mut cache);
    let root = built.result.expect("build 应当成功");
    let texts: Vec<String> = root
        .items
        .iter()
        .filter_map(|item| match item {
            TypstItem::Text(text) if !text.is_empty() => Some(text.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(texts, expected_texts, "文本条目不符：{source_text:?}");
}

#[test]
fn test_typst_escape_single_char() {
    // `\*` 只产生字面 `*`，Escape token 覆盖 0..2。
    assert_escape_case("\\*", &[(TypstTokenType::Escape, 0, 2)], &["*"]);
}

#[test]
fn test_typst_escape_then_ascii() {
    // 转义后的 ASCII 文本坐标必须从第 2 字节开始，不能继承反斜杠造成的错位。
    assert_escape_case("\\*abc", &[(TypstTokenType::Escape, 0, 2), (TypstTokenType::Identifier, 2, 5)], &["*", "abc"]);
}

#[test]
fn test_typst_escape_then_cjk() {
    // 多字节中文从第 2 字节开始，转义不能让 span 落到字符内部。
    assert_escape_case("\\*中文", &[(TypstTokenType::Escape, 0, 2), (TypstTokenType::Text, 2, 8)], &["*", "中文"]);
}

#[test]
fn test_typst_escape_consecutive() {
    // 连续转义逐个产出字面字符，其后的 ASCII 仍落在正确的字节区间。
    assert_escape_case("\\*\\*\\*x", &[(TypstTokenType::Escape, 0, 2), (TypstTokenType::Escape, 2, 4), (TypstTokenType::Escape, 4, 6), (TypstTokenType::Identifier, 6, 7)], &["*", "*", "*", "x"]);
}
