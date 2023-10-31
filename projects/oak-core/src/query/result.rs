use super::view::ElementRef;

/// High-level selector status distinct from parser diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelectorOutcome {
    /// Query succeeded and may return zero or more nodes.
    Matched,
    /// Selector syntax or semantics are not supported by the active executor.
    Unsupported,
    /// `select_one` found multiple matches.
    Ambiguous,
    /// View revision does not match the query snapshot.
    StaleSnapshot,
    /// Query exceeded [`super::QueryBudget`].
    BudgetExceeded,
}

/// Result of a multi-match selector query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorResult {
    /// Overall query status.
    pub outcome: SelectorOutcome,
    /// Matched elements in document order.
    pub matches: Vec<ElementRef>,
}

impl SelectorResult {
    /// Creates an empty successful result.
    #[must_use]
    pub fn empty() -> Self {
        Self { outcome: SelectorOutcome::Matched, matches: Vec::new() }
    }

    /// Returns whether the query completed without structural failure.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self.outcome, SelectorOutcome::Matched)
    }
}

/// Result of a single-match selector query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectOneOutcome {
    /// Exactly one match.
    One(ElementRef),
    /// Zero matches.
    None,
    /// More than one match.
    Ambiguous(Vec<ElementRef>),
    /// Query failed before matching.
    Failed(SelectorOutcome),
}

impl SelectOneOutcome {
    /// Returns the single match when present.
    #[must_use]
    pub fn ok(self) -> Option<ElementRef> {
        match self {
            Self::One(reference) => Some(reference),
            Self::None | Self::Ambiguous(_) | Self::Failed(_) => None,
        }
    }
}
