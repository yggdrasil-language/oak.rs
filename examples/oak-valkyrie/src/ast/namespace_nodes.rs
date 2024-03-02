//! Namespace and import nodes for the Valkyrie language AST.

use crate::ast::{Attribute, Identifier, NamePath, Span, items_nodes::StatementNode};

/// A namespace declaration.
///
/// # V Language Example
/// ```v
/// namespace core::text;
///
/// namespace game::player {
///     micro spawn() -> Player { }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamespaceDeclaration {
    /// Annotations applied to the namespace.
    pub annotations: Vec<Attribute>,
    /// The name path of the namespace.
    pub name: NamePath,
    /// Items declared within the namespace.
    pub items: Vec<StatementNode>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A using (import) statement.
///
/// # V Language Example
/// ```v
/// using core::primitive::{Never, Unit}
/// using core::primitive::Bool as bool
/// using std::text::utf8
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UsingDeclaration {
    /// The path to import.
    pub path: NamePath,
    /// Optional alias for the import.
    pub alias: Option<Identifier>,
    /// Selective import list (e.g., `{Never, Unit}` in `using core::primitive.{Never, Unit}`)
    pub imports: Vec<Identifier>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
