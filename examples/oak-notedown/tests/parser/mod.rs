use oak_core::parser::session::ParseSession;
use oak_core::{Parser, RedNode, RedTree, SourceText};
use oak_notedown::{NoteElementType, NoteLanguage, NoteParser};

fn block_kinds(source: &str) -> Vec<NoteElementType> {
    let language = NoteLanguage::default();
    let text = SourceText::new(source);
    let mut cache = ParseSession::<NoteLanguage>::default();
    let parser = NoteParser::new(&language);
    let parsed = parser.parse(&text, &[], &mut cache);
    let green_tree = parsed.result.expect("parse notedown");
    let root = RedNode::new(&green_tree, 0);
    root.children()
        .filter_map(|child| match child {
            RedTree::Node(node) => Some(node.element_type()),
            RedTree::Leaf(_) => None,
        })
        .collect()
}

#[test]
fn parses_heading_list_and_paragraph_blocks() {
    let kinds = block_kinds("# Title\n\n- Alpha\n- Beta\n\nBody text.\n");
    assert!(kinds.contains(&NoteElementType::Heading));
    assert!(kinds.iter().filter(|kind| *kind == &NoteElementType::ListItem).count() >= 2);
    assert!(kinds.contains(&NoteElementType::Paragraph));
}

#[test]
fn parses_inline_links() {
    let kinds = block_kinds("Visit [home](https://example.com) today.\n");
    assert!(kinds.contains(&NoteElementType::Paragraph));
    let source = SourceText::new("Visit [home](https://example.com) today.\n");
    let language = NoteLanguage::default();
    let mut cache = ParseSession::<NoteLanguage>::default();
    let parser = NoteParser::new(&language);
    let parsed = parser.parse(&source, &[], &mut cache);
    let green_tree = parsed.result.expect("parse notedown");
    let root = RedNode::new(&green_tree, 0);
    let paragraph = root
        .children()
        .find_map(|child| match child {
            RedTree::Node(node) if node.element_type() == NoteElementType::Paragraph => Some(node),
            _ => None,
        })
        .expect("paragraph");
    let has_link = paragraph.children().any(|child| match child {
        RedTree::Node(node) => node.element_type() == NoteElementType::Link,
        _ => false,
    });
    assert!(has_link, "expected Link inline node in paragraph");
}

#[test]
fn parses_fenced_code_block() {
    let kinds = block_kinds("```rust\nfn main() {}\n```\n");
    assert!(kinds.contains(&NoteElementType::CodeBlock));
}
