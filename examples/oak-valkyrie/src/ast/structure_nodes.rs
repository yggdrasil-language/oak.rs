//! Structure nodes for the Valkyrie language AST.
//!
//! Value-type `struct` / `structure` declarations use [`StructureDeclaration`].
//! Anonymous `structure { ... }` literals use [`AnonymousStructure`].
//! Anonymous `class { ... }` literals use [`AnonymousClass`](crate::ast::class_nodes::AnonymousClass).

use crate::ast::{Attribute, Block, GenericParam, Identifier, Param, Parent, Span, TermExpression, TypeExpression, trait_nodes::AssociatedType};

/// Named value-type declaration keyword (`struct` / `structure`).
///
/// # V Language Example
/// ```v
/// struct LegacyPoint { x: f64, y: f64 }
/// structure Point { x: f64, y: f64 }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StructureKind {
    /// `struct <name>? { ... }` spelling (value type), deprecated.
    Struct,
    /// `structure <name>? { ... }` (value type).
    #[default]
    Structure,
}

/// A named `struct` / `structure` declaration.
///
/// # V Language Example
/// ```v
/// structure Point {
///     x: f64
///     y: f64
///
///     micro distance(self, other: Point) -> f64 {
///         let dx = other.x - self.x
///         let dy = other.y - self.y
///         return sqrt(dx * dx + dy * dy)
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructureDeclaration {
    /// Whether this item uses `struct` or `structure`.
    pub kind: StructureKind,
    /// The declared type name.
    pub name: Identifier,
    /// Annotations applied to the declaration.
    pub annotations: Vec<Attribute>,
    /// Generic parameters declared on the header.
    pub generics: Vec<GenericParam>,
    /// Parents listed on the header.
    pub parents: Vec<Parent>,
    /// Declaration body.
    pub body: StructureBody,
    /// The source code span covering the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An anonymous `structure { ... }` expression literal.
///
/// # V Language Example
/// ```v
/// let point = structure {
///     x: 1.0
///     y: 2.0
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnonymousStructure {
    /// Annotations applied to the literal.
    pub annotations: Vec<Attribute>,
    /// Generic parameters on the header, when present.
    pub generics: Vec<GenericParam>,
    /// Parents listed on the header, when present.
    pub parents: Vec<Parent>,
    /// Literal body.
    pub body: StructureBody,
    /// Variables captured from the enclosing scope.
    pub captures: Vec<Identifier>,
    /// The source code span covering the whole literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// Field/method body shared by class, structure, singleton, trait, and anonymous literals.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructureBody {
    /// Fields declared in the body.
    pub fields: Vec<FieldDeclaration>,
    /// Methods declared in the body.
    pub methods: Vec<MethodDeclaration>,
    /// Associated types declared in trait bodies.
    pub associated_types: Vec<AssociatedType>,
    /// The source code span covering the body braces.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A field declaration inside a class, structure, or singleton.
///
/// # V Language Example
/// ```v
/// class Packet {
///     id: i32
///     payload: utf8 = ""
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldDeclaration {
    /// Annotations applied to the field.
    pub annotations: Vec<Attribute>,
    /// The field name.
    pub name: Identifier,
    /// The field type.
    pub typing: TypeExpression,
    /// Optional default value expression.
    pub default: Option<TermExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A method declaration inside a class, structure, trait, singleton, or imply block.
///
/// # V Language Example
/// ```v
/// structure Rect {
///     width: f64
///     height: f64
///
///     micro area(self) -> f64 {
///         return self.width * self.height
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MethodDeclaration {
    /// The method name.
    pub name: Identifier,
    /// Generic parameters for the method.
    pub generics: Vec<GenericParam>,
    /// The method parameters.
    pub parameters: Vec<Param>,
    /// Optional return type annotation.
    pub return_type: Option<TypeExpression>,
    /// The optional method body.
    pub body: Option<Block>,
    /// Annotations applied to the method.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
