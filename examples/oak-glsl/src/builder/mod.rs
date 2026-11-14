use crate::{GlslParser, ast::*, language::GlslLanguage, lexer::token_type::GlslTokenType, parser::element_type::GlslElementType};
use oak_core::{Builder, BuilderCache, GreenNode, OakDiagnostics, OakError, Parser, RedLeaf, RedNode, RedTree, TextEdit, TokenType, builder::BuildOutput, parser::session::ParseSession, source::Source};

/// AST builder for the GLSL language.
#[derive(Clone, Copy)]
pub struct GlslBuilder<'config> {
    /// Language configuration.
    config: &'config GlslLanguage,
}

impl<'config> GlslBuilder<'config> {
    /// Creates a new GLSL builder with the given language configuration.
    pub const fn new(config: &'config GlslLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<GlslLanguage> for GlslBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<GlslLanguage>) -> BuildOutput<GlslLanguage> {
        let parser = GlslParser::new(self.config);
        let mut session = ParseSession::<GlslLanguage>::default();
        let parse_result = parser.parse(source, edits, &mut session);

        match parse_result.result {
            Ok(green_tree) => match self.build_root(green_tree, source) {
                Ok(ast_root) => OakDiagnostics { result: Ok(ast_root), diagnostics: parse_result.diagnostics },
                Err(build_error) => {
                    let mut diagnostics = parse_result.diagnostics;
                    diagnostics.push(build_error.clone());
                    OakDiagnostics { result: Err(build_error), diagnostics }
                }
            },
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics: parse_result.diagnostics },
        }
    }
}

impl<'config> GlslBuilder<'config> {
    fn build_root<'a, S: Source + ?Sized>(&self, green_tree: &'a GreenNode<'a, GlslLanguage>, source: &S) -> Result<GlslRoot, OakError> {
        let red_root = RedNode::new(green_tree, 0);
        let mut items = Vec::new();

        for child in red_root.children() {
            if let RedTree::Node(node) = child {
                match node.element_type() {
                    GlslElementType::FunctionDeclaration => items.push(GlslItem::Function(build_function(node, source))),
                    GlslElementType::StructDeclaration => items.push(GlslItem::Struct(build_struct(node, source))),
                    GlslElementType::VariableDeclaration => items.push(GlslItem::GlobalVar(build_global_var(node, source))),
                    GlslElementType::GlobalConstDeclaration => items.push(GlslItem::Const(build_const(node, source))),
                    _ => {}
                }
            }
        }

        Ok(GlslRoot { items })
    }
}

/// Builds a function node.
fn build_function<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslFunction {
    let mut params = Vec::new();
    let mut return_type = None;
    let mut body = Vec::new();

    for child in node.children() {
        if let RedTree::Node(child) = child {
            match child.element_type() {
                GlslElementType::ParameterList => params = build_parameter_list(child, source),
                GlslElementType::TypeReference => {
                    if return_type.is_none() {
                        return_type = Some(build_type(child, source));
                    }
                }
                GlslElementType::Block => body = build_block(child, source),
                _ => {}
            }
        }
    }

    GlslFunction { layout: build_layouts(node, source), name: declared_name(node, source), params, return_type, body }
}

/// Builds a struct node.
fn build_struct<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslStruct {
    let mut members = Vec::new();

    for child in node.children() {
        if let RedTree::Node(child) = child {
            if child.element_type() == GlslElementType::StructMemberDeclaration {
                members.push(build_struct_member(child, source));
            }
        }
    }

    GlslStruct { name: declared_name(node, source), members }
}

/// Builds a module-scope declaration node.
fn build_global_var<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslGlobalVar {
    let mut ty = None;
    let mut members = Vec::new();
    let mut init = None;

    for child in node.children() {
        if let RedTree::Node(child) = child {
            match child.element_type() {
                GlslElementType::StructMemberDeclaration => members.push(build_struct_member(child, source)),
                GlslElementType::TypeReference if ty.is_none() => ty = Some(build_type(child, source)),
                kind if is_expression_kind(kind) => init = Some(build_expression(child, source)),
                _ => {}
            }
        }
    }

    GlslGlobalVar { layout: build_layouts(node, source), qualifiers: qualifiers(node), name: declared_name(node, source), ty, members, init }
}

/// Builds a module-scope `const` declaration node.
fn build_const<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslConst {
    let mut ty = None;
    let mut init = None;

    for child in node.children() {
        if let RedTree::Node(child) = child {
            match child.element_type() {
                GlslElementType::TypeReference => ty = Some(build_type(child, source)),
                kind if is_expression_kind(kind) => init = Some(build_expression(child, source)),
                _ => {}
            }
        }
    }

    GlslConst { name: declared_name(node, source), ty, init: init.unwrap_or_else(empty_expression) }
}

/// Builds the parameters of a parameter list node.
fn build_parameter_list<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> Vec<GlslParam> {
    let mut params = Vec::new();
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if child.element_type() == GlslElementType::Parameter {
                params.push(build_param(child, source));
            }
        }
    }
    params
}

/// Builds a function parameter node.
fn build_param<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslParam {
    let mut ty = empty_type();
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if child.element_type() == GlslElementType::TypeReference {
                ty = build_type(child, source);
            }
        }
    }
    GlslParam { qualifiers: qualifiers(node), name: declared_name(node, source), ty }
}

/// Builds a struct or interface block member node.
fn build_struct_member<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslStructMember {
    let mut ty = empty_type();

    for child in node.children() {
        if let RedTree::Node(child) = child {
            if child.element_type() == GlslElementType::TypeReference {
                ty = build_type(child, source);
            }
        }
    }

    GlslStructMember { layout: build_layouts(node, source), qualifiers: qualifiers(node), name: declared_name(node, source), ty }
}

/// Builds every `layout(...)` entry attached to a node.
fn build_layouts<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> Vec<GlslLayoutQualifier> {
    let mut layouts = Vec::new();
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if child.element_type() == GlslElementType::LayoutQualifier {
                layouts.push(build_layout(child, source));
            }
        }
    }
    layouts
}

/// Builds one `layout(...)` entry such as `binding = 0`.
fn build_layout<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslLayoutQualifier {
    let mut name = String::new();
    let mut value = None;

    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if name.is_empty() && !GlslTokenType::is_ignored(&token.kind) && token.kind != GlslTokenType::Assign {
                    name = leaf_text(&token, source);
                }
            }
            RedTree::Node(child) => {
                if value.is_none() && is_expression_kind(child.element_type()) {
                    value = Some(build_expression(child, source));
                }
            }
        }
    }

    GlslLayoutQualifier { name, value }
}

/// Builds the statements of a block node.
fn build_block<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> Vec<GlslStatement> {
    let mut statements = Vec::new();
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if is_statement_kind(child.element_type()) {
                statements.push(build_statement(child, source));
            }
        }
    }
    statements
}

/// Builds a single statement node.
fn build_statement<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslStatement {
    match node.element_type() {
        GlslElementType::VarStatement => {
            let mut ty = None;
            let mut init = None;

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        GlslElementType::TypeReference => ty = Some(build_type(child, source)),
                        kind if is_expression_kind(kind) => init = Some(build_expression(child, source)),
                        _ => {}
                    }
                }
            }

            let qualifiers = qualifiers(node);
            let is_const = qualifiers.contains(&GlslQualifier::Const);
            GlslStatement::Var { qualifiers, name: declared_name(node, source), ty, init, is_const }
        }
        GlslElementType::AssignmentStatement => {
            let mut expressions = Vec::new();
            let mut operator_kinds = Vec::new();

            collect_expressions_and_leaves(node, source, &mut expressions, &mut operator_kinds);

            let target = expressions.first().cloned().unwrap_or_else(empty_expression);
            let value = expressions.get(1).cloned().unwrap_or_else(empty_expression);

            GlslStatement::Assign { target, op: assignment_op(&operator_kinds), value }
        }
        GlslElementType::IfStatement => {
            let mut cond = empty_expression();
            let mut then_body = Vec::new();
            let mut else_body = Vec::new();
            let mut seen_then = false;

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        GlslElementType::Block => {
                            if seen_then {
                                else_body = build_block(child, source);
                            }
                            else {
                                then_body = build_block(child, source);
                                seen_then = true;
                            }
                        }
                        GlslElementType::IfStatement => else_body = vec![build_statement(child, source)],
                        kind if is_expression_kind(kind) => cond = build_expression(child, source),
                        _ => {}
                    }
                }
            }

            GlslStatement::If { cond, then_body, else_body }
        }
        GlslElementType::ForStatement => {
            let mut init = None;
            let mut cond = None;
            let mut update = None;
            let mut body = Vec::new();

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        GlslElementType::ForInit => init = first_statement(child, source).map(Box::new),
                        GlslElementType::ForCond => cond = first_expression(child, source),
                        GlslElementType::ForUpdate => update = first_statement(child, source).map(Box::new),
                        GlslElementType::Block => body = build_block(child, source),
                        _ => {}
                    }
                }
            }

            GlslStatement::For { init, cond, update, body }
        }
        GlslElementType::WhileStatement => {
            let mut cond = empty_expression();
            let mut body = Vec::new();

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        GlslElementType::Block => body = build_block(child, source),
                        kind if is_expression_kind(kind) => cond = build_expression(child, source),
                        _ => {}
                    }
                }
            }

            GlslStatement::While { cond, body }
        }
        GlslElementType::DoWhileStatement => {
            let mut cond = empty_expression();
            let mut body = Vec::new();

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        GlslElementType::Block => body = build_block(child, source),
                        kind if is_expression_kind(kind) => cond = build_expression(child, source),
                        _ => {}
                    }
                }
            }

            GlslStatement::DoWhile { body, cond }
        }
        GlslElementType::BreakStatement => GlslStatement::Break,
        GlslElementType::ContinueStatement => GlslStatement::Continue,
        GlslElementType::DiscardStatement => GlslStatement::Discard,
        GlslElementType::ReturnStatement => GlslStatement::Return(first_expression(node, source)),
        GlslElementType::ExpressionStatement => GlslStatement::Expr(first_expression(node, source).unwrap_or_else(empty_expression)),
        GlslElementType::Block => GlslStatement::Block(build_block(node, source)),
        _ => GlslStatement::Expr(empty_expression()),
    }
}

/// Returns the first statement nested in a header node.
fn first_statement<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> Option<GlslStatement> {
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if is_statement_kind(child.element_type()) {
                return Some(build_statement(child, source));
            }
        }
    }
    None
}

/// Returns the first expression nested in a node.
fn first_expression<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> Option<GlslExpression> {
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if is_expression_kind(child.element_type()) {
                return Some(build_expression(child, source));
            }
        }
    }
    None
}

/// Collects nested expressions and significant leaf kinds in source order.
fn collect_expressions_and_leaves<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S, expressions: &mut Vec<GlslExpression>, leaves: &mut Vec<GlslTokenType>) {
    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if !GlslTokenType::is_ignored(&token.kind) {
                    leaves.push(token.kind);
                }
            }
            RedTree::Node(child) => {
                if is_expression_kind(child.element_type()) {
                    expressions.push(build_expression(child, source));
                }
            }
        }
    }
}

/// Returns the declaration name of a node, which is its first own identifier.
fn declared_name<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> String {
    for child in node.children() {
        if let RedTree::Leaf(token) = child {
            if token.kind == GlslTokenType::Identifier {
                return leaf_text(&token, source);
            }
        }
    }
    String::new()
}

/// Returns the qualifiers attached to a declaration node.
fn qualifiers(node: RedNode<GlslLanguage>) -> Vec<GlslQualifier> {
    let mut result = Vec::new();
    for child in node.children() {
        if let RedTree::Leaf(token) = child {
            if let Some(qualifier) = qualifier_of(token.kind) {
                if !result.contains(&qualifier) {
                    result.push(qualifier);
                }
            }
        }
    }
    result
}

/// Maps a token kind to a declaration qualifier.
fn qualifier_of(kind: GlslTokenType) -> Option<GlslQualifier> {
    Some(match kind {
        GlslTokenType::Const => GlslQualifier::Const,
        GlslTokenType::Uniform => GlslQualifier::Uniform,
        GlslTokenType::In => GlslQualifier::In,
        GlslTokenType::Out => GlslQualifier::Out,
        GlslTokenType::Inout => GlslQualifier::InOut,
        GlslTokenType::Buffer => GlslQualifier::Buffer,
        GlslTokenType::Shared => GlslQualifier::Shared,
        GlslTokenType::Readonly => GlslQualifier::Readonly,
        GlslTokenType::Writeonly => GlslQualifier::Writeonly,
        GlslTokenType::Coherent => GlslQualifier::Coherent,
        GlslTokenType::Volatile => GlslQualifier::Volatile,
        GlslTokenType::Restrict => GlslQualifier::Restrict,
        GlslTokenType::Flat => GlslQualifier::Flat,
        GlslTokenType::Smooth => GlslQualifier::Smooth,
        GlslTokenType::Noperspective => GlslQualifier::NoPerspective,
        GlslTokenType::Centroid => GlslQualifier::Centroid,
        GlslTokenType::Sample => GlslQualifier::Sample,
        GlslTokenType::Invariant => GlslQualifier::Invariant,
        GlslTokenType::Patch => GlslQualifier::Patch,
        GlslTokenType::Precision => GlslQualifier::Precision,
        GlslTokenType::Highp => GlslQualifier::Highp,
        GlslTokenType::Mediump => GlslQualifier::Mediump,
        GlslTokenType::Lowp => GlslQualifier::Lowp,
        _ => return None,
    })
}

/// Builds a type node.
fn build_type<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslType {
    let mut name = String::new();
    let mut args = Vec::new();

    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if name.is_empty() && !GlslTokenType::is_ignored(&token.kind) {
                    name = leaf_text(&token, source);
                }
            }
            RedTree::Node(child) => {
                if child.element_type() == GlslElementType::TypeReference {
                    args.push(build_type(child, source));
                }
            }
        }
    }

    GlslType { name, args }
}

/// Builds an expression node.
fn build_expression<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslExpression {
    match node.element_type() {
        GlslElementType::ExprUnary => {
            let mut op = GlslUnaryOp::Neg;
            let mut expr = None;
            let mut postfix = false;

            for child in node.children() {
                match child {
                    RedTree::Leaf(token) => {
                        if GlslTokenType::is_ignored(&token.kind) {
                            continue;
                        }
                        if expr.is_some() {
                            postfix = true;
                        }
                        op = unary_op(token.kind);
                    }
                    RedTree::Node(child) => {
                        if is_expression_kind(child.element_type()) {
                            expr = Some(build_expression(child, source));
                        }
                    }
                }
            }

            if postfix {
                op = match op {
                    GlslUnaryOp::PreIncrement => GlslUnaryOp::PostIncrement,
                    GlslUnaryOp::PreDecrement => GlslUnaryOp::PostDecrement,
                    other => other,
                };
            }

            GlslExpression::Unary { op, expr: Box::new(expr.unwrap_or_else(empty_expression)) }
        }
        GlslElementType::ExprBinary => {
            let mut expressions = Vec::new();
            let mut operator_kinds = Vec::new();
            collect_expressions_and_leaves(node, source, &mut expressions, &mut operator_kinds);

            let lhs = expressions.first().cloned().unwrap_or_else(empty_expression);
            let rhs = expressions.get(1).cloned().unwrap_or_else(empty_expression);

            GlslExpression::Binary { op: binary_op(&operator_kinds), lhs: Box::new(lhs), rhs: Box::new(rhs) }
        }
        GlslElementType::ExprParen => GlslExpression::Paren(Box::new(first_expression(node, source).unwrap_or_else(empty_expression))),
        GlslElementType::ExprMember => {
            let mut base = empty_expression();
            let mut field = String::new();

            for child in node.children() {
                match child {
                    RedTree::Leaf(token) => {
                        if !GlslTokenType::is_ignored(&token.kind) && token.kind != GlslTokenType::Dot {
                            field = leaf_text(&token, source);
                        }
                    }
                    RedTree::Node(child) => {
                        if is_expression_kind(child.element_type()) {
                            base = build_expression(child, source);
                        }
                    }
                }
            }

            GlslExpression::Member { base: Box::new(base), field }
        }
        GlslElementType::ExprIndex => {
            let mut expressions = Vec::new();
            let mut operator_kinds = Vec::new();
            collect_expressions_and_leaves(node, source, &mut expressions, &mut operator_kinds);

            let base = expressions.first().cloned().unwrap_or_else(empty_expression);
            let index = expressions.get(1).cloned().unwrap_or_else(empty_expression);
            GlslExpression::Index { base: Box::new(base), index: Box::new(index) }
        }
        GlslElementType::ExprCall => build_call(node, source),
        _ => build_atomic_expression(node, source),
    }
}

/// Builds a call expression, keeping the callee text including member paths.
fn build_call<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslExpression {
    let mut callee = String::new();
    let mut args = Vec::new();
    let mut after_left_paren = false;

    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if token.kind == GlslTokenType::LeftParen {
                    after_left_paren = true;
                }
                else if !after_left_paren && !GlslTokenType::is_ignored(&token.kind) {
                    callee.push_str(&leaf_text(&token, source));
                }
            }
            RedTree::Node(child) => {
                if after_left_paren {
                    if is_expression_kind(child.element_type()) {
                        args.push(build_expression(child, source));
                    }
                }
                else {
                    callee.push_str(&compact_text(child, source));
                }
            }
        }
    }

    GlslExpression::Call { callee, args }
}

/// Builds a literal or name expression.
fn build_atomic_expression<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> GlslExpression {
    for child in node.children() {
        if let RedTree::Leaf(token) = child {
            if GlslTokenType::is_ignored(&token.kind) {
                continue;
            }

            let text = leaf_text(&token, source);
            return match token.kind {
                GlslTokenType::IntegerLiteral => GlslExpression::IntegerLiteral(parse_integer(&text)),
                GlslTokenType::FloatLiteral => GlslExpression::FloatLiteral(parse_float(&text)),
                GlslTokenType::BooleanLiteral => GlslExpression::BoolLiteral(text == "true"),
                _ => GlslExpression::Ident(text),
            };
        }
    }

    empty_expression()
}

/// Parses an integer literal, ignoring its suffix.
fn parse_integer(text: &str) -> i64 {
    let value = text.trim().trim_end_matches(['u', 'U', 'l', 'L']);
    if let Some(hex) = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) { i64::from_str_radix(hex, 16).unwrap_or(0) } else { value.parse().unwrap_or(0) }
}

/// Parses a floating-point literal, ignoring its suffix.
fn parse_float(text: &str) -> f64 {
    text.trim().trim_end_matches(['f', 'F', 'h', 'H', 'l', 'L']).parse().unwrap_or(0.0)
}

/// Maps a leaf kind to a unary operator.
fn unary_op(kind: GlslTokenType) -> GlslUnaryOp {
    match kind {
        GlslTokenType::Minus => GlslUnaryOp::Neg,
        GlslTokenType::LogicalNot => GlslUnaryOp::Not,
        GlslTokenType::BitwiseNot => GlslUnaryOp::BitNot,
        GlslTokenType::Increment => GlslUnaryOp::PreIncrement,
        GlslTokenType::Decrement => GlslUnaryOp::PreDecrement,
        _ => GlslUnaryOp::Neg,
    }
}

/// Maps the significant leaf kinds of an assignment to an operator.
fn assignment_op(kinds: &[GlslTokenType]) -> GlslAssignOp {
    match kinds.first() {
        Some(GlslTokenType::PlusAssign) => GlslAssignOp::AddAssign,
        Some(GlslTokenType::MinusAssign) => GlslAssignOp::SubAssign,
        Some(GlslTokenType::MultiplyAssign) => GlslAssignOp::MulAssign,
        Some(GlslTokenType::DivideAssign) => GlslAssignOp::DivAssign,
        Some(GlslTokenType::ModuloAssign) => GlslAssignOp::ModAssign,
        Some(GlslTokenType::BitwiseAndAssign) => GlslAssignOp::AndAssign,
        Some(GlslTokenType::BitwiseOrAssign) => GlslAssignOp::OrAssign,
        Some(GlslTokenType::BitwiseXorAssign) => GlslAssignOp::XorAssign,
        Some(GlslTokenType::LeftShiftAssign) => GlslAssignOp::ShlAssign,
        Some(GlslTokenType::RightShiftAssign) => GlslAssignOp::ShrAssign,
        _ => GlslAssignOp::Assign,
    }
}

/// Maps the significant leaf kinds of a binary expression to an operator.
fn binary_op(kinds: &[GlslTokenType]) -> GlslBinaryOp {
    match kinds.first() {
        Some(GlslTokenType::Plus) => GlslBinaryOp::Add,
        Some(GlslTokenType::Minus) => GlslBinaryOp::Sub,
        Some(GlslTokenType::Multiply) => GlslBinaryOp::Mul,
        Some(GlslTokenType::Divide) => GlslBinaryOp::Div,
        Some(GlslTokenType::Modulo) => GlslBinaryOp::Mod,
        Some(GlslTokenType::Equal) => GlslBinaryOp::Eq,
        Some(GlslTokenType::NotEqual) => GlslBinaryOp::Ne,
        Some(GlslTokenType::Less) => GlslBinaryOp::Lt,
        Some(GlslTokenType::LessEqual) => GlslBinaryOp::Le,
        Some(GlslTokenType::Greater) => GlslBinaryOp::Gt,
        Some(GlslTokenType::GreaterEqual) => GlslBinaryOp::Ge,
        Some(GlslTokenType::LogicalAnd) => GlslBinaryOp::And,
        Some(GlslTokenType::LogicalOr) => GlslBinaryOp::Or,
        Some(GlslTokenType::BitwiseAnd) => GlslBinaryOp::BitAnd,
        Some(GlslTokenType::BitwiseOr) => GlslBinaryOp::BitOr,
        Some(GlslTokenType::BitwiseXor) => GlslBinaryOp::BitXor,
        Some(GlslTokenType::LeftShift) => GlslBinaryOp::Shl,
        Some(GlslTokenType::RightShift) => GlslBinaryOp::Shr,
        _ => GlslBinaryOp::Add,
    }
}

/// Returns `true` for expression element kinds.
fn is_expression_kind(kind: GlslElementType) -> bool {
    matches!(kind, GlslElementType::Expr | GlslElementType::ExprUnary | GlslElementType::ExprBinary | GlslElementType::ExprCall | GlslElementType::ExprMember | GlslElementType::ExprIndex | GlslElementType::ExprParen)
}

/// Returns `true` for element kinds that can appear as a statement.
fn is_statement_kind(kind: GlslElementType) -> bool {
    matches!(
        kind,
        GlslElementType::VarStatement
            | GlslElementType::AssignmentStatement
            | GlslElementType::IfStatement
            | GlslElementType::ForStatement
            | GlslElementType::WhileStatement
            | GlslElementType::DoWhileStatement
            | GlslElementType::BreakStatement
            | GlslElementType::ContinueStatement
            | GlslElementType::ReturnStatement
            | GlslElementType::DiscardStatement
            | GlslElementType::ExpressionStatement
            | GlslElementType::Block
    )
}

/// Returns the source text of a leaf.
fn leaf_text<S: Source + ?Sized>(leaf: &RedLeaf<GlslLanguage>, source: &S) -> String {
    source.get_text_in(leaf.span()).to_string()
}

/// Collects the non-trivia leaf text of a node, without whitespace or comments.
fn compact_text<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S) -> String {
    let mut text = String::new();
    collect_compact_text(node, source, &mut text);
    text
}

fn collect_compact_text<S: Source + ?Sized>(node: RedNode<GlslLanguage>, source: &S, text: &mut String) {
    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if !GlslTokenType::is_ignored(&token.kind) {
                    text.push_str(&leaf_text(&token, source));
                }
            }
            RedTree::Node(child) => collect_compact_text(child, source, text),
        }
    }
}

/// Creates an empty type used for error recovery.
fn empty_type() -> GlslType {
    GlslType { name: String::new(), args: Vec::new() }
}

/// Creates an empty expression used for error recovery.
fn empty_expression() -> GlslExpression {
    GlslExpression::Ident(String::new())
}
