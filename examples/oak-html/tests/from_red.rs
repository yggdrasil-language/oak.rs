use oak_core::{Builder, Lexer, Parser, Source, SourceText, parser::session::ParseSession};
use oak_html::{HtmlBuilder, HtmlLanguage, HtmlLexer, HtmlNode, HtmlParser, ast::Element, lexer::token_type::HtmlTokenType};

#[test]
fn builder_lowers_nested_elements_and_attributes() {
    const SOURCE: &str = r#"<article class="doc"><h1>Title</h1><p>Body</p></article>"#;
    let language = HtmlLanguage::default();
    let builder = HtmlBuilder::new(language);
    let source = SourceText::new(SOURCE);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let built = builder.build(&source, &[], &mut cache);
    let document = built.result.expect("build html ast");
    assert_eq!(document.nodes.len(), 1);
    let HtmlNode::Element(article) = &document.nodes[0]
    else {
        panic!("expected article element");
    };
    assert_eq!(article.tag_name, "article");
    assert_eq!(article.attributes.iter().find(|attr| attr.name == "class").and_then(|attr| attr.value.as_deref()), Some("doc"));
    assert_eq!(article.children.len(), 2);
}

#[test]
fn builder_lowers_nav_links_for_selector_execution() {
    const SOURCE: &str = r#"<nav epub:type="toc"><ol><li><a href="part.xhtml">Part</a></li></ol></nav>"#;
    let language = HtmlLanguage::default();
    let source = SourceText::new(SOURCE);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let document = HtmlBuilder::new(language.clone()).build(&source, &[], &mut cache).result.expect("build html ast");
    let view = crate::query::HtmlDocumentView::from_document(&document);
    let matched = crate::query::select_css(&view, "nav a[href]", oak_core::query::QueryBudget::default()).expect("css selector");
    assert_eq!(matched.matches.len(), 1);
}

#[test]
fn lexes_nested_nav_closing_tags() {
    use oak_core::{Lexer, Source};
    const SOURCE: &str = r#"<li><a href="part.xhtml">Part One</a>
<ol>
  <li><a href="part.xhtml#ch1">Chapter 1</a></li>
</ol>
  </li>"#;
    let language = HtmlLanguage::default();
    let source = SourceText::new(SOURCE);
    let lexer = HtmlLexer::new(&language);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let tokens = lexer.lex(&source, &[], &mut cache).result.expect("lex");
    let rendered = tokens.iter().map(|token| format!("{:?}:{}", token.kind, source.get_text_in(token.span.clone()))).collect::<Vec<_>>().join("\n");
    assert!(rendered.contains("TagSlashOpen:</"), "missing closing tags in token stream:\n{rendered}");
    assert!(rendered.contains("Text:Part One"), "missing anchor text token:\n{rendered}");
}

#[test]
fn parser_builds_nested_nav_green_tree() {
    const SOURCE: &str = r#"<nav epub:type="toc">
<ol>
  <li><a href="part.xhtml">Part One</a>
<ol>
  <li><a href="part.xhtml#ch1">Chapter 1</a></li>
</ol>
  </li>
</ol>
</nav>"#;
    let language = HtmlLanguage::default();
    let source = SourceText::new(SOURCE);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let document = HtmlBuilder::new(language.clone()).build(&source, &[], &mut cache).result.expect("build html ast");
    let HtmlNode::Element(nav) = &document.nodes[0]
    else {
        panic!("expected nav root element");
    };
    fn count_tag(element: &Element, tag: &str) -> usize {
        let mut count = if element.tag_name.eq_ignore_ascii_case(tag) { 1 } else { 0 };
        for child in &element.children {
            if let HtmlNode::Element(child_element) = child {
                count += count_tag(child_element, tag);
            }
        }
        count
    }
    assert_eq!(nav.tag_name, "nav");
    assert_eq!(count_tag(nav, "a"), 2);
}

#[test]
fn builder_lowers_html_body_section_paragraph() {
    const SOURCE: &str = r#"<html><body><section><div><p>Wrapped</p></div></section></body></html>"#;
    let language = HtmlLanguage::default();
    let source = SourceText::new(SOURCE);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let built = HtmlBuilder::new(language).build(&source, &[], &mut cache);
    let document = built.result.expect("build html ast");
    fn find_p_text(element: &Element) -> Option<String> {
        if element.tag_name.eq_ignore_ascii_case("p") {
            for child in &element.children {
                if let HtmlNode::Text(text) = child {
                    return Some(text.content.clone());
                }
            }
        }
        for child in &element.children {
            if let HtmlNode::Element(child_element) = child {
                if let Some(found) = find_p_text(child_element) {
                    return Some(found);
                }
            }
        }
        None
    }
    let text = document
        .nodes
        .iter()
        .filter_map(|node| match node {
            HtmlNode::Element(element) => find_p_text(element),
            _ => None,
        })
        .next()
        .expect("paragraph text");
    assert_eq!(text, "Wrapped");
}

#[test]
fn builder_lowers_nested_nav_anchor_text() {
    const SOURCE: &str = r#"<nav epub:type="toc">
<ol>
  <li><a href="part.xhtml">Part One</a>
<ol>
  <li><a href="part.xhtml#ch1">Chapter 1</a></li>
</ol>
  </li>
</ol>
</nav>"#;
    let language = HtmlLanguage::default();
    let source = SourceText::new(SOURCE);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let built = HtmlBuilder::new(language).build(&source, &[], &mut cache);
    let document = built.result.expect("build html ast");
    fn count_tag(element: &Element, tag: &str) -> usize {
        let mut count = if element.tag_name.eq_ignore_ascii_case(tag) { 1 } else { 0 };
        for child in &element.children {
            if let HtmlNode::Element(child_element) = child {
                count += count_tag(child_element, tag);
            }
        }
        count
    }
    let HtmlNode::Element(nav) = &document.nodes[0]
    else {
        panic!("expected nav root");
    };
    fn dump(element: &Element, depth: usize) -> String {
        let mut out = format!("{}{}\n", "  ".repeat(depth), element.tag_name);
        for child in &element.children {
            match child {
                HtmlNode::Element(child_element) => out.push_str(&dump(child_element, depth + 1)),
                HtmlNode::Text(text) => out.push_str(&format!("{}\"{}\"\n", "  ".repeat(depth + 1), text.content)),
                HtmlNode::Comment(_) => {}
            }
        }
        out
    }
    let tree_dump = dump(nav, 0);
    assert_eq!(count_tag(nav, "a"), 2, "expected two anchors in nav tree:\n{tree_dump}");
    let view = crate::query::HtmlDocumentView::from_document(&document);
    let (_, anchors) = crate::query::select_css_elements(&view, "nav a[href]", oak_core::query::QueryBudget::default()).expect("css selector");
    assert_eq!(anchors.len(), 2);
    let labels = anchors
        .iter()
        .map(|anchor| {
            anchor
                .children
                .iter()
                .filter_map(|child| match child {
                    HtmlNode::Text(text) => Some(text.content.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .collect::<Vec<_>>();
    assert_eq!(labels, vec!["Part One", "Chapter 1"]);
}
#[test]
fn lexes_text_content_outside_tags() {
    use oak_core::{Lexer, Source, SourceText, parser::session::ParseSession};
    const SOURCE: &str = r#"<a href="part.xhtml">Part One</a>"#;
    let language = HtmlLanguage::default();
    let source = SourceText::new(SOURCE);
    let lexer = HtmlLexer::new(&language);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let lexed = lexer.lex(&source, &[], &mut cache);
    let tokens = lexed.result.expect("lex");
    let text = tokens.iter().find(|token| token.kind == HtmlTokenType::Text).map(|token| source.get_text_in(token.span.clone()).to_string());
    assert_eq!(text.as_deref(), Some("Part One"));
}
