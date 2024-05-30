#![warn(missing_docs)]
#![doc = include_str!("readme.md")]
use oak_core::{
    language::Language,
    source::{LineMap, Source},
    tree::RedNode,
};

/// Represents a semantic token for syntax highlighting.
///
/// Benchmarked against LSP's Semantic Tokens.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticToken {
    /// The delta line from the previous token.
    pub delta_line: u32,
    /// The delta start character from the previous token or line start.
    pub delta_start: u32,
    /// The length of the token in characters.
    pub length: u32,
    /// The type of the token (index into the legend).
    pub token_type: u32,
    /// The modifiers of the token (bitset).
    pub token_modifiers_bitset: u32,
}

/// Trait for languages that support semantic highlighting.
pub trait SemanticTokensProvider<L: Language> {
    /// Returns the semantic tokens for the given document.
    ///
    /// 源码以 `&dyn Source` 传入，使本能力可以在适配层以引用持有。
    fn semantic_tokens(&self, root: &RedNode<L>, source: &dyn Source, line_map: &LineMap) -> Vec<SemanticToken>;
}
