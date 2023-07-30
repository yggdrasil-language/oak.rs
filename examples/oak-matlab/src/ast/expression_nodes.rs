//! Expression nodes for MATLAB.

use crate::{
    ast::root_nodes::{Identifier, Span},
    lexer::token_type::MatlabTokenType,
};

/// A MATLAB expression (owned).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Expression {
    /// Identifier / `end` / lone `:`.
    Symbol(Identifier),
    /// Number / string / character literal.
    Literal {
        /// Raw literal text.
        value: String,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// Array `[…]` with row groups.
    Array {
        /// Rows of elements.
        rows: Vec<Vec<Expression>>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// Call / indexing `f(…)` / `A(…)`.
    Call {
        /// Head expression.
        head: Box<Expression>,
        /// Arguments / indices.
        arguments: Vec<Expression>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// Binary / infix operator.
    Binary(Box<BinaryExpr>),
    /// Prefix operator.
    Prefix(Box<UnaryExpr>),
    /// Postfix operator (transpose).
    Postfix(Box<UnaryExpr>),
    /// Parenthesized expression.
    Grouped {
        /// Inner expression.
        expression: Box<Expression>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// `@(x, y) body` anonymous function.
    AnonymousFunction {
        /// Parameter expressions (typically symbols).
        parameters: Vec<Expression>,
        /// Function body.
        body: Box<Expression>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// `@sin` / `@name` function handle.
    FunctionHandle {
        /// Target name / expression.
        target: Box<Expression>,
        /// Source span.
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
}

/// Binary operator application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BinaryExpr {
    /// Operator token kind.
    pub operator: MatlabTokenType,
    /// Left operand.
    pub lhs: Expression,
    /// Right operand.
    pub rhs: Expression,
    /// Source span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// Unary (prefix or postfix) operator application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnaryExpr {
    /// Operator token kind.
    pub operator: MatlabTokenType,
    /// Operand.
    pub operand: Expression,
    /// Source span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

impl Expression {
    /// Source span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Self::Symbol(id) => id.span.clone(),
            Self::Literal { span, .. }
            | Self::Array { span, .. }
            | Self::Call { span, .. }
            | Self::Grouped { span, .. }
            | Self::AnonymousFunction { span, .. }
            | Self::FunctionHandle { span, .. } => span.clone(),
            Self::Binary(b) => b.span.clone(),
            Self::Prefix(u) | Self::Postfix(u) => u.span.clone(),
        }
    }

    /// Identifier / `end` / lone `:`.
    pub fn as_symbol(&self) -> Option<&Identifier> {
        match self {
            Self::Symbol(id) => Some(id),
            _ => None,
        }
    }

    /// Raw literal text and span.
    pub fn as_literal(&self) -> Option<(&str, &Span)> {
        match self {
            Self::Literal { value, span } => Some((value.as_str(), span)),
            _ => None,
        }
    }

    /// Array rows.
    pub fn as_array(&self) -> Option<&[Vec<Expression>]> {
        match self {
            Self::Array { rows, .. } => Some(rows.as_slice()),
            _ => None,
        }
    }

    /// Call / indexing head and arguments.
    pub fn as_call(&self) -> Option<(&Expression, &[Expression])> {
        match self {
            Self::Call { head, arguments, .. } => Some((head.as_ref(), arguments.as_slice())),
            _ => None,
        }
    }

    /// Binary / infix node.
    pub fn as_binary(&self) -> Option<&BinaryExpr> {
        match self {
            Self::Binary(bin) => Some(bin.as_ref()),
            _ => None,
        }
    }

    /// Prefix unary node.
    pub fn as_prefix(&self) -> Option<&UnaryExpr> {
        match self {
            Self::Prefix(u) => Some(u.as_ref()),
            _ => None,
        }
    }

    /// Postfix unary node (transpose).
    pub fn as_postfix(&self) -> Option<&UnaryExpr> {
        match self {
            Self::Postfix(u) => Some(u.as_ref()),
            _ => None,
        }
    }

    /// Parenthesized inner expression.
    pub fn as_grouped(&self) -> Option<&Expression> {
        match self {
            Self::Grouped { expression, .. } => Some(expression.as_ref()),
            _ => None,
        }
    }

    /// Assignment `lhs = rhs`.
    pub fn as_assignment(&self) -> Option<(&Expression, &Expression)> {
        let bin = self.as_binary()?;
        if bin.operator != MatlabTokenType::Assign {
            return None;
        }
        Some((&bin.lhs, &bin.rhs))
    }

    /// Colon range / slice `lhs : rhs` (possibly nested for `a:b:c`).
    pub fn as_colon(&self) -> Option<(&Expression, &Expression)> {
        let bin = self.as_binary()?;
        if bin.operator != MatlabTokenType::Colon {
            return None;
        }
        Some((&bin.lhs, &bin.rhs))
    }

    /// Elementwise binary (`.*` / `./` / `.^` / `.\`).
    pub fn as_elementwise(&self) -> Option<&BinaryExpr> {
        let bin = self.as_binary()?;
        match bin.operator {
            MatlabTokenType::DotTimes | MatlabTokenType::DotDivide | MatlabTokenType::DotPower | MatlabTokenType::DotLeftDivide => Some(bin),
            _ => None,
        }
    }

    /// Matrix left divide `A \ b`.
    pub fn as_left_divide(&self) -> Option<(&Expression, &Expression)> {
        let bin = self.as_binary()?;
        if bin.operator != MatlabTokenType::LeftDivide {
            return None;
        }
        Some((&bin.lhs, &bin.rhs))
    }

    /// Transpose postfix (`'` / `.'`).
    pub fn as_transpose(&self) -> Option<&UnaryExpr> {
        let u = self.as_postfix()?;
        match u.operator {
            MatlabTokenType::Transpose | MatlabTokenType::DotTranspose => Some(u),
            _ => None,
        }
    }
}
