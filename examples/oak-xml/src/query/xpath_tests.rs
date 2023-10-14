#[cfg(test)]
mod tests {
    use crate::ast::{XmlAttribute, XmlElement, XmlRoot, XmlValue};
    use crate::query::{select_xpath, select_xpath_elements, XmlDocumentView};
    use core::range::Range;
    use oak_core::query::QueryBudget;

    fn sample_root() -> XmlRoot {
        XmlRoot {
            value: XmlValue::Element(XmlElement {
                name: "w:body".to_string(),
                attributes: Vec::new(),
                children: vec![
                    XmlValue::Element(XmlElement {
                        name: "w:p".to_string(),
                        attributes: vec![XmlAttribute {
                            name: "w14:paraId".to_string(),
                            value: "abc".to_string(),
                            span: Range::from(0..0),
                        }],
                        children: vec![XmlValue::Text("hello".to_string())],
                        span: Range::from(0..0),
                    }),
                    XmlValue::Element(XmlElement {
                        name: "w:p".to_string(),
                        attributes: Vec::new(),
                        children: Vec::new(),
                        span: Range::from(0..0),
                    }),
                ],
                span: Range::from(0..0),
            }),
        }
    }

    #[test]
    fn selects_child_paragraphs() {
        let root = sample_root();
        let view = XmlDocumentView::from_root(&root);
        let result = select_xpath(&view, "/w:body/w:p", QueryBudget::default()).expect("parse");
        assert_eq!(result.matches.len(), 2);
    }

    #[test]
    fn selects_with_attribute_predicate() {
        let root = sample_root();
        let view = XmlDocumentView::from_root(&root);
        let result = select_xpath(&view, "//w:p[@w14:paraId=\"abc\"]", QueryBudget::default()).expect("parse");
        assert_eq!(result.matches.len(), 1);
    }
    #[test]
    fn maps_matches_back_to_source_elements() {
        let root = sample_root();
        let view = XmlDocumentView::from_root(&root);
        let (result, elements) =
            select_xpath_elements(&view, "/w:body/w:p", QueryBudget::default()).expect("parse");
        assert_eq!(result.matches.len(), 2);
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0].name, "w:p");
        assert!(view.source_element(result.matches[0].node_id).is_some());
    }
}
