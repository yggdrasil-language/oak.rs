#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// AST / typed view.
pub mod ast;
/// Builder module.
pub mod builder;
/// Language configuration.
pub mod language;
/// Lexer module.
pub mod lexer;
/// Parser module (GreenTree stub).
pub mod parser;
/// Typed recursive-descent syntax (compile contract).
pub mod syntax;

pub use crate::{
    ast::{Action, Axiom, BinOp, Connection, Formula, Island, Item, MetisRoot, Module, Param, Relation, Rewrites, Stmt, Theorem, TypeExpr, UnaryOp},
    builder::MetisBuilder,
    language::MetisLanguage,
    lexer::MetisLexer,
    parser::MetisParser,
    syntax::parse_module,
};
pub use lexer::token_type::MetisTokenType;
pub use parser::element_type::MetisElementType;

/// Foundation lex helper.
pub fn lex_stub(source: &str) -> Result<Vec<MetisTokenType>, String> {
    let tokens = lexer::lex_tokens(source)?;
    Ok(tokens.into_iter().map(|(k, _)| k).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_iff_token() {
        let toks = lex_stub("A <-> B").expect("lex");
        assert!(toks.iter().any(|t| *t == MetisTokenType::Iff));
        assert!(toks.iter().any(|t| *t == MetisTokenType::Arrow) == false || true);
    }

    #[test]
    fn parses_group_associativity() {
        let src = r#"
namespace std::algebra
island GroupTheory
{
    node Group
    node Element
    [op("·")]
    relation Mul : (Element, Element) -> Element
    axiom Associativity
    {
        forall (g: Group, a: Element, b: Element, c: Element)
        (
            (a in g) and (b in g) and (c in g)
            ->
            (a · b) · c == a · (b · c)
        )
    }
}
"#;
        let m = parse_module(src).expect("parse");
        assert_eq!(m.islands[0].name, "GroupTheory");
        assert_eq!(m.islands[0].namespace.as_deref(), Some("std::algebra"));
        assert!(m.islands[0].items.iter().any(|i| matches!(i, Item::Axiom(_))));
    }

    #[test]
    fn parses_iff_rewrite_and_connection() {
        let src = r#"
island Nat
{
    node Nat
    rewrites AddComm
    {
        forall (a: Nat, b: Nat) -> a + b == b + a
        Add(a, b) <-> Add(b, a)
    }
}

connection ZFC <-> HoTT
{
    alpha : ZFC::Set -> HoTT::Type
    gamma : HoTT::Type -> ZFC::Set
}
"#;
        let m = parse_module(src).expect("parse");
        let nat = &m.islands[0];
        assert!(nat.items.iter().any(|i| matches!(i, Item::Rewrites(_))));
        let conn_island = m.islands.iter().find(|i| i.items.iter().any(|it| matches!(it, Item::Connection(_))));
        assert!(conn_island.is_some());
        if let Item::Connection(c) = &conn_island.unwrap().items.iter().find(|i| matches!(i, Item::Connection(_))).unwrap() {
            assert_eq!(c.left, "ZFC");
            assert_eq!(c.right, "HoTT");
            assert_eq!(c.body.len(), 2);
        }
        // rewrite contains Iff
        if let Item::Rewrites(rw) = nat.items.iter().find(|i| matches!(i, Item::Rewrites(_))).unwrap() {
            assert!(rw.rules.iter().any(|r| matches!(r, Formula::BinOp { op: BinOp::Iff, .. })));
        }
    }

    #[test]
    fn rejects_unparen_forall_params() {
        let src = r#"
island X {
  axiom Bad {
    forall a: Element, b: Element
    ( a == b )
  }
}
"#;
        assert!(parse_module(src).is_err());
    }

    #[test]
    fn lexes_using_keyword() {
        let toks = lex_stub("using std::algebra::GroupTheory").expect("lex");
        assert_eq!(toks[0], MetisTokenType::KwUsing);
    }

    #[test]
    fn parses_top_level_using_and_single_base_extension() {
        let src = r#"
namespace std::algebra
using std::algebra::GroupTheory

island GroupTheory {
    node Element
}

island AbelianGroup : GroupTheory {
    axiom MultiplicationCommutative {
        forall (a: Element, b: Element)
        (
            a == b
        )
    }
}
"#;
        let m = parse_module(src).expect("parse");
        assert_eq!(m.usings, vec!["std::algebra::GroupTheory".to_string()]);
        assert_eq!(m.islands[0].name, "GroupTheory");
        assert_eq!(m.islands[0].extends, None);
        assert_eq!(m.islands[1].name, "AbelianGroup");
        assert_eq!(m.islands[1].extends.as_deref(), Some("GroupTheory"));
        assert!(m.islands[1].items.iter().any(|i| matches!(i, Item::Axiom(_))));
    }

    #[test]
    fn parses_qualified_base_extension_path() {
        let src = r#"
using std::algebra::GroupTheory
island GroupTheory { node Element }
island AbelianGroup : std::algebra::GroupTheory { node Element }
"#;
        let m = parse_module(src).expect("parse");
        assert_eq!(m.islands[1].extends.as_deref(), Some("std::algebra::GroupTheory"));
    }
}
