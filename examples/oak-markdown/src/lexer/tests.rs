use crate::{
    language::MarkdownLanguage,
    lexer::{MarkdownLexer, token_type::MarkdownTokenType},
    parser::{MarkdownParser, element_type::MarkdownElementType},
};
use oak_core::{Parser, RedNode, RedTree, Source, SourceText, parser::ParseSession};

/// Lexes `source` and returns `(kind, text, start, end)` for every non-trivia token.
fn lex_tokens(source: &str, language: MarkdownLanguage) -> Vec<(MarkdownTokenType, String, usize, usize)> {
    let text = SourceText::new(source);
    let lexer = MarkdownLexer::new(&language);
    let output = lexer.lex_internal(&text);
    let tokens = output.result.expect("lexing should succeed");
    let mut result = Vec::new();

    for token in tokens.iter() {
        if matches!(token.kind, MarkdownTokenType::Whitespace | MarkdownTokenType::Newline) {
            continue;
        }
        let content = text.get_text_in((token.span.start..token.span.end).into()).into_owned();
        result.push((token.kind, content, token.span.start, token.span.end));
    }

    result
}

fn top_level_elements(source: &str, language: MarkdownLanguage) -> Vec<MarkdownElementType> {
    let text = SourceText::new(source);
    let parser = MarkdownParser::new(&language);
    let mut session = ParseSession::<MarkdownLanguage>::default();
    let output = parser.parse(&text, &[], &mut session);
    let tree = output.result.expect("parsing should succeed");
    let root = RedNode::new(&tree, 0);

    root.children()
        .filter_map(|child| match child {
            RedTree::Node(node) => Some(node.element_type()),
            RedTree::Leaf(_) => None,
        })
        .collect()
}

fn has_tag(tokens: &[(MarkdownTokenType, String, usize, usize)]) -> bool {
    tokens.iter().any(|(kind, ..)| matches!(kind, MarkdownTokenType::HtmlTag | MarkdownTokenType::XmlTag | MarkdownTokenType::HtmlComment | MarkdownTokenType::XmlComment))
}

/// A lone `<` must not swallow the rest of the document when a later line contains `>`.
#[test]
fn angle_bracket_stops_at_newline() {
    let source = "a < b\n\n> This is a simple blockquote.\n";
    let tokens = lex_tokens(source, MarkdownLanguage::common_mark());

    assert!(!has_tag(&tokens), "a lone `<` must not become a tag: {tokens:#?}");
    assert!(tokens.iter().any(|(kind, text, ..)| *kind == MarkdownTokenType::Less && text == "<"));
    assert!(tokens.iter().any(|(kind, text, ..)| *kind == MarkdownTokenType::BlockquoteMarker && text == ">"));
}

/// The regression reported downstream: `<` inside a fenced block followed by a `>` later
/// in the document used to consume the closing fence and every following section.
#[test]
fn fenced_code_angle_bracket_does_not_swallow_the_closing_fence() {
    let source = "```python\nif n <= 1:\n    return n\n```\n\n> quote\n";
    let tokens = lex_tokens(source, MarkdownLanguage::github_flavored());

    assert!(!has_tag(&tokens), "`<` inside a fence must not become a tag: {tokens:#?}");

    let fences: Vec<_> = tokens.iter().filter(|(kind, ..)| *kind == MarkdownTokenType::CodeFence).collect();
    assert_eq!(fences.len(), 2, "both fences must stay separate tokens: {tokens:#?}");
    assert_eq!(fences[0].1, "```");
    assert_eq!(fences[1].1, "```");
    assert!(fences[1].2 > fences[0].3);

    assert!(tokens.iter().any(|(kind, text, ..)| *kind == MarkdownTokenType::BlockquoteMarker && text == ">"));
    assert!(tokens.iter().any(|(kind, text, ..)| *kind == MarkdownTokenType::CodeLanguage && text == "python"));
}

/// A CommonMark autolink is a single token and must not be reported as raw HTML.
#[test]
fn autolink_is_not_a_raw_html_tag() {
    let source = "<https://example.com/path?q=1>\n";
    let tokens = lex_tokens(source, MarkdownLanguage::common_mark());

    assert_eq!(tokens.len(), 1, "expected exactly one autolink token: {tokens:#?}");
    assert_eq!(tokens[0].0, MarkdownTokenType::AutoLink);
    assert_eq!(tokens[0].1, "<https://example.com/path?q=1>");
    assert!(!has_tag(&tokens));
}

/// Single-line raw HTML still lexes as a tag, so the newline guard is not over-broad.
#[test]
fn single_line_html_tag_still_lexes() {
    let source = "<div class=\"note\">hi</div>\n";
    let tokens = lex_tokens(source, MarkdownLanguage::common_mark());

    let tags: Vec<_> = tokens.iter().filter(|(kind, ..)| *kind == MarkdownTokenType::HtmlTag).collect();
    assert_eq!(tags.len(), 2, "opening and closing tags must both lex: {tokens:#?}");
    assert_eq!(tags[0].1, "<div class=\"note\">");
    assert_eq!(tags[1].1, "</div>");
}

/// A candidate tag that crosses a newline is rejected even when a `>` appears on the next line.
#[test]
fn multiline_tag_candidate_is_not_a_tag() {
    let source = "<a\nhref>\n";
    let tokens = lex_tokens(source, MarkdownLanguage::common_mark());

    assert!(!has_tag(&tokens), "a tag must not span a line break: {tokens:#?}");
    assert_eq!(tokens[0].0, MarkdownTokenType::Less);
    assert_eq!(tokens[0].1, "<");
}

/// Parsed structure keeps both the fenced block and everything after it.
#[test]
fn parser_keeps_structure_after_fenced_block_with_angle_bracket() {
    let source = "```python\nif n <= 1:\n    return n\n```\n\n> This is a simple blockquote.\n";
    let elements = top_level_elements(source, MarkdownLanguage::github_flavored());

    assert_eq!(elements.first(), Some(&MarkdownElementType::CodeBlock), "the fence must close: {elements:?}");
    assert_eq!(elements.get(1), Some(&MarkdownElementType::Blockquote), "the quote after the fence must remain: {elements:?}");
}

/// MDX component tags may span lines, so their scan must still reject a `<`
/// that is not a JSX tag start, such as a comparison operator.
#[test]
fn mdx_mode_does_not_swallow_comparison_angle() {
    let source = "a < b\n\n> This is a simple blockquote.\n";
    let tokens = lex_tokens(source, MarkdownLanguage::new(true));

    assert!(!tokens.iter().any(|(kind, ..)| *kind == MarkdownTokenType::MdxComponent), "a comparison `<` must not open a JSX tag: {tokens:#?}");
    assert!(tokens.iter().any(|(kind, text, ..)| *kind == MarkdownTokenType::BlockquoteMarker && text == ">"));
}

#[test]
fn mdx_mode_fenced_block_keeps_its_closing_fence() {
    let source = "```python\nif n <= 1:\n    return n\n```\n\n> quote\n";
    let tokens = lex_tokens(source, MarkdownLanguage::new(true));

    let fences: Vec<_> = tokens.iter().filter(|(kind, ..)| *kind == MarkdownTokenType::CodeFence).collect();
    assert_eq!(fences.len(), 2, "both fences must stay separate tokens: {tokens:#?}");
    assert!(tokens.iter().any(|(kind, text, ..)| *kind == MarkdownTokenType::BlockquoteMarker && text == ">"));
}

/// A `#` line inside a fenced block is code content, not a heading.
#[test]
fn fence_content_keeps_hash_out_of_heading_tokens() {
    let source = "```markdown\n# Code block with Markdown\nThis is a **code block** that contains Markdown syntax.\n```\n";
    let tokens = lex_tokens(source, MarkdownLanguage::github_flavored());

    assert!(!tokens.iter().any(|(kind, ..)| is_heading(*kind)), "fence content must not lex as a heading: {tokens:#?}");

    let fences: Vec<_> = tokens.iter().filter(|(kind, ..)| *kind == MarkdownTokenType::CodeFence).collect();
    assert_eq!(fences.len(), 2, "both fences must stay separate tokens: {tokens:#?}");

    let hash = tokens.iter().find(|(kind, text, ..)| *kind == MarkdownTokenType::Hash && *text == "#").expect("the fence body still lexes its `#`");
    assert!(hash.2 > fences[0].3 && hash.3 < fences[1].2, "the `#` must stay between the fences: {tokens:#?}");
}

/// A heading after a blockquote marker is still a heading block.
#[test]
fn blockquote_heading_lexes_at_its_marker() {
    let source = "> ### Heading in Blockquote\n";
    let tokens = lex_tokens(source, MarkdownLanguage::github_flavored());

    let heading = tokens.iter().find(|(kind, ..)| *kind == MarkdownTokenType::Heading3).expect("`> ### ...` must keep a `Heading3` token");
    assert_eq!(heading.1, "###");
    assert_eq!(heading.2, 2, "the heading marker starts after the `> ` prefix: {tokens:#?}");
}

fn is_heading(kind: MarkdownTokenType) -> bool {
    matches!(kind, MarkdownTokenType::Heading1 | MarkdownTokenType::Heading2 | MarkdownTokenType::Heading3 | MarkdownTokenType::Heading4 | MarkdownTokenType::Heading5 | MarkdownTokenType::Heading6)
}

/// A real multi-line JSX component still lexes as one MDX component.
#[test]
fn mdx_mode_keeps_multiline_jsx_component() {
    let source = "<Card\n  title=\"Hello\"\n/>\n";
    let tokens = lex_tokens(source, MarkdownLanguage::new(true));

    let components: Vec<_> = tokens.iter().filter(|(kind, ..)| *kind == MarkdownTokenType::MdxComponent).collect();
    assert_eq!(components.len(), 1, "multi-line JSX must stay a single component: {tokens:#?}");
    assert_eq!(components[0].1, "<Card\n  title=\"Hello\"\n/>");
}
