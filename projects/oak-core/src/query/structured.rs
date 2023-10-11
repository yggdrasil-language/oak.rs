use super::view::ExpandedName;

/// Syntax-agnostic structured query for languages without a selector DSL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StructuredQuery {
    /// Direct element children.
    Children,
    /// All descendant elements in document order.
    Descendants,
    /// Parent element.
    Parent,
    /// Children whose expanded name matches.
    ChildNamed(ExpandedName),
    /// Descendants whose expanded name matches.
    DescendantNamed(ExpandedName),
    /// Read one attribute on the current node.
    Attribute(ExpandedName),
}
