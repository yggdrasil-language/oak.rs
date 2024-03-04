//! Valkyrie AST → `oak_pretty_print::Document`（printer 路径，不属于 `ast` 模块）。

use oak_pretty_print::{Document, SOFT_LINE as soft_line, doc, indent};

use crate::{
    ast::{Attribute, Block, Identifier, Let, MicroDeclaration, NamePath, NamespaceDeclaration, Param, Pattern, Return, Statement, StatementNode, StringLiteral, StringSegment, TermBinaryNode, TermExpression, TypeExpression, ValkyrieRoot},
    lexer::token_type::ValkyrieTokenType,
};

use super::PrintError;

/// 将 AST 根节点转为 pretty-print 文档。
pub(super) fn root_document(root: &ValkyrieRoot) -> Result<Document<'static>, PrintError> {
    if root.items.is_empty() {
        return Ok(Document::Nil);
    }
    let items = join_result_documents(&root.items, item_document, doc!(soft_line))?;
    Ok(Document::group(items))
}

fn item_document(item: &StatementNode) -> Result<Document<'static>, PrintError> {
    match item {
        StatementNode::Namespace(namespace) => namespace_document(namespace.as_ref()),
        StatementNode::Micro(micro) => micro_document(micro.as_ref()),
        other => Err(PrintError::Unsupported { context: format!("top-level item `{other:?}`") }),
    }
}

fn namespace_document(namespace: &NamespaceDeclaration) -> Result<Document<'static>, PrintError> {
    let name = namespace_name(namespace);
    if namespace.items.is_empty() {
        return Ok(doc!("namespace ", name, ";"));
    }
    let items = join_result_documents(&namespace.items, item_document, doc!(soft_line))?;
    Ok(Document::group(doc!("namespace ", name, " {", indent(doc!(soft_line, items)), "}")))
}

fn namespace_name(namespace: &NamespaceDeclaration) -> String {
    namespace.name.parts.iter().map(|part| part.name.as_str()).collect::<Vec<_>>().join("::")
}

fn micro_document(micro: &MicroDeclaration) -> Result<Document<'static>, PrintError> {
    let mut parts = Vec::new();
    for attribute in &micro.annotations {
        parts.push(attribute_document(attribute)?);
        parts.push(Document::text(" "));
    }
    parts.push(Document::text("micro "));
    parts.push(identifier_text(&micro.name));
    parts.push(Document::text("("));
    parts.push(join_result_documents(&micro.params, param_document, doc!(","))?);
    parts.push(Document::text(")"));
    if let Some(return_type) = &micro.return_type {
        parts.push(Document::text(" -> "));
        parts.push(type_document(return_type)?);
    }
    parts.push(Document::text(" "));
    parts.push(block_document(&micro.body)?);
    Ok(Document::concat(parts))
}

fn block_document(block: &Block) -> Result<Document<'static>, PrintError> {
    if block.statements.is_empty() {
        return Ok(Document::text("{}"));
    }
    let statements = join_result_documents(&block.statements, statement_document, doc!(soft_line))?;
    Ok(Document::group(doc!("{", indent(doc!(soft_line, statements)), "}")))
}

fn statement_document(statement: &Statement) -> Result<Document<'static>, PrintError> {
    match statement {
        Statement::Let(let_stmt) => let_document(let_stmt),
        Statement::Expression(expr_stmt) => {
            let mut parts = vec![term_document(&expr_stmt.expr)?];
            if expr_stmt.semi {
                parts.push(Document::text(";"));
            }
            Ok(Document::concat(parts))
        }
        Statement::Template(_) => Err(PrintError::Unsupported { context: "TGrammar template statement".into() }),
    }
}

fn let_document(let_stmt: &Let) -> Result<Document<'static>, PrintError> {
    let mut parts = vec![Document::text("let ")];
    if let_stmt.annotations.iter().any(|attribute| attribute.name.name == "mut") {
        parts.push(Document::text("mut "));
    }
    parts.push(pattern_document(&let_stmt.pattern)?);
    parts.push(Document::text("="));
    parts.push(term_document(&let_stmt.expr)?);
    Ok(Document::concat(parts))
}

fn pattern_document(pattern: &Pattern) -> Result<Document<'static>, PrintError> {
    match pattern {
        Pattern::Variable(variable) => Ok(identifier_text(&variable.name)),
        other => Err(PrintError::Unsupported { context: format!("pattern `{other:?}`") }),
    }
}

fn term_document(expression: &TermExpression) -> Result<Document<'static>, PrintError> {
    match expression {
        TermExpression::Binary(node) => binary_document(node),
        TermExpression::IntegerLiteral { value, .. } => Ok(Document::text(value.clone())),
        TermExpression::FloatLiteral { value, .. } => Ok(Document::text(value.clone())),
        TermExpression::Bool { value, .. } => Ok(Document::text(if *value { "true" } else { "false" })),
        TermExpression::StringLiteral(literal) => Ok(string_literal_document(literal)),
        TermExpression::NamePath(path) => Ok(Document::text(name_path_text(path))),
        TermExpression::ApplyCall { callee, args, .. } => apply_call_document(callee, args),
        TermExpression::Return(node) => return_document(node.as_ref()),
        other => Err(PrintError::Unsupported { context: format!("term expression `{other:?}`") }),
    }
}

fn apply_call_document(callee: &TermExpression, args: &[TermExpression]) -> Result<Document<'static>, PrintError> {
    let mut parts = vec![term_document(callee)?, Document::text("(")];
    parts.push(join_result_documents(args, |arg| term_document(&arg), doc!(","))?);
    parts.push(Document::text(")"));
    Ok(Document::concat(parts))
}

fn return_document(return_stmt: &Return) -> Result<Document<'static>, PrintError> {
    let mut parts = vec![Document::text("return")];
    if let Some(value) = &return_stmt.base {
        parts.push(Document::text(" "));
        parts.push(term_document(value)?);
    }
    Ok(Document::concat(parts))
}

fn param_document(param: &Param) -> Result<Document<'static>, PrintError> {
    let mut parts = vec![identifier_text(&param.name)];
    if let Some(ty) = &param.ty {
        parts.push(Document::text(": "));
        parts.push(type_document(ty)?);
    }
    Ok(Document::concat(parts))
}

fn type_document(ty: &TypeExpression) -> Result<Document<'static>, PrintError> {
    match ty {
        TypeExpression::Namepath(path) => Ok(Document::text(name_path_text(path.as_ref()))),
        other => Err(PrintError::Unsupported { context: format!("type expression `{other:?}`") }),
    }
}

fn attribute_document(attribute: &Attribute) -> Result<Document<'static>, PrintError> {
    let mut parts = vec![Document::text("["), identifier_text(&attribute.name)];
    if !attribute.args.is_empty() {
        parts.push(Document::text("("));
        parts.push(join_result_documents(&attribute.args, attribute_argument_document, doc!(","))?);
        parts.push(Document::text(")"));
    }
    parts.push(Document::text("]"));
    Ok(Document::concat(parts))
}

fn attribute_argument_document(argument: &crate::ast::AttributeArgument) -> Result<Document<'static>, PrintError> {
    let mut parts = Vec::new();
    if let Some(key) = &argument.key {
        parts.push(identifier_text(key));
        parts.push(Document::text(": "));
    }
    parts.push(term_document(&argument.value)?);
    Ok(Document::concat(parts))
}

fn binary_document(node: &TermBinaryNode) -> Result<Document<'static>, PrintError> {
    let operator = operator_text(node.operator).ok_or_else(|| PrintError::Unsupported { context: format!("binary operator `{:?}`", node.operator) })?;
    Ok(doc!(term_document(&node.lhs)?, operator, term_document(&node.rhs)?))
}

fn string_literal_document(literal: &StringLiteral) -> Document<'static> {
    let mut text = String::from("\"");
    for segment in &literal.segments {
        match segment {
            StringSegment::Text(text_segment) => text.push_str(&text_segment.content),
            StringSegment::Interpolation(_) => text.push('…'),
        }
    }
    text.push('"');
    Document::text(text)
}

fn identifier_text(id: &Identifier) -> Document<'static> {
    Document::text(id.name.clone())
}

fn name_path_text(path: &NamePath) -> String {
    path.parts.iter().map(|part| part.name.as_str()).collect::<Vec<_>>().join("::")
}

fn join_documents<I>(docs: I, separator: Document<'static>) -> Document<'static>
where
    I: IntoIterator<Item = Document<'static>>,
{
    Document::join(docs, separator)
}

fn join_result_documents<T, I, F>(items: I, map: F, separator: Document<'static>) -> Result<Document<'static>, PrintError>
where
    I: IntoIterator<Item = T>,
    F: Fn(T) -> Result<Document<'static>, PrintError>,
{
    let docs = items.into_iter().map(map).collect::<Result<Vec<_>, _>>()?;
    Ok(Document::join(docs, separator))
}

fn operator_text(operator: ValkyrieTokenType) -> Option<&'static str> {
    match operator {
        ValkyrieTokenType::Plus => Some("+"),
        ValkyrieTokenType::Minus => Some("-"),
        ValkyrieTokenType::Star => Some("*"),
        ValkyrieTokenType::Slash => Some("/"),
        ValkyrieTokenType::Percent => Some("%"),
        ValkyrieTokenType::EqEq => Some("=="),
        ValkyrieTokenType::NotEq => Some("!="),
        ValkyrieTokenType::LessThan => Some("<"),
        ValkyrieTokenType::GreaterThan => Some(">"),
        ValkyrieTokenType::LessEq => Some("<="),
        ValkyrieTokenType::GreaterEq => Some(">="),
        ValkyrieTokenType::AndAnd => Some("&&"),
        ValkyrieTokenType::OrOr => Some("||"),
        ValkyrieTokenType::Eq => Some("="),
        _ => None,
    }
}
