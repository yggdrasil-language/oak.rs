use crate::{
    WolframLanguage,
    ast::{BinaryExpr, Expression, Identifier, UnaryExpr},
    builder::{WolframBuilder, text, utils},
    lexer::token_type::WolframTokenType,
    parser::element_type::WolframElementType,
};
use oak_core::{OakError, RedNode, RedTree, Source, TokenType};

impl<'config> WolframBuilder<'config> {
    pub(crate) fn build_expr<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        match node.element_type() {
            WolframElementType::Symbol => self.build_symbol(node, source),
            WolframElementType::Literal => {
                let value = text(source, span.clone());
                Ok(Expression::Literal { value, span })
            }
            WolframElementType::List => {
                let mut elements = Vec::new();
                for child in utils::child_expr_nodes(node) {
                    elements.push(self.build_expr(child, source)?);
                }
                Ok(Expression::List { elements, span })
            }
            WolframElementType::Call => self.build_call(node, source),
            WolframElementType::Part => self.build_part(node, source),
            WolframElementType::BinaryExpr => self.build_binary(node, source),
            WolframElementType::PrefixExpr => self.build_prefix(node, source),
            WolframElementType::PostfixExpr => self.build_postfix(node, source),
            WolframElementType::Blank => self.build_blank(node, source),
            WolframElementType::Pattern => self.build_pattern(node, source),
            WolframElementType::Expression => {
                let mut inner = None;
                for child in utils::child_expr_nodes(node) {
                    inner = Some(self.build_expr(child, source)?);
                    break;
                }
                let expression = inner.ok_or_else(|| source.syntax_error("Empty grouped expression".into(), span.start))?;
                Ok(Expression::Grouped { expression: Box::new(expression), span })
            }
            WolframElementType::Error => Ok(Expression::Error { span }),
            other => Err(source.syntax_error(format!("Unexpected Wolfram expression kind: {other:?}"), span.start)),
        }
    }

    fn build_symbol<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                if !t.kind().is_ignored() && utils::is_symbol_like(t.kind()) {
                    return Ok(Expression::Symbol(Identifier { name: text(source, t.span()), span: t.span() }));
                }
            }
        }
        Ok(Expression::Symbol(Identifier { name: text(source, span.clone()).trim().to_string(), span }))
    }

    fn build_call<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut head: Option<Expression> = None;
        let mut arg_groups: Vec<Vec<Expression>> = Vec::new();

        for child in node.children() {
            if utils::should_skip(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) if utils::is_symbol_like(t.kind()) && head.is_none() => {
                    head = Some(Expression::Symbol(Identifier { name: text(source, t.span()), span: t.span() }));
                }
                RedTree::Node(n) if n.element_type() == WolframElementType::Arguments => {
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
        for (i, args) in arg_groups.into_iter().enumerate() {
            let call_span = if i + 1 == 1 { span.clone() } else { span.clone() };
            expr = Expression::Call { head: Box::new(expr), arguments: args, span: call_span };
        }
        Ok(expr)
    }

    fn build_part<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut expression = None;
        let mut indices = Vec::new();
        for child in node.children() {
            if utils::should_skip(&child) {
                continue;
            }
            match child {
                RedTree::Node(n) if n.element_type() == WolframElementType::Arguments => {
                    indices.extend(self.build_arguments(n, source)?);
                }
                RedTree::Node(n) if expression.is_none() => {
                    expression = Some(self.build_expr(n, source)?);
                }
                _ => {}
            }
        }
        let expression = expression.ok_or_else(|| source.syntax_error("Part missing expression".into(), span.start))?;
        Ok(Expression::Part { expression: Box::new(expression), indices, span })
    }

    fn build_arguments<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Vec<Expression>, OakError> {
        let mut args = Vec::new();
        for child in node.children() {
            if utils::should_skip(&child) {
                continue;
            }
            if let RedTree::Node(n) = child {
                if n.element_type() != WolframElementType::Error {
                    args.push(self.build_expr(n, source)?);
                }
            }
        }
        Ok(args)
    }

    fn build_binary<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut lhs = None;
        let mut rhs = None;
        let mut operator = None;
        for child in node.children() {
            if utils::should_skip(&child) {
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
        // Juxtaposition builds BinaryExpr without a Times leaf token.
        let operator = operator.unwrap_or(WolframTokenType::Times);
        Ok(Expression::Binary(Box::new(BinaryExpr { operator, lhs, rhs, span })))
    }

    fn build_prefix<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut operator = None;
        let mut operand = None;
        for child in node.children() {
            if utils::should_skip(&child) {
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

    fn build_postfix<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut operator = None;
        let mut operand = None;
        for child in node.children() {
            if utils::should_skip(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) if matches!(t.kind(), WolframTokenType::Ampersand | WolframTokenType::Factorial | WolframTokenType::Underscore | WolframTokenType::DoubleUnderscore | WolframTokenType::TripleUnderscore) => {
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

    fn build_blank<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut kind = WolframTokenType::Underscore;
        let mut head = None;
        let mut saw_blank = false;
        for child in node.children() {
            if utils::should_skip(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) => match t.kind() {
                    WolframTokenType::Underscore | WolframTokenType::DoubleUnderscore | WolframTokenType::TripleUnderscore => {
                        kind = t.kind();
                        saw_blank = true;
                    }
                    k if saw_blank && utils::is_symbol_like(k) => {
                        head = Some(Box::new(Expression::Symbol(Identifier { name: text(source, t.span()), span: t.span() })));
                    }
                    _ => {}
                },
                RedTree::Node(n) => head = Some(Box::new(self.build_expr(n, source)?)),
            }
        }
        Ok(Expression::Blank { kind, head, span })
    }

    fn build_pattern<S: Source + ?Sized>(&self, node: RedNode<'_, WolframLanguage>, source: &S) -> Result<Expression, OakError> {
        let span = node.span();
        let mut name = None;
        let mut blank = WolframTokenType::Underscore;
        for child in node.children() {
            if utils::should_skip(&child) {
                continue;
            }
            match child {
                RedTree::Leaf(t) if matches!(t.kind(), WolframTokenType::Underscore | WolframTokenType::DoubleUnderscore | WolframTokenType::TripleUnderscore) => {
                    blank = t.kind();
                }
                RedTree::Node(n) => name = Some(self.build_expr(n, source)?),
                _ => {}
            }
        }
        let name = name.ok_or_else(|| source.syntax_error("Pattern missing name".into(), span.start))?;
        Ok(Expression::Pattern { name: Box::new(name), blank, span })
    }
}
