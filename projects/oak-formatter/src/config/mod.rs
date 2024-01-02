//! Configuration primitives owned by the formatter algorithm.

/// The layout decision for one source gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapConstraint {
    /// Do not emit a byte in the gap.
    NoSpace,
    /// Emit one space unless a preserved line break owns the gap.
    OptionalSpace,
    /// Emit one space.
    RequiredSpace,
    /// Keep the source gap unchanged.
    Preserve,
    /// A language rule requires a line break.
    HardLine,
}

/// Local options for applying token-gap edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GapFormatOptions {
    /// Whether final newline policy is controlled by the caller.
    pub preserve_final_newline: bool,
}

impl Default for GapFormatOptions {
    fn default() -> Self {
        Self { preserve_final_newline: true }
    }
}
