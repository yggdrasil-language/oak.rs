use crate::ast::{ClassMember, Decorator, EnumMember, Expression, TypeAnnotation, TypeParameter};
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the declaration is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the declaration is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the declaration is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the declaration is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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

/// Represents a switch statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SwitchStatement {
    /// Decorators associated with the statement.
    pub decorators: Vec<Decorator>,
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the statement is declared with `declare`.
    pub is_declare: bool,
    /// The statements within the block.
    pub statements: Vec<Statement>,
    /// Source span of the block statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents an import specifier.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportSpecifier {
    /// Default import: `import local from "source"`.
    Default(String),
    /// Namespace import: `import * as local from "source"`.
    Namespace(String),
    /// Named import: `import { local as imported } from "source"`.
    Named {
        /// Local name.
        local: String,
        /// Imported name.
        imported: String,
    },
}

/// Represents an export specifier.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExportSpecifier {
    /// Local name.
    pub local: String,
    /// Exported name.
    pub exported: String,
    /// Whether this specifier is type-only.
    pub is_type_only: bool,
}

/// Represents an import declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportDeclaration {
    /// The module specifier being imported from.
    pub module_specifier: String,
    /// The specifiers within the import.
    pub specifiers: Vec<ImportSpecifier>,
    /// Whether the import is type-only.
    pub is_type_only: bool,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
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
    pub source: Option<String>,
    /// Whether this is a default export.
    pub is_default: bool,
    /// Whether the export is type-only.
    pub is_type_only: bool,
    /// Source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents a variable declaration.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariableDeclaration {
    /// Decorators associated with the declaration.
    pub decorators: Vec<Decorator>,
    /// Whether the declaration is declared with `declare`.
    pub is_declare: bool,
    /// The name of the variable.
    pub name: String,
    /// The JavaScript declaration keyword.
    pub kind: VariableKind,
    /// The type annotation of the variable.
    pub ty: Option<TypeAnnotation>,
    /// The initial value of the variable.
    pub value: Option<Expression>,
    /// Source span of the declaration.
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
    /// Whether the declaration is declared with `declare`.
    pub is_declare: bool,
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
    /// Whether the declaration is declared with `declare`.
    pub is_declare: bool,
    /// The name of the class.
    pub name: String,
    /// Type parameters of the class.
    pub type_params: Vec<TypeParameter>,
    /// The class that this class extends.
    pub extends: Option<TypeAnnotation>,
    /// The interfaces that this class implements.
    pub implements: Vec<TypeAnnotation>,
    /// Whether the class is abstract.
    pub is_abstract: bool,
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
    /// The name of the parameter.
    pub name: String,
    /// The type annotation of the parameter.
    pub ty: Option<TypeAnnotation>,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Source span of the parameter.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}
