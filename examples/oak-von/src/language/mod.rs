#![doc = include_str!("readme.md")]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use oak_core::{Language, LanguageCategory};

/// VON value representation.
pub mod value;
pub use value::VonValue;

#[cfg(feature = "serde")]
mod de;
#[cfg(feature = "serde")]
mod ser;
#[cfg(feature = "serde")]
pub use self::{de::deserialize, de::from_str, ser::serialize, ser::to_string, ser::to_string_indented};

/// Configuration for comments in VON.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommentConfig {
    /// The character sequence that starts a line comment.
    pub line_comment: Option<String>,
    /// The character sequences that start and end a block comment.
    pub block_comment: Option<(String, String)>,
}

/// Configuration for strings in VON.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringConfig {
    /// The characters that can be used as string quotes.
    pub quotes: Vec<char>,
    /// The character used for escaping characters within strings.
    pub escape_char: Option<char>,
}

/// Configuration for whitespace in VON.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhitespaceConfig {
    /// The characters treated as non-newline whitespace.
    pub characters: Vec<char>,
    /// The characters treated as newlines.
    pub new_line_characters: Vec<char>,
}

/// Implementation of the VON language.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonLanguage {
    /// Comment configuration.
    pub commentconfig: CommentConfig,
    /// String configuration.
    pub stringconfig: StringConfig,
    /// Whitespace configuration.
    pub whitespaceconfig: WhitespaceConfig,
}

impl VonLanguage {
    /// Creates a new `VonLanguage` instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for VonLanguage {
    fn default() -> Self {
        Self {
            commentconfig: CommentConfig { line_comment: Some("#".to_string()), block_comment: None },
            stringconfig: StringConfig { quotes: vec!['"', '\''], escape_char: Some('\\') },
            whitespaceconfig: WhitespaceConfig { characters: vec![' ', '\t'], new_line_characters: vec!['\n', '\r'] },
        }
    }
}

impl Language for VonLanguage {
    const NAME: &'static str = "von";
    const CATEGORY: LanguageCategory = LanguageCategory::Programming;

    type TokenType = crate::lexer::token_type::VonTokenType;
    type ElementType = crate::parser::element_type::VonElementType;
    type TypedRoot = crate::ast::VonRoot;
}
