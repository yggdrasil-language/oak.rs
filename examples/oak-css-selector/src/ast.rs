/// Parsed CSS selector list.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectorList {
    /// Selectors in source order.
    pub selectors: Vec<Selector>,
}

/// One complex selector with optional combinator chain.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Selector {
    /// Right-most compound selector.
    pub compound: CompoundSelector,
    /// Ancestor chain from nearest to farthest.
    pub ancestors: Vec<(Combinator, CompoundSelector)>,
}

/// Combinator between compound selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Combinator {
    /// Descendant combinator (whitespace).
    Descendant,
    /// Child combinator `>`.
    Child,
    /// Adjacent sibling `+`.
    NextSibling,
    /// General sibling `~`.
    SubsequentSibling,
}

/// A sequence of simple selectors without combinators.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompoundSelector {
    /// Simple selectors applied to the same element.
    pub simple: Vec<SimpleSelector>,
}

/// Supported simple selector variants in the Oak subset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SimpleSelector {
    /// Universal selector `*`.
    Universal,
    /// Type selector such as `div`.
    Type(String),
    /// Class selector such as `.title`.
    Class(String),
    /// ID selector such as `#main`.
    Id(String),
    /// Attribute selector such as `[href^="/docs/"]`.
    Attribute(AttributeSelector),
    /// Pseudo-class such as `:checked`.
    PseudoClass(String),
}

/// Attribute selector operator and value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeSelector {
    /// Attribute local name.
    pub name: String,
    /// Matching operator.
    pub operator: AttributeOperator,
    /// Comparison value when required by the operator.
    pub value: Option<String>,
}

/// Attribute matching operators supported in phase one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AttributeOperator {
    /// Presence `[name]`.
    Present,
    /// Exact `[name="value"]`.
    Equals,
    /// Starts-with `[name^="value"]`.
    StartsWith,
    /// Contains token `[name~="value"]`.
    Includes,
    /// Substring `[name*="value"]`.
    Substring,
    /// Suffix `[name$="value"]`.
    Suffix,
    /// Dash match `[name|="value"]`.
    DashMatch,
}
