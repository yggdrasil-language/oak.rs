#![doc = include_str!("readme.md")]
use oak_core::{Language, LanguageCategory};

/// How unquoted values are recognized after `=`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IniValueStyle {
    /// Typed literals only (identifier / string / number / bool / datetime).
    /// Closer to TOML-inspired config INI.
    #[default]
    Typed,
    /// Everything after `=` until end of line is one bare value (Westwood / classic INI).
    LineRemainder,
}

/// INI language definition and dialect switches.
///
/// INI has no single standard. Dialects differ on numeric keys, `#` comments,
/// and whether values are typed tokens or the rest of the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IniLanguage {
    /// Allow integer tokens as keys (e.g. `0=MTNK` in type lists).
    pub numeric_keys: bool,
    /// Treat `#` as a line-comment marker (in addition to `;`).
    pub hash_comments: bool,
    /// Value recognition strategy after `=`.
    pub value_style: IniValueStyle,
}

impl Default for IniLanguage {
    fn default() -> Self {
        Self {
            numeric_keys: false,
            hash_comments: true,
            value_style: IniValueStyle::Typed,
        }
    }
}

impl IniLanguage {
    /// Creates a new `IniLanguage` with default (typed) dialect.
    pub fn new() -> Self {
        Self::default()
    }

    /// Westwood / classic game INI dialect (Red Alert 2 rules, maps, art).
    ///
    /// - numeric keys allowed
    /// - only `;` starts a line comment (`#` is data)
    /// - values run to end of line
    pub fn westwood() -> Self {
        Self {
            numeric_keys: true,
            hash_comments: false,
            value_style: IniValueStyle::LineRemainder,
        }
    }
}

impl Language for IniLanguage {
    const NAME: &'static str = "ini";
    const CATEGORY: LanguageCategory = LanguageCategory::Config;

    type TokenType = crate::lexer::token_type::IniTokenType;
    type ElementType = crate::parser::element_type::IniElementType;
    type TypedRoot = crate::ast::IniRoot;
}
