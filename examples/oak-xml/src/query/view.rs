use crate::ast::{XmlElement, XmlRoot, XmlValue};
use core::range::Range;
use oak_core::{
    Arc,
    query::{ElementRef, ElementView, ExpandedName, TextPolicy},
};

/// Indexed read-only view over an XML AST for selector execution.
#[derive(Debug, Clone)]
pub struct XmlDocumentView<'a> {
    revision: u64,
    nodes: Vec<XmlNodeData>,
    sources: Vec<&'a XmlElement>,
}

#[derive(Debug, Clone)]
pub(crate) struct XmlNodeData {
    expanded_name: ExpandedName,
    raw_name: String,
    attributes: Vec<(String, String)>,
    pub(crate) parent: Option<u64>,
    pub(crate) children: Vec<u64>,
    direct_text: String,
    descendant_text: String,
    span: Range<usize>,
}

/// Element view bound to one node in a [`XmlDocumentView`] snapshot.
#[derive(Debug, Clone)]
pub struct XmlElementView<'a> {
    document: XmlDocumentView<'a>,
    node_id: u64,
}

impl<'a> XmlDocumentView<'a> {
    /// Builds a view from a parsed XML root. Assigns stable node ids in document order.
    #[must_use]
    pub fn from_root(root: &'a XmlRoot) -> Self {
        let mut nodes = Vec::new();
        let mut sources = Vec::new();
        match &root.value {
            XmlValue::Element(element) => {
                index_element(element, None, &mut nodes, &mut sources);
            }
            XmlValue::Fragment(values) => {
                for value in values {
                    if let XmlValue::Element(element) = value {
                        index_element(element, None, &mut nodes, &mut sources);
                    }
                }
            }
            _ => {}
        }
        Self { revision: 1, nodes, sources }
    }

    /// Builds a view rooted at one element subtree.
    #[must_use]
    pub fn from_element(element: &'a XmlElement) -> Self {
        let mut nodes = Vec::new();
        let mut sources = Vec::new();
        index_element(element, None, &mut nodes, &mut sources);
        Self { revision: 1, nodes, sources }
    }

    /// Returns the document revision for this snapshot.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns a view handle for a node id in this snapshot.
    #[must_use]
    pub fn element(&self, node_id: u64) -> Option<XmlElementView<'a>> {
        if self.nodes.get(node_id as usize).is_some() { Some(XmlElementView { document: self.clone(), node_id }) } else { None }
    }

    /// Returns the source AST element for a node id in this snapshot.
    #[must_use]
    pub fn source_element(&self, node_id: u64) -> Option<&'a XmlElement> {
        self.sources.get(node_id as usize).copied()
    }

    /// Maps selector matches back to source AST elements in match order.
    #[must_use]
    pub fn source_elements(&self, references: &[ElementRef]) -> Vec<&'a XmlElement> {
        references.iter().filter_map(|reference| self.source_element(reference.node_id)).collect()
    }

    /// Returns all element node ids in document order.
    #[must_use]
    pub fn all_elements(&self) -> Vec<ElementRef> {
        self.nodes.iter().enumerate().map(|(index, _)| ElementRef { node_id: index as u64, revision: self.revision }).collect()
    }

    pub(crate) fn node(&self, node_id: u64) -> Option<&XmlNodeData> {
        self.nodes.get(node_id as usize)
    }
}

fn index_element<'a>(element: &'a XmlElement, parent: Option<u64>, nodes: &mut Vec<XmlNodeData>, sources: &mut Vec<&'a XmlElement>) -> u64 {
    let node_id = nodes.len() as u64;
    sources.push(element);
    let local = split_local_name(&element.name);
    let expanded_name = ExpandedName::local(local);
    let attributes = element.attributes.iter().map(|attr| (attr.name.clone(), attr.value.clone())).collect();
    let direct_text = collect_direct_text(&element.children);
    let descendant_text = collect_descendant_text_values(&element.children);
    let mut children = Vec::new();
    nodes.push(XmlNodeData { expanded_name, raw_name: element.name.clone(), attributes, parent, children: Vec::new(), direct_text, descendant_text, span: element.span });
    for child in &element.children {
        if let XmlValue::Element(child_element) = child {
            let child_id = index_element(child_element, Some(node_id), nodes, sources);
            children.push(child_id);
        }
    }
    nodes[node_id as usize].children = children;
    node_id
}

fn split_local_name(name: &str) -> String {
    name.split_once(':').map(|(_, local)| local.to_string()).unwrap_or_else(|| name.to_string())
}

fn split_qname(name: &str) -> (Option<String>, String) {
    match name.split_once(':') {
        Some((prefix, local)) => (Some(prefix.to_string()), local.to_string()),
        None => (None, name.to_string()),
    }
}

fn collect_direct_text(children: &[XmlValue]) -> String {
    let mut out = String::new();
    for child in children {
        match child {
            XmlValue::Text(text) | XmlValue::CData(text) => out.push_str(text),
            _ => {}
        }
    }
    out
}

fn collect_descendant_text_values(children: &[XmlValue]) -> String {
    let mut out = String::new();
    for child in children {
        match child {
            XmlValue::Text(text) | XmlValue::CData(text) => out.push_str(text),
            XmlValue::Element(element) => out.push_str(&collect_descendant_text_values(&element.children)),
            XmlValue::Fragment(values) => out.push_str(&collect_descendant_text_values(values)),
            _ => {}
        }
    }
    out
}

impl<'a> ElementView for XmlElementView<'a> {
    fn element_ref(&self) -> ElementRef {
        ElementRef { node_id: self.node_id, revision: self.document.revision }
    }

    fn expanded_name(&self) -> ExpandedName {
        self.document.node(self.node_id).map(|node| node.expanded_name.clone()).unwrap_or_else(|| ExpandedName::local(""))
    }

    fn parent(&self) -> Option<ElementRef> {
        self.document.node(self.node_id).and_then(|node| node.parent).map(|parent| ElementRef { node_id: parent, revision: self.document.revision })
    }

    fn element_children(&self) -> Vec<ElementRef> {
        let revision = self.document.revision;
        self.document.node(self.node_id).map(|node| node.children.iter().map(|child| ElementRef { node_id: *child, revision }).collect()).unwrap_or_default()
    }

    fn resolve(&self, reference: ElementRef) -> Option<Self> {
        if reference.revision != self.document.revision {
            return None;
        }
        self.document.element(reference.node_id)
    }

    fn attribute(&self, name: &ExpandedName) -> Option<Arc<str>> {
        let node = self.document.node(self.node_id)?;
        node.attributes.iter().find(|(attr_name, _)| attribute_matches(attr_name, name)).map(|(_, value)| Arc::from(value.as_str()))
    }

    fn text_content(&self, policy: TextPolicy) -> Arc<str> {
        let Some(node) = self.document.node(self.node_id)
        else {
            return Arc::from("");
        };
        match policy {
            TextPolicy::Direct => Arc::from(node.direct_text.as_str()),
            TextPolicy::Descendant => Arc::from(node.descendant_text.as_str()),
        }
    }

    fn source_span(&self) -> Option<Range<usize>> {
        self.document.node(self.node_id).map(|node| node.span.clone())
    }
}

fn attribute_matches(attr_name: &str, query: &ExpandedName) -> bool {
    if query.local_name.contains(':') {
        return attr_name == query.local_name;
    }
    let (_, local) = split_qname(attr_name);
    query.local_name == local
}

impl XmlNodeData {
    pub(crate) fn matches_qname(&self, qname: &oak_xpath::QName) -> bool {
        let (prefix, local) = split_qname(&self.raw_name);
        if qname.local != "*" && qname.local != local {
            return false;
        }
        match (&qname.prefix, prefix) {
            (None, _) => true,
            (Some(expected), Some(actual)) => *expected == actual,
            (Some(_), None) => false,
        }
    }
}
