use crate::{ast::*, builder::TypeScriptBuilder, language::TypeScriptLanguage, parser::element_type::TypeScriptElementType};
use core::range::Range;
use oak_core::{OakError, RedNode, RedTree, SourceText};

impl<'config> TypeScriptBuilder<'config> {
    pub(crate) fn build_root(&self, green_tree: &oak_core::GreenNode<TypeScriptLanguage>, source: &SourceText) -> Result<TypeScriptRoot, OakError> {
        let red_root = RedNode::new(green_tree, 0);
        let span = red_root.span();
        let mut statements = Vec::new();

        self.collect_statements(&red_root, source, &mut statements)?;

        let mut top_level_await_spans = Vec::new();
        for stmt in &statements {
            collect_top_level_await_statement(stmt, &mut top_level_await_spans);
        }

        Ok(TypeScriptRoot { statements, top_level_await_spans, span: span.into() })
    }

    pub(crate) fn collect_statements(&self, node: &RedNode<TypeScriptLanguage>, source: &SourceText, statements: &mut Vec<Statement>) -> Result<(), OakError> {
        let kind = node.green.kind;

        if kind == TypeScriptElementType::SourceFile || kind == TypeScriptElementType::Root {
            for child in node.children() {
                if let RedTree::Node(child_node) = child {
                    if let Some(stmt) = self.build_statement(&child_node, source)? {
                        statements.push(stmt);
                    }
                }
            }
        }
        else {
            if let Some(stmt) = self.build_statement(node, source)? {
                // If erase_types is true, we skip type-only statements.
                if self.erase_types {
                    match &stmt {
                        Statement::Interface(_) | Statement::TypeAlias(_) => return Ok(()),
                        _ => {}
                    }
                }
                statements.push(stmt)
            }
        }
        Ok(())
    }
}

/// Records the spans of `await` occurrences that are not nested inside a function boundary.
fn collect_top_level_await_statement(stmt: &Statement, spans: &mut Vec<Range<usize>>) {
    match stmt {
        Statement::VariableDeclaration(decl) => {
            if let Some(value) = &decl.value {
                collect_top_level_await_expression(value, spans);
            }
        }
        Statement::ExpressionStatement(s) => collect_top_level_await_expression(&s.expression, spans),
        Statement::ReturnStatement(s) => {
            if let Some(arg) = &s.argument {
                collect_top_level_await_expression(arg, spans);
            }
        }
        Statement::ThrowStatement(s) => collect_top_level_await_expression(&s.argument, spans),
        Statement::IfStatement(s) => {
            collect_top_level_await_expression(&s.test, spans);
            collect_top_level_await_statement(&s.consequent, spans);
            if let Some(alternate) = &s.alternate {
                collect_top_level_await_statement(alternate, spans);
            }
        }
        Statement::WhileStatement(s) => {
            collect_top_level_await_expression(&s.test, spans);
            collect_top_level_await_statement(&s.body, spans);
        }
        Statement::DoWhileStatement(s) => {
            collect_top_level_await_statement(&s.body, spans);
            collect_top_level_await_expression(&s.test, spans);
        }
        Statement::ForStatement(s) => {
            if let Some(initializer) = &s.initializer {
                collect_top_level_await_statement(initializer, spans);
            }
            if let Some(test) = &s.test {
                collect_top_level_await_expression(test, spans);
            }
            if let Some(incrementor) = &s.incrementor {
                collect_top_level_await_expression(incrementor, spans);
            }
            collect_top_level_await_statement(&s.body, spans);
        }
        Statement::ForInStatement(s) => {
            collect_top_level_await_statement(&s.left, spans);
            collect_top_level_await_expression(&s.right, spans);
            collect_top_level_await_statement(&s.body, spans);
        }
        Statement::ForOfStatement(s) => {
            if let Some(marker) = &s.await_marker {
                spans.push(marker.span.clone());
            }
            collect_top_level_await_statement(&s.left, spans);
            collect_top_level_await_expression(&s.right, spans);
            collect_top_level_await_statement(&s.body, spans);
        }
        Statement::SwitchStatement(s) => {
            collect_top_level_await_expression(&s.discriminant, spans);
            for case in &s.cases {
                if let Some(test) = &case.test {
                    collect_top_level_await_expression(test, spans);
                }
                for inner in &case.consequent {
                    collect_top_level_await_statement(inner, spans);
                }
            }
        }
        Statement::TryStatement(s) => {
            for inner in &s.block {
                collect_top_level_await_statement(inner, spans);
            }
            if let Some(handler) = &s.handler {
                for inner in &handler.body {
                    collect_top_level_await_statement(inner, spans);
                }
            }
            if let Some(finalizer) = &s.finalizer {
                for inner in finalizer {
                    collect_top_level_await_statement(inner, spans);
                }
            }
        }
        Statement::BlockStatement(s) => {
            for inner in &s.statements {
                collect_top_level_await_statement(inner, spans);
            }
        }
        Statement::ExportDeclaration(s) => {
            if let Some(declaration) = &s.declaration {
                collect_top_level_await_statement(declaration, spans);
            }
        }
        // Function boundaries and non-expression declarations stop the descent.
        Statement::FunctionDeclaration(_)
        | Statement::ClassDeclaration(_)
        | Statement::Namespace(_)
        | Statement::ImportDeclaration(_)
        | Statement::Interface(_)
        | Statement::TypeAlias(_)
        | Statement::Enum(_)
        | Statement::BreakStatement(_)
        | Statement::ContinueStatement(_) => {}
    }
}

/// Records top-level `await` spans inside an expression, stopping at function boundaries.
fn collect_top_level_await_expression(expr: &Expression, spans: &mut Vec<Range<usize>>) {
    match &*expr.kind {
        ExpressionKind::AwaitExpression(argument) => {
            spans.push(expr.span.clone());
            collect_top_level_await_expression(argument, spans);
        }
        ExpressionKind::UnaryExpression { argument, .. } | ExpressionKind::UpdateExpression { argument, .. } | ExpressionKind::NonNullExpression(argument) | ExpressionKind::SpreadElement(argument) => {
            collect_top_level_await_expression(argument, spans);
        }
        ExpressionKind::BinaryExpression { left, right, .. } | ExpressionKind::AssignmentExpression { left, right, .. } => {
            collect_top_level_await_expression(left, spans);
            collect_top_level_await_expression(right, spans);
        }
        ExpressionKind::ConditionalExpression { test, consequent, alternate } => {
            collect_top_level_await_expression(test, spans);
            collect_top_level_await_expression(consequent, spans);
            collect_top_level_await_expression(alternate, spans);
        }
        ExpressionKind::MemberExpression { object, property, .. } => {
            collect_top_level_await_expression(object, spans);
            collect_top_level_await_expression(property, spans);
        }
        ExpressionKind::CallExpression { func, args } | ExpressionKind::NewExpression { func, args } => {
            collect_top_level_await_expression(func, spans);
            for arg in args {
                collect_top_level_await_expression(arg, spans);
            }
        }
        ExpressionKind::AsExpression { expression, .. } | ExpressionKind::TypeAssertionExpression { expression, .. } => {
            collect_top_level_await_expression(expression, spans);
        }
        ExpressionKind::ObjectLiteral { properties } => {
            for property in properties {
                match property {
                    ObjectProperty::Property { value, .. } => collect_top_level_await_expression(value, spans),
                    ObjectProperty::Spread(expr) => collect_top_level_await_expression(expr, spans),
                }
            }
        }
        ExpressionKind::ArrayLiteral { elements } => {
            for element in elements {
                collect_top_level_await_expression(element, spans);
            }
        }
        ExpressionKind::YieldExpression(Some(argument)) => collect_top_level_await_expression(argument, spans),
        ExpressionKind::ImportExpression { module_specifier } => collect_top_level_await_expression(module_specifier, spans),
        ExpressionKind::TaggedTemplateExpression { tag, template } => {
            collect_top_level_await_expression(tag, spans);
            collect_top_level_await_expression(template, spans);
        }
        // Function boundaries stop the descent.
        ExpressionKind::ArrowFunction { .. } | ExpressionKind::FunctionExpression { .. } => {}
        ExpressionKind::Identifier(_)
        | ExpressionKind::NumericLiteral(_)
        | ExpressionKind::StringLiteral(_)
        | ExpressionKind::BigIntLiteral(_)
        | ExpressionKind::BooleanLiteral(_)
        | ExpressionKind::NullLiteral
        | ExpressionKind::RegexLiteral(_)
        | ExpressionKind::TemplateString(_)
        | ExpressionKind::MetaProperty { .. }
        | ExpressionKind::YieldExpression(None)
        | ExpressionKind::JsxElement(_)
        | ExpressionKind::JsxFragment(_)
        | ExpressionKind::JsxSelfClosingElement(_) => {}
    }
}
