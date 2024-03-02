use crate::{ValkyrieLanguage, ast::*, lexer::token_type::ValkyrieTokenType};
use oak_core::{OakError, RedNode, RedTree, Source};

/// 跳过空白、换行和注释节点
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

/// 遍历子节点，跳过空白和注释
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

/// 获取必需的子表达式，如果不存在则返回错误
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

/// 获取必需的子节点，如果不存在则返回错误
pub(crate) fn get_required_child<'a, S: Source + ?Sized>(node: &'a RedNode<'a, ValkyrieLanguage>, source: &S, error_msg: &str, span_start: usize) -> Result<RedTree<'a, ValkyrieLanguage>, OakError> {
    for child in node.children() {
        if !should_skip_node(&child) {
            return Ok(child);
        }
    }
    Err(source.syntax_error(error_msg.to_string(), span_start))
}

/// 构建标识符表达式
pub(crate) fn build_identifier<S: Source + ?Sized>(node: &RedNode<ValkyrieLanguage>, source: &S, span: oak_core::Range<usize>) -> Result<Identifier, OakError> {
    let mut name = String::new();
    for child in node.children() {
        if let RedTree::Leaf(t) = child {
            if t.kind == ValkyrieTokenType::Identifier {
                name = crate::builder::identifier_text(source, t.span);
                return Ok(Identifier { name, span: t.span });
            }
        }
    }
    Ok(Identifier { name, span })
}

/// 构建名称路径
pub(crate) fn build_name_path<S: Source + ?Sized>(node: &RedNode<ValkyrieLanguage>, source: &S) -> Result<NamePath, OakError> {
    let span = node.span();
    let mut parts = Vec::new();

    for child in node.children() {
        if let RedTree::Leaf(t) = child {
            if !should_skip_node(&child) && t.kind == ValkyrieTokenType::Identifier {
                parts.push(Identifier { name: crate::builder::identifier_text(source, t.span), span: t.span });
            }
        }
    }
    Ok(NamePath { parts, span })
}

/// 构建参数列表
pub(crate) fn build_arg_list<S: Source + ?Sized>(node: &RedNode<ValkyrieLanguage>, source: &S, build_expr: &impl Fn(&RedNode<ValkyrieLanguage>, &S) -> Result<TermExpression, OakError>) -> Vec<TermExpression> {
    let mut args = Vec::new();
    for child in node.children() {
        if let RedTree::Node(arg_n) = child {
            if !should_skip_node(&child) {
                if let Ok(arg) = build_expr(&arg_n, source) {
                    args.push(arg);
                }
            }
        }
    }
    args
}

/// 构建二元表达式的通用函数
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

    for child in node.children() {
        if should_skip_node(&child) {
            continue;
        }

        match child {
            RedTree::Leaf(t) => {
                if op.is_none() && left.is_some() {
                    op = Some(t.kind);
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

/// 构建一元表达式的通用函数
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

/// 构建字段访问表达式的通用函数
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
                if t.kind == ValkyrieTokenType::Identifier && receiver.is_some() && field.is_none() {
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

/// 构建索引表达式的通用函数
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

/// 构建调用表达式的通用函数
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
                    args = build_arg_list(&n, source, &build_expr);
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

/// 构建块表达式的通用函数
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
                    if let Ok(stmt) = build_let(&n, source) {
                        statements.push(stmt);
                    }
                }
                crate::parser::element_type::ValkyrieElementType::ExprStatement => {
                    if let Ok(stmt) = build_expr_stmt(&n, source) {
                        statements.push(stmt);
                    }
                }
                crate::parser::element_type::ValkyrieElementType::TemplateIfStatement
                | crate::parser::element_type::ValkyrieElementType::TemplateLoop
                | crate::parser::element_type::ValkyrieElementType::TemplateMatch
                | crate::parser::element_type::ValkyrieElementType::TemplateStatement => {
                    if let Ok(stmt) = build_template_stmt(&n, source) {
                        statements.push(stmt);
                    }
                }
                _ => {
                    if let Ok(expr) = build_expr(&n, source) {
                        statements.push(Statement::Expression(Box::new(ExprStmt { annotations: Vec::new(), expr, semi: false, span: n.span() })));
                    }
                }
            }
        }
    }

    Ok(statements)
}
