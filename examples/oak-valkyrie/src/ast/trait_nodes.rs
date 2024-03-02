//! Trait declaration nodes for the Valkyrie language AST.
//!
//! This module defines `trait` declarations, associated types, and `imply` blocks.

use crate::ast::{Attribute, GenericParam, Identifier, MethodDeclaration, Span, StructureBody, TypeExpression};

/// A trait declaration.
///
/// # V Language Example
/// ```v
/// trait Text {
///     type View: TextView<Text = Self>
///     micro is_empty(self) -> bool
///     micro view(self, span: TextSpan) -> Self::View
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trait {
    /// The trait name.
    pub name: Identifier,
    /// Generic parameters for the trait.
    pub generics: Vec<GenericParam>,
    /// Annotations applied to the trait.
    pub annotations: Vec<Attribute>,
    /// Trait body (`type` members and `micro` methods).
    pub body: StructureBody,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An associated type declaration in a trait.
///
/// # V Language Example
/// ```v
/// trait Iterator {
///     type Item
///     micro next(self) -> Option<Self::Item>
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssociatedType {
    /// The associated type name.
    pub name: Identifier,
    /// Type bounds that the associated type must satisfy.
    pub bounds: Vec<TypeExpression>,
    /// Default type for the associated type, if any.
    pub default: Option<TypeExpression>,
    /// Annotations applied to the associated type.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An `imply` block attaching methods to a type or trait witness.
///
/// # V Language Example
/// ```v
/// imply Utf8Text: Text {
///     micro is_empty(self) -> bool {
///         return self.byte_length() == 0
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImplyDeclaration {
    /// Annotations applied to the imply block.
    pub annotations: Vec<Attribute>,
    /// Generic parameters declared on the imply header.
    pub generics: Vec<GenericParam>,
    /// Target type being extended or implemented.
    pub target_type: TypeExpression,
    /// Optional trait or protocol being implemented.
    pub trait_type: Option<TypeExpression>,
    /// Methods defined in the imply block.
    pub methods: Vec<MethodDeclaration>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
