use crate::{Language, ParserState, source::Source, tree::GreenNode};

/// Associativity of an operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    /// Left-associative (e.g., a + b + c is (a + b) + c).
    Left,
    /// Right-associative (e.g., a = b = c is a = (b = c)).
    Right,
    /// Non-associative.
    None,
}

/// Precedence and associativity of an operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperatorInfo {
    /// Precedence of the operator.
    pub precedence: u8,
    /// Associativity of the operator.
    pub associativity: Associativity,
}

impl OperatorInfo {
    /// Creates a left-associative operator info.
    pub fn left(precedence: u8) -> Self {
        Self { precedence, associativity: Associativity::Left }
    }

    /// Creates a right-associative operator info.
    pub fn right(precedence: u8) -> Self {
        Self { precedence, associativity: Associativity::Right }
    }

    /// Creates a non-associative operator info.
    pub fn none(precedence: u8) -> Self {
        Self { precedence, associativity: Associativity::None }
    }
}

/// A specification for a Pratt parser.
///
/// Users implement this trait to define the grammar rules for expressions.
/// Using a trait allows the compiler to optimize operator lookups using match statements.
///
/// # Example
///
/// ```rust
/// # use oak_core::{Language, ParserState, Pratt, binary, unary, OperatorInfo, Associativity, TokenType, ElementType, UniversalTokenRole, UniversalElementRole};
/// # use oak_core::tree::GreenNode;
/// # use oak_core::source::Source;
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// # #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// # enum MyToken { End, Plus, Star, Int }
/// # impl TokenType for MyToken {
/// #     const END_OF_STREAM: Self = MyToken::End;
/// #     type Role = UniversalTokenRole;
/// #     fn role(&self) -> Self::Role { UniversalTokenRole::None }
/// # }
/// #
/// # #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// # #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// # enum MyElement { Root }
/// # impl ElementType for MyElement {
/// #     type Role = UniversalElementRole;
/// #     fn role(&self) -> Self::Role { UniversalElementRole::None }
/// # }
/// # impl From<MyToken> for MyElement {
/// #     fn from(_: MyToken) -> Self { MyElement::Root }
/// # }
/// #
/// # struct MyLanguage;
/// # impl Language for MyLanguage {
/// #     const NAME: &'static str = "test";
/// #     type TokenType = MyToken;
/// #     type ElementType = MyElement;
/// #     type TypedRoot = ();
/// # }
/// # #[derive(Clone, Copy)]
/// # struct MyPratt;
/// # impl Pratt<MyLanguage> for MyPratt {
/// #     fn primary<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, MyLanguage, S>) -> &'a GreenNode<'a, MyLanguage> {
/// #         // Parse literals, identifiers, or parenthesized expressions
/// #         let cp = state.checkpoint();
/// #         state.finish_at(cp, MyElement::Root)
/// #     }
/// #
/// #     fn infix<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, MyLanguage, S>, left: &'a GreenNode<'a, MyLanguage>, min_precedence: u8) -> Option<&'a GreenNode<'a, MyLanguage>> {
/// #         let token = state.peek_at(0)?;
/// #         let info = match token.kind {
/// #             MyToken::Plus => Some(OperatorInfo::left(10)), // '+'
/// #             MyToken::Star => Some(OperatorInfo::left(20)), // '*'
/// #             _ => None,
/// #         }?;
/// #
/// #         if info.precedence < min_precedence { return None }
/// #
/// #         Some(binary(state, left, token.kind, info.precedence, info.associativity, MyElement::Root, |s, p| self.parse(s, p)))
/// #     }
/// # }
/// # impl MyPratt {
/// #     fn parse<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, MyLanguage, S>, min_precedence: u8) -> &'a GreenNode<'a, MyLanguage> {
/// #         oak_core::parser::PrattParser::new(*self).parse_expr(state, min_precedence)
/// #     }
/// # }
/// ```
pub trait Pratt<L: Language> {
    /// Parses a primary expression (e.g., literals, identifiers, group).
    fn primary<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, L, S>) -> &'a GreenNode<'a, L>;

    /// Handles prefix operators and primary expressions.
    ///
    /// Default implementation just calls `primary`.
    fn prefix<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, L, S>) -> &'a GreenNode<'a, L> {
        self.primary(state)
    }

    /// Handles infix and postfix operators.
    ///
    /// Should return `Some(new_node)` if an operator was parsed, or `None` if no operator matches
    /// or its precedence is lower than `min_precedence`.
    fn infix<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, L, S>, left: &'a GreenNode<'a, L>, min_precedence: u8) -> Option<&'a GreenNode<'a, L>>;
}

/// A helper for parsing binary (infix) expressions.
#[inline(always)]
pub fn binary<'a, L, S, F>(state: &mut ParserState<'a, L, S>, left: &'a GreenNode<'a, L>, op_kind: L::TokenType, op_precedence: u8, assoc: Associativity, result_kind: L::ElementType, mut parse_expr: F) -> &'a GreenNode<'a, L>
where
    L: Language,
    S: Source + ?Sized,
    F: FnMut(&mut ParserState<'a, L, S>, u8) -> &'a GreenNode<'a, L>,
    L::ElementType: From<L::TokenType>,
{
    let cp = state.checkpoint_before(left);
    state.expect(op_kind).ok();

    let next_prec = match assoc {
        Associativity::Left => op_precedence + 1,
        Associativity::Right => op_precedence,
        Associativity::None => op_precedence + 1,
    };

    let _right = parse_expr(state, next_prec);
    state.finish_at(cp, result_kind)
}

/// A helper for parsing unary (prefix) expressions.
#[inline(always)]
pub fn unary<'a, L, S, F>(state: &mut ParserState<'a, L, S>, op_kind: L::TokenType, op_precedence: u8, result_kind: L::ElementType, mut parse_expr: F) -> &'a GreenNode<'a, L>
where
    L: Language,
    S: Source + ?Sized,
    F: FnMut(&mut ParserState<'a, L, S>, u8) -> &'a GreenNode<'a, L>,
    L::ElementType: From<L::TokenType>,
{
    let cp = state.checkpoint();
    state.expect(op_kind).ok();
    let _right = parse_expr(state, op_precedence);
    state.finish_at(cp, result_kind)
}

/// A helper for parsing postfix expressions.
#[inline(always)]
pub fn postfix<'a, L, S>(state: &mut ParserState<'a, L, S>, left: &'a GreenNode<'a, L>, op_kind: L::TokenType, result_kind: L::ElementType) -> &'a GreenNode<'a, L>
where
    L: Language,
    S: Source + ?Sized,
    L::ElementType: From<L::TokenType>,
{
    let cp = state.checkpoint_before(left);
    state.expect(op_kind).ok();
    state.finish_at(cp, result_kind)
}

/// A Pratt parser implementation.
///
/// # Examples
///
/// ```rust
/// # use oak_core::{Language, TokenType, ElementType, UniversalTokenRole, UniversalElementRole, ParserState, Pratt, PrattParser, binary, unary, SyntaxArena, LexOutput, SourceText};
/// # use triomphe::Arc;
/// #
/// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// enum Token { Number, Plus, Minus, Star, Slash, Eof }
/// impl TokenType for Token {
///     const END_OF_STREAM: Self = Token::Eof;
///     type Role = UniversalTokenRole;
///     fn role(&self) -> Self::Role { UniversalTokenRole::None }
/// }
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// enum Element { Expr, Number, Plus, Minus, Star, Slash }
/// impl ElementType for Element {
///     type Role = UniversalElementRole;
///     fn role(&self) -> Self::Role { UniversalElementRole::None }
/// }
/// impl From<Token> for Element {
///     fn from(t: Token) -> Self {
///         match t {
///             Token::Number => Element::Number,
///             Token::Plus => Element::Plus,
///             Token::Minus => Element::Minus,
///             Token::Star => Element::Star,
///             Token::Slash => Element::Slash,
///             Token::Eof => unreachable!(),
///         }
///     }
/// }
///
/// #[derive(Clone, Copy)]
/// struct Lang;
/// impl Language for Lang {
///     const NAME: &'static str = "test";
///     type TokenType = Token;
///     type ElementType = Element;
///     type TypedRoot = ();
/// }
///
/// struct ExprParser;
/// impl Pratt<Lang> for ExprParser {
///     fn primary<'a, S: oak_core::Source + ?Sized>(&self, state: &mut ParserState<'a, Lang, S>) -> &'a oak_core::GreenNode<'a, Lang> {
///         let cp = state.checkpoint();
///         state.bump(); // bump number
///         state.finish_at(cp, Element::Number)
///     }
///
///     fn infix<'a, S: oak_core::Source + ?Sized>(&self, state: &mut ParserState<'a, Lang, S>, left: &'a oak_core::GreenNode<'a, Lang>, min_prec: u8) -> Option<&'a oak_core::GreenNode<'a, Lang>> {
///         let kind = state.peek_kind()?;
///         let (prec, assoc) = match kind {
///             Token::Plus | Token::Minus => (1, oak_core::Associativity::Left),
///             Token::Star | Token::Slash => (2, oak_core::Associativity::Left),
///             _ => return None,
///         };
///         if prec < min_prec { return None }
///         Some(binary(state, left, kind, prec, assoc, Element::Expr, |s, p| self.parse_expr(s, p)))
///     }
/// }
///
/// impl ExprParser {
///     fn parse_expr<'a, S: oak_core::Source + ?Sized>(&self, state: &mut ParserState<'a, Lang, S>, min_prec: u8) -> &'a oak_core::GreenNode<'a, Lang> {
///         PrattParser::new(ExprParser).parse_expr(state, min_prec)
///     }
/// }
/// ```
pub struct PrattParser<L: Language, T: Pratt<L>> {
    spec: T,
    _marker: core::marker::PhantomData<L>,
}

impl<L: Language, T: Pratt<L>> PrattParser<L, T> {
    /// Creates a new Pratt parser with the given specification.
    pub const fn new(spec: T) -> Self {
        Self { spec, _marker: core::marker::PhantomData }
    }

    /// Parses an expression with the given minimum precedence.
    pub fn parse_expr<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, L, S>, min_precedence: u8) -> &'a GreenNode<'a, L> {
        Self::parse(state, min_precedence, &self.spec)
    }

    /// Static version of parse_expr that takes a specification reference.
    pub fn parse<'a, S: Source + ?Sized>(state: &mut ParserState<'a, L, S>, min_precedence: u8, spec: &T) -> &'a GreenNode<'a, L> {
        let mut left = spec.prefix(state);
        while state.not_at_end() {
            let before = state.checkpoint();
            let Some(node) = spec.infix(state, left, min_precedence)
            else {
                break;
            };
            if state.checkpoint().0 == before.0 {
                break;
            }
            left = node
        }
        left
    }
}
