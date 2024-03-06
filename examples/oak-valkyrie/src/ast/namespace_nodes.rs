//! Namespace and import nodes for the Valkyrie language AST.

use crate::{
    ValkyrieRoot,
    ast::{Attribute, Identifier, NamePath, Span, items_nodes::StatementNode, root_nodes::ProgramBody},
};

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
    /// 该 namespace 是否有 body。
    ///
    /// ```v
    /// namespace name {} -> Some([])
    /// namespace name -> None;
    /// ```
    pub body: Option<ProgramBody>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

impl NamespaceDeclaration {
    /// 是否为花括号作用域；空 body 仍然是作用域，不能被当成分号式指令。
    pub fn has_body(&self) -> bool {
        self.body.is_some()
    }
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
