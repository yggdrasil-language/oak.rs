//! Statement nodes for MATLAB.

use crate::ast::{expression_nodes::Expression, root_nodes::Span};

/// A MATLAB statement (owned).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Statement {
    /// Expression used as a statement.
    Expr(Expression),
    /// `if … end`.
    If {
        /// Condition.
        condition: Expression,
        /// Then body.
        then_body: Vec<Statement>,
        /// `elseif` arms `(condition, body)`.
        elseifs: Vec<(Expression, Vec<Statement>)>,
        /// Optional `else` body.
        else_body: Vec<Statement>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// `while … end`.
    While {
        /// Loop condition.
        condition: Expression,
        /// Loop body.
        body: Vec<Statement>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// `for … end`.
    For {
        /// Header expression (typically `i = 1:n`).
        header: Expression,
        /// Loop body.
        body: Vec<Statement>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// `switch … case … otherwise … end`.
    Switch {
        /// Discriminant expression.
        discriminant: Expression,
        /// `case` arms `(value, body)`.
        cases: Vec<(Expression, Vec<Statement>)>,
        /// Optional `otherwise` body.
        otherwise: Vec<Statement>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// `try … catch … end`.
    Try {
        /// Protected body.
        body: Vec<Statement>,
        /// Optional catch identifier (`catch ME`).
        catch_name: Option<Expression>,
        /// Catch body.
        catch_body: Vec<Statement>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// Recovery / error node.
    Error {
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
}

impl Statement {
    /// Source span.
    pub fn span(&self) -> Span {
        match self {
            Self::Expr(e) => e.span(),
            Self::If { span, .. }
            | Self::While { span, .. }
            | Self::For { span, .. }
            | Self::Switch { span, .. }
            | Self::Try { span, .. }
            | Self::Error { span } => span.clone(),
        }
    }

    /// Expression statement.
    pub fn as_expr(&self) -> Option<&Expression> {
        match self {
            Self::Expr(e) => Some(e),
            _ => None,
        }
    }

    /// `if` statement parts.
    pub fn as_if(&self) -> Option<IfView<'_>> {
        match self {
            Self::If { condition, then_body, elseifs, else_body, .. } => Some(IfView { condition, then_body: then_body.as_slice(), elseifs: elseifs.as_slice(), else_body: else_body.as_slice() }),
            _ => None,
        }
    }

    /// `while` statement parts.
    pub fn as_while(&self) -> Option<(&Expression, &[Statement])> {
        match self {
            Self::While { condition, body, .. } => Some((condition, body.as_slice())),
            _ => None,
        }
    }

    /// `for` statement parts.
    pub fn as_for(&self) -> Option<(&Expression, &[Statement])> {
        match self {
            Self::For { header, body, .. } => Some((header, body.as_slice())),
            _ => None,
        }
    }

    /// `switch` statement parts.
    pub fn as_switch(&self) -> Option<SwitchView<'_>> {
        match self {
            Self::Switch { discriminant, cases, otherwise, .. } => Some(SwitchView {
                discriminant,
                cases: cases.as_slice(),
                otherwise: otherwise.as_slice(),
            }),
            _ => None,
        }
    }

    /// `try` / `catch` statement parts.
    pub fn as_try(&self) -> Option<TryView<'_>> {
        match self {
            Self::Try { body, catch_name, catch_body, .. } => Some(TryView { body: body.as_slice(), catch_name: catch_name.as_ref(), catch_body: catch_body.as_slice() }),
            _ => None,
        }
    }

    /// Recovery / error node.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

/// Typed view over an `if` statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IfView<'a> {
    /// Condition expression.
    pub condition: &'a Expression,
    /// Then body.
    pub then_body: &'a [Statement],
    /// `elseif` arms.
    pub elseifs: &'a [(Expression, Vec<Statement>)],
    /// Else body.
    pub else_body: &'a [Statement],
}

/// Typed view over a `switch` statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwitchView<'a> {
    /// Discriminant expression.
    pub discriminant: &'a Expression,
    /// `case` arms.
    pub cases: &'a [(Expression, Vec<Statement>)],
    /// `otherwise` body.
    pub otherwise: &'a [Statement],
}

/// Typed view over a `try` statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TryView<'a> {
    /// Protected body.
    pub body: &'a [Statement],
    /// Optional catch identifier.
    pub catch_name: Option<&'a Expression>,
    /// Catch body.
    pub catch_body: &'a [Statement],
}
