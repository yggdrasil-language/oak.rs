#![doc = include_str!("readme.md")]
/// Class declaration nodes.
pub mod class_nodes;
/// Shared AST nodes such as attributes, literals, and modifiers.
pub mod common_nodes;
/// ECS declaration nodes for components, events, and systems.
pub mod ecs_nodes;
/// Top-level item nodes such as classes, enums, traits, and micros.
pub mod items_nodes;
/// Namespace and using declaration nodes.
pub mod namespace_nodes;
/// Pattern nodes used by match and binding forms.
pub mod pattern_nodes;
/// Root AST nodes such as identifiers, name paths, and the program root.
pub mod root_nodes;
/// Shader declaration nodes.
pub mod shader_nodes;
/// Singleton declaration nodes.
pub mod singleton_nodes;
/// Statement nodes such as `let` and expression statements.
pub mod statement_nodes;
/// Structure declaration nodes for classes, fields, and methods.
pub mod structure_nodes;
/// Template nodes used by metaprogramming constructs.
pub mod template_nodes;
/// Term expression and control-flow nodes.
pub mod term_nodes;
/// Trait declaration nodes.
pub mod trait_nodes;
/// Type expression and generic parameter nodes.
pub mod type_nodes;
/// Union declaration nodes.
pub mod union_nodes;
/// Widget declaration nodes.
pub mod widget_nodes;

pub use self::{
    class_nodes::{AnonymousClass, ClassDeclaration},
    common_nodes::{Attribute, AttributeArgument, EnumVariant, InterpolationSegment, Modifier, StringLiteral, StringSegment, TextSegment, VariantCase},
    ecs_nodes::{ComponentDeclaration, EventDeclaration, SystemDeclaration},
    items_nodes::{Effect, Enums, Flags, MicroDeclaration, Parent, Property, PropertyKind, StatementNode, TypeFunction, Variant},
    namespace_nodes::{NamespaceDeclaration, UsingDeclaration},
    pattern_nodes::{ClassPattern, LiteralPattern, MatchArm, Pattern, TypePattern, VariablePattern, WildcardPattern},
    root_nodes::{EnumsKind, Identifier, LoopKind, NamePath, Span, ValkyrieRoot},
    shader_nodes::ShaderDeclaration,
    singleton_nodes::SingletonDeclaration,
    statement_nodes::{ExprStmt, Let, Statement},
    structure_nodes::{AnonymousStructure, FieldDeclaration, MethodDeclaration, StructureBody, StructureDeclaration, StructureKind},
    template_nodes::{TemplateIf, TemplateIfArm, TemplateLoop, TemplateMatch, TemplateMatchArm, TemplateNode, TemplateTokenStream},
    term_nodes::{AnonymousMicro, Block, Break, Continue, Raise, Resume, Return, TermBinaryNode, TermExpression, TermUnaryNode},
    trait_nodes::{AssociatedType, ImplyDeclaration, Trait},
    type_nodes::{ApplyType, GenericParam, Param, TypeExpression},
    union_nodes::{UnionBody, UnionDeclaration},
    widget_nodes::WidgetDeclaration,
};
