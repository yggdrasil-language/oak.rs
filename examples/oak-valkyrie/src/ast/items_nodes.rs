use super::*;
use crate::ast::{
    ecs_nodes::{ComponentDeclaration, SystemDeclaration},
    statement_nodes::{ExprStmt, Let},
    structure_nodes::SingletonDeclaration,
};

/// A root node item in a Valkyrie module
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
}

/// A parent class with optional alias for renamed inheritance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Parent {
    /// Optional alias for disambiguation (e.g., "primary" in "primary: Parent1").
    pub alias: Option<Identifier>,
    /// Parent class name path.
    pub name: NamePath,
    /// Source span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A flags (bitflags) declaration
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

/// An enum declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Enums {
    /// The keyword kind used for this enum (enums, enum, unity).
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

/// A trait declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trait {
    /// The trait name.
    pub name: Identifier,
    /// Generic parameters for the trait.
    pub generics: Vec<GenericParam>,
    /// Methods declared in the trait.
    pub methods: Vec<MethodDeclaration>,
    /// Associated types declared in the trait.
    pub associated_types: Vec<AssociatedType>,
    /// Annotations applied to the trait.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An associated type declaration in a trait.
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

/// A widget declaration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WidgetDeclaration {
    /// The widget name.
    pub name: Identifier,
    /// Generic parameters for the widget.
    pub generics: Vec<GenericParam>,
    /// Items (properties, methods) declared within the widget.
    pub items: Vec<StatementNode>,
    /// Annotations applied to the widget.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An `imply` block attaching methods to a type or trait witness.
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

/// A micro (small function) declaration
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

/// A type function declaration
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

/// A variant declaration
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

/// An effect declaration
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
    /// Parameters for the property (self for getter, self + value for setter).
    pub params: Vec<Param>,
    /// Return type for getter, None for setter.
    pub return_type: Option<TypeExpression>,
    /// The body of the property.
    pub body: Block,
    /// Source span.
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
