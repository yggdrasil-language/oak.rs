#[cfg(test)]
mod tests {
    use crate::ast::{Attribute, Element, HtmlDocument, HtmlNode, Text};
    use crate::query::{select_css, HtmlDocumentView};
    use core::range::Range;
    use oak_core::query::QueryBudget;

    fn sample_document() -> HtmlDocument {
        HtmlDocument {
            nodes: vec![HtmlNode::Element(Element {
                tag_name: "article".to_string(),
                attributes: Vec::new(),
                children: vec![
                    HtmlNode::Element(Element {
                        tag_name: "h1".to_string(),
                        attributes: vec![Attribute {
                            name: "class".to_string(),
                            value: Some("title".to_string()),
                            span: Range::from(0..0),
                        }],
                        children: vec![HtmlNode::Text(Text {
                            content: "Hello".to_string(),
                            span: Range::from(0..0),
                        })],
                        span: Range::from(0..0),
                    }),
                    HtmlNode::Element(Element {
                        tag_name: "nav".to_string(),
                        attributes: Vec::new(),
                        children: vec![HtmlNode::Element(Element {
                            tag_name: "a".to_string(),
                            attributes: vec![Attribute {
                                name: "href".to_string(),
                                value: Some("/docs/start".to_string()),
                                span: Range::from(0..0),
                            }],
                            children: Vec::new(),
                            span: Range::from(0..0),
                        })],
                        span: Range::from(0..0),
                    }),
                ],
                span: Range::from(0..0),
            })],
        }
    }

    #[test]
    fn selects_child_with_class() {
        let document = sample_document();
        let view = HtmlDocumentView::from_document(&document);
        let result = select_css(&view, "article > h1.title", QueryBudget::default()).expect("parse");
        assert_eq!(result.matches.len(), 1);
    }

    #[test]
    fn selects_attribute_prefix() {
        let document = sample_document();
        let view = HtmlDocumentView::from_document(&document);
        let result = select_css(&view, "nav a[href^=\"/docs/\"]", QueryBudget::default()).expect("parse");
        assert_eq!(result.matches.len(), 1);
    }

    #[test]
    fn selects_adjacent_sibling() {
        let document = HtmlDocument {
            nodes: vec![HtmlNode::Element(Element {
                tag_name: "article".to_string(),
                attributes: Vec::new(),
                children: vec![
                    HtmlNode::Element(Element {
                        tag_name: "h1".to_string(),
                        attributes: Vec::new(),
                        children: vec![HtmlNode::Text(Text {
                            content: "Title".to_string(),
                            span: Range::from(0..0),
                        })],
                        span: Range::from(0..0),
                    }),
                    HtmlNode::Element(Element {
                        tag_name: "p".to_string(),
                        attributes: Vec::new(),
                        children: vec![HtmlNode::Text(Text {
                            content: "Body".to_string(),
                            span: Range::from(0..0),
                        })],
                        span: Range::from(0..0),
                    }),
                ],
                span: Range::from(0..0),
            })],
        };
        let view = HtmlDocumentView::from_document(&document);
        let result = select_css(&view, "h1 + p", QueryBudget::default()).expect("parse");
        assert_eq!(result.matches.len(), 1);
    }

    #[test]
    fn maps_matches_back_to_source_elements() {
        let document = sample_document();
        let view = HtmlDocumentView::from_document(&document);
        let (_, elements) =
            crate::query::select_css_elements(&view, "nav a[href]", QueryBudget::default()).expect("parse");
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].tag_name, "a");
    }
}
