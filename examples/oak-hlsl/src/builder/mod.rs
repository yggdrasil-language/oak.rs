use crate::{HlslParser, ast::*, language::HlslLanguage, lexer::token_type::HlslTokenType, parser::element_type::HlslElementType};
use oak_core::{Builder, BuilderCache, GreenNode, OakDiagnostics, OakError, Parser, RedLeaf, RedNode, RedTree, TextEdit, TokenType, builder::BuildOutput, parser::session::ParseSession, source::Source};

/// AST builder for the HLSL language.
#[derive(Clone, Copy)]
pub struct HlslBuilder<'config> {
    /// Language configuration.
    config: &'config HlslLanguage,
}

impl<'config> HlslBuilder<'config> {
    /// Creates a new HLSL builder with the given language configuration.
    pub const fn new(config: &'config HlslLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<HlslLanguage> for HlslBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<HlslLanguage>) -> BuildOutput<HlslLanguage> {
        let parser = HlslParser::new(self.config);
        let mut session = ParseSession::<HlslLanguage>::default();
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

impl<'config> HlslBuilder<'config> {
    fn build_root<'a, S: Source + ?Sized>(&self, green_tree: &'a GreenNode<'a, HlslLanguage>, source: &S) -> Result<HlslRoot, OakError> {
        let red_root = RedNode::new(green_tree, 0);
        let mut items = Vec::new();

        for child in red_root.children() {
            if let RedTree::Node(node) = child {
                match node.element_type() {
                    HlslElementType::FunctionDeclaration => items.push(HlslItem::Function(self.build_function(node, source))),
                    HlslElementType::StructDeclaration => items.push(HlslItem::Struct(self.build_struct(node, source))),
                    HlslElementType::VariableDeclaration => items.push(HlslItem::GlobalVar(self.build_global_var(node, source))),
                    HlslElementType::GlobalConstDeclaration => items.push(HlslItem::Const(self.build_const(node, source))),
                    HlslElementType::TypeAliasDeclaration => items.push(HlslItem::TypeAlias(self.build_type_alias(node, source))),
                    _ => {}
                }
            }
        }

        Ok(HlslRoot { items })
    }

    fn build_function<'a, S: Source + ?Sized>(&self, node: RedNode<'a, HlslLanguage>, source: &S) -> HlslFunction {
        let mut attributes = Vec::new();
        let mut params = Vec::new();
        let mut return_type = None;
        let mut body = Vec::new();

        for child in node.children() {
            if let RedTree::Node(child) = child {
                match child.element_type() {
                    HlslElementType::Attribute => attributes.push(build_attribute(child, source)),
                    HlslElementType::ParameterList => {
                        params = build_parameter_list(child, source);
                    }
                    HlslElementType::TypeReference => {
                        if return_type.is_none() {
                            return_type = Some(build_type(child, source));
                        }
                    }
                    HlslElementType::Block => body = build_block(child, source),
                    _ => {}
                }
            }
        }

        HlslFunction { attributes, name: declared_name(node, source), params, return_type, semantic: colon_semantic(node, source), body }
    }

    fn build_struct<'a, S: Source + ?Sized>(&self, node: RedNode<'a, HlslLanguage>, source: &S) -> HlslStruct {
        let mut members = Vec::new();

        for child in node.children() {
            if let RedTree::Node(child) = child {
                if child.element_type() == HlslElementType::StructMemberDeclaration {
                    members.push(build_struct_member(child, source));
                }
            }
        }

        HlslStruct { name: declared_name(node, source), members }
    }

    fn build_global_var<'a, S: Source + ?Sized>(&self, node: RedNode<'a, HlslLanguage>, source: &S) -> HlslGlobalVar {
        let mut attributes = Vec::new();
        let mut register = None;
        let mut ty = None;
        let mut init = None;

        for child in node.children() {
            if let RedTree::Node(child) = child {
                match child.element_type() {
                    HlslElementType::Attribute => attributes.push(build_attribute(child, source)),
                    HlslElementType::RegisterClause => register = Some(build_register(child, source)),
                    HlslElementType::TypeReference => ty = Some(build_type(child, source)),
                    kind if is_expression_kind(kind) => init = Some(build_expression(child, source)),
                    _ => {}
                }
            }
        }

        HlslGlobalVar { attributes, qualifiers: qualifiers(node), register, semantic: colon_semantic(node, source), name: declared_name(node, source), ty, init }
    }

    fn build_const<'a, S: Source + ?Sized>(&self, node: RedNode<'a, HlslLanguage>, source: &S) -> HlslConst {
        let mut ty = None;
        let mut init = None;

        for child in node.children() {
            if let RedTree::Node(child) = child {
                match child.element_type() {
                    HlslElementType::TypeReference => ty = Some(build_type(child, source)),
                    kind if is_expression_kind(kind) => init = Some(build_expression(child, source)),
                    _ => {}
                }
            }
        }

        HlslConst { name: declared_name(node, source), ty, init: init.unwrap_or_else(empty_expression) }
    }

    fn build_type_alias<'a, S: Source + ?Sized>(&self, node: RedNode<'a, HlslLanguage>, source: &S) -> HlslTypeAlias {
        let mut ty = empty_type();

        for child in node.children() {
            if let RedTree::Node(child) = child {
                if child.element_type() == HlslElementType::TypeReference {
                    ty = build_type(child, source);
                }
            }
        }

        HlslTypeAlias { name: declared_name(node, source), ty }
    }
}

/// Builds the parameters of a parameter list node.
fn build_parameter_list<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> Vec<HlslParam> {
    let mut params = Vec::new();
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if child.element_type() == HlslElementType::Parameter {
                params.push(build_param(child, source));
            }
        }
    }
    params
}

/// Builds a function parameter node.
fn build_param<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> HlslParam {
    let mut ty = empty_type();
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if child.element_type() == HlslElementType::TypeReference {
                ty = build_type(child, source);
            }
        }
    }
    HlslParam { qualifiers: qualifiers(node), name: declared_name(node, source), ty, semantic: colon_semantic(node, source) }
}

/// Builds a struct member node.
fn build_struct_member<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> HlslStructMember {
    let mut attributes = Vec::new();
    let mut ty = empty_type();

    for child in node.children() {
        if let RedTree::Node(child) = child {
            match child.element_type() {
                HlslElementType::Attribute => attributes.push(build_attribute(child, source)),
                HlslElementType::TypeReference => ty = build_type(child, source),
                _ => {}
            }
        }
    }

    HlslStructMember { attributes, qualifiers: qualifiers(node), name: declared_name(node, source), ty, semantic: colon_semantic(node, source) }
}

/// Builds a resource register node such as `register(b0)`.
fn build_register<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> HlslRegister {
    for child in node.children() {
        if let RedTree::Leaf(token) = child {
            if HlslTokenType::is_ignored(&token.kind) {
                continue;
            }
            if token.kind == HlslTokenType::Identifier {
                let text = leaf_text(&token, source);
                return split_register(&text);
            }
        }
    }
    HlslRegister { class: None, index: None }
}

/// Splits a register name such as `b0` or `t12` into its class and slot.
fn split_register(text: &str) -> HlslRegister {
    let class_len = text.find(|ch: char| ch.is_ascii_digit()).unwrap_or(text.len());
    let (class, index) = text.split_at(class_len);
    HlslRegister { class: if class.is_empty() { None } else { Some(class.to_ascii_lowercase()) }, index: index.parse().ok() }
}

/// Builds the statements of a block node.
fn build_block<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> Vec<HlslStatement> {
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
fn build_statement<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> HlslStatement {
    match node.element_type() {
        HlslElementType::VarStatement => {
            let mut ty = None;
            let mut init = None;

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        HlslElementType::TypeReference => ty = Some(build_type(child, source)),
                        kind if is_expression_kind(kind) => init = Some(build_expression(child, source)),
                        _ => {}
                    }
                }
            }

            let qualifiers = qualifiers(node);
            let is_const = qualifiers.contains(&HlslQualifier::Const);
            HlslStatement::Var { qualifiers, name: declared_name(node, source), ty, init, is_const }
        }
        HlslElementType::AssignmentStatement => {
            let mut expressions = Vec::new();
            let mut operator_kinds = Vec::new();

            collect_expressions_and_leaves(node, source, &mut expressions, &mut operator_kinds);

            let target = expressions.first().cloned().unwrap_or_else(empty_expression);
            let value = expressions.get(1).cloned().unwrap_or_else(empty_expression);

            HlslStatement::Assign { target, op: assignment_op(&operator_kinds), value }
        }
        HlslElementType::IfStatement => {
            let mut cond = empty_expression();
            let mut then_body = Vec::new();
            let mut else_body = Vec::new();
            let mut seen_then = false;

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        HlslElementType::Block => {
                            if seen_then {
                                else_body = build_block(child, source);
                            }
                            else {
                                then_body = build_block(child, source);
                                seen_then = true;
                            }
                        }
                        HlslElementType::IfStatement => else_body = vec![build_statement(child, source)],
                        kind if is_expression_kind(kind) => {
                            if cond_is_empty(&cond) {
                                cond = build_expression(child, source);
                            }
                        }
                        _ => {}
                    }
                }
            }

            HlslStatement::If { cond, then_body, else_body }
        }
        HlslElementType::ForStatement => {
            let mut init = None;
            let mut cond = None;
            let mut update = None;
            let mut body = Vec::new();

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        HlslElementType::ForInit => init = first_statement(child, source).map(Box::new),
                        HlslElementType::ForCond => cond = first_expression(child, source),
                        HlslElementType::ForUpdate => update = first_statement(child, source).map(Box::new),
                        HlslElementType::Block => body = build_block(child, source),
                        _ => {}
                    }
                }
            }

            HlslStatement::For { init, cond, update, body }
        }
        HlslElementType::WhileStatement => {
            let mut cond = empty_expression();
            let mut body = Vec::new();

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        HlslElementType::Block => body = build_block(child, source),
                        kind if is_expression_kind(kind) => cond = build_expression(child, source),
                        _ => {}
                    }
                }
            }

            HlslStatement::While { cond, body }
        }
        HlslElementType::DoWhileStatement => {
            let mut cond = empty_expression();
            let mut body = Vec::new();

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        HlslElementType::Block => body = build_block(child, source),
                        kind if is_expression_kind(kind) => cond = build_expression(child, source),
                        _ => {}
                    }
                }
            }

            HlslStatement::DoWhile { body, cond }
        }
        HlslElementType::BreakStatement => HlslStatement::Break,
        HlslElementType::ContinueStatement => HlslStatement::Continue,
        HlslElementType::DiscardStatement => HlslStatement::Discard,
        HlslElementType::ReturnStatement => HlslStatement::Return(first_expression(node, source)),
        HlslElementType::ExpressionStatement => HlslStatement::Expr(first_expression(node, source).unwrap_or_else(empty_expression)),
        HlslElementType::Block => HlslStatement::Block(build_block(node, source)),
        _ => HlslStatement::Expr(empty_expression()),
    }
}

/// Returns `true` when an expression is the placeholder produced for missing input.
fn cond_is_empty(expression: &HlslExpression) -> bool {
    matches!(expression, HlslExpression::Ident(name) if name.is_empty())
}

/// Returns the first statement nested in a header node.
fn first_statement<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> Option<HlslStatement> {
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
fn first_expression<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> Option<HlslExpression> {
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
fn collect_expressions_and_leaves<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S, expressions: &mut Vec<HlslExpression>, leaves: &mut Vec<HlslTokenType>) {
    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if !HlslTokenType::is_ignored(&token.kind) {
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
fn declared_name<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> String {
    for child in node.children() {
        if let RedTree::Leaf(token) = child {
            if token.kind == HlslTokenType::Identifier {
                return leaf_text(&token, source);
            }
        }
    }
    String::new()
}

/// Returns the identifier that follows the last `:` in a node, which is the
/// semantic of a declaration such as `float4 pos : POSITION`.
fn colon_semantic<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> Option<String> {
    let mut after_colon = false;
    for child in node.children() {
        if let RedTree::Leaf(token) = child {
            match token.kind {
                kind if HlslTokenType::is_ignored(&kind) => {}
                HlslTokenType::Colon => after_colon = true,
                HlslTokenType::Identifier if after_colon => return Some(leaf_text(&token, source)),
                _ => after_colon = false,
            }
        }
    }
    None
}

/// Returns the qualifiers attached to a declaration node.
fn qualifiers(node: RedNode<HlslLanguage>) -> Vec<HlslQualifier> {
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
fn qualifier_of(kind: HlslTokenType) -> Option<HlslQualifier> {
    Some(match kind {
        HlslTokenType::Uniform => HlslQualifier::Uniform,
        HlslTokenType::Const => HlslQualifier::Const,
        HlslTokenType::Static => HlslQualifier::Static,
        HlslTokenType::Extern => HlslQualifier::Extern,
        HlslTokenType::Inline => HlslQualifier::Inline,
        HlslTokenType::In => HlslQualifier::In,
        HlslTokenType::Out => HlslQualifier::Out,
        HlslTokenType::Inout => HlslQualifier::InOut,
        HlslTokenType::Shared => HlslQualifier::Shared,
        HlslTokenType::Groupshared => HlslQualifier::Groupshared,
        HlslTokenType::Varying => HlslQualifier::Varying,
        HlslTokenType::Volatile => HlslQualifier::Volatile,
        HlslTokenType::Precise => HlslQualifier::Precise,
        HlslTokenType::Nointerpolation => HlslQualifier::NoInterpolation,
        HlslTokenType::Linear => HlslQualifier::Linear,
        HlslTokenType::Centroid => HlslQualifier::Centroid,
        HlslTokenType::Sample => HlslQualifier::Sample,
        HlslTokenType::Noperspective => HlslQualifier::NoPerspective,
        HlslTokenType::Target => HlslQualifier::Target,
        _ => return None,
    })
}

/// Builds an attribute node.
fn build_attribute<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> HlslAttribute {
    let mut name = String::new();
    let mut args = Vec::new();

    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if HlslTokenType::is_ignored(&token.kind) {
                    continue;
                }
                if name.is_empty() && token.kind != HlslTokenType::LeftBracket {
                    name = leaf_text(&token, source);
                }
            }
            RedTree::Node(child) => {
                if is_expression_kind(child.element_type()) {
                    args.push(build_expression(child, source));
                }
            }
        }
    }

    HlslAttribute { name, args }
}

/// Builds a type node.
fn build_type<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> HlslType {
    let mut name = String::new();
    let mut args = Vec::new();

    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if name.is_empty() && !HlslTokenType::is_ignored(&token.kind) {
                    name = leaf_text(&token, source);
                }
            }
            RedTree::Node(child) => {
                if child.element_type() == HlslElementType::TypeReference {
                    args.push(build_type(child, source));
                }
            }
        }
    }

    HlslType { name, args }
}

/// Builds an expression node.
fn build_expression<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> HlslExpression {
    match node.element_type() {
        HlslElementType::ExprUnary => {
            let mut op = HlslUnaryOp::Neg;
            let mut expr = None;
            let mut postfix = false;

            for child in node.children() {
                match child {
                    RedTree::Leaf(token) => {
                        if HlslTokenType::is_ignored(&token.kind) {
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
                    HlslUnaryOp::PreIncrement => HlslUnaryOp::PostIncrement,
                    HlslUnaryOp::PreDecrement => HlslUnaryOp::PostDecrement,
                    other => other,
                };
            }

            HlslExpression::Unary { op, expr: Box::new(expr.unwrap_or_else(empty_expression)) }
        }
        HlslElementType::ExprBinary => {
            let mut expressions = Vec::new();
            let mut operator_kinds = Vec::new();
            collect_expressions_and_leaves(node, source, &mut expressions, &mut operator_kinds);

            let lhs = expressions.first().cloned().unwrap_or_else(empty_expression);
            let rhs = expressions.get(1).cloned().unwrap_or_else(empty_expression);

            HlslExpression::Binary { op: binary_op(&operator_kinds), lhs: Box::new(lhs), rhs: Box::new(rhs) }
        }
        HlslElementType::ExprParen => HlslExpression::Paren(Box::new(first_expression(node, source).unwrap_or_else(empty_expression))),
        HlslElementType::ExprMember => {
            let mut base = empty_expression();
            let mut field = String::new();

            for child in node.children() {
                match child {
                    RedTree::Leaf(token) => {
                        if !HlslTokenType::is_ignored(&token.kind) && token.kind != HlslTokenType::Dot {
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

            HlslExpression::Member { base: Box::new(base), field }
        }
        HlslElementType::ExprIndex => {
            let mut expressions = Vec::new();
            let mut operator_kinds = Vec::new();
            collect_expressions_and_leaves(node, source, &mut expressions, &mut operator_kinds);

            let base = expressions.first().cloned().unwrap_or_else(empty_expression);
            let index = expressions.get(1).cloned().unwrap_or_else(empty_expression);
            HlslExpression::Index { base: Box::new(base), index: Box::new(index) }
        }
        HlslElementType::ExprCall => build_call(node, source),
        _ => build_atomic_expression(node, source),
    }
}

/// Builds a call expression, keeping the callee text including member paths.
fn build_call<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> HlslExpression {
    let mut callee = String::new();
    let mut args = Vec::new();
    let mut after_left_paren = false;

    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if token.kind == HlslTokenType::LeftParen {
                    after_left_paren = true;
                }
                else if !after_left_paren && !HlslTokenType::is_ignored(&token.kind) {
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

    HlslExpression::Call { callee, args }
}

/// Builds a literal or name expression.
fn build_atomic_expression<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> HlslExpression {
    for child in node.children() {
        if let RedTree::Leaf(token) = child {
            if HlslTokenType::is_ignored(&token.kind) {
                continue;
            }

            let text = leaf_text(&token, source);
            return match token.kind {
                HlslTokenType::NumberLiteral => number_expression(&text),
                HlslTokenType::BooleanLiteral => HlslExpression::BoolLiteral(text == "true"),
                _ => HlslExpression::Ident(text),
            };
        }
    }

    empty_expression()
}

/// Classifies a numeric literal as an integer or a floating-point value.
fn number_expression(text: &str) -> HlslExpression {
    let value = text.trim();
    let lower = value.to_ascii_lowercase();
    let is_hex = lower.starts_with("0x");
    let is_float = !is_hex && (lower.contains('.') || lower.contains('e') || lower.ends_with('f') || lower.ends_with('h'));

    if is_float { HlslExpression::FloatLiteral(parse_float(&lower)) } else { HlslExpression::IntegerLiteral(parse_integer(&lower)) }
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
fn unary_op(kind: HlslTokenType) -> HlslUnaryOp {
    match kind {
        HlslTokenType::Minus => HlslUnaryOp::Neg,
        HlslTokenType::LogicalNot => HlslUnaryOp::Not,
        HlslTokenType::BitwiseNot => HlslUnaryOp::BitNot,
        HlslTokenType::Increment => HlslUnaryOp::PreIncrement,
        HlslTokenType::Decrement => HlslUnaryOp::PreDecrement,
        _ => HlslUnaryOp::Neg,
    }
}

/// Maps the significant leaf kinds of an assignment to an operator.
fn assignment_op(kinds: &[HlslTokenType]) -> HlslAssignOp {
    match kinds.first() {
        Some(HlslTokenType::PlusAssign) => HlslAssignOp::AddAssign,
        Some(HlslTokenType::MinusAssign) => HlslAssignOp::SubAssign,
        Some(HlslTokenType::MultiplyAssign) => HlslAssignOp::MulAssign,
        Some(HlslTokenType::DivideAssign) => HlslAssignOp::DivAssign,
        Some(HlslTokenType::ModuloAssign) => HlslAssignOp::ModAssign,
        Some(HlslTokenType::BitwiseAndAssign) => HlslAssignOp::AndAssign,
        Some(HlslTokenType::BitwiseOrAssign) => HlslAssignOp::OrAssign,
        Some(HlslTokenType::BitwiseXorAssign) => HlslAssignOp::XorAssign,
        Some(HlslTokenType::LeftShiftAssign) => HlslAssignOp::ShlAssign,
        Some(HlslTokenType::RightShiftAssign) => HlslAssignOp::ShrAssign,
        _ => HlslAssignOp::Assign,
    }
}

/// Maps the significant leaf kinds of a binary expression to an operator.
fn binary_op(kinds: &[HlslTokenType]) -> HlslBinaryOp {
    match kinds.first() {
        Some(HlslTokenType::Plus) => HlslBinaryOp::Add,
        Some(HlslTokenType::Minus) => HlslBinaryOp::Sub,
        Some(HlslTokenType::Multiply) => HlslBinaryOp::Mul,
        Some(HlslTokenType::Divide) => HlslBinaryOp::Div,
        Some(HlslTokenType::Modulo) => HlslBinaryOp::Mod,
        Some(HlslTokenType::Equal) => HlslBinaryOp::Eq,
        Some(HlslTokenType::NotEqual) => HlslBinaryOp::Ne,
        Some(HlslTokenType::Less) => HlslBinaryOp::Lt,
        Some(HlslTokenType::LessEqual) => HlslBinaryOp::Le,
        Some(HlslTokenType::Greater) => HlslBinaryOp::Gt,
        Some(HlslTokenType::GreaterEqual) => HlslBinaryOp::Ge,
        Some(HlslTokenType::LogicalAnd) => HlslBinaryOp::And,
        Some(HlslTokenType::LogicalOr) => HlslBinaryOp::Or,
        Some(HlslTokenType::BitwiseAnd) => HlslBinaryOp::BitAnd,
        Some(HlslTokenType::BitwiseOr) => HlslBinaryOp::BitOr,
        Some(HlslTokenType::BitwiseXor) => HlslBinaryOp::BitXor,
        Some(HlslTokenType::LeftShift) => HlslBinaryOp::Shl,
        Some(HlslTokenType::RightShift) => HlslBinaryOp::Shr,
        _ => HlslBinaryOp::Add,
    }
}

/// Returns `true` for expression element kinds.
fn is_expression_kind(kind: HlslElementType) -> bool {
    matches!(kind, HlslElementType::Expr | HlslElementType::ExprUnary | HlslElementType::ExprBinary | HlslElementType::ExprCall | HlslElementType::ExprMember | HlslElementType::ExprIndex | HlslElementType::ExprParen)
}

/// Returns `true` for element kinds that can appear as a statement.
fn is_statement_kind(kind: HlslElementType) -> bool {
    matches!(
        kind,
        HlslElementType::VarStatement
            | HlslElementType::AssignmentStatement
            | HlslElementType::IfStatement
            | HlslElementType::ForStatement
            | HlslElementType::WhileStatement
            | HlslElementType::DoWhileStatement
            | HlslElementType::BreakStatement
            | HlslElementType::ContinueStatement
            | HlslElementType::ReturnStatement
            | HlslElementType::DiscardStatement
            | HlslElementType::ExpressionStatement
            | HlslElementType::Block
    )
}

/// Returns the source text of a leaf.
fn leaf_text<S: Source + ?Sized>(leaf: &RedLeaf<HlslLanguage>, source: &S) -> String {
    source.get_text_in(leaf.span()).to_string()
}

/// Collects the non-trivia leaf text of a node, without whitespace or comments.
fn compact_text<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S) -> String {
    let mut text = String::new();
    collect_compact_text(node, source, &mut text);
    text
}

fn collect_compact_text<S: Source + ?Sized>(node: RedNode<HlslLanguage>, source: &S, text: &mut String) {
    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if !HlslTokenType::is_ignored(&token.kind) {
                    text.push_str(&leaf_text(&token, source));
                }
            }
            RedTree::Node(child) => collect_compact_text(child, source, text),
        }
    }
}

/// Creates an empty type used for error recovery.
fn empty_type() -> HlslType {
    HlslType { name: String::new(), args: Vec::new() }
}

/// Creates an empty expression used for error recovery.
fn empty_expression() -> HlslExpression {
    HlslExpression::Ident(String::new())
}
