//! Recursive-descent parser for the Living Metis grammar.
//!
//! Parameter lists are always `(x: T, ...)`. `->` is implication / function type.
//! `<->` is bidirectional (iff / reversible rewrite / connection head).

use crate::{
    ast::{Action, Axiom, BinOp, Connection, Formula, Island, Item, Module, Param, Relation, Rewrites, Stmt, Theorem, TypeExpr, UnaryOp},
    lexer::{lex_tokens, token_type::MetisTokenType},
};

/// Parse a Metis source string into a typed [`Module`].
pub fn parse_module(source: &str) -> Result<Module, String> {
    let raw = lex_tokens(source)?;
    let tokens: Vec<(MetisTokenType, String)> = raw.into_iter().filter(|(k, _)| !matches!(k, MetisTokenType::Whitespace | MetisTokenType::Comment | MetisTokenType::Eof)).collect();
    let mut p = Parser { tokens, idx: 0 };
    p.parse_module()
}

struct Parser {
    tokens: Vec<(MetisTokenType, String)>,
    idx: usize,
}

impl Parser {
    fn peek(&self) -> Option<&(MetisTokenType, String)> {
        self.tokens.get(self.idx)
    }

    fn peek_kind(&self) -> Option<MetisTokenType> {
        self.peek().map(|(k, _)| *k)
    }

    fn bump(&mut self) -> Result<(MetisTokenType, String), String> {
        let t = self.peek().cloned().ok_or_else(|| "unexpected end of input".to_string())?;
        self.idx += 1;
        Ok(t)
    }

    fn expect(&mut self, kind: MetisTokenType) -> Result<String, String> {
        let (k, text) = self.bump()?;
        if k != kind {
            return Err(format!("expected {kind:?}, got {k:?} ({text})"));
        }
        Ok(text)
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        let (k, text) = self.bump()?;
        if k != MetisTokenType::Ident {
            return Err(format!("expected identifier, got {k:?} ({text})"));
        }
        Ok(text)
    }

    fn parse_module(&mut self) -> Result<Module, String> {
        let mut usings = Vec::new();
        let mut islands = Vec::new();
        let mut actions = Vec::new();
        let mut current_ns: Option<String> = None;
        while let Some(k) = self.peek_kind() {
            match k {
                MetisTokenType::KwNamespace => {
                    current_ns = Some(self.parse_namespace()?);
                }
                MetisTokenType::KwUsing => {
                    usings.push(self.parse_using()?);
                }
                MetisTokenType::KwIsland => {
                    let mut island = self.parse_island()?;
                    island.namespace = current_ns.clone();
                    islands.push(island);
                }
                MetisTokenType::KwAction => actions.push(self.parse_action()?),
                MetisTokenType::KwRewrites => {
                    let rw = self.parse_rewrites()?;
                    if let Some(island) = islands.last_mut() {
                        island.items.push(Item::Rewrites(rw));
                    }
                    else {
                        islands.push(Island {
                            namespace: current_ns.clone(),
                            name: "_".into(),
                            extends: None,
                            items: vec![Item::Rewrites(rw)],
                        });
                    }
                }
                MetisTokenType::KwConnection => {
                    let c = self.parse_connection()?;
                    if let Some(island) = islands.last_mut() {
                        island.items.push(Item::Connection(c));
                    }
                    else {
                        islands.push(Island {
                            namespace: current_ns.clone(),
                            name: "_".into(),
                            extends: None,
                            items: vec![Item::Connection(c)],
                        });
                    }
                }
                other => return Err(format!("expected namespace/using/island/action/rewrites/connection, got {other:?}")),
            }
        }
        Ok(Module { usings, islands, actions })
    }

    fn parse_namespace(&mut self) -> Result<String, String> {
        self.expect(MetisTokenType::KwNamespace)?;
        let path = self.parse_path_string()?;
        self.maybe_semi();
        Ok(path)
    }

    fn parse_using(&mut self) -> Result<String, String> {
        self.expect(MetisTokenType::KwUsing)?;
        let path = self.parse_path_string()?;
        self.maybe_semi();
        Ok(path)
    }

    fn skip_attrs(&mut self) -> Result<(), String> {
        while self.peek_kind() == Some(MetisTokenType::LBracket) {
            self.bump()?;
            // consume until matching `]`
            let mut depth = 1;
            while depth > 0 {
                match self.peek_kind() {
                    Some(MetisTokenType::LBracket) => {
                        depth += 1;
                        self.bump()?;
                    }
                    Some(MetisTokenType::RBracket) => {
                        depth -= 1;
                        self.bump()?;
                    }
                    Some(_) => {
                        self.bump()?;
                    }
                    None => return Err("unclosed attribute `[`".into()),
                }
            }
        }
        Ok(())
    }

    fn parse_island(&mut self) -> Result<Island, String> {
        self.expect(MetisTokenType::KwIsland)?;
        let name = self.expect_ident()?;
        let extends = if self.peek_kind() == Some(MetisTokenType::Colon) {
            self.bump()?;
            Some(self.parse_path_string()?)
        }
        else {
            None
        };
        self.expect(MetisTokenType::LBrace)?;
        let mut items = Vec::new();
        while self.peek_kind() != Some(MetisTokenType::RBrace) {
            self.skip_attrs()?;
            items.push(self.parse_item()?);
        }
        self.expect(MetisTokenType::RBrace)?;
        Ok(Island { namespace: None, name, extends, items })
    }

    fn parse_item(&mut self) -> Result<Item, String> {
        match self.peek_kind() {
            Some(MetisTokenType::KwUse) => {
                self.bump()?;
                Ok(Item::Use(self.expect_ident()?))
            }
            Some(MetisTokenType::KwNode) => {
                self.bump()?;
                Ok(Item::Node(self.expect_ident()?))
            }
            Some(MetisTokenType::KwRelation) => Ok(Item::Relation(self.parse_relation()?)),
            Some(MetisTokenType::KwAxiom) => Ok(Item::Axiom(self.parse_axiom()?)),
            Some(MetisTokenType::KwTheorem) => Ok(Item::Theorem(self.parse_theorem()?)),
            Some(MetisTokenType::KwRewrites) => Ok(Item::Rewrites(self.parse_rewrites()?)),
            Some(MetisTokenType::KwConnection) => Ok(Item::Connection(self.parse_connection()?)),
            other => Err(format!("expected island item, got {other:?}")),
        }
    }

    fn parse_relation(&mut self) -> Result<Relation, String> {
        self.expect(MetisTokenType::KwRelation)?;
        let name = self.expect_ident()?;
        let mut ty = None;
        if self.peek_kind() == Some(MetisTokenType::Colon) {
            self.bump()?;
            ty = Some(self.parse_type()?);
        }
        let body = if self.peek_kind() == Some(MetisTokenType::LBrace) {
            self.bump()?;
            let f = self.parse_formula()?;
            self.expect(MetisTokenType::RBrace)?;
            Some(f)
        }
        else {
            None
        };
        Ok(Relation { name, ty, body })
    }

    fn parse_axiom(&mut self) -> Result<Axiom, String> {
        self.expect(MetisTokenType::KwAxiom)?;
        let name = self.expect_ident()?;
        self.expect(MetisTokenType::LBrace)?;
        let formula = self.parse_formula()?;
        self.expect(MetisTokenType::RBrace)?;
        Ok(Axiom { name, formula })
    }

    fn parse_theorem(&mut self) -> Result<Theorem, String> {
        self.expect(MetisTokenType::KwTheorem)?;
        let name = self.expect_ident()?;
        self.expect(MetisTokenType::LBrace)?;
        let formula = self.parse_formula()?;
        self.expect(MetisTokenType::RBrace)?;
        Ok(Theorem { name, formula })
    }

    fn parse_rewrites(&mut self) -> Result<Rewrites, String> {
        self.expect(MetisTokenType::KwRewrites)?;
        let name = self.expect_ident()?;
        self.expect(MetisTokenType::LBrace)?;
        let mut rules = Vec::new();
        while self.peek_kind() != Some(MetisTokenType::RBrace) {
            rules.push(self.parse_formula()?);
        }
        self.expect(MetisTokenType::RBrace)?;
        Ok(Rewrites { name, rules })
    }

    fn parse_connection(&mut self) -> Result<Connection, String> {
        self.expect(MetisTokenType::KwConnection)?;
        let left = self.expect_ident()?;
        self.expect(MetisTokenType::Iff)?;
        let right = self.expect_ident()?;
        self.expect(MetisTokenType::LBrace)?;
        let mut body = Vec::new();
        while self.peek_kind() != Some(MetisTokenType::RBrace) {
            body.push(self.parse_connection_item()?);
        }
        self.expect(MetisTokenType::RBrace)?;
        Ok(Connection { left, right, body })
    }

    fn parse_connection_item(&mut self) -> Result<Formula, String> {
        // `name : Type` or general formula
        if self.peek_kind() == Some(MetisTokenType::Ident) {
            let save = self.idx;
            let path = self.parse_path_string()?;
            if self.peek_kind() == Some(MetisTokenType::Colon) {
                self.bump()?;
                let ty = self.parse_type()?;
                return Ok(Formula::TypedName { name: path, ty });
            }
            self.idx = save;
        }
        self.parse_formula()
    }

    fn parse_action(&mut self) -> Result<Action, String> {
        self.expect(MetisTokenType::KwAction)?;
        let name = self.expect_ident()?;
        self.expect(MetisTokenType::LBrace)?;
        let mut body = Vec::new();
        while self.peek_kind() != Some(MetisTokenType::RBrace) {
            body.push(self.parse_stmt()?);
        }
        self.expect(MetisTokenType::RBrace)?;
        Ok(Action { name, body })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek_kind() {
            Some(MetisTokenType::KwLet) => {
                self.bump()?;
                let name = self.expect_ident()?;
                self.expect(MetisTokenType::Eq)?;
                let value = self.parse_formula()?;
                self.maybe_semi();
                Ok(Stmt::Let { name, value })
            }
            Some(MetisTokenType::KwIf) => {
                self.bump()?;
                let cond = self.parse_formula()?;
                self.expect(MetisTokenType::LBrace)?;
                let mut then_body = Vec::new();
                while self.peek_kind() != Some(MetisTokenType::RBrace) {
                    then_body.push(self.parse_stmt()?);
                }
                self.expect(MetisTokenType::RBrace)?;
                Ok(Stmt::If { cond, then_body })
            }
            Some(_) => {
                let expr = self.parse_formula()?;
                self.maybe_semi();
                Ok(Stmt::Expr(expr))
            }
            None => Err("expected statement".into()),
        }
    }

    fn maybe_semi(&mut self) {
        if self.peek_kind() == Some(MetisTokenType::Semi) {
            let _ = self.bump();
        }
    }

    fn parse_type(&mut self) -> Result<TypeExpr, String> {
        let left = self.parse_type_atom()?;
        if self.peek_kind() == Some(MetisTokenType::Arrow) {
            self.bump()?;
            let result = self.parse_type()?;
            return Ok(TypeExpr::Func { params: vec![left], result: Box::new(result) });
        }
        Ok(left)
    }

    fn parse_type_atom(&mut self) -> Result<TypeExpr, String> {
        match self.peek_kind() {
            Some(MetisTokenType::LParen) => {
                self.bump()?;
                let mut params = Vec::new();
                if self.peek_kind() != Some(MetisTokenType::RParen) {
                    loop {
                        params.push(self.parse_type()?);
                        match self.peek_kind() {
                            Some(MetisTokenType::Comma) => {
                                self.bump()?;
                            }
                            Some(MetisTokenType::RParen) => break,
                            other => return Err(format!("expected `,` or `)` in type, got {other:?}")),
                        }
                    }
                }
                self.expect(MetisTokenType::RParen)?;
                if self.peek_kind() == Some(MetisTokenType::Arrow) {
                    self.bump()?;
                    let result = self.parse_type()?;
                    Ok(TypeExpr::Func { params, result: Box::new(result) })
                }
                else if params.len() == 1 {
                    Ok(params.remove(0))
                }
                else {
                    Ok(TypeExpr::Product(params))
                }
            }
            Some(MetisTokenType::Ident) => Ok(TypeExpr::Name(self.parse_path_string()?)),
            other => Err(format!("expected type, got {other:?}")),
        }
    }

    fn parse_path_string(&mut self) -> Result<String, String> {
        let mut segs = vec![self.expect_ident()?];
        while self.peek_kind() == Some(MetisTokenType::PathSep) {
            self.bump()?;
            segs.push(self.expect_ident()?);
        }
        Ok(segs.join("::"))
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, String> {
        self.expect(MetisTokenType::LParen)?;
        let mut params = Vec::new();
        if self.peek_kind() != Some(MetisTokenType::RParen) {
            loop {
                let name = self.expect_ident()?;
                self.expect(MetisTokenType::Colon)?;
                let ty = self.parse_type()?;
                params.push(Param { name, ty });
                match self.peek_kind() {
                    Some(MetisTokenType::Comma) => {
                        self.bump()?;
                    }
                    Some(MetisTokenType::RParen) => break,
                    other => return Err(format!("expected `,` or `)` in params, got {other:?}")),
                }
            }
        }
        self.expect(MetisTokenType::RParen)?;
        Ok(params)
    }

    fn parse_formula(&mut self) -> Result<Formula, String> {
        self.parse_iff()
    }

    /// `<->` and `->` (right-associative chain).
    fn parse_iff(&mut self) -> Result<Formula, String> {
        let mut left = self.parse_and()?;
        loop {
            match self.peek_kind() {
                Some(MetisTokenType::Iff) => {
                    self.bump()?;
                    let right = self.parse_iff()?;
                    left = Formula::BinOp { op: BinOp::Iff, left: Box::new(left), right: Box::new(right) };
                }
                Some(MetisTokenType::Arrow) => {
                    self.bump()?;
                    let right = self.parse_iff()?;
                    left = Formula::BinOp { op: BinOp::Arrow, left: Box::new(left), right: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Formula, String> {
        let mut left = self.parse_cmp()?;
        loop {
            match self.peek_kind() {
                Some(MetisTokenType::KwAnd) => {
                    self.bump()?;
                    let right = self.parse_cmp()?;
                    left = Formula::BinOp { op: BinOp::And, left: Box::new(left), right: Box::new(right) };
                }
                Some(MetisTokenType::KwOr) => {
                    self.bump()?;
                    let right = self.parse_cmp()?;
                    left = Formula::BinOp { op: BinOp::Or, left: Box::new(left), right: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<Formula, String> {
        let mut left = self.parse_add()?;
        loop {
            let op = match self.peek_kind() {
                Some(MetisTokenType::EqEq) => BinOp::Eq,
                Some(MetisTokenType::OpLe) => BinOp::Le,
                Some(MetisTokenType::KwIn) => BinOp::In,
                Some(MetisTokenType::OpSubseteq) => BinOp::Subseteq,
                Some(MetisTokenType::OpSupseteq) => BinOp::Supseteq,
                Some(MetisTokenType::OpIso) => BinOp::Iso,
                _ => break,
            };
            self.bump()?;
            let right = self.parse_add()?;
            left = Formula::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<Formula, String> {
        let mut left = self.parse_postfix()?;
        loop {
            let op = match self.peek_kind() {
                Some(MetisTokenType::OpMul) => BinOp::Mul,
                Some(MetisTokenType::OpPlus) => BinOp::Plus,
                _ => break,
            };
            self.bump()?;
            let right = self.parse_postfix()?;
            left = Formula::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_postfix(&mut self) -> Result<Formula, String> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek_kind() {
                Some(MetisTokenType::OpInv) => {
                    self.bump()?;
                    expr = Formula::UnaryOp { op: UnaryOp::Inv, expr: Box::new(expr) };
                }
                Some(MetisTokenType::LParen) => {
                    // only if expr is Name/Call — treat as call on name path
                    let path = match expr {
                        Formula::Name(n) => n.split("::").map(|s| s.to_string()).collect(),
                        Formula::Call { path, args } if args.is_empty() => path,
                        other => {
                            // juxtaposition call not supported; leftover paren is grouping for next?
                            // If we already have a term and see `(`, it's a call only for names.
                            return Err(format!("unexpected call on non-name term: {other:?}"));
                        }
                    };
                    self.bump()?;
                    let mut args = Vec::new();
                    if self.peek_kind() != Some(MetisTokenType::RParen) {
                        loop {
                            args.push(self.parse_formula()?);
                            match self.peek_kind() {
                                Some(MetisTokenType::Comma) => {
                                    self.bump()?;
                                }
                                Some(MetisTokenType::RParen) => break,
                                other => return Err(format!("expected `,` or `)` in call, got {other:?}")),
                            }
                        }
                    }
                    self.expect(MetisTokenType::RParen)?;
                    expr = Formula::Call { path, args };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Formula, String> {
        match self.peek_kind() {
            Some(MetisTokenType::KwForall) => {
                self.bump()?;
                let params = self.parse_param_list()?;
                let body = self.parse_quant_body()?;
                Ok(Formula::Forall { params, body: Box::new(body) })
            }
            Some(MetisTokenType::KwExists) => {
                self.bump()?;
                let params = self.parse_param_list()?;
                let body = self.parse_quant_body()?;
                Ok(Formula::Exists { params, body: Box::new(body) })
            }
            Some(MetisTokenType::KwNot) => {
                self.bump()?;
                let expr = self.parse_postfix()?;
                Ok(Formula::UnaryOp { op: UnaryOp::Not, expr: Box::new(expr) })
            }
            Some(MetisTokenType::LParen) => {
                self.bump()?;
                // Could be param-looking or just grouping. Always parse formula then expect ).
                let inner = self.parse_formula()?;
                self.expect(MetisTokenType::RParen)?;
                Ok(Formula::Group(Box::new(inner)))
            }
            Some(MetisTokenType::LBrace) => {
                self.bump()?;
                let head = self.parse_formula()?;
                if self.peek_kind() == Some(MetisTokenType::Pipe) {
                    self.bump()?;
                    let pred = self.parse_formula()?;
                    self.expect(MetisTokenType::RBrace)?;
                    Ok(Formula::SetComp { head: Box::new(head), pred: Box::new(pred) })
                }
                else {
                    self.expect(MetisTokenType::RBrace)?;
                    Ok(head)
                }
            }
            Some(MetisTokenType::String) => {
                let (_, text) = self.bump()?;
                Ok(Formula::String(unquote(&text)?))
            }
            Some(MetisTokenType::Ident) => Ok(Formula::Name(self.parse_path_string()?)),
            other => Err(format!("expected formula, got {other:?}")),
        }
    }

    /// After `forall (params)`, body may start with `->` meaning the implication chain is the body.
    fn parse_quant_body(&mut self) -> Result<Formula, String> {
        if self.peek_kind() == Some(MetisTokenType::Arrow) {
            self.bump()?;
            self.parse_formula()
        }
        else {
            self.parse_formula()
        }
    }
}

fn unquote(text: &str) -> Result<String, String> {
    let t = text.strip_prefix('"').and_then(|s| s.strip_suffix('"')).ok_or_else(|| format!("bad string literal: {text}"))?;
    Ok(t.replace("\\\"", "\"").replace("\\\\", "\\"))
}
