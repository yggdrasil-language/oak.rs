//! Owned Maple syntax, independent of any computation runtime.

/// An owned source range measured in UTF-8 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapleSpan {
    pub start: usize,
    pub end: usize,
}

/// A sequence of Maple expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapleRoot {
    pub expressions: Vec<MapleExpression>,
    pub span: MapleSpan,
}

/// A Maple expression with its original source extent.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapleExpression {
    pub kind: MapleExpressionKind,
    pub span: MapleSpan,
}

/// Syntax-only expression variants. Numeric spelling is preserved exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MapleExpressionKind {
    Symbol(String),
    Integer(String),
    Decimal(String),
    List(Vec<MapleExpression>),
    Call { name: String, arguments: Vec<MapleExpression> },
    Grouped(Box<MapleExpression>),
    Unary { operator: MapleOperator, operand: Box<MapleExpression> },
    Binary { operator: MapleOperator, left: Box<MapleExpression>, right: Box<MapleExpression> },
}

/// Arithmetic operators recognized by the Maple expression subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MapleOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}
