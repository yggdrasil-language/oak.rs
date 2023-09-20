use oak_xml::ast::XmlValue;

fn parse_xml(source: &str) -> XmlValue {
    oak_xml::parse(source).expect("parse xml")
}

#[test]
fn parses_hyphenated_attributes_on_self_closing_tag() {
    let value = parse_xml(r#"<rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>"#);
    let element = value.as_element().expect("root element");
    assert_eq!(element.name, "rootfile");
    assert_eq!(element.attributes.len(), 2);
    assert_eq!(element.children.len(), 0);
}

#[test]
fn parses_epub_container_shape() {
    let value = parse_xml(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
    );
    let container = value.as_element().expect("container");
    assert_eq!(container.name, "container");
    let rootfiles = container
        .children
        .iter()
        .filter_map(XmlValue::as_element)
        .find(|child| child.name == "rootfiles")
        .expect("rootfiles");
    let rootfile = rootfiles
        .children
        .iter()
        .filter_map(XmlValue::as_element)
        .find(|child| child.name == "rootfile")
        .expect("rootfile");
    assert_eq!(
        rootfile
            .attributes
            .iter()
            .find(|attr| attr.name == "full-path")
            .map(|attr| attr.value.as_str()),
        Some("OEBPS/content.opf")
    );
}

#[test]
fn parses_plain_dc_title_element() {
    let value = parse_xml("<dc:title>Sample Book</dc:title>");
    let title = value.as_element().expect("dc:title");
    assert_eq!(title.name, "dc:title");
    assert_eq!(
        title
            .children
            .iter()
            .filter_map(XmlValue::as_str)
            .collect::<String>(),
        "Sample Book"
    );
}

#[test]
fn parses_namespaced_dc_metadata() {
    let value = parse_xml(
        r#"<metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Sample Book</dc:title><dc:language>en</dc:language></metadata>"#,
    );
    let metadata = value.as_element().expect("metadata");
    let title = metadata
        .children
        .iter()
        .filter_map(XmlValue::as_element)
        .find(|child| child.name == "dc:title")
        .expect("dc:title");
    assert_eq!(
        title
            .children
            .iter()
            .filter_map(XmlValue::as_str)
            .collect::<String>(),
        "Sample Book"
    );
}
