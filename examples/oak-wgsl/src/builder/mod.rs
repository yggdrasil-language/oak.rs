use crate::{WgslParser, ast::*, language::WgslLanguage, lexer::token_type::WgslTokenType, parser::element_type::WgslElementType};
use oak_core::{Builder, BuilderCache, GreenNode, OakDiagnostics, OakError, Parser, RedLeaf, RedNode, RedTree, TextEdit, TokenType, builder::BuildOutput, parser::session::ParseSession, source::Source};

/// AST builder for the WebGPU Shading Language (WGSL).
#[derive(Clone, Copy)]
pub struct WgslBuilder<'config> {
    /// Language configuration
    config: &'config WgslLanguage,
}

impl<'config> WgslBuilder<'config> {
    /// Creates a new `WgslBuilder` with the given configuration.
    pub const fn new(config: &'config WgslLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<WgslLanguage> for WgslBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<WgslLanguage>) -> BuildOutput<WgslLanguage> {
        let parser = WgslParser::new(self.config);
        let mut session = ParseSession::<WgslLanguage>::default();
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

impl<'config> WgslBuilder<'config> {
    fn build_root<'a, S: Source + ?Sized>(&self, green_tree: &'a GreenNode<'a, WgslLanguage>, source: &S) -> Result<WgslRoot, OakError> {
        let red_root = RedNode::new(green_tree, 0);
        let mut items = Vec::new();

        for child in red_root.children() {
            if let RedTree::Node(node) = child {
                match node.element_type() {
                    WgslElementType::Function => items.push(WgslItem::Function(self.build_function(node, source))),
                    WgslElementType::Struct => items.push(WgslItem::Struct(self.build_struct(node, source))),
                    WgslElementType::Variable => items.push(WgslItem::GlobalVar(self.build_global_var(node, source))),
                    WgslElementType::GlobalConst => items.push(WgslItem::Const(self.build_const(node, source))),
                    WgslElementType::TypeAlias => items.push(WgslItem::TypeAlias(self.build_type_alias(node, source))),
                    _ => {}
                }
            }
        }

        Ok(WgslRoot { items })
    }

    fn build_function<'a, S: Source + ?Sized>(&self, node: RedNode<'a, WgslLanguage>, source: &S) -> WgslFunction {
        let mut attributes = Vec::new();
        let mut name = String::new();
        let mut params = Vec::new();
        let mut return_type = None;
        let mut body = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(token) => {
                    if token.kind == WgslTokenType::Identifier && name.is_empty() {
                        name = leaf_text(&token, source);
                    }
                }
                RedTree::Node(child) => match child.element_type() {
                    WgslElementType::Attribute => attributes.push(build_attribute(child, source)),
                    WgslElementType::Param => params.push(self.build_param(child, source)),
                    WgslElementType::Type => {
                        if return_type.is_none() {
                            return_type = Some(build_type(child, source));
                        }
                    }
                    WgslElementType::Block => body = build_block(child, source),
                    _ => {}
                },
            }
        }

        WgslFunction { attributes, name, params, return_type, body }
    }

    fn build_param<'a, S: Source + ?Sized>(&self, node: RedNode<'a, WgslLanguage>, source: &S) -> WgslParam {
        let mut attributes = Vec::new();
        let mut name = String::new();
        let mut ty = empty_type();

        for child in node.children() {
            match child {
                RedTree::Leaf(token) => {
                    if name.is_empty() && !WgslTokenType::is_ignored(&token.kind) {
                        name = leaf_text(&token, source);
                    }
                }
                RedTree::Node(child) => match child.element_type() {
                    WgslElementType::Attribute => attributes.push(build_attribute(child, source)),
                    WgslElementType::Type => ty = build_type(child, source),
                    _ => {}
                },
            }
        }

        WgslParam { attributes, name, ty }
    }

    fn build_struct<'a, S: Source + ?Sized>(&self, node: RedNode<'a, WgslLanguage>, source: &S) -> WgslStruct {
        let mut name = String::new();
        let mut members = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(token) => {
                    if token.kind == WgslTokenType::Identifier && name.is_empty() {
                        name = leaf_text(&token, source);
                    }
                }
                RedTree::Node(child) => {
                    if child.element_type() == WgslElementType::StructMember {
                        members.push(self.build_struct_member(child, source));
                    }
                }
            }
        }

        WgslStruct { name, members }
    }

    fn build_struct_member<'a, S: Source + ?Sized>(&self, node: RedNode<'a, WgslLanguage>, source: &S) -> WgslStructMember {
        let mut attributes = Vec::new();
        let mut name = String::new();
        let mut ty = empty_type();

        for child in node.children() {
            match child {
                RedTree::Leaf(token) => {
                    if name.is_empty() && !WgslTokenType::is_ignored(&token.kind) {
                        name = leaf_text(&token, source);
                    }
                }
                RedTree::Node(child) => match child.element_type() {
                    WgslElementType::Attribute => attributes.push(build_attribute(child, source)),
                    WgslElementType::Type => ty = build_type(child, source),
                    _ => {}
                },
            }
        }

        WgslStructMember { attributes, name, ty }
    }

    fn build_global_var<'a, S: Source + ?Sized>(&self, node: RedNode<'a, WgslLanguage>, source: &S) -> WgslGlobalVar {
        let mut attributes = Vec::new();
        let mut address_space = None;
        let mut access = None;
        let mut name = String::new();
        let mut ty = None;
        let mut init = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(token) => match token.kind {
                    WgslTokenType::FunctionKw => address_space = Some(WgslAddressSpace::Function),
                    WgslTokenType::PrivateKw => address_space = Some(WgslAddressSpace::Private),
                    WgslTokenType::WorkgroupKw => address_space = Some(WgslAddressSpace::Workgroup),
                    WgslTokenType::UniformKw => address_space = Some(WgslAddressSpace::Uniform),
                    WgslTokenType::StorageKw => address_space = Some(WgslAddressSpace::Storage),
                    WgslTokenType::ReadKw => access = Some(WgslAccessMode::Read),
                    WgslTokenType::WriteKw => access = Some(WgslAccessMode::Write),
                    WgslTokenType::ReadWriteKw => access = Some(WgslAccessMode::ReadWrite),
                    WgslTokenType::Identifier if name.is_empty() => name = leaf_text(&token, source),
                    _ => {}
                },
                RedTree::Node(child) => match child.element_type() {
                    WgslElementType::Attribute => attributes.push(build_attribute(child, source)),
                    WgslElementType::Type => ty = Some(build_type(child, source)),
                    kind if is_expression_kind(kind) => init = Some(build_expression(child, source)),
                    _ => {}
                },
            }
        }

        WgslGlobalVar { attributes, address_space, access, name, ty, init }
    }

    fn build_const<'a, S: Source + ?Sized>(&self, node: RedNode<'a, WgslLanguage>, source: &S) -> WgslConst {
        let mut name = String::new();
        let mut ty = None;
        let mut init = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(token) => {
                    if token.kind == WgslTokenType::Identifier && name.is_empty() {
                        name = leaf_text(&token, source);
                    }
                }
                RedTree::Node(child) => {
                    if child.element_type() == WgslElementType::Type {
                        ty = Some(build_type(child, source));
                    }
                    else if is_expression_kind(child.element_type()) {
                        init = Some(build_expression(child, source));
                    }
                }
            }
        }

        WgslConst { name, ty, init: init.unwrap_or_else(empty_expression) }
    }

    fn build_type_alias<'a, S: Source + ?Sized>(&self, node: RedNode<'a, WgslLanguage>, source: &S) -> WgslTypeAlias {
        let mut name = String::new();
        let mut ty = empty_type();

        for child in node.children() {
            match child {
                RedTree::Leaf(token) => {
                    if token.kind == WgslTokenType::Identifier && name.is_empty() {
                        name = leaf_text(&token, source);
                    }
                }
                RedTree::Node(child) => {
                    if child.element_type() == WgslElementType::Type {
                        ty = build_type(child, source);
                    }
                }
            }
        }

        WgslTypeAlias { name, ty }
    }
}

/// Builds the statements of a block node.
fn build_block<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> Vec<WgslStatement> {
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
fn build_statement<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> WgslStatement {
    match node.element_type() {
        WgslElementType::VarStatement => {
            let mut name = String::new();
            let mut ty = None;
            let mut init = None;
            let mut is_const = false;

            for child in node.children() {
                match child {
                    RedTree::Leaf(token) => match token.kind {
                        WgslTokenType::LetKw | WgslTokenType::ConstKw => is_const = true,
                        WgslTokenType::Identifier if name.is_empty() => name = leaf_text(&token, source),
                        _ => {}
                    },
                    RedTree::Node(child) => match child.element_type() {
                        WgslElementType::Type => ty = Some(build_type(child, source)),
                        kind if is_expression_kind(kind) => init = Some(build_expression(child, source)),
                        _ => {}
                    },
                }
            }

            WgslStatement::Var { name, ty, init, is_const }
        }
        WgslElementType::AssignmentStatement => {
            let mut expressions = Vec::new();
            let mut operator_kinds = Vec::new();

            collect_expressions_and_leaves(node, source, &mut expressions, &mut operator_kinds);

            let target = expressions.first().cloned().unwrap_or_else(empty_expression);
            let value = expressions.get(1).cloned().unwrap_or_else(empty_expression);

            WgslStatement::Assign { target, op: assignment_op(&operator_kinds), value }
        }
        WgslElementType::IfStatement => {
            let mut cond = empty_expression();
            let mut then_body = Vec::new();
            let mut else_body = Vec::new();
            let mut seen_then = false;

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        WgslElementType::Block => {
                            if seen_then {
                                else_body = build_block(child, source);
                            }
                            else {
                                then_body = build_block(child, source);
                                seen_then = true;
                            }
                        }
                        WgslElementType::IfStatement => else_body = vec![build_statement(child, source)],
                        kind if is_expression_kind(kind) => cond = build_expression(child, source),
                        _ => {}
                    }
                }
            }

            WgslStatement::If { cond, then_body, else_body }
        }
        WgslElementType::ForStatement => {
            let mut init = None;
            let mut cond = None;
            let mut update = None;
            let mut body = Vec::new();

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        WgslElementType::ForInit => {
                            init = first_statement(child, source).map(Box::new);
                        }
                        WgslElementType::ForCond => {
                            cond = first_expression(child, source);
                        }
                        WgslElementType::ForUpdate => {
                            update = first_statement(child, source).map(Box::new);
                        }
                        WgslElementType::Block => body = build_block(child, source),
                        _ => {}
                    }
                }
            }

            WgslStatement::For { init, cond, update, body }
        }
        WgslElementType::WhileStatement => {
            let mut cond = empty_expression();
            let mut body = Vec::new();

            for child in node.children() {
                if let RedTree::Node(child) = child {
                    match child.element_type() {
                        WgslElementType::Block => body = build_block(child, source),
                        kind if is_expression_kind(kind) => cond = build_expression(child, source),
                        _ => {}
                    }
                }
            }

            WgslStatement::While { cond, body }
        }
        WgslElementType::LoopStatement => WgslStatement::Loop { body: body_of(node, source) },
        WgslElementType::BreakStatement => WgslStatement::Break,
        WgslElementType::ContinueStatement => WgslStatement::Continue,
        WgslElementType::DiscardStatement => WgslStatement::Discard,
        WgslElementType::ReturnStatement => WgslStatement::Return(first_expression(node, source)),
        WgslElementType::ExpressionStatement => WgslStatement::Expr(first_expression(node, source).unwrap_or_else(empty_expression)),
        WgslElementType::Block => WgslStatement::Block(build_block(node, source)),
        _ => WgslStatement::Expr(empty_expression()),
    }
}

/// Builds the body block of a loop-like statement.
fn body_of<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> Vec<WgslStatement> {
    for child in node.children() {
        if let RedTree::Node(child) = child {
            if child.element_type() == WgslElementType::Block {
                return build_block(child, source);
            }
        }
    }
    Vec::new()
}

/// Returns the first statement nested in a header node.
fn first_statement<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> Option<WgslStatement> {
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
fn first_expression<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> Option<WgslExpression> {
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
fn collect_expressions_and_leaves<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S, expressions: &mut Vec<WgslExpression>, leaves: &mut Vec<WgslTokenType>) {
    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if !WgslTokenType::is_ignored(&token.kind) {
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

/// Builds an attribute node.
fn build_attribute<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> WgslAttribute {
    let mut name = String::new();
    let mut args = Vec::new();

    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if name.is_empty() && token.kind == WgslTokenType::At {
                    continue;
                }
                if name.is_empty() && !WgslTokenType::is_ignored(&token.kind) {
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

    WgslAttribute { name, args }
}

/// Builds a type node.
fn build_type<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> WgslType {
    let mut name = String::new();
    let mut args = Vec::new();

    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if name.is_empty() && !WgslTokenType::is_ignored(&token.kind) {
                    name = leaf_text(&token, source);
                }
            }
            RedTree::Node(child) => {
                if child.element_type() == WgslElementType::Type {
                    args.push(build_type(child, source));
                }
            }
        }
    }

    WgslType { name, args }
}

/// Builds an expression node.
fn build_expression<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> WgslExpression {
    match node.element_type() {
        WgslElementType::ExprUnary => {
            let mut op = WgslUnaryOp::Neg;
            let mut expr = None;

            for child in node.children() {
                match child {
                    RedTree::Leaf(token) => match token.kind {
                        WgslTokenType::Minus => op = WgslUnaryOp::Neg,
                        WgslTokenType::Bang => op = WgslUnaryOp::Not,
                        WgslTokenType::Tilde => op = WgslUnaryOp::BitNot,
                        _ => {}
                    },
                    RedTree::Node(child) => {
                        if is_expression_kind(child.element_type()) {
                            expr = Some(build_expression(child, source));
                        }
                    }
                }
            }

            WgslExpression::Unary { op, expr: Box::new(expr.unwrap_or_else(empty_expression)) }
        }
        WgslElementType::ExprBinary => {
            let mut expressions = Vec::new();
            let mut operator_kinds = Vec::new();
            collect_expressions_and_leaves(node, source, &mut expressions, &mut operator_kinds);

            let lhs = expressions.first().cloned().unwrap_or_else(empty_expression);
            let rhs = expressions.get(1).cloned().unwrap_or_else(empty_expression);

            WgslExpression::Binary { op: binary_op(&operator_kinds), lhs: Box::new(lhs), rhs: Box::new(rhs) }
        }
        WgslElementType::ExprParen => WgslExpression::Paren(Box::new(first_expression(node, source).unwrap_or_else(empty_expression))),
        WgslElementType::ExprMember => {
            let mut base = empty_expression();
            let mut field = String::new();

            for child in node.children() {
                match child {
                    RedTree::Leaf(token) => {
                        if token.kind == WgslTokenType::Identifier {
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

            WgslExpression::Member { base: Box::new(base), field }
        }
        WgslElementType::ExprIndex => {
            let mut expressions = Vec::new();
            let mut operator_kinds = Vec::new();
            collect_expressions_and_leaves(node, source, &mut expressions, &mut operator_kinds);

            let base = expressions.first().cloned().unwrap_or_else(empty_expression);
            let index = expressions.get(1).cloned().unwrap_or_else(empty_expression);
            WgslExpression::Index { base: Box::new(base), index: Box::new(index) }
        }
        WgslElementType::ExprCall => build_call(node, source),
        WgslElementType::ExprTypeCtor => WgslExpression::Ident(compact_text(node, source)),
        _ => build_atomic_expression(node, source),
    }
}

/// Builds a call expression, keeping the callee text including type arguments.
fn build_call<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> WgslExpression {
    let mut callee = String::new();
    let mut args = Vec::new();
    let mut after_left_paren = false;

    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if token.kind == WgslTokenType::LeftParen {
                    after_left_paren = true;
                }
                else if !after_left_paren && !WgslTokenType::is_ignored(&token.kind) {
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

    WgslExpression::Call { callee, args }
}

/// Builds a literal or name expression.
fn build_atomic_expression<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> WgslExpression {
    for child in node.children() {
        if let RedTree::Leaf(token) = child {
            if WgslTokenType::is_ignored(&token.kind) {
                continue;
            }

            let text = leaf_text(&token, source);
            return match token.kind {
                WgslTokenType::IntegerLiteral => WgslExpression::IntegerLiteral(parse_integer(&text)),
                WgslTokenType::FloatLiteral => WgslExpression::FloatLiteral(parse_float(&text)),
                WgslTokenType::BoolLiteral => WgslExpression::BoolLiteral(text == "true"),
                _ => WgslExpression::Ident(text),
            };
        }
    }

    empty_expression()
}

/// Parses an integer literal, ignoring the `u`/`i` suffix.
fn parse_integer(text: &str) -> i64 {
    let mut value = text.trim();
    if value.ends_with('u') || value.ends_with('i') {
        value = &value[..value.len() - 1];
    }

    if let Some(hex) = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) { i64::from_str_radix(hex, 16).unwrap_or(0) } else { value.parse().unwrap_or(0) }
}

/// Parses a floating-point literal, ignoring the `f`/`h` suffix.
fn parse_float(text: &str) -> f64 {
    let mut value = text.trim();
    if value.ends_with('f') || value.ends_with('h') {
        value = &value[..value.len() - 1];
    }
    value.parse().unwrap_or(0.0)
}

/// Maps the significant leaf kinds of an assignment to an operator.
fn assignment_op(kinds: &[WgslTokenType]) -> WgslAssignOp {
    let gt_count = kinds.iter().filter(|kind| **kind == WgslTokenType::Gt).count();
    if gt_count >= 2 && kinds.contains(&WgslTokenType::Assign) {
        return WgslAssignOp::ShrAssign;
    }

    match kinds.first() {
        Some(WgslTokenType::PlusAssign) => WgslAssignOp::AddAssign,
        Some(WgslTokenType::MinusAssign) => WgslAssignOp::SubAssign,
        Some(WgslTokenType::StarAssign) => WgslAssignOp::MulAssign,
        Some(WgslTokenType::SlashAssign) => WgslAssignOp::DivAssign,
        Some(WgslTokenType::PercentAssign) => WgslAssignOp::ModAssign,
        Some(WgslTokenType::AmpersandAssign) => WgslAssignOp::AndAssign,
        Some(WgslTokenType::PipeAssign) => WgslAssignOp::OrAssign,
        Some(WgslTokenType::CaretAssign) => WgslAssignOp::XorAssign,
        Some(WgslTokenType::LeftShiftAssign) => WgslAssignOp::ShlAssign,
        Some(WgslTokenType::RightShiftAssign) => WgslAssignOp::ShrAssign,
        _ => WgslAssignOp::Assign,
    }
}

/// Maps the significant leaf kinds of a binary expression to an operator.
fn binary_op(kinds: &[WgslTokenType]) -> WgslBinaryOp {
    let gt_count = kinds.iter().filter(|kind| **kind == WgslTokenType::Gt).count();
    if gt_count >= 2 {
        return WgslBinaryOp::Shr;
    }

    match kinds.first() {
        Some(WgslTokenType::Plus) => WgslBinaryOp::Add,
        Some(WgslTokenType::Minus) => WgslBinaryOp::Sub,
        Some(WgslTokenType::Star) => WgslBinaryOp::Mul,
        Some(WgslTokenType::Slash) => WgslBinaryOp::Div,
        Some(WgslTokenType::Percent) => WgslBinaryOp::Mod,
        Some(WgslTokenType::EqEq) => WgslBinaryOp::Eq,
        Some(WgslTokenType::BangEq) | Some(WgslTokenType::Ne) => WgslBinaryOp::Ne,
        Some(WgslTokenType::Lt) => WgslBinaryOp::Lt,
        Some(WgslTokenType::Le) => WgslBinaryOp::Le,
        Some(WgslTokenType::Gt) => WgslBinaryOp::Gt,
        Some(WgslTokenType::Ge) => WgslBinaryOp::Ge,
        Some(WgslTokenType::AmpersandAmpersand) => WgslBinaryOp::And,
        Some(WgslTokenType::PipePipe) => WgslBinaryOp::Or,
        Some(WgslTokenType::Ampersand) => WgslBinaryOp::BitAnd,
        Some(WgslTokenType::Pipe) => WgslBinaryOp::BitOr,
        Some(WgslTokenType::Caret) => WgslBinaryOp::BitXor,
        Some(WgslTokenType::LeftShift) | Some(WgslTokenType::LtLt) => WgslBinaryOp::Shl,
        Some(WgslTokenType::RightShift) | Some(WgslTokenType::GtGt) => WgslBinaryOp::Shr,
        _ => WgslBinaryOp::Add,
    }
}

/// Returns `true` for expression element kinds.
fn is_expression_kind(kind: WgslElementType) -> bool {
    matches!(kind, WgslElementType::Expr | WgslElementType::ExprUnary | WgslElementType::ExprBinary | WgslElementType::ExprTypeCtor | WgslElementType::ExprCall | WgslElementType::ExprMember | WgslElementType::ExprIndex | WgslElementType::ExprParen)
}

/// Returns `true` for element kinds that can appear as a statement.
fn is_statement_kind(kind: WgslElementType) -> bool {
    matches!(
        kind,
        WgslElementType::VarStatement
            | WgslElementType::AssignmentStatement
            | WgslElementType::IfStatement
            | WgslElementType::ForStatement
            | WgslElementType::WhileStatement
            | WgslElementType::LoopStatement
            | WgslElementType::BreakStatement
            | WgslElementType::ContinueStatement
            | WgslElementType::ReturnStatement
            | WgslElementType::DiscardStatement
            | WgslElementType::ExpressionStatement
            | WgslElementType::Block
    )
}

/// Returns the source text of a leaf.
fn leaf_text<S: Source + ?Sized>(leaf: &RedLeaf<WgslLanguage>, source: &S) -> String {
    source.get_text_in(leaf.span()).to_string()
}

/// Collects the non-trivia leaf text of a node, without whitespace or comments.
fn compact_text<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S) -> String {
    let mut text = String::new();
    collect_compact_text(node, source, &mut text);
    text
}

fn collect_compact_text<S: Source + ?Sized>(node: RedNode<WgslLanguage>, source: &S, text: &mut String) {
    for child in node.children() {
        match child {
            RedTree::Leaf(token) => {
                if !WgslTokenType::is_ignored(&token.kind) {
                    text.push_str(&leaf_text(&token, source));
                }
            }
            RedTree::Node(child) => collect_compact_text(child, source, text),
        }
    }
}

/// Creates an empty type used for error recovery.
fn empty_type() -> WgslType {
    WgslType { name: String::new(), args: Vec::new() }
}

/// Creates an empty expression used for error recovery.
fn empty_expression() -> WgslExpression {
    WgslExpression::Ident(String::new())
}
