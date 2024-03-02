//! Class declaration nodes for the Valkyrie language AST.

use crate::ast::{Attribute, GenericParam, Identifier, Parent, Span, StructureBody};

/// A named `class` declaration.
///
/// # V Language Example
/// ```v
/// class Player {
///     name: utf8
///
///     micro greet(self) -> utf8 {
///         return "hello"
///     }
/// }
///
/// class Derived(Base) {
///     micro speak(self) -> utf8 { "woof" }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassDeclaration {
    /// The declared class name.
    pub name: Identifier,
    /// Annotations applied to the declaration.
    pub annotations: Vec<Attribute>,
    /// Generic parameters declared on the header.
    pub generics: Vec<GenericParam>,
    /// Parents listed on the header (`class Derived(Base)`).
    pub parents: Vec<Parent>,
    /// Class body.
    pub body: StructureBody,
    /// The source code span covering the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An anonymous `class { ... }` expression literal.
///
/// # V Language Example
/// ```v
/// let counter = class {
///     value: i32 = start
///
///     micro next(self) -> i32 {
///         self.value = self.value + 1
///         return self.value
///     }
/// }
///
/// let impl_trait = class: Drawable { radius: 1.0 }
/// let derived = class(Animal) { micro speak(self) -> utf8 { "woof" } }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnonymousClass {
    /// Annotations applied to the literal.
    pub annotations: Vec<Attribute>,
    /// Generic parameters on the header, when present.
    pub generics: Vec<GenericParam>,
    /// Parents (`class(Base)` / `class: Trait` lowered to parent entries).
    pub parents: Vec<Parent>,
    /// Literal body.
    pub body: StructureBody,
    /// Variables captured from the enclosing scope.
    pub captures: Vec<Identifier>,
    /// The source code span covering the whole literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
