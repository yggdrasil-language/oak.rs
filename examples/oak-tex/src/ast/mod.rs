#![doc = include_str!("readme.md")]
use core::range::Range;

/// Root node of the TeX AST.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexRoot {
    /// The span of the root node.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The items contained in the root node.
    pub items: Vec<TexItem>,
}

/// A top-level item in a TeX document.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexItem {
    /// A TeX command.
    Command(TexCommand),
    /// A TeX environment.
    Environment(TexEnvironment),
    /// A row of an environment body.
    Row(TexRow),
    /// A TeX group.
    Group(TexGroup),
    /// A TeX math environment.
    Math(TexMath),
    /// A `\left...\middle...\right` paired delimiter construct.
    LeftRightPair(TexLeftRightPair),
    /// A `\limits` / `\nolimits` command.
    Limits(TexLimits),
    /// A superscript.
    Superscript(TexSuperscript),
    /// A subscript.
    Subscript(TexSubscript),
    /// Plain text.
    Text {
        /// The span of the text.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
        /// The content of the text.
        content: String,
    },
    /// A comment.
    Comment {
        /// The span of the comment.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Range<usize>,
        /// The content of the comment.
        content: String,
    },
}

/// A row of a TeX environment body.
///
/// `&` separates cells and `\\` separates rows, so the parser wraps an
/// environment body into rows of cells instead of leaving the separators as
/// plain text. Downstream semantic lowering reads this structure; it never
/// rescans the environment source for alignment tabs or row breaks.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexRow {
    /// The span of the row, from its first cell to the row break when present.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The cells in this row, in source order.
    pub cells: Vec<TexCell>,
}

/// One cell of an environment row.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexCell {
    /// The span of the cell.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The cell content.
    pub content: TexRoot,
}

/// A TeX environment (e.g., \begin{matrix} ... \end{matrix}).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexEnvironment {
    /// The span of the environment.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The name of the environment (without trailing `*`).
    pub name: String,
    /// Whether the environment name had a trailing `*` (e.g. `align*`).
    pub starred: bool,
    /// Whether this is a known math environment (`equation`, `align`, `cases`, `matrix`, etc.).
    pub is_math: bool,
    /// The arguments to the environment.
    pub arguments: Vec<TexArgument>,
    /// The content of the environment, one [`TexItem::Row`] per row.
    pub content: TexRoot,
}

/// A TeX superscript (^).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexSuperscript {
    /// The span of the superscript.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The target of the superscript.
    pub target: Option<Box<TexItem>>,
    /// The content of the superscript.
    pub content: Box<TexRoot>,
}

/// A TeX subscript (_).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexSubscript {
    /// The span of the subscript.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The target of the subscript.
    pub target: Option<Box<TexItem>>,
    /// The content of the subscript.
    pub content: Box<TexRoot>,
}

/// The delimiter style of a math mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MathDelimiter {
    /// `$...$` inline math
    Dollar,
    /// `$$...$$` display math
    DoubleDollar,
    /// `\(...\)` inline math
    Paren,
    /// `\[...\]` display math
    Bracket,
    /// Math environment like `equation`, `align` (delimiter-less)
    Environment,
}

impl MathDelimiter {
    /// Whether this delimiter produces display math.
    pub fn is_display(&self) -> bool {
        matches!(self, Self::DoubleDollar | Self::Bracket | Self::Environment)
    }
}

/// A TeX math environment ($...$ or $$...$$).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexMath {
    /// The span of the math environment.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The content of the math environment.
    pub content: TexRoot,
    /// Whether this is a display math environment.
    pub is_display: bool,
    /// The delimiter style that opened this math mode.
    pub delimiter: MathDelimiter,
}

/// A TeX command (e.g., \section, \textbf).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexCommand {
    /// The span of the command.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The name of the command (without backslash and without trailing `*`).
    pub name: String,
    /// Whether the command has a trailing `*` (e.g. `\section*`).
    pub starred: bool,
    /// The arguments to the command.
    pub arguments: Vec<TexArgument>,
}

/// A TeX argument.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexArgument {
    /// An optional argument ([...]).
    Optional(TexRoot),
    /// A mandatory argument ({...}).
    Required(TexRoot),
}

/// A TeX group (e.g., { ... }).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexGroup {
    /// The span of the group.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The content of the group.
    pub content: TexRoot,
}

/// A `\left ... \right` construct, possibly with one or more `\middle`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexLeftRightPair {
    /// The full span of the pair, from `\left` to the closing delimiter.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// The opening delimiter after `\left` (e.g. `"("`, `"["`, `"\{"`, `"."` for invisible).
    pub left_delim: String,
    /// The closing delimiter after `\right` (e.g. `")"`, `"]"`, `"\}"`, `"."` for invisible).
    pub right_delim: String,
    /// Content between `\left` and `\right`, including any `\middle` delimiters treated as text within.
    pub content: TexRoot,
}

/// A `\limits` or `\nolimits` command that modifies the preceding operator.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexLimits {
    /// The span of the `\limits` or `\nolimits` command.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
    /// True for `\limits`, false for `\nolimits`.
    pub is_limits: bool,
}

impl TexRoot {
    /// Creates a new TeX root node.
    pub fn new(span: Range<usize>) -> Self {
        Self { span, items: Vec::new() }
    }
}
