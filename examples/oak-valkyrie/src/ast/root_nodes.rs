//! Root nodes for the Valkyrie language AST.
//!
//! This module defines the core root-level nodes for the Valkyrie language, including:
//! - Source code spans
//! - Loop and enum keyword kinds
//! - Identifiers
//! - Name paths
//! - The root AST node

use crate::ast::items_nodes::StatementNode;

/// Source code span representing a range of positions in the source code.
pub type Span = oak_core::Range<usize>;

/// Loop keyword kind for deprecation warnings.
///
/// # V Language Example
/// ```v
/// // Preferred syntax
/// loop i in 0..10 {
///     println(i)
/// }
///
/// // Deprecated syntax (use `loop` instead)
/// for i in 0..10 {
///     println(i)
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoopKind {
    /// Using `loop` keyword (preferred).
    #[default]
    Loop,
    /// Using `for` keyword (deprecated, use `loop` instead).
    For,
}

/// Enums keyword kind for deprecation warnings.
///
/// # V Language Example
/// ```v
/// // Preferred syntax
/// enums Color {
///     Red
///     Green
///     Blue
/// }
///
/// // Alternative preferred syntax
/// unity Color {
///     Red
///     Green
///     Blue
/// }
///
/// // Deprecated syntax (use `enums` or `unity` instead)
/// enum Color {
///     Red
///     Green
///     Blue
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EnumsKind {
    /// Using `enums` keyword (preferred).
    #[default]
    Enums,
    /// Using `enum` keyword (deprecated, use `unity` instead).
    Enum,
    /// Using `unity` keyword (preferred alternative).
    Unity,
}

/// An identifier representing a name in the source code.
///
/// # V Language Example
/// ```v
/// // Identifier examples
/// let name = "John"  // 'name' is an identifier
/// let age: i32 = 30   // 'age' is an identifier
///
/// micro greet(name: String) -> String {
///     return "Hello, " + name
/// }
/// // 'greet' is an identifier
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Identifier {
    /// The identifier name as a string.
    pub name: String,
    /// The source code span where this identifier appears.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

impl Default for Identifier {
    fn default() -> Self {
        Self { name: String::new(), span: Span::default() }
    }
}

/// A name path representing a qualified name (e.g., `std::collections::HashMap`).
///
/// # V Language Example
/// ```v
/// // Name path examples
/// using std::collections::HashMap
///
/// class Person {
///     name: String
///     age: i32
/// }
///
/// let map: HashMap<String, Person> = HashMap::new()
/// // 'std::collections::HashMap' is a name path
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamePath {
    /// The individual identifier parts of the path.
    pub parts: Vec<Identifier>,
    /// The source code span covering the entire name path.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// Valkyrie root node representing the entire AST of a Valkyrie module.
///
/// # V Language Example
/// ```v
/// // A complete Valkyrie module
/// using std::io
///
/// class Person {
///     name: String
///     age: i32
///
///     micro new(name: String, age: i32) -> Person {
///         return Person { name: name, age: age }
///     }
///
///     micro greet(self) -> String {
///         return "Hello, my name is " + self.name
///     }
/// }
///
/// let person = Person::new("John", 30)
/// println(person.greet())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValkyrieRoot {
    /// The collection of top-level items in the Valkyrie module.
    pub items: Vec<StatementNode>,
}

/// 花括号作用域或模块根的 body 载荷。
///
/// `namespace name { ... }` 与 `namespace name {}` 都产出 body；分号式
/// `namespace name;` 的 body 为 `None`。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProgramBody {
    /// The collection of top-level items in the Valkyrie module.
    pub items: Vec<StatementNode>,
    /// The source code span where this identifier appears.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
