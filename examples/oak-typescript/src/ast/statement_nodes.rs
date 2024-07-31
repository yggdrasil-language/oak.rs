use crate::ast::{ClassMember, Decorator, EnumMember, Expression, Modifier, ModifierKind, OptionalMarker, TypeAnnotation, TypeParameter, has_modifier};
use core::range::Range;

/// Represents a TypeScript statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Statement {
    /// Variable declaration.
    VariableDeclaration(VariableDeclaration),
    /// Function declaration.
    FunctionDeclaration(FunctionDeclaration),
    /// Class declaration.
    ClassDeclaration(ClassDeclaration),
    /// Expression statement.
    ExpressionStatement(ExpressionStatement),
    /// Import declaration.
    ImportDeclaration(ImportDeclaration),
    /// Export declaration.
    ExportDeclaration(ExportDeclaration),
    /// Interface declaration.
    Interface(InterfaceDeclaration),
    /// Type alias declaration.
    TypeAlias(TypeAliasDeclaration),
    /// Enum declaration.
    Enum(EnumDeclaration),
    /// Return statement.
    ReturnStatement(ReturnStatement),
    /// If statement.
    IfStatement(IfStatement),
    /// While statement.
    WhileStatement(WhileStatement),
    /// Do-while statement.
    DoWhileStatement(DoWhileStatement),
    /// For statement.
    ForStatement(ForStatement),
    /// For-in statement.
    ForInStatement(ForInStatement),
    /// For-of statement.
    ForOfStatement(ForOfStatement),
    /// Switch statement.
    SwitchStatement(SwitchStatement),
    /// Try statement.
    TryStatement(TryStatement),
    /// Throw statement.
    ThrowStatement(ThrowStatement),
    /// Break statement.
    BreakStatement(BreakStatement),
    /// Continue statement.
    ContinueStatement(ContinueStatement),
    /// Block statement.
    BlockStatement(BlockStatement),
    /// Namespace declaration.
    Namespace(NamespaceDeclaration),
}

impl Statement {
    /// Gets the span of the statement.
    pub fn span(&self) -> Range<usize> {
        match self {
            Statement::VariableDeclaration(d) => d.span.clone(),
            Statement::FunctionDeclaration(d) => d.span.clone(),
            Statement::ClassDeclaration(d) => d.span.clone(),
            Statement::ExpressionStatement(s) => s.span.clone(),
            Statement::ImportDeclaration(d) => d.span.clone(),
            Statement::ExportDeclaration(d) => d.span.clone(),
            Statement::Interface(d) => d.span.clone(),
            Statement::TypeAlias(d) => d.span.clone(),
            Statement::Enum(d) => d.span.clone(),
            Statement::ReturnStatement(s) => s.span.clone(),
            Statement::IfStatement(s) => s.span.clone(),
            Statement::WhileStatement(s) => s.span.clone(),
            Statement::DoWhileStatement(s) => s.span.clone(),
            Statement::ForStatement(s) => s.span.clone(),
            Statement::ForInStatement(s) => s.span.clone(),
            Statement::ForOfStatement(s) => s.span.clone(),
            Statement::SwitchStatement(s) => s.span.clone(),
            Statement::TryStatement(s) => s.span.clone(),
            Statement::ThrowStatement(s) => s.span.clone(),
            Statement::BreakStatement(s) => s.span.clone(),
            Statement::ContinueStatement(s) => s.span.clone(),
            Statement::BlockStatement(s) => s.span.clone(),
            Statement::Namespace(s) => s.span.clone(),
        }
    }
}

/// Represents an expression statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The expression being evaluated.
    pub expression: Expression,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a return statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReturnStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The expression being returned, if any.
    pub argument: Option<Expression>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a throw statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ThrowStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The expression being thrown.
    pub argument: Expression,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a break statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BreakStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The label being broken to, if any.
    pub label: Option<String>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a continue statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContinueStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The label being continued to, if any.
    pub label: Option<String>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a namespace declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamespaceDeclaration {
    /// Decorators associated with the declaration.
    pub decorators: Vec<Decorator>,
    /// Keyword modifiers written before the declaration (`declare`).
    pub modifiers: Vec<Modifier>,
    /// The name of the namespace.
    pub name: String,
    /// The statements within the namespace.
    pub body: Vec<Statement>,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents an interface declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InterfaceDeclaration {
    /// Decorators associated with the declaration.
    pub decorators: Vec<Decorator>,
    /// Keyword modifiers written before the declaration (`declare`).
    pub modifiers: Vec<Modifier>,
    /// The name of the interface.
    pub name: String,
    /// Type parameters of the interface.
    pub type_params: Vec<TypeParameter>,
    /// Types that this interface extends.
    pub extends: Vec<TypeAnnotation>,
    /// Members of the interface.
    pub body: Vec<ClassMember>,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a type alias declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeAliasDeclaration {
    /// Decorators associated with the declaration.
    pub decorators: Vec<Decorator>,
    /// Keyword modifiers written before the declaration (`declare`).
    pub modifiers: Vec<Modifier>,
    /// The name of the type alias.
    pub name: String,
    /// Type parameters of the type alias.
    pub type_params: Vec<TypeParameter>,
    /// The type being aliased.
    pub ty: TypeAnnotation,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents an enum declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnumDeclaration {
    /// Decorators associated with the declaration.
    pub decorators: Vec<Decorator>,
    /// Keyword modifiers written before the declaration (`declare`).
    pub modifiers: Vec<Modifier>,
    /// The name of the enum.
    pub name: String,
    /// Members of the enum.
    pub members: Vec<EnumMember>,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents an if statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The condition being tested.
    pub test: Expression,
    /// The statement to execute if the condition is true.
    pub consequent: Box<Statement>,
    /// The statement to execute if the condition is false.
    pub alternate: Option<Box<Statement>>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a while statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhileStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The condition being tested.
    pub test: Expression,
    /// The statement to execute as long as the condition is true.
    pub body: Box<Statement>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a do-while statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DoWhileStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The statement to execute at least once.
    pub body: Box<Statement>,
    /// The condition being tested after each iteration.
    pub test: Expression,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a for statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The initialization statement.
    pub initializer: Option<Box<Statement>>,
    /// The condition being tested before each iteration.
    pub test: Option<Expression>,
    /// The expression being evaluated after each iteration.
    pub incrementor: Option<Expression>,
    /// The statement to execute in each iteration.
    pub body: Box<Statement>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a for-in statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForInStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The left-hand side of the for-in loop.
    pub left: Box<Statement>,
    /// The expression being iterated over.
    pub right: Expression,
    /// The statement to execute in each iteration.
    pub body: Box<Statement>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a for-of statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForOfStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The `await` keyword marker written after `for` in `for await`.
    pub await_marker: Option<Modifier>,
    /// The left-hand side of the for-of loop.
    pub left: Box<Statement>,
    /// The expression being iterated over.
    pub right: Expression,
    /// The statement to execute in each iteration.
    pub body: Box<Statement>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ForOfStatement {
    /// Returns `true` when the loop is written as `for await`.
    pub fn is_await(&self) -> bool {
        self.await_marker.is_some()
    }
}

/// Represents a switch statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SwitchStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The expression being switched on.
    pub discriminant: Expression,
    /// Cases within the switch statement.
    pub cases: Vec<SwitchCase>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a case within a switch statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SwitchCase {
    /// The condition for this case. `None` for the default case.
    pub test: Option<Expression>,
    /// The statements to execute if the condition matches.
    pub consequent: Vec<Statement>,
    /// Source span of the case.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a try statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TryStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The try block.
    pub block: Vec<Statement>,
    /// The catch clause.
    pub handler: Option<CatchClause>,
    /// The finally block.
    pub finalizer: Option<Vec<Statement>>,
    /// Source span of the statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a catch clause within a try statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CatchClause {
    /// The parameter of the catch clause.
    pub param: Option<String>,
    /// The catch block.
    pub body: Vec<Statement>,
    /// Source span of the catch clause.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a block statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// The statements within the block.
    pub statements: Vec<Statement>,
    /// Source span of the block statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A module specifier string literal written in an import or export declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModuleSpecifier {
    /// The specifier text without its surrounding quotes.
    pub value: String,
    /// Source span of the string literal token, quotes included.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents an import specifier.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportSpecifier {
    /// Default import: `import local from "source"`.
    Default {
        /// Local name.
        local: String,
        /// Source span of the binding token.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// Namespace import: `import * as local from "source"`.
    Namespace {
        /// Local name bound to the module namespace.
        local: String,
        /// Source span of the `* as local` clause.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// Named import: `import { imported as local } from "source"`.
    Named {
        /// Local name.
        local: String,
        /// Imported name.
        imported: String,
        /// `type` keyword written before this specifier, e.g. `{ type Foo }`.
        type_modifier: Option<Modifier>,
        /// Source span of the whole specifier.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
}

impl ImportSpecifier {
    /// Returns `true` when this specifier carries an inline `type` keyword.
    ///
    /// Declaration-level `import type …` is recorded on
    /// [`ImportDeclaration::is_type_only`], not here.
    pub fn is_inline_type_only(&self) -> bool {
        matches!(self, ImportSpecifier::Named { type_modifier: Some(_), .. })
    }

    /// Returns the source span of the whole specifier.
    pub fn span(&self) -> &Range<usize> {
        match self {
            ImportSpecifier::Default { span, .. } | ImportSpecifier::Namespace { span, .. } | ImportSpecifier::Named { span, .. } => span,
        }
    }
}

/// Represents an export specifier.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExportSpecifier {
    /// `*` in `export * from "source"`.
    All {
        /// Source span of the `*` token.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// `* as exported` in `export * as exported from "source"`.
    Namespace {
        /// Exported namespace name.
        exported: String,
        /// Source span of the `* as exported` clause.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
    /// `local as exported` in `export { local as exported }`.
    Named {
        /// Local name.
        local: String,
        /// Exported name.
        exported: String,
        /// `type` keyword written before this specifier, e.g. `{ type Foo }`.
        type_modifier: Option<Modifier>,
        /// Source span of the whole specifier.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
    },
}

impl ExportSpecifier {
    /// Returns `true` when this specifier carries an inline `type` keyword.
    ///
    /// Declaration-level `export type …` is recorded on
    /// [`ExportDeclaration::is_type_only`], not here.
    pub fn is_inline_type_only(&self) -> bool {
        matches!(self, ExportSpecifier::Named { type_modifier: Some(_), .. })
    }

    /// Returns the source span of the whole specifier.
    pub fn span(&self) -> &Range<usize> {
        match self {
            ExportSpecifier::All { span, .. } | ExportSpecifier::Namespace { span, .. } | ExportSpecifier::Named { span, .. } => span,
        }
    }
}

/// Represents an import declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportDeclaration {
    /// The module specifier being imported from.
    pub module_specifier: ModuleSpecifier,
    /// The specifiers within the import.
    pub specifiers: Vec<ImportSpecifier>,
    /// Keyword modifiers written before the bindings (`type`).
    pub modifiers: Vec<Modifier>,
    /// Raw text of a trailing import attribute clause (`with { type: "json" }`).
    pub attributes: Option<String>,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ImportDeclaration {
    /// Returns `true` for `import type …` declarations.
    pub fn is_type_only(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Type)
    }
}

/// Represents an export declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExportDeclaration {
    /// The declaration being exported.
    pub declaration: Option<Box<Statement>>,
    /// The specifiers within the export.
    pub specifiers: Vec<ExportSpecifier>,
    /// The module specifier being exported from.
    pub source: Option<ModuleSpecifier>,
    /// Keyword modifiers written directly after `export` (`default`, `type`).
    pub modifiers: Vec<Modifier>,
    /// Raw text of a trailing import attribute clause (`with { type: "json" }`).
    pub attributes: Option<String>,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ExportDeclaration {
    /// Returns `true` for `export default …`.
    pub fn is_default(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Default)
    }

    /// Returns `true` for `export type …`.
    pub fn is_type_only(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Type)
    }
}

/// Represents a variable declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariableDeclaration {
    /// Decorators associated with the declaration.
    pub decorators: Vec<Decorator>,
    /// Keyword modifiers written before the declaration (`declare`).
    pub modifiers: Vec<Modifier>,
    /// The JavaScript declaration keyword.
    pub kind: VariableKind,
    /// One entry per declarator, in source order (`const a = 1, b = 2` has two).
    pub declarators: Vec<VariableDeclarator>,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A single declarator inside a [`VariableDeclaration`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariableDeclarator {
    /// Binding identifiers introduced by this declarator, in source order.
    ///
    /// A plain binding gives `["a"]`, an object pattern `{ a, b: c, ...rest }`
    /// gives `["a", "c", "rest"]` and an array pattern `[x, , ...ys]` gives
    /// `["x", "ys"]`. Nested patterns and default values expand the same way.
    pub binding_names: Vec<String>,
    /// The binding pattern exactly as written (`{ a, b: c }`), or `None` for a
    /// plain binding identifier. The identities themselves live in
    /// [`Self::binding_names`] and are never read back from this text.
    pub pattern_text: Option<String>,
    /// The type annotation of the declarator.
    pub ty: Option<TypeAnnotation>,
    /// The initial value of the declarator.
    pub value: Option<Expression>,
    /// Source span of the declarator, from its binding to the last of its
    /// type annotation and initializer.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A JavaScript variable declaration keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VariableKind {
    /// `const` declaration.
    Const,
    /// `let` declaration.
    Let,
    /// `var` declaration.
    Var,
}

/// Represents a function declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionDeclaration {
    /// Decorators associated with the declaration.
    pub decorators: Vec<Decorator>,
    /// Keyword modifiers written before the declaration (`declare`).
    pub modifiers: Vec<Modifier>,
    /// The name of the function.
    pub name: String,
    /// Type parameters of the function.
    pub type_params: Vec<TypeParameter>,
    /// Parameters of the function.
    pub params: Vec<FunctionParam>,
    /// Return type of the function.
    pub return_type: Option<TypeAnnotation>,
    /// Body of the function.
    pub body: Vec<Statement>,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a class declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassDeclaration {
    /// Decorators associated with the declaration.
    pub decorators: Vec<Decorator>,
    /// Keyword modifiers written before the declaration (`declare`, `abstract`).
    pub modifiers: Vec<Modifier>,
    /// The name of the class.
    pub name: String,
    /// Type parameters of the class.
    pub type_params: Vec<TypeParameter>,
    /// The class that this class extends.
    pub extends: Option<TypeAnnotation>,
    /// The interfaces that this class implements.
    pub implements: Vec<TypeAnnotation>,
    /// Members of the class.
    pub body: Vec<ClassMember>,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a parameter in a function declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionParam {
    /// Decorators associated with the parameter.
    pub decorators: Vec<Decorator>,
    /// The simple identifier name of the parameter (empty for destructuring patterns).
    pub name: String,
    /// Every identifier this parameter introduces (aliases, rest elements and
    /// nested pattern targets resolve to their bound names).
    pub binding_names: Vec<String>,
    /// Raw source text of a destructuring pattern, `None` for plain identifiers.
    pub pattern_text: Option<String>,
    /// Default value written after `=`.
    pub default: Option<Expression>,
    /// `true` for a rest parameter (`...args`).
    pub is_rest: bool,
    /// The type annotation of the parameter.
    pub ty: Option<TypeAnnotation>,
    /// `?` marker written after the parameter name.
    pub optional_marker: Option<OptionalMarker>,
    /// Source span of the parameter.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl FunctionParam {
    /// Returns `true` when the parameter is marked optional with `?`.
    pub fn is_optional(&self) -> bool {
        self.optional_marker.is_some()
    }
}

impl VariableDeclaration {
    /// Returns `true` when the declaration carries a `declare` modifier.
    pub fn is_declare(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Declare)
    }

    /// The first binding identifier of the first declarator, or an empty string.
    ///
    /// Multi-declarator and destructuring declarations are read through
    /// [`Self::declarators`].
    pub fn name(&self) -> &str {
        self.declarators.first().and_then(|declarator| declarator.binding_names.first()).map(String::as_str).unwrap_or("")
    }

    /// The type annotation of the first declarator.
    pub fn ty(&self) -> Option<&TypeAnnotation> {
        self.declarators.first().and_then(|declarator| declarator.ty.as_ref())
    }

    /// The initializer of the first declarator.
    pub fn value(&self) -> Option<&Expression> {
        self.declarators.first().and_then(|declarator| declarator.value.as_ref())
    }
}

impl FunctionDeclaration {
    /// Returns `true` when the declaration carries a `declare` modifier.
    pub fn is_declare(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Declare)
    }
}

impl ClassDeclaration {
    /// Returns `true` when the declaration carries a `declare` modifier.
    pub fn is_declare(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Declare)
    }

    /// Returns `true` when the class carries an `abstract` modifier.
    pub fn is_abstract(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Abstract)
    }
}

impl InterfaceDeclaration {
    /// Returns `true` when the declaration carries a `declare` modifier.
    pub fn is_declare(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Declare)
    }
}

impl TypeAliasDeclaration {
    /// Returns `true` when the declaration carries a `declare` modifier.
    pub fn is_declare(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Declare)
    }
}

impl EnumDeclaration {
    /// Returns `true` when the declaration carries a `declare` modifier.
    pub fn is_declare(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Declare)
    }
}

impl NamespaceDeclaration {
    /// Returns `true` when the declaration carries a `declare` modifier.
    pub fn is_declare(&self) -> bool {
        has_modifier(&self.modifiers, ModifierKind::Declare)
    }
}
