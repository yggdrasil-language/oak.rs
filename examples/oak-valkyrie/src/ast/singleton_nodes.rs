//! Singleton declaration nodes for the Valkyrie language AST.

use crate::ast::{Attribute, GenericParam, Identifier, Parent, Span, StructureBody};

/// A named `singleton` declaration.
///
/// # V Language Example
/// ```v
/// singleton Config {
///     port: i32 = 8080
///
///     initiate(mut self) {
///         self.port = 8080
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SingletonDeclaration {
    /// The declared singleton name.
    pub name: Identifier,
    /// Annotations applied to the declaration.
    pub annotations: Vec<Attribute>,
    /// Generic parameters declared on the header.
    pub generics: Vec<GenericParam>,
    /// Parents listed on the header.
    pub parents: Vec<Parent>,
    /// Singleton body.
    pub body: StructureBody,
    /// The source code span covering the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
