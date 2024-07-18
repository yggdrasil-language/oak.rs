use crate::ast::{Decorator, Expression, FunctionParam, Modifier, ModifierKind, OptionalMarker, Statement, TypeAnnotation, TypeParameter, has_modifier};
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
        /// Keyword modifiers written before the property (`static`, `readonly`, `abstract`).
        modifiers: Vec<Modifier>,
        /// `?` marker written after the property name.
        optional_marker: Option<OptionalMarker>,
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
        /// Keyword modifiers written before the method (`static`, `abstract`, `async`, `get`, `set`).
        modifiers: Vec<Modifier>,
        /// `?` marker written after the method name.
        optional_marker: Option<OptionalMarker>,
        /// Source span of the method.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
}

impl ClassMember {
    /// Returns `true` when the member carries `kind` among its keyword modifiers.
    pub fn has_modifier(&self, kind: ModifierKind) -> bool {
        match self {
            ClassMember::Property { modifiers, .. } | ClassMember::Method { modifiers, .. } => has_modifier(modifiers, kind),
        }
    }

    /// Returns `true` when the member is marked optional with `?`.
    pub fn is_optional(&self) -> bool {
        match self {
            ClassMember::Property { optional_marker, .. } | ClassMember::Method { optional_marker, .. } => optional_marker.is_some(),
        }
    }

    /// Returns `true` when the member carries a `static` modifier.
    pub fn is_static(&self) -> bool {
        self.has_modifier(ModifierKind::Static)
    }

    /// Returns `true` when the member carries a `readonly` modifier.
    pub fn is_readonly(&self) -> bool {
        self.has_modifier(ModifierKind::Readonly)
    }

    /// Returns `true` when the member carries an `abstract` modifier.
    pub fn is_abstract(&self) -> bool {
        self.has_modifier(ModifierKind::Abstract)
    }

    /// Returns `true` when the member carries an `async` modifier.
    pub fn is_async(&self) -> bool {
        self.has_modifier(ModifierKind::Async)
    }

    /// Returns `true` when the member carries a `get` modifier.
    pub fn is_getter(&self) -> bool {
        self.has_modifier(ModifierKind::Get)
    }

    /// Returns `true` when the member carries a `set` modifier.
    pub fn is_setter(&self) -> bool {
        self.has_modifier(ModifierKind::Set)
    }
}
