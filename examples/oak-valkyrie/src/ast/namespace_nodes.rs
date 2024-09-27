//! Namespace and import nodes for the Valkyrie language AST.

use crate::{
    ValkyrieRoot,
    ast::{Attribute, Identifier, NamePath, Span, items_nodes::StatementNode, root_nodes::ProgramBody},
};

/// A namespace declaration.
///
/// # V Language Example
/// ```v
/// namespace core::text;
///
/// namespace game::player {
///     micro spawn() -> Player { }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamespaceDeclaration {
    /// Annotations applied to the namespace.
    pub annotations: Vec<Attribute>,
    /// 是否为 `namespace!` 主空间声明。
    ///
    /// `namespace!` 是独立构造，不是普通 `namespace` 的别名；身份必须保留到这里，
    /// 由语言 frontend 校验「全工程唯一」与 `using!` 的放置。
    #[cfg_attr(feature = "serde", serde(default))]
    pub primary: bool,
    /// The name path of the namespace.
    pub name: NamePath,
    /// 该 namespace 是否有 body。
    ///
    /// ```v
    /// namespace name {} -> Some([])
    /// namespace name -> None;
    /// ```
    pub body: Option<ProgramBody>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

impl NamespaceDeclaration {
    /// 是否为花括号作用域；空 body 仍然是作用域，不能被当成分号式指令。
    pub fn has_body(&self) -> bool {
        self.body.is_some()
    }
}

/// A using (import) statement.
///
/// # V Language Example
/// ```v
/// using core::primitive::{Never, Unit}
/// using core::primitive::Bool as bool
/// using std::text::utf8
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UsingDeclaration {
    /// 是否为 `using!` 主空间导入。
    ///
    /// `using!` 只能在 `namespace!` 内使用，其导入在所有 namespace 共享；
    /// 普通 `using` 是文件局部导入。两者身份不得以空 `path` 或同名节点混同。
    #[cfg_attr(feature = "serde", serde(default))]
    pub primary: bool,
    /// The path to import.
    pub path: NamePath,
    /// Optional alias for the import.
    pub alias: Option<Identifier>,
    /// Selective import list (e.g., `{Never, Unit}` in `using core::primitive.{Never, Unit}`)
    pub imports: Vec<Identifier>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
