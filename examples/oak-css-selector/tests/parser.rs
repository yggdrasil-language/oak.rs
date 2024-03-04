use oak_css_selector::{AttributeOperator, AttributeSelector, Combinator, CompoundSelector, Selector, SelectorList, SimpleSelector, parse_css_selector};

#[test]
fn parses_type_and_class() {
    let list = parse_css_selector("article > h1.title").expect("parse");
    assert_eq!(
        list,
        SelectorList {
            selectors: vec![Selector {
                compound: CompoundSelector { simple: vec![SimpleSelector::Type("h1".to_string()), SimpleSelector::Class("title".to_string()),] },
                ancestors: vec![(Combinator::Child, CompoundSelector { simple: vec![SimpleSelector::Type("article".to_string())] },)],
            }],
        }
    );
}

#[test]
fn parses_attribute_prefix() {
    let list = parse_css_selector("nav a[href^=\"/docs/\"]").expect("parse");
    let compound = &list.selectors[0].compound;
    assert!(compound.simple.iter().any(|simple| {
        matches!(
            simple,
            SimpleSelector::Attribute(AttributeSelector {
                name,
                operator: AttributeOperator::StartsWith,
                value: Some(value),
            }) if name == "href" && value == "/docs/"
        )
    }));
}

#[test]
fn parses_selector_list() {
    let list = parse_css_selector("article > h1, article > h2").expect("parse");
    assert_eq!(list.selectors.len(), 2);
}
