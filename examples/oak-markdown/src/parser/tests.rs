use crate::{ast::Block, builder::MarkdownBuilder, language::MarkdownLanguage};
use oak_core::{Builder, parser::ParseSession};

fn build_blocks(source: &str) -> Vec<Block> {
    let language = MarkdownLanguage::github_flavored();
    let builder = MarkdownBuilder::new(&language);
    let mut session = ParseSession::<MarkdownLanguage>::default();
    let output = builder.build(source, &[], &mut session);
    output.result.expect("building should succeed").blocks
}

fn contains_heading(blocks: &[Block]) -> bool {
    blocks.iter().any(|block| match block {
        Block::Heading(_) => true,
        Block::Blockquote(quote) => contains_heading(&quote.content),
        Block::List(list) => list.items.iter().any(|item| contains_heading(&item.content)),
        _ => false,
    })
}

/// The `#` line of the reported source stays fenced code and never becomes a heading block.
#[test]
fn fenced_hash_line_stays_code_content() {
    let source = "```markdown\n# Code block with Markdown\nThis is a **code block** that contains Markdown syntax.\n```\n";
    let blocks = build_blocks(source);

    assert!(!contains_heading(&blocks), "fence content must not build a heading: {blocks:#?}");

    assert!(matches!(&blocks[0], Block::CodeBlock(_)), "the fenced block must build a `CodeBlock`: {blocks:#?}");
}

/// `> ### Heading in Blockquote` keeps a level-3 heading inside the blockquote.
#[test]
fn blockquote_heading_keeps_its_level_and_span() {
    let source = "> ### Heading in Blockquote\n";
    let blocks = build_blocks(source);

    let Block::Blockquote(quote) = &blocks[0]
    else {
        panic!("`> ### ...` must build a `Blockquote`: {blocks:#?}")
    };
    let heading = quote
        .content
        .iter()
        .find_map(|block| match block {
            Block::Heading(heading) => Some(heading),
            _ => None,
        })
        .expect("the blockquote must contain a heading block");

    assert_eq!(heading.level, 3, "the heading keeps the `###` level");
    assert_eq!(heading.span.start, 2, "the heading starts at the `###` marker after `> `");
}
