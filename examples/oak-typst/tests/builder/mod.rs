use oak_core::{Builder, OakError, parser::ParseSession};
use oak_testing::{building::BuilderTester, source_from_path};
use oak_typst::{
    TypstBuilder, TypstLanguage,
    ast::{TypstItem, TypstRoot},
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
