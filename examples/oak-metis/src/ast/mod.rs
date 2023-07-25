//! Typed AST for the Metis island language (Living grammar).

use oak_core::tree::GreenNode;

/// Green-tree root placeholder.
pub struct MetisRoot {
    /// Underlying green node.
    pub node: GreenNode<'static, super::language::MetisLanguage>,
}

impl MetisRoot {
    /// Wrap a green root.
    pub fn new(node: &GreenNode<'static, super::language::MetisLanguage>) -> Self {
        Self { node: node.clone() }
    }
}

/// Compilation unit: islands and top-level actions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    /// Top-level `using Path` entries (name resolution only; no math).
    pub usings: Vec<String>,
    /// `island` declarations.
    pub islands: Vec<Island>,
    /// Top-level `action` blocks.
    pub actions: Vec<Action>,
}

/// `island Name { ... }` or `island Name : Base { ... }`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Island {
    /// Optional `namespace path::…` prefix applied to this island.
    pub namespace: Option<String>,
    /// Island name.
    pub name: String,
    /// Optional single-base extension (`island C : B`).
    pub extends: Option<String>,
    /// Body items.
    pub items: Vec<Item>,
}

/// Island body item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    /// Island-body `use OtherIsland` (legacy; prefer `island C : B` / top-level `using`).
    Use(String),
    /// `node Name`
    Node(String),
    /// `relation` declaration.
    Relation(Relation),
    /// `axiom` declaration.
    Axiom(Axiom),
    /// `theorem` declaration (formula statement).
    Theorem(Theorem),
    /// `rewrites Name { ... }`
    Rewrites(Rewrites),
    /// `connection A <-> B { ... }`
    Connection(Connection),
}

/// `rewrites Name { rules/formulas }`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rewrites {
    /// Rule set name.
    pub name: String,
    /// Body formulas / rewrite equations.
    pub rules: Vec<Formula>,
}

/// `connection Left <-> Right { ... }`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    /// Left island / sort name.
    pub left: String,
    /// Right island / sort name.
    pub right: String,
    /// Body formulas (alpha / gamma signatures as formulas for now).
    pub body: Vec<Formula>,
}

/// `relation Name : Type { formula? }` or without body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relation {
    /// Relation name.
    pub name: String,
    /// Optional type signature.
    pub ty: Option<TypeExpr>,
    /// Optional defining formula.
    pub body: Option<Formula>,
}

/// `axiom Name { formula }`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Axiom {
    /// Axiom name.
    pub name: String,
    /// Formula.
    pub formula: Formula,
}

/// `theorem Name { formula }`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theorem {
    /// Theorem name.
    pub name: String,
    /// Statement formula.
    pub formula: Formula,
}

/// `action Name { stmts }`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    /// Action name.
    pub name: String,
    /// Statements.
    pub body: Vec<Stmt>,
}

/// Type expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeExpr {
    /// Named type / sort.
    Name(String),
    /// `(T1, T2, ...)` product / tuple type.
    Product(Vec<TypeExpr>),
    /// `(T1, T2, ...) -> U` or `T -> U`.
    Func {
        /// Parameter types (product).
        params: Vec<TypeExpr>,
        /// Result type.
        result: Box<TypeExpr>,
    },
}

/// Logical / relational formula or term (foundation: one expression AST).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Formula {
    /// Name / path segment use.
    Name(String),
    /// String literal.
    String(String),
    /// `forall (params) body`
    Forall {
        /// Binders.
        params: Vec<Param>,
        /// Body.
        body: Box<Formula>,
    },
    /// `exists (params) body`
    Exists {
        /// Binders.
        params: Vec<Param>,
        /// Body.
        body: Box<Formula>,
    },
    /// Infix binary op.
    BinOp {
        /// Operator.
        op: BinOp,
        /// Left.
        left: Box<Formula>,
        /// Right.
        right: Box<Formula>,
    },
    /// Unary op / postfix inv.
    UnaryOp {
        /// Operator.
        op: UnaryOp,
        /// Operand.
        expr: Box<Formula>,
    },
    /// `f(args...)` or `Path::f(args...)`.
    Call {
        /// Path segments.
        path: Vec<String>,
        /// Arguments.
        args: Vec<Formula>,
    },
    /// Parenthesized group (preserved only if needed; usually flattened).
    Group(Box<Formula>),
    /// `name : Type` (connection maps, signatures in bodies).
    TypedName {
        /// Name / path joined.
        name: String,
        /// Type.
        ty: TypeExpr,
    },
    /// Set comprehension `{ binder | pred }` (opaque text body for now as nested formula).
    SetComp {
        /// Left of `|`.
        head: Box<Formula>,
        /// Right of `|`.
        pred: Box<Formula>,
    },
}

/// `(name: Type)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub ty: TypeExpr,
}

/// Binary operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    /// `->` implication or type arrow in formula position.
    Arrow,
    /// `and`
    And,
    /// `or`
    Or,
    /// `==`
    Eq,
    /// `<=`
    Le,
    /// `<->` bidirectional (iff / reversible rewrite / connection).
    Iff,
    /// `in`
    In,
    /// `·`
    Mul,
    /// `+`
    Plus,
    /// `⊆`
    Subseteq,
    /// `⊇`
    Supseteq,
    /// `≅`
    Iso,
}

/// Unary operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    /// `not`
    Not,
    /// postfix `⁻¹`
    Inv,
}

/// Action statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stmt {
    /// `let name = expr`
    Let {
        /// Binding.
        name: String,
        /// Value.
        value: Formula,
    },
    /// Expression statement.
    Expr(Formula),
    /// `if cond { stmts }`
    If {
        /// Condition.
        cond: Formula,
        /// Then body.
        then_body: Vec<Stmt>,
    },
}
