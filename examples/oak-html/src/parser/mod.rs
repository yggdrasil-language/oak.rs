/// Element type module for HTML.
pub mod element_type;

use crate::{
    language::HtmlLanguage,
    lexer::{HtmlLexer, token_type::HtmlTokenType},
    parser::element_type::HtmlElementType,
};
use oak_core::{
    OakError,
    parser::{ParseCache, ParseOutput, Parser, ParserState, parse_with_lexer},
    source::{Source, TextEdit},
};

pub(crate) type State<'a, S> = ParserState<'a, HtmlLanguage, S>;

/// Parser for the HTML language.
///
/// This parser transforms a stream of tokens into a green tree of HTML syntax nodes.
pub struct HtmlParser<'config> {
    pub(crate) config: &'config HtmlLanguage,
}

impl<'config> HtmlParser<'config> {
    /// Creates a new `HtmlParser` with the given configuration.
    pub fn new(config: &'config HtmlLanguage) -> Self {
        Self { config }
    }

    /// Parses an HTML tag, including its attributes and potentially its children.
    ///
    /// This method handles both self-closing tags (e.g., `<br/>`) and tags with
    /// separate closing tags (e.g., `<div>...</div>`).
    fn parse_tag<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::HtmlTokenType::*;
        let cp = state.checkpoint();
        state.expect(TagOpen).ok();
        if !state.at(TagName) {
            state.finish_at(cp, HtmlElementType::Element);
            return Ok(());
        }
        let tag_name = state
            .peek_text()
            .map(|text| text.trim().to_ascii_lowercase())
            .unwrap_or_default();
        state.bump();

        while state.not_at_end() && !matches!(state.peek_kind(), Some(TagClose) | Some(TagSelfClose)) {
            if state.at(AttributeName) || state.at(TagName) {
                let attr_cp = state.checkpoint();
                state.bump(); // attribute name token
                if state.eat(Equal) {
                    state.eat(Quote);
                    state.eat(AttributeValue);
                    state.eat(Quote);
                }
                state.finish_at(attr_cp, HtmlElementType::Attribute);
            }
            else {
                state.advance();
            }
        }

        if state.eat(TagSelfClose) {
            // Self-closing tag
        }
        else if state.eat(TagClose) {
            loop {
                if !state.not_at_end() {
                    break;
                }
                if Self::peek_closing_tag_name(state).as_deref() == Some(tag_name.as_str()) {
                    state.eat(TagSlashOpen);
                    state.eat(TagName);
                    state.expect(TagClose).ok();
                    break;
                }
                if state.at(TagOpen) {
                    self.parse_tag(state)?;
                }
                else if state.not_at_end() {
                    state.bump();
                }
            }
        }

        state.finish_at(cp, HtmlElementType::Element);
        Ok(())
    }

    fn peek_closing_tag_name<'a, S: Source + ?Sized>(
        state: &State<'a, S>,
    ) -> Option<String> {
        use crate::lexer::token_type::HtmlTokenType::*;
        if !state.at(TagSlashOpen) {
            return None;
        }
        match state.peek_kind_at(1) {
            Some(TagName) => state.peek_at(1).map(|token| {
                state
                    .source
                    .get_text_in(token.span.clone())
                    .trim()
                    .to_ascii_lowercase()
            }),
            _ => None,
        }
    }
}

impl<'config> Parser<HtmlLanguage> for HtmlParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<HtmlLanguage>) -> ParseOutput<'a, HtmlLanguage> {
        let lexer = HtmlLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();

            while state.not_at_end() {
                match state.peek_kind() {
                    Some(HtmlTokenType::TagOpen) => self.parse_tag(state)?,
                    Some(HtmlTokenType::Doctype) => {
                        state.bump();
                    }
                    Some(HtmlTokenType::Comment) => {
                        state.bump();
                    }
                    _ => {
                        state.bump();
                    }
                }
            }

            Ok(state.finish_at(checkpoint, crate::parser::element_type::HtmlElementType::Document))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oak_core::parser::session::ParseSession;
    use oak_core::{Parser, RedNode, RedTree, SourceText};

    #[test]
    fn parse_simple_anchor() {
        const SOURCE: &str = r#"<a href="part.xhtml">Part One</a>"#;
        let language = HtmlLanguage::default();
        let source = SourceText::new(SOURCE);
        let mut cache = ParseSession::<HtmlLanguage>::default();
        let parsed = HtmlParser::new(&language).parse(&source, &[], &mut cache);
        let green = parsed.result.expect("parse html");
        let root = RedNode::new(&green, 0);
        let mut element_tags = Vec::new();
        fn walk(node: RedNode<'_, HtmlLanguage>, source: &SourceText, out: &mut Vec<String>) {
            if node.element_type() == HtmlElementType::Element {
                out.push(node.text(source).to_string());
            }
            for child in node.children() {
                if let RedTree::Node(child_node) = child {
                    walk(child_node, source, out);
                }
            }
        }
        for child in root.children() {
            if let RedTree::Node(child_node) = child {
                walk(child_node, &source, &mut element_tags);
            }
        }
        assert_eq!(element_tags, vec![SOURCE.to_string()]);
    }

    #[test]
    fn parse_li_with_nested_anchor_and_list() {
        const SOURCE: &str = r#"<li><a href="part.xhtml">Part One</a><ol><li><a href="part.xhtml#ch1">Chapter 1</a></li></ol></li>"#;
        let language = HtmlLanguage::default();
        let source = SourceText::new(SOURCE);
        let mut cache = ParseSession::<HtmlLanguage>::default();
        let parsed = HtmlParser::new(&language).parse(&source, &[], &mut cache);
        let green = parsed.result.expect("parse html");
        let root = RedNode::new(&green, 0);
        let mut element_tags = Vec::new();
        fn walk(node: RedNode<'_, HtmlLanguage>, source: &SourceText, out: &mut Vec<String>) {
            if node.element_type() == HtmlElementType::Element {
                out.push(node.text(source).to_string());
            }
            for child in node.children() {
                if let RedTree::Node(child_node) = child {
                    walk(child_node, source, out);
                }
            }
        }
        for child in root.children() {
            if let RedTree::Node(child_node) = child {
                walk(child_node, &source, &mut element_tags);
            }
        }
        assert_eq!(
            element_tags
                .iter()
                .filter(|tag| tag.starts_with("<a"))
                .count(),
            2,
            "anchors: {element_tags:?}"
        );
    }

    #[test]
    fn parse_nested_nav_green_structure() {
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
        let parsed = HtmlParser::new(&language).parse(&source, &[], &mut cache);
        let green = parsed.result.expect("parse html");
        let root = RedNode::new(&green, 0);
        let mut anchor_count = 0usize;
        fn walk(node: RedNode<'_, HtmlLanguage>, source: &SourceText, count: &mut usize) {
            if node.element_type() == HtmlElementType::Element {
                let text = node.text(source);
                if text.starts_with("<a") {
                    *count += 1;
                }
            }
            for child in node.children() {
                if let RedTree::Node(child_node) = child {
                    walk(child_node, source, count);
                }
            }
        }
        for child in root.children() {
            if let RedTree::Node(child_node) = child {
                walk(child_node, &source, &mut anchor_count);
            }
        }
        assert_eq!(anchor_count, 2);
    }
}
