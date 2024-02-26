//! AST (Abstract Syntax Tree) module for the Valkyrie language.
//!
//! This module defines the syntax tree structure for the Valkyrie language, including:
//! - Common nodes (attributes, string literals, etc.)
//! - Item nodes (classes, enums, traits, etc.)
//! - Namespace nodes (namespaces, using declarations)
//! - Pattern nodes (patterns for matching)
//! - Root nodes (root AST structure, identifiers, etc.)
//! - Shader nodes (shader declarations)
//! - Statement nodes (let statements, expression statements)
//! - Structure nodes (class declarations, field declarations, etc.)
//! - Template nodes (template-related nodes)
//! - Term nodes (expressions, blocks, control flow, etc.)
//! - Trait nodes (trait declarations)
//! - Type nodes (type expressions, generic parameters, etc.)
//! - Widget nodes (widget declarations)

pub mod common_nodes;
pub mod ecs_nodes;
pub mod items_nodes;
pub mod namespace_nodes;
pub mod pattern_nodes;
pub mod root_nodes;
pub mod shader_nodes;
pub mod statement_nodes;
pub mod structure_nodes;
pub mod template_nodes;
pub mod term_nodes;
pub mod trait_nodes;
pub mod type_nodes;
pub mod widget_nodes;

pub use self::{
    common_nodes::{Attribute, AttributeArgument, EnumVariant, InterpolationSegment, Modifier, StringLiteral, StringSegment, TextSegment, VariantCase},
    ecs_nodes::{ComponentDeclaration, EventDeclaration, SystemDeclaration},
    items_nodes::{AssociatedType, Effect, Enums, Flags, ImplyDeclaration, MicroDeclaration, Parent, Property, PropertyKind, StatementNode, Trait, TypeFunction, Variant, WidgetDeclaration},
    namespace_nodes::{NamespaceDeclaration, UsingDeclaration},
    pattern_nodes::{ClassPattern, LiteralPattern, MatchArm, Pattern, TypePattern, VariablePattern, WildcardPattern},
    root_nodes::{EnumsKind, Identifier, LoopKind, NamePath, Span, ValkyrieRoot},
    shader_nodes::ShaderDeclaration,
    statement_nodes::{ExprStmt, Let, Statement},
    structure_nodes::{AnonymousClass, ClassDeclaration, FieldDeclaration, MethodDeclaration, SingletonDeclaration, StructureDeclaration},
    template_nodes::{TemplateControlNode, TemplateInterpolationNode, TemplateTextNode},
    term_nodes::{AnonymousMicro, Block, Break, Continue, Raise, Resume, Return, TermBinaryNode, TermExpression, TermUnaryNode},
    type_nodes::{GenericParam, Param, TypeExpression},
};
