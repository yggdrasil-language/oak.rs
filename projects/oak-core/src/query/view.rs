use core::range::Range;

/// Expanded element or attribute name with optional namespace URI.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpandedName {
    /// Namespace URI when known. `None` means no namespace or unresolved prefix.
    pub namespace_uri: Option<String>,
    /// Local name without prefix.
    pub local_name: String,
}

impl ExpandedName {
    /// Creates a name without namespace information.
    #[must_use]
    pub fn local(local_name: impl Into<String>) -> Self {
        Self {
            namespace_uri: None,
            local_name: local_name.into(),
        }
    }

    /// Creates a fully expanded name.
    #[must_use]
    pub fn namespaced(namespace_uri: impl Into<String>, local_name: impl Into<String>) -> Self {
        Self {
            namespace_uri: Some(namespace_uri.into()),
            local_name: local_name.into(),
        }
    }
}

/// Policy for reading element text during queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextPolicy {
    /// Direct text node children only.
    #[default]
    Direct,
    /// All descendant text concatenated in document order.
    Descendant,
}

/// Opaque, revision-bound handle to a node in a document view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementRef {
    /// Stable node identity within one view snapshot.
    pub node_id: u64,
    /// Source revision that produced this view.
    pub revision: u64,
}

/// Read-only element view used by structured queries and selector executors.
pub trait ElementView {
    /// Returns the element reference for this view node.
    fn element_ref(&self) -> ElementRef;

    /// Expanded element name.
    fn expanded_name(&self) -> ExpandedName;

    /// Parent element, if any.
    fn parent(&self) -> Option<ElementRef>;

    /// Direct element children in document order.
    fn element_children(&self) -> Vec<ElementRef>;

    /// Resolves a child reference back to a view node in the same snapshot.
    fn resolve(&self, reference: ElementRef) -> Option<Self>
    where
        Self: Sized;

    /// Reads an attribute by expanded name.
    fn attribute(&self, name: &ExpandedName) -> Option<crate::Arc<str>>;

    /// Reads text according to `policy`.
    fn text_content(&self, policy: TextPolicy) -> crate::Arc<str>;

    /// Byte range of this element in the backing source, when available.
    fn source_span(&self) -> Option<Range<usize>>;
}
