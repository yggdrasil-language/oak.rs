/// Resource limits for tree queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryBudget {
    /// Maximum number of matched nodes returned.
    pub max_matches: usize,
    /// Maximum traversal depth from the query root.
    pub max_depth: usize,
    /// Maximum predicate evaluations per query.
    pub max_predicates: usize,
    /// Maximum bytes materialized for text extraction.
    pub max_text_bytes: usize,
}

impl Default for QueryBudget {
    fn default() -> Self {
        Self {
            max_matches: 10_000,
            max_depth: 256,
            max_predicates: 10_000,
            max_text_bytes: 8 * 1024 * 1024,
        }
    }
}

impl QueryBudget {
    /// Creates a budget suitable for small package members such as OPC parts.
    #[must_use]
    pub fn package_member() -> Self {
        Self {
            max_matches: 50_000,
            max_depth: 512,
            max_predicates: 50_000,
            max_text_bytes: 16 * 1024 * 1024,
        }
    }
}
