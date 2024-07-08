use crate::{ValkyrieLanguage, ast::*, lexer::token_type::ValkyrieTokenType};
use oak_core::{OakError, RedNode, RedTree, Source};

/// Skip whitespace, newline, and comment nodes
pub(crate) fn should_skip_node(node: &RedTree<ValkyrieLanguage>) -> bool {
    match node {
        RedTree::Leaf(t) => {
            matches!(t.kind, ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment)
        }
        RedTree::Node(n) => {
            matches!(
                n.green.kind,
                crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment
            )
        }
    }
}

/// Iterate child nodes, skipping whitespace and comments
pub(crate) fn for_each_non_skip_node<F>(node: &RedNode<ValkyrieLanguage>, f: &mut F)
where
    F: FnMut(&RedTree<ValkyrieLanguage>),
{
    for child in node.children() {
        if !should_skip_node(&child) {
            f(&child);
        }
    }
}

/// Get a required child expression, or return an error if missing
pub(crate) fn get_required_expr<S: Source + ?Sized>(
    node: &RedNode<ValkyrieLanguage>,
    source: &S,
    build_expr: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<TermExpression, OakError>,
    error_msg: &str,
    span_start: usize,
) -> Result<Box<TermExpression>, OakError> {
    let mut expr = None;
    for child in node.children() {
        if let RedTree::Node(n) = child {
            if !should_skip_node(&child) {
                expr = Some(Box::new(build_expr(&n, source)?));
                break;
            }
        }
    }
    expr.ok_or_else(|| source.syntax_error(error_msg.to_string(), span_start))
}

/// Get a required child node, or return an error if missing
pub(crate) fn get_required_child<'a, S: Source + ?Sized>(node: &'a RedNode<'a, ValkyrieLanguage>, source: &S, error_msg: &str, span_start: usize) -> Result<RedTree<'a, ValkyrieLanguage>, OakError> {
    for child in node.children() {
        if !should_skip_node(&child) {
            return Ok(child);
        }
    }
    Err(source.syntax_error(error_msg.to_string(), span_start))
}

/// Build an identifier expression
pub(crate) fn build_identifier<S: Source + ?Sized>(node: &RedNode<ValkyrieLanguage>, source: &S, span: oak_core::Range<usize>) -> Result<Identifier, OakError> {
    let mut name = String::new();
    for child in node.children() {
        if let RedTree::Leaf(t) = child {
            if t.kind.is_soft_identifier() || t.kind == ValkyrieTokenType::Keyword(crate::lexer::ValkyrieKeywords::SelfType) {
                name = crate::builder::identifier_text(source, t.span);
                return Ok(Identifier { name, span: t.span });
            }
        }
    }
    Ok(Identifier { name, span })
}

/// Build a name path
pub(crate) fn build_name_path<S: Source + ?Sized>(node: &RedNode<ValkyrieLanguage>, source: &S) -> Result<NamePath, OakError> {
    let span = node.span();
    let mut parts = Vec::new();

    for child in node.children() {
        if let RedTree::Leaf(t) = child {
            if should_skip_node(&child) {
                continue;
            }
            match t.kind {
                ValkyrieTokenType::Identifier => {
                    parts.push(Identifier { name: crate::builder::identifier_text(source, t.span), span: t.span });
                }
                ValkyrieTokenType::Keyword(crate::lexer::ValkyrieKeywords::SelfType) => {
                    parts.push(Identifier { name: "Self".to_string(), span: t.span });
                }
                _ => {}
            }
        }
    }
    Ok(NamePath { parts, span })
}

/// Build an argument list
pub(crate) fn build_arg_list<S: Source + ?Sized>(node: &RedNode<ValkyrieLanguage>, source: &S, build_expr: &impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<TermExpression, OakError>) -> Result<Vec<TermExpression>, OakError> {
    let mut args = Vec::new();
    for child in node.children() {
        if let RedTree::Node(arg_n) = child {
            if !should_skip_node(&child) {
                args.push(build_expr(&arg_n, source)?);
            }
        }
    }
    Ok(args)
}

/// Shared helper to build binary expressions
pub(crate) fn build_binary_expr<S: Source + ?Sized>(
    node: &RedNode<ValkyrieLanguage>,
    source: &S,
    build_expr: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<TermExpression, OakError>,
    default_op: ValkyrieTokenType,
) -> Result<(Box<TermExpression>, ValkyrieTokenType, Box<TermExpression>), OakError> {
    let span = node.span();
    let mut left = None;
    let mut right = None;
    let mut op = None;
    let mut operator_end = None;

    for child in node.children() {
        if should_skip_node(&child) {
            continue;
        }

        match child {
            RedTree::Leaf(t) => {
                if op.is_none() && left.is_some() {
                    op = Some(t.kind);
                    operator_end = Some(t.span.end);
                }
                else if op == Some(ValkyrieTokenType::GreaterThan) && t.kind == ValkyrieTokenType::GreaterThan && operator_end == Some(t.span.start) {
                    op = Some(ValkyrieTokenType::RightShift);
                }
            }
            RedTree::Node(n) => {
                if left.is_none() {
                    left = Some(Box::new(build_expr(&n, source)?));
                }
                else if right.is_none() {
                    right = Some(Box::new(build_expr(&n, source)?));
                }
            }
        }
    }

    let left = left.ok_or_else(|| source.syntax_error("Missing left operand".to_string(), span.start))?;
    let right = right.ok_or_else(|| source.syntax_error("Missing right operand".to_string(), span.start))?;
    let op = op.unwrap_or(default_op);

    Ok((left, op, right))
}

/// Shared helper to build unary expressions
pub(crate) fn build_unary_expr<S: Source + ?Sized>(
    node: &RedNode<ValkyrieLanguage>,
    source: &S,
    build_expr: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<TermExpression, OakError>,
    default_op: ValkyrieTokenType,
) -> Result<(ValkyrieTokenType, Box<TermExpression>), OakError> {
    let span = node.span();
    let mut expr = None;
    let mut op = None;

    for child in node.children() {
        if should_skip_node(&child) {
            continue;
        }

        match child {
            RedTree::Leaf(t) => {
                if op.is_none() {
                    op = Some(t.kind);
                }
            }
            RedTree::Node(n) => {
                expr = Some(Box::new(build_expr(&n, source)?));
            }
        }
    }

    let expr = expr.ok_or_else(|| source.syntax_error("Missing operand".to_string(), span.start))?;
    let op = op.unwrap_or(default_op);

    Ok((op, expr))
}

/// Shared helper to build field-access expressions
pub(crate) fn build_field_expr<S: Source + ?Sized>(node: &RedNode<ValkyrieLanguage>, source: &S, build_expr: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<TermExpression, OakError>) -> Result<(Box<TermExpression>, Identifier), OakError> {
    let span = node.span();
    let mut receiver = None;
    let mut field = None;

    for child in node.children() {
        if should_skip_node(&child) {
            continue;
        }

        match child {
            RedTree::Leaf(t) => {
                if t.kind.is_soft_identifier() && receiver.is_some() && field.is_none() {
                    field = Some(Identifier { name: crate::builder::text(source, t.span), span: t.span });
                }
            }
            RedTree::Node(n) => {
                if receiver.is_none() {
                    receiver = Some(Box::new(build_expr(&n, source)?));
                }
            }
        }
    }

    let receiver = receiver.ok_or_else(|| source.syntax_error("Missing receiver".to_string(), span.start))?;
    let field = field.ok_or_else(|| source.syntax_error("Missing field".to_string(), span.start))?;

    Ok((receiver, field))
}

/// Shared helper to build index expressions
pub(crate) fn build_index_expr<S: Source + ?Sized>(
    node: &RedNode<ValkyrieLanguage>,
    source: &S,
    build_expr: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<TermExpression, OakError>,
    error_msg: &str,
) -> Result<(Box<TermExpression>, Box<TermExpression>), OakError> {
    let span = node.span();
    let mut receiver = None;
    let mut index = None;

    for child in node.children() {
        if should_skip_node(&child) {
            continue;
        }

        if let RedTree::Node(n) = child {
            if receiver.is_none() {
                receiver = Some(Box::new(build_expr(&n, source)?));
            }
            else if index.is_none() {
                index = Some(Box::new(build_expr(&n, source)?));
            }
        }
    }

    let receiver = receiver.ok_or_else(|| source.syntax_error("Missing receiver".to_string(), span.start))?;
    let index = index.ok_or_else(|| source.syntax_error(error_msg.to_string(), span.start))?;

    Ok((receiver, index))
}

/// Shared helper to build call expressions
pub(crate) fn build_call_expr<S: Source + ?Sized>(node: &RedNode<ValkyrieLanguage>, source: &S, build_expr: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<TermExpression, OakError>) -> Result<(Box<TermExpression>, Vec<TermExpression>), OakError> {
    let span = node.span();
    let mut callee = None;
    let mut args = Vec::new();

    for child in node.children() {
        if should_skip_node(&child) {
            continue;
        }

        if let RedTree::Node(n) = child {
            match n.green.kind {
                crate::parser::element_type::ValkyrieElementType::ArgList => {
                    args = build_arg_list(&n, source, &build_expr)?;
                }
                _ => {
                    if callee.is_none() {
                        callee = Some(Box::new(build_expr(&n, source)?));
                    }
                }
            }
        }
    }

    let callee = callee.ok_or_else(|| source.syntax_error("Missing callee".to_string(), span.start))?;

    Ok((callee, args))
}

/// Shared helper to build block expressions
pub(crate) fn build_block_expr<S: Source + ?Sized>(
    node: &RedNode<ValkyrieLanguage>,
    source: &S,
    build_let: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<Statement, OakError>,
    build_expr_stmt: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<Statement, OakError>,
    build_template_stmt: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<Statement, OakError>,
    build_expr: impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<TermExpression, OakError>,
) -> Result<Vec<Statement>, OakError> {
    let mut statements = Vec::new();

    for child in node.children() {
        if should_skip_node(&child) {
            continue;
        }

        if let RedTree::Node(n) = child {
            match n.green.kind {
                crate::parser::element_type::ValkyrieElementType::LetStatement => {
                    statements.push(build_let(&n, source)?);
                }
                crate::parser::element_type::ValkyrieElementType::ExprStatement => {
                    statements.push(build_expr_stmt(&n, source)?);
                }
                crate::parser::element_type::ValkyrieElementType::TemplateIfStatement
                | crate::parser::element_type::ValkyrieElementType::TemplateLoop
                | crate::parser::element_type::ValkyrieElementType::TemplateMatch
                | crate::parser::element_type::ValkyrieElementType::TemplateStatement => {
                    statements.push(build_template_stmt(&n, source)?);
                }
                _ => {
                    let expr = build_expr(&n, source)?;
                    statements.push(Statement::Expression(Box::new(ExprStmt { annotations: Vec::new(), expr, semi: false, span: n.span() })));
                }
            }
        }
    }

    Ok(statements)
}
