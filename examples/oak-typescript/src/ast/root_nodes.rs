use crate::ast::Statement;
use core::range::Range;

/// TypeScript AST root node.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeScriptRoot {
    /// Statements in the root.
    pub statements: Vec<Statement>,
    /// Source spans of `await` occurrences that are not nested inside a function body.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range::vec"))]
    pub top_level_await_spans: Vec<Range<usize>>,
    /// Source span of the root.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl TypeScriptRoot {
    /// Creates a new `TypeScriptRoot`.
    pub fn new(span: Range<usize>) -> Self {
        Self { statements: vec![], top_level_await_spans: vec![], span }
    }

    /// Returns `true` when the root recorded at least one top-level `await`.
    pub fn has_top_level_await(&self) -> bool {
        !self.top_level_await_spans.is_empty()
    }
}
