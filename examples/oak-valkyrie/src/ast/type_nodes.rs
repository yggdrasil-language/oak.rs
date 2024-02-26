use super::{Identifier, NamePath, Span, TermExpression};
use crate::ValkyrieTokenType;

/// A type expression
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypeExpression {
    /// A binary operation expression.
    Binary(Box<TypeBinaryNode>),
    /// A unary operation expression.
    Unary(Box<TypeUnaryNode>),
    /// A generic type parameter.
    Generic(Box<GenericType>),
    /// A tuple type.
    Tuple(Box<TupleType>),
    /// A function type.
    Function(Box<FunctionType>),
    /// An optional type.
    Optional(Box<OptionalType>),
    /// An associated type projection (e.g., `Self::Item`, `T::Output`).
    AssociatedType(Box<AssociatedType>),
    /// A qualified associated type (e.g., `<T as Trait>::Item`).
    QualifiedAssociatedType(Box<QualifiedAssociatedType>),
    /// A name path expression (e.g., `std::collections::HashMap`).
    Namepath(Box<NamePath>),
}

/// A generic type parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericType {
    /// The generic parameter name.
    pub name: Identifier,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A tuple type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleType {
    /// The element types.
    pub elements: Vec<TypeExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A function type.
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

/// An optional type.
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeUnaryNode {
    /// The unary operator.
    pub operator: ValkyrieTokenType,
    /// The operand expression.
    pub base: TypeExpression,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

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

/// A generic parameter
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

/// A function parameter
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
