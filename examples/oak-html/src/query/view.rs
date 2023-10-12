use crate::ast::{Element, HtmlDocument, HtmlNode};
use core::range::Range;
use oak_core::query::{ElementRef, ElementView, ExpandedName, TextPolicy};
use oak_core::Arc;

/// Indexed read-only view over an HTML AST for selector execution.
#[derive(Debug, Clone)]
pub struct HtmlDocumentView {
    revision: u64,
    nodes: Vec<HtmlNodeData>,
}

#[derive(Debug, Clone)]
pub(crate) struct HtmlNodeData {
    pub(crate) tag_name: String,
    attributes: Vec<(String, Option<String>)>,
    pub(crate) classes: Vec<String>,
    pub(crate) id: Option<String>,
    pub(crate) parent: Option<u64>,
    pub(crate) children: Vec<u64>,
    direct_text: String,
    descendant_text: String,
    span: Range<usize>,
}

/// Element view bound to one node in a [`HtmlDocumentView`] snapshot.
#[derive(Debug, Clone)]
pub struct HtmlElementView {
    document: HtmlDocumentView,
    node_id: u64,
}

impl HtmlDocumentView {
    /// Builds a view from a parsed HTML document.
    #[must_use]
    pub fn from_document(document: &HtmlDocument) -> Self {
        let mut nodes = Vec::new();
        for node in &document.nodes {
            if let HtmlNode::Element(element) = node {
                index_element(element, None, &mut nodes);
            }
        }
        Self {
            revision: 1,
            nodes,
        }
    }

    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn element(&self, node_id: u64) -> Option<HtmlElementView> {
        if self.nodes.get(node_id as usize).is_some() {
            Some(HtmlElementView {
                document: self.clone(),
                node_id,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub fn all_elements(&self) -> Vec<ElementRef> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, _)| ElementRef {
                node_id: index as u64,
                revision: self.revision,
            })
            .collect()
    }

    pub(crate) fn node(&self, node_id: u64) -> Option<&HtmlNodeData> {
        self.nodes.get(node_id as usize)
    }
}

fn index_element(element: &Element, parent: Option<u64>, nodes: &mut Vec<HtmlNodeData>) -> u64 {
    let node_id = nodes.len() as u64;
    let attributes = element
        .attributes
        .iter()
        .map(|attr| (attr.name.clone(), attr.value.clone()))
        .collect::<Vec<_>>();
    let classes = attributes
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("class"))
        .and_then(|(_, value)| value.as_ref())
        .map(|value| value.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default();
    let id = attributes
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("id"))
        .and_then(|(_, value)| value.clone());
    let direct_text = collect_direct_text(&element.children);
    let descendant_text = collect_descendant_text(&element.children);
    let mut children = Vec::new();
    nodes.push(HtmlNodeData {
        tag_name: element.tag_name.clone(),
        attributes,
        classes,
        id,
        parent,
        children: Vec::new(),
        direct_text,
        descendant_text,
        span: element.span,
    });
    for child in &element.children {
        if let HtmlNode::Element(child_element) = child {
            children.push(index_element(child_element, Some(node_id), nodes));
        }
    }
    nodes[node_id as usize].children = children;
    node_id
}

fn collect_direct_text(children: &[HtmlNode]) -> String {
    let mut out = String::new();
    for child in children {
        if let HtmlNode::Text(text) = child {
            out.push_str(&text.content);
        }
    }
    out
}

fn collect_descendant_text(children: &[HtmlNode]) -> String {
    let mut out = String::new();
    for child in children {
        match child {
            HtmlNode::Text(text) => out.push_str(&text.content),
            HtmlNode::Element(element) => out.push_str(&collect_descendant_text(&element.children)),
            HtmlNode::Comment(_) => {}
        }
    }
    out
}

impl ElementView for HtmlElementView {
    fn element_ref(&self) -> ElementRef {
        ElementRef {
            node_id: self.node_id,
            revision: self.document.revision,
        }
    }

    fn expanded_name(&self) -> ExpandedName {
        ExpandedName::local(
            self.document
                .node(self.node_id)
                .map(|node| node.tag_name.clone())
                .unwrap_or_default(),
        )
    }

    fn parent(&self) -> Option<ElementRef> {
        self.document
            .node(self.node_id)
            .and_then(|node| node.parent)
            .map(|parent| ElementRef {
                node_id: parent,
                revision: self.document.revision,
            })
    }

    fn element_children(&self) -> Vec<ElementRef> {
        let revision = self.document.revision;
        self.document
            .node(self.node_id)
            .map(|node| {
                node.children
                    .iter()
                    .map(|child| ElementRef {
                        node_id: *child,
                        revision,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn resolve(&self, reference: ElementRef) -> Option<Self> {
        if reference.revision != self.document.revision {
            return None;
        }
        self.document.element(reference.node_id)
    }

    fn attribute(&self, name: &ExpandedName) -> Option<Arc<str>> {
        let node = self.document.node(self.node_id)?;
        node.attributes
            .iter()
            .find(|(attr_name, _)| attr_name.eq_ignore_ascii_case(&name.local_name))
            .and_then(|(_, value)| value.as_ref())
            .map(|value| Arc::from(value.as_str()))
    }

    fn text_content(&self, policy: TextPolicy) -> Arc<str> {
        let Some(node) = self.document.node(self.node_id) else {
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

impl HtmlNodeData {
    pub(crate) fn attribute_value(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(attr_name, _)| attr_name.eq_ignore_ascii_case(name))
            .and_then(|(_, value)| value.as_deref())
    }
}
