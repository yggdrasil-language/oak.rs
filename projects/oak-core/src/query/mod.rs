//! Structured element query primitives for Oak document trees.
//!
//! Selector DSL parsing lives in `oak-xpath` and `oak-css-selector`. This module
//! provides syntax-agnostic views, budgets, results, and structured queries.

mod budget;
mod result;
mod structured;
mod view;

pub use budget::QueryBudget;
pub use result::{SelectOneOutcome, SelectorOutcome, SelectorResult};
pub use structured::StructuredQuery;
pub use view::{ElementRef, ElementView, ExpandedName, TextPolicy};
