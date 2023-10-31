/// Parsed XPath expression. Union members are evaluated independently.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct XpathExpr {
    /// Union branches in source order.
    pub paths: Vec<PathExpr>,
}

/// One location path within an XPath expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PathExpr {
    /// Whether the path starts with `/` or `//`.
    pub absolute: bool,
    /// Steps after the initial slash, if any.
    pub steps: Vec<Step>,
}

/// A single axis step with optional predicates.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Step {
    /// Traversal axis for this step.
    pub axis: Axis,
    /// Node test applied at this step.
    pub test: NodeTest,
    /// Bracket predicates applied to candidate nodes.
    pub predicates: Vec<Predicate>,
}

/// Supported XPath axes in the Oak subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Axis {
    /// `child::`
    Child,
    /// `descendant::`
    Descendant,
    /// `descendant-or-self::`
    DescendantOrSelf,
    /// `attribute::`
    Attribute,
}

/// Node tests supported by the Oak XPath subset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeTest {
    /// Any element node (`*`).
    AnyElement,
    /// Prefixed wildcard (`prefix:*`).
    PrefixedWildcard(QName),
    /// Qualified or local element name.
    Name(QName),
}

/// Qualified name with optional prefix.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QName {
    /// Namespace prefix when present.
    pub prefix: Option<String>,
    /// Local part after `:`.
    pub local: String,
}

/// Bracket predicate attached to a step.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Predicate {
    /// Numeric position such as `[1]`.
    Position(u32),
    /// Attribute equality such as `[@id="main"]`.
    AttributeEquals(QName, String),
    /// Unsupported predicate retained for diagnostics.
    Unsupported(String),
}
