/// Product-facing options for [`super::format_source`].
///
/// The Oak formatter edits whitespace between CST tokens and keeps every token
/// text intact, so `quote_style` is accepted without rewriting string literals.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FormatOptions {
    /// Spaces per indent level.
    pub indent_width: u8,
    /// Soft wrap width for formatted output.
    pub line_width: usize,
    /// Reserved for the separate TypeScript transform pipeline.
    pub type_erasure: bool,
    /// Requested string literal quote style. Literal quoting is preserved.
    pub quote_style: QuoteStyle,
}

/// Requested string literal quote style.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum QuoteStyle {
    /// Keep the quote character used in the source.
    #[default]
    Preserve,
    /// Source style asks for single quotes.
    Single,
    /// Source style asks for double quotes.
    Double,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { indent_width: 4, line_width: 144, type_erasure: false, quote_style: QuoteStyle::Preserve }
    }
}

impl FormatOptions {
    /// Enable or disable TypeScript type erasure.
    pub fn with_type_erasure(mut self, enabled: bool) -> Self {
        self.type_erasure = enabled;
        self
    }

    /// Set the requested string literal quote style.
    pub fn with_quote_style(mut self, quote_style: QuoteStyle) -> Self {
        self.quote_style = quote_style;
        self
    }
}
