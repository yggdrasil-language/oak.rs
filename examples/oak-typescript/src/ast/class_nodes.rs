use crate::ast::{Decorator, Expression, FunctionParam, Statement, TypeAnnotation, TypeParameter};
use core::range::Range;

/// Represents the visibility of a class member.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Visibility {
    /// Public visibility.
    Public,
    /// Private visibility.
    Private,
    /// Protected visibility.
    Protected,
}

/// Represents a member of a class.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClassMember {
    /// A property member.
    Property {
        /// Decorators associated with the property.
        decorators: Vec<Decorator>,
        /// Name of the property.
        name: String,
        /// Type annotation of the property.
        ty: Option<TypeAnnotation>,
        /// Initializer expression of the property.
        initializer: Option<Expression>,
        /// Visibility of the property.
        visibility: Option<Visibility>,
        /// Whether the property is static.
        is_static: bool,
        /// Whether the property is readonly.
        is_readonly: bool,
        /// Whether the property is abstract.
        is_abstract: bool,
        /// Whether the property is optional.
        is_optional: bool,
        /// Source span of the property.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// A method member.
    Method {
        /// Decorators associated with the method.
        decorators: Vec<Decorator>,
        /// Name of the method.
        name: String,
        /// Type parameters of the method.
        type_params: Vec<TypeParameter>,
        /// Parameters of the method.
        params: Vec<FunctionParam>,
        /// Return type of the method.
        return_type: Option<TypeAnnotation>,
        /// Body of the method.
        body: Vec<Statement>,
        /// Visibility of the method.
        visibility: Option<Visibility>,
        /// Whether the method is static.
        is_static: bool,
        /// Whether the method is abstract.
        is_abstract: bool,
        /// Whether the method is `async`.
        is_async: bool,
        /// Whether the method is a getter.
        is_getter: bool,
        /// Whether the method is a setter.
        is_setter: bool,
        /// Whether the method is optional.
        is_optional: bool,
        /// Source span of the method.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
}
