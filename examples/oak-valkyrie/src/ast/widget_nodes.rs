//! Widget declaration nodes for the Valkyrie language AST.

use crate::ast::{Attribute, GenericParam, Identifier, Span, StatementNode};

/// A `widget` declaration.
///
/// Widgets are UI composition roots that contain nested module items rather than
/// a class-like field/method body.
///
/// # V Language Example
/// ```v
/// widget CounterLabel {
///     value: i32 = 0
///
///     micro render(self) -> utf8 {
///         return f"Count: {self.value}"
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WidgetDeclaration {
    /// The widget name.
    pub name: Identifier,
    /// Generic parameters for the widget.
    pub generics: Vec<GenericParam>,
    /// Items declared within the widget.
    pub items: Vec<StatementNode>,
    /// Annotations applied to the widget.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
