//! Union declaration nodes for the Valkyrie language AST.

use crate::ast::{Attribute, FieldDeclaration, GenericParam, Identifier, Span};

/// A named `union` declaration.
///
/// # V Language Example
/// ```v
/// union Integer {
///     lamb1: i64
///     lamb2: [i64; 2]
///     buffer: ArrayList<i64>
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnionDeclaration {
    /// The union name.
    pub name: Identifier,
    /// Generic parameters declared on the header.
    pub generics: Vec<GenericParam>,
    /// Annotations applied to the declaration.
    pub annotations: Vec<Attribute>,
    /// Overlapping field body.
    pub body: UnionBody,
    /// The source code span covering the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// Field body of a `union` declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnionBody {
    /// Overlapping union fields.
    pub fields: Vec<FieldDeclaration>,
    /// The source code span covering the body braces.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
