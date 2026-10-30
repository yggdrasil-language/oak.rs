use core::range::Range;

/// A keyword written before a declaration or class member.
///
/// Modifiers are recorded in source order together with their keyword span, so
/// the syntax tree never stores a derived `is_*` boolean: presence questions are
/// answered by [`has_modifier`] or by the `is_*` methods on the owning node.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Modifier {
    /// Which keyword this modifier records.
    pub kind: ModifierKind,
    /// Source span of the keyword.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl Modifier {
    /// Builds a modifier for `kind` written at `span`.
    pub fn new(kind: ModifierKind, span: Range<usize>) -> Self {
        Self { kind, span }
    }
}

/// Keyword modifiers recognised by the TypeScript builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModifierKind {
    /// `declare`.
    Declare,
    /// `abstract`.
    Abstract,
    /// `static`.
    Static,
    /// `readonly`.
    Readonly,
    /// `async`.
    Async,
    /// `get`.
    Get,
    /// `set`.
    Set,
    /// `type` in `import type` / `export type`.
    Type,
    /// `default` in `export default`.
    Default,
    /// `await` in `for await`.
    Await,
}

/// Returns `true` when `modifiers` contains `kind`.
pub fn has_modifier(modifiers: &[Modifier], kind: ModifierKind) -> bool {
    modifiers.iter().any(|modifier| modifier.kind == kind)
}

/// A `?` marker written after a parameter, property or method name.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionalMarker {
    /// Source span of the `?` token.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl OptionalMarker {
    /// Builds an optional marker at `span`.
    pub fn new(span: Range<usize>) -> Self {
        Self { span }
    }
}
