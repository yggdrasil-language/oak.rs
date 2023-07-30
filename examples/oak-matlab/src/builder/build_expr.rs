use crate::{
    MatlabLanguage,
    ast::{BinaryExpr, Expression, Identifier, UnaryExpr},
    builder::{MatlabBuilder, text, utils},
    lexer::token_type::MatlabTokenType,
    parser::element_type::MatlabElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source, TokenType};

impl<'config> MatlabBuilder<'config> {
    pub(crate) fn build_expr<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        match node.element_type() {
            MatlabElementType::Symbol => self.build_symbol(node, source),
            MatlabElementType::Literal => Ok(Expression::Literal { value: text(source, span.clone()), span }),
            MatlabElementType::Array => self.build_array(node, source),
            MatlabElementType::Call => self.build_call(node, source),
            MatlabElementType::BinaryExpr => self.build_binary(node, source),
            MatlabElementType::PrefixExpr => self.build_prefix(node, source),
            MatlabElementType::PostfixExpr => self.build_postfix(node, source),
            MatlabElementType::AnonymousFunction => self.build_anonymous_function(node, source),
            MatlabElementType::FunctionHandle => self.build_function_handle(node, source),
            MatlabElementType::Expression => {
                let mut inner = None;
                for child in node.children() {
                    if utils::is_trivia(&child) {
                        continue;
                    }
                    if let RedTree::Node(n) = child {
                        inner = Some(self.build_expr(n, source)?);
                        break;
                    }
                }
                let expression = inner.ok_or_else(|| source.syntax_error("Empty grouped expression".into(), span.start))?;
                Ok(Expression::Grouped { expression: Box::new(expression), span })
            }
            other => Err(source.syntax_error(format!("Unexpected MATLAB expression kind: {other:?}"), span.start)),
        }
    }

    fn build_symbol<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                if !t.kind().is_ignored() {
                    return Ok(Expression::Symbol(Identifier { name: text(source, t.span()), span: t.span() }));
                }
            }
        }
        Ok(Expression::Symbol(Identifier { name: text(source, span.clone()).trim().to_string(), span }))
    }

    fn build_array<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut rows: Vec<Vec<Expression>> = vec![Vec::new()];
        for child in node.children() {
            if utils::is_trivia(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) if t.kind() == MatlabTokenType::Semicolon => {
                    rows.push(Vec::new());
                }
                RedTree::Leaf(_) => {}
                RedTree::Node(n) => {
                    rows.last_mut().unwrap().push(self.build_expr(n, source)?);
                }
            }
        }
        if rows.last().is_some_and(|r| r.is_empty()) && rows.len() > 1 {
            rows.pop();
        }
        Ok(Expression::Array { rows, span })
    }

    fn build_call<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut head = None;
        let mut arg_groups: Vec<Vec<Expression>> = Vec::new();
        for child in node.children() {
            if utils::is_trivia(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) if t.kind() == MatlabTokenType::Identifier && head.is_none() => {
                    head = Some(Expression::Symbol(Identifier { name: text(source, t.span()), span: t.span() }));
                }
                RedTree::Node(n) if n.element_type() == MatlabElementType::Arguments => {
                    arg_groups.push(self.build_arguments(n, source)?);
                }
                RedTree::Node(n) if head.is_none() => {
                    head = Some(self.build_expr(n, source)?);
                }
                _ => {}
            }
        }
        let mut expr = head.ok_or_else(|| source.syntax_error("Call missing head".into(), span.start))?;
        if arg_groups.is_empty() {
            return Ok(Expression::Call { head: Box::new(expr), arguments: vec![], span });
        }
        for args in arg_groups {
            expr = Expression::Call { head: Box::new(expr), arguments: args, span: span.clone() };
        }
        Ok(expr)
    }

    fn build_arguments<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Vec<Expression>, OakError> {
        let mut args = Vec::new();
        for child in node.children() {
            if utils::is_trivia(&child) {
                continue;
            }
            if let RedTree::Node(n) = child {
                if n.element_type() != MatlabElementType::Error {
                    args.push(self.build_expr(n, source)?);
                }
            }
        }
        Ok(args)
    }

    fn build_binary<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut lhs = None;
        let mut rhs = None;
        let mut operator = None;
        for child in node.children() {
            if utils::is_trivia(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) => {
                    if lhs.is_some() && operator.is_none() {
                        operator = Some(t.kind());
                    }
                }
                RedTree::Node(n) => {
                    let expr = self.build_expr(n, source)?;
                    if lhs.is_none() {
                        lhs = Some(expr);
                    }
                    else {
                        rhs = Some(expr);
                    }
                }
            }
        }
        let lhs = lhs.ok_or_else(|| source.syntax_error("Binary missing lhs".into(), span.start))?;
        let rhs = rhs.ok_or_else(|| source.syntax_error("Binary missing rhs".into(), span.start))?;
        let operator = operator.ok_or_else(|| source.syntax_error("Binary missing operator".into(), span.start))?;
        Ok(Expression::Binary(Box::new(BinaryExpr { operator, lhs, rhs, span })))
    }

    fn build_prefix<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut operator = None;
        let mut operand = None;
        for child in node.children() {
            if utils::is_trivia(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) if operator.is_none() => operator = Some(t.kind()),
                RedTree::Node(n) => operand = Some(self.build_expr(n, source)?),
                _ => {}
            }
        }
        let operator = operator.ok_or_else(|| source.syntax_error("Prefix missing operator".into(), span.start))?;
        let operand = operand.ok_or_else(|| source.syntax_error("Prefix missing operand".into(), span.start))?;
        Ok(Expression::Prefix(Box::new(UnaryExpr { operator, operand, span })))
    }

    fn build_postfix<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut operator = None;
        let mut operand = None;
        for child in node.children() {
            if utils::is_trivia(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) if matches!(t.kind(), MatlabTokenType::Transpose | MatlabTokenType::DotTranspose) => {
                    operator = Some(t.kind());
                }
                RedTree::Node(n) => operand = Some(self.build_expr(n, source)?),
                _ => {}
            }
        }
        let operator = operator.ok_or_else(|| source.syntax_error("Postfix missing operator".into(), span.start))?;
        let operand = operand.ok_or_else(|| source.syntax_error("Postfix missing operand".into(), span.start))?;
        Ok(Expression::Postfix(Box::new(UnaryExpr { operator, operand, span })))
    }

    fn build_anonymous_function<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut parameters = Vec::new();
        let mut body = None;
        for child in node.children() {
            if utils::is_trivia(&child) {
                continue;
            }
            match child {
                RedTree::Node(n) if n.element_type() == MatlabElementType::Arguments => {
                    parameters = self.build_arguments(n, source)?;
                }
                RedTree::Node(n) => {
                    // First non-Arguments node after params is the body.
                    if body.is_none() {
                        body = Some(self.build_expr(n, source)?);
                    }
                }
                _ => {}
            }
        }
        let body = body.ok_or_else(|| source.syntax_error("Anonymous function missing body".into(), span.start))?;
        Ok(Expression::AnonymousFunction { parameters, body: Box::new(body), span })
    }

    fn build_function_handle<S: Source + ?Sized>(&self, node: RedNode<'_, MatlabLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut target = None;
        for child in node.children() {
            if utils::is_trivia(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) if t.kind() == MatlabTokenType::Identifier && target.is_none() => {
                    target = Some(Expression::Symbol(Identifier { name: text(source, t.span()), span: t.span() }));
                }
                RedTree::Node(n) if target.is_none() => {
                    target = Some(self.build_expr(n, source)?);
                }
                _ => {}
            }
        }
        let target = target.ok_or_else(|| source.syntax_error("Function handle missing target".into(), span.start))?;
        Ok(Expression::FunctionHandle { target: Box::new(target), span })
    }
}
