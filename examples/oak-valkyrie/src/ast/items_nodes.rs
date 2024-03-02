//! Top-level item nodes for the Valkyrie language AST.
//!
//! This module defines module-level declarations such as classes, traits,
//! enums, micros, imply blocks, and the `StatementNode` root item enum.

use super::*;
use crate::ast::{
    ecs_nodes::{ComponentDeclaration, SystemDeclaration},
    statement_nodes::{ExprStmt, Let},
    singleton_nodes::SingletonDeclaration,
    template_nodes::TemplateNode,
    trait_nodes::{AssociatedType, ImplyDeclaration, Trait},
    union_nodes::UnionDeclaration,
    widget_nodes::WidgetDeclaration,
};

/// A top-level item in a Valkyrie module.
///
/// Each variant corresponds to one syntactic form that may appear at module
/// scope or inside a namespace.
///
/// # V Language Example
/// ```v
/// namespace game::player {
///     structure Vec2 { x: f64, y: f64 }
///
///     trait Movable {
///         micro move_by(self, delta: Vec2)
///     }
///
///     micro spawn(name: utf8) -> Player {
///         return Player { name: name, position: Vec2 { x: 0.0, y: 0.0 } }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatementNode {
    /// A namespace declaration.
    Namespace(Box<NamespaceDeclaration>),
    /// A using (import) statement.
    Using(Box<UsingDeclaration>),
    /// A class declaration.
    Class(Box<ClassDeclaration>),
    /// A value type structure (immutable, copied on assignment).
    Structure(Box<StructureDeclaration>),
    /// A named union declaration.
    Union(Box<UnionDeclaration>),
    /// A singleton declaration.
    Singleton(Box<SingletonDeclaration>),
    /// A flags (bitflags) declaration.
    Flags(Box<Flags>),
    /// An enum declaration.
    Enums(Box<Enums>),
    /// A trait declaration.
    Trait(Box<Trait>),
    /// A widget declaration.
    Widget(Box<WidgetDeclaration>),
    /// An `imply` implementation block.
    Imply(Box<ImplyDeclaration>),
    /// A micro (small function) declaration.
    Micro(Box<MicroDeclaration>),
    /// A type function declaration.
    TypeFunction(Box<TypeFunction>),
    /// A statement at module level.
    Statement(Box<StatementNode>),
    /// A variant declaration.
    Variant(Box<Variant>),
    /// An effect declaration.
    Effect(Box<Effect>),
    /// A property declaration (getter or setter).
    Property(Box<Property>),
    /// A shader declaration.
    Shader(Box<ShaderDeclaration>),
    /// A component declaration for ECS.
    Component(Box<ComponentDeclaration>),
    /// A system declaration for ECS.
    System(Box<SystemDeclaration>),

    /// A let binding statement.
    Let(Box<Let>),
    /// An expression statement.
    ExprStmt(Box<ExprStmt>),
    /// A TGrammar template node.
    Template(Box<TemplateNode>),
}

/// A parent class or trait with optional alias for renamed inheritance.
///
/// # V Language Example
/// ```v
/// class Sprite implements primary: Drawable, secondary: Updatable {
///     micro draw(self) { primary.draw(self) }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parent {
    /// Optional alias for disambiguation (e.g., `primary` in `primary: Drawable`).
    pub alias: Option<Identifier>,
    /// Parent class or trait name path.
    pub name: NamePath,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A flags (bitflags) declaration.
///
/// # V Language Example
/// ```v
/// flags FileMode {
///     Read = 1
///     Write = 2
///     Execute = 4
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Flags {
    /// The flags name.
    pub name: Identifier,
    /// The flag variants.
    pub variants: Vec<EnumVariant>,
    /// Annotations applied to the flags.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An enum declaration.
///
/// # V Language Example
/// ```v
/// enums Direction {
///     North
///     East
///     South
///     West
/// }
///
/// unite Option<T> {
///     Some { value: T }
///     None
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Enums {
    /// The keyword kind used for this enum (`enums`, `enum`, or `unity`).
    pub kind: EnumsKind,
    /// The enum name.
    pub name: Identifier,
    /// Generic parameters for the enum.
    pub generics: Vec<GenericParam>,
    /// The enum variants.
    pub variants: Vec<EnumVariant>,
    /// Annotations applied to the enum.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A micro (small function) declaration.
///
/// # V Language Example
/// ```v
/// micro add(left: i32, right: i32) -> i32 {
///     return left + right
/// }
///
/// micro<T> identity(value: T) -> T {
///     return value
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MicroDeclaration {
    /// The micro name.
    pub name: Identifier,
    /// Generic parameters for the micro.
    pub generics: Vec<GenericParam>,
    /// Parameters for the micro.
    pub params: Vec<Param>,
    /// Return type annotation, if any.
    pub return_type: Option<TypeExpression>,
    /// The body of the micro.
    pub body: Block,
    /// Annotations applied to the micro.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
    /// Whether this function is abstract (has no body implementation).
    pub is_abstract: bool,
    /// Whether this function is final (cannot be overridden).
    pub is_final: bool,
}

/// A type function declaration.
///
/// # V Language Example
/// ```v
/// mezzo Pair<A, B>(left: A, right: B) -> (A, B) {
///     return (left, right)
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeFunction {
    /// The type function name.
    pub name: Identifier,
    /// Generic parameters for the type function.
    pub generics: Vec<GenericParam>,
    /// Parameters for the type function.
    pub params: Vec<Param>,
    /// Return type annotation, if any.
    pub return_type: Option<TypeExpression>,
    /// The body of the type function.
    pub body: Block,
    /// Annotations applied to the type function.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A variant declaration.
///
/// # V Language Example
/// ```v
/// variant HttpStatus {
///     Ok { code: i32 }
///     Redirect { code: i32, location: utf8 }
///     Error { code: i32, message: utf8 }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Variant {
    /// The variant name.
    pub name: Identifier,
    /// Generic parameters for the variant.
    pub generics: Vec<GenericParam>,
    /// The variant cases.
    pub cases: Vec<VariantCase>,
    /// Annotations applied to the variant.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An effect declaration.
///
/// # V Language Example
/// ```v
/// effect Console {
///     micro print_line(message: utf8)
///     micro read_line() -> utf8
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Effect {
    /// The effect name.
    pub name: Identifier,
    /// Operations defined by the effect.
    pub operations: Vec<MethodDeclaration>,
    /// Annotations applied to the effect.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// The kind of a property (getter or setter).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PropertyKind {
    /// A getter property.
    Getter,
    /// A setter property.
    Setter,
}

/// A property declaration (getter or setter).
///
/// # V Language Example
/// ```v
/// class Temperature {
///     _celsius: f64 = 0.0
///
///     property celsius: f64 {
///         get { return self._celsius }
///         set(value) { self._celsius = value }
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Property {
    /// The name of the property.
    pub name: Identifier,
    /// Whether this is a getter or setter.
    pub kind: PropertyKind,
    /// Generic parameters for the property.
    pub generics: Vec<GenericParam>,
    /// Annotations on the property.
    pub annotations: Vec<Attribute>,
    /// Parameters for the property (`self` for getter, `self` + value for setter).
    pub params: Vec<Param>,
    /// Return type for getter, `None` for setter.
    pub return_type: Option<TypeExpression>,
    /// The body of the property.
    pub body: Block,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
    /// Whether this property is abstract (has no body implementation).
    ///
    /// Abstract properties are declared without a body in abstract classes
    /// and must be implemented by concrete subclasses.
    pub is_abstract: bool,
    /// Whether this property is final (cannot be overridden).
    pub is_final: bool,
    /// Whether this property is static (belongs to the class, not instances).
    ///
    /// Static properties are accessed via `ClassName.property_name` syntax
    /// and do not have access to `self`.
    pub is_static: bool,
    /// Whether this property is virtual (can be overridden by subclasses).
    ///
    /// Virtual properties use dynamic dispatch through the vtable,
    /// allowing subclasses to provide their own implementation.
    pub is_virtual: bool,
    /// Whether this property overrides a parent class property.
    ///
    /// Override properties must match the signature of the parent property
    /// and are verified during type checking.
    pub is_override: bool,
    /// Whether this property uses lazy initialization.
    ///
    /// Lazy properties cache their computed value after first access.
    /// The getter is only called once, and subsequent accesses return
    /// the cached value.
    pub is_lazy: bool,
}
