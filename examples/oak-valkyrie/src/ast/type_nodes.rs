//! Type nodes for the Valkyrie language AST.
//!
//! This module defines type expressions used in signatures, generics, casts,
//! and associated-type projections.

use super::{Identifier, NamePath, Span, TermExpression};
use crate::ValkyrieTokenType;

/// A type expression in the Valkyrie language.
///
/// Type expressions describe static types in signatures, generics, casts, and
/// associated-type projections.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeExpression {
    /// A binary type expression.
    ///
    /// # V Language Example
    /// ```v
    /// micro accept(value: i32 | f64) { }
    /// micro both(value: Send & Sync) { }
    /// ```
    Binary(Box<TypeBinaryNode>),
    /// A unary type expression.
    ///
    /// # V Language Example
    /// ```v
    /// micro store(values: [i32]) { }
    /// ```
    Unary(Box<TypeUnaryNode>),
    /// A generic type parameter reference.
    ///
    /// # V Language Example
    /// ```v
    /// micro identity<T>(value: T) -> T { return value }
    /// ```
    Generic(Box<GenericType>),
    /// A tuple type.
    ///
    /// # V Language Example
    /// ```v
    /// micro pair(): (i32, bool) { return (1, true) }
    /// ```
    Tuple(Box<TupleType>),
    /// A function type.
    ///
    /// # V Language Example
    /// ```v
    /// micro invoke(callback: micro(i32) -> i32) { }
    /// ```
    Function(Box<FunctionType>),
    /// A nullable type suffix.
    ///
    /// # V Language Example
    /// ```v
    /// micro maybe_name(): utf8? { return None }
    /// ```
    Optional(Box<OptionalType>),
    /// An associated type projection (e.g., `Self::Item`, `T::Output`).
    ///
    /// # V Language Example
    /// ```v
    /// trait Iterator {
    ///     type Item
    ///     micro next(self) -> Self::Item
    /// }
    /// ```
    AssociatedType(Box<AssociatedType>),
    /// A qualified associated type (e.g., `<T as Trait>::Item`).
    ///
    /// # V Language Example
    /// ```v
    /// micro read<T>(value: T) -> <T as Iterator>::Item { }
    /// ```
    QualifiedAssociatedType(Box<QualifiedAssociatedType>),
    /// A name path type (e.g., `std::collections::HashMap`).
    ///
    /// # V Language Example
    /// ```v
    /// micro count(): i32 { return 0 }
    /// ```
    Namepath(Box<NamePath>),
    /// A generic type application.
    ///
    /// Mirrors term-level [`TermExpression::Turbofish`], but for type syntax such
    /// as `Option<T>` and `Result<T, E>`.
    ///
    /// # V Language Example
    /// ```v
    /// micro take(value: Option<i32>) { }
    /// micro parse(): Result<utf8, i32> { return Fine("") }
    /// micro nested(): Option<Option<i32>> { return None }
    /// ```
    Apply(Box<ApplyType>),
}

/// A generic type parameter reference node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericType {
    /// The generic parameter name.
    pub name: Identifier,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A tuple type node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleType {
    /// The element types.
    pub elements: Vec<TypeExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A function type node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionType {
    /// The parameter types.
    pub params: Vec<TypeExpression>,
    /// The return type.
    pub return_type: Box<TypeExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A generic type application node.
///
/// Represents `Constructor<Args...>` in type position, such as `Option<i32>`
/// or `Result<utf8, i32>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplyType {
    /// The applied type constructor.
    pub base: TypeExpression,
    /// Generic arguments in source order.
    pub arguments: Vec<TypeExpression>,
    /// The source code span covering the full application.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A nullable type node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionalType {
    /// The inner type.
    pub inner: Box<TypeExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An associated type projection (e.g., `Self::Item`, `T::Output`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssociatedType {
    /// The base type (e.g., `Self` or a type parameter name).
    pub base: Identifier,
    /// The associated type name.
    pub name: Identifier,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A qualified associated type (e.g., `<T as Trait>::Item`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QualifiedAssociatedType {
    /// The type being projected from.
    pub ty: Box<TypeExpression>,
    /// The trait providing the associated type.
    pub trait_path: NamePath,
    /// The associated type name.
    pub name: Identifier,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A unary type expression node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeUnaryNode {
    /// The unary operator.
    pub operator: ValkyrieTokenType,
    /// The operand expression.
    pub base: TypeExpression,
    /// Fixed array length for `[T; N]` spellings.
    pub length: Option<TermExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A binary type expression node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeBinaryNode {
    /// The binary operator.
    pub operator: ValkyrieTokenType,
    /// The left operand.
    pub lhs: TypeExpression,
    /// The right operand.
    pub rhs: TypeExpression,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A generic parameter declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericParam {
    /// The generic parameter name.
    pub name: Identifier,
    /// Type constraints (bounds) for the generic parameter.
    pub constraints: Vec<TypeExpression>,
    /// Default type for the generic parameter.
    pub default: Option<TypeExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A function parameter declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Param {
    /// The parameter name.
    pub name: Identifier,
    /// Optional type annotation.
    pub ty: Option<TypeExpression>,
    /// Optional default value expression.
    pub default: Option<TermExpression>,
    /// Modifiers such as `mut` lowered from [`Modifier`] nodes.
    pub annotations: Vec<super::Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
