use crate::{
    ast::{ExpressionNode, LiteralNode, RubyRoot, StatementNode},
    language::RubyLanguage,
    lexer::token_type::RubyTokenType,
    parser::{element_type::RubyElementType, RubyParser},
};
use core::range::Range;
use oak_core::{
    Builder, BuilderCache, Lexer, Parser, Source, TextEdit,
    builder::BuildOutput,
    parser::session::ParseSession,
    tree::{GreenNode, GreenTree},
};

/// Ruby AST builder。把语法树降成 [`RubyRoot`]，不再丢弃语句。
pub struct RubyBuilder<'config> {
    config: &'config RubyLanguage,
}

impl<'config> RubyBuilder<'config> {
    /// Creates a new `RubyBuilder` with the given configuration.
    pub fn new(config: &'config RubyLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<RubyLanguage> for RubyBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(
        &self,
        source: &'a S,
        edits: &[TextEdit],
        _cache: &'a mut impl BuilderCache<RubyLanguage>,
    ) -> BuildOutput<RubyLanguage> {
        let parser = RubyParser::new(self.config);
        let lexer = crate::lexer::RubyLexer::new(self.config);

        let mut session = ParseSession::<RubyLanguage>::default();
        lexer.lex(source, edits, &mut session);
        let parse_result = parser.parse(source, edits, &mut session);

        match parse_result.result {
            Ok(green_tree) => match self.build_root(green_tree, source) {
                Ok(ast_root) => oak_core::errors::OakDiagnostics {
                    result: Ok(ast_root),
                    diagnostics: parse_result.diagnostics,
                },
                Err(build_error) => {
                    let mut diagnostics = parse_result.diagnostics;
                    diagnostics.push(build_error.clone());
                    oak_core::errors::OakDiagnostics {
                        result: Err(build_error),
                        diagnostics,
                    }
                }
            },
            Err(parse_error) => oak_core::errors::OakDiagnostics {
                result: Err(parse_error),
                diagnostics: parse_result.diagnostics,
            },
        }
    }
}

impl<'config> RubyBuilder<'config> {
    fn build_root<S: Source + ?Sized>(&self, green_tree: &GreenNode<RubyLanguage>, source: &S) -> Result<RubyRoot, oak_core::OakError> {
        let statements = self.build_statement_list(green_tree, source, 0)?;
        Ok(RubyRoot {
            statements,
            span: Range {
                start: 0,
                end: source.length(),
            },
        })
    }

    fn build_statement_list<S: Source + ?Sized>(&self, node: &GreenNode<RubyLanguage>, source: &S, base: usize) -> Result<Vec<StatementNode>, oak_core::OakError> {
        let mut statements = Vec::new();
        let mut offset = base;
        for child in node.children() {
            if let GreenTree::Node(child_node) = child {
                if child_node.kind == RubyElementType::BeginStatement {
                    statements.extend(self.build_begin_body(child_node, source, offset)?);
                } else if let Some(stmt) = self.build_statement(child_node, source, offset)? {
                    statements.push(stmt);
                }
            }
            offset += child.len() as usize;
        }
        Ok(statements)
    }

    /// `begin ... rescue ... end`：本阶段只展开 begin 主体，rescue / ensure 暂跳过。
    fn build_begin_body<S: Source + ?Sized>(
        &self,
        node: &GreenNode<RubyLanguage>,
        source: &S,
        base: usize,
    ) -> Result<Vec<StatementNode>, oak_core::OakError> {
        let mut statements = Vec::new();
        let mut offset = base;
        for child in node.children() {
            if let GreenTree::Node(child_node) = child {
                if matches!(
                    child_node.kind,
                    RubyElementType::RescueClause | RubyElementType::EnsureClause
                ) {
                    break;
                }
                if child_node.kind == RubyElementType::BeginStatement {
                    statements.extend(self.build_begin_body(child_node, source, offset)?);
                } else if let Some(stmt) = self.build_statement(child_node, source, offset)? {
                    statements.push(stmt);
                }
            }
            offset += child.len() as usize;
        }
        Ok(statements)
    }

    fn build_statement<S: Source + ?Sized>(&self, node: &GreenNode<RubyLanguage>, source: &S, offset: usize) -> Result<Option<StatementNode>, oak_core::OakError> {
        let span = span_at(offset, node.byte_length);
        match node.kind {
            RubyElementType::MethodDefinition => {
                let (name, params, body) = self.split_method(node, source, offset)?;
                Ok(Some(StatementNode::MethodDef { name, params, body, span }))
            }
            RubyElementType::ClassDefinition | RubyElementType::ModuleDefinition => {
                let name = class_or_module_name(node, source, offset).unwrap_or_else(|| "_".into());
                let body = self.build_statement_list(node, source, offset)?;
                Ok(Some(StatementNode::ClassDef {
                    name,
                    superclass: None,
                    body,
                    span,
                }))
            }
            RubyElementType::IfStatement | RubyElementType::UnlessStatement => {
                let (condition, then_body, else_body) = self.split_if(node, source, offset)?;
                let condition = if node.kind == RubyElementType::UnlessStatement {
                    ExpressionNode::UnaryOp {
                        operator: "!".into(),
                        operand: Box::new(condition),
                        span: span.clone(),
                    }
                } else {
                    condition
                };
                Ok(Some(StatementNode::If {
                    condition,
                    then_body,
                    else_body,
                    span,
                }))
            }
            RubyElementType::WhileStatement => {
                let (condition, body) = self.split_cond_body(node, source, offset)?;
                Ok(Some(StatementNode::While { condition, body, span }))
            }
            RubyElementType::UntilStatement => {
                let (condition, body) = self.split_cond_body(node, source, offset)?;
                Ok(Some(StatementNode::Until { condition, body, span }))
            }
            RubyElementType::ForStatement => {
                let var = first_token_text(node, source, offset, |kind| {
                    matches!(kind, RubyTokenType::Identifier | RubyTokenType::Constant)
                })
                .unwrap_or_else(|| "_".into());
                let iterable = self
                    .first_expr(node, source, offset)?
                    .unwrap_or(ExpressionNode::Literal(LiteralNode::Nil { span: span.clone() }));
                let body = self.build_statement_list(node, source, offset)?;
                Ok(Some(StatementNode::For {
                    var,
                    iterable,
                    body,
                    span,
                }))
            }
            RubyElementType::ReturnStatement => {
                let value = self.first_expr(node, source, offset)?;
                Ok(Some(StatementNode::Return { value, span }))
            }
            RubyElementType::BreakStatement => Ok(Some(StatementNode::Break { span })),
            RubyElementType::AssignmentStatement => {
                let target = first_token_text(node, source, offset, |kind| {
                    matches!(
                        kind,
                        RubyTokenType::Identifier
                            | RubyTokenType::Constant
                            | RubyTokenType::GlobalVariable
                            | RubyTokenType::InstanceVariable
                            | RubyTokenType::ClassVariable
                    )
                })
                .unwrap_or_else(|| "_".into());
                let op = first_token_text(node, source, offset, |kind| {
                    matches!(
                        kind,
                        RubyTokenType::Assign
                            | RubyTokenType::PlusAssign
                            | RubyTokenType::MinusAssign
                            | RubyTokenType::MultiplyAssign
                            | RubyTokenType::DivideAssign
                            | RubyTokenType::OrOrAssign
                            | RubyTokenType::AndAndAssign
                    )
                })
                .unwrap_or_else(|| "=".into());
                let rhs = self
                    .first_expr(node, source, offset)?
                    .unwrap_or(ExpressionNode::Literal(LiteralNode::Nil { span: span.clone() }));
                let value = match op.as_str() {
                    "+=" => ExpressionNode::BinaryOp {
                        left: Box::new(ExpressionNode::Identifier {
                            name: target.clone(),
                            span: span.clone(),
                        }),
                        operator: "+".into(),
                        right: Box::new(rhs),
                        span: span.clone(),
                    },
                    "-=" => ExpressionNode::BinaryOp {
                        left: Box::new(ExpressionNode::Identifier {
                            name: target.clone(),
                            span: span.clone(),
                        }),
                        operator: "-".into(),
                        right: Box::new(rhs),
                        span: span.clone(),
                    },
                    "*=" => ExpressionNode::BinaryOp {
                        left: Box::new(ExpressionNode::Identifier {
                            name: target.clone(),
                            span: span.clone(),
                        }),
                        operator: "*".into(),
                        right: Box::new(rhs),
                        span: span.clone(),
                    },
                    _ => rhs,
                };
                Ok(Some(StatementNode::Assignment { target, value, span }))
            }
            RubyElementType::BinaryExpression
            | RubyElementType::UnaryExpression
            | RubyElementType::LiteralExpression
            | RubyElementType::Identifier
            | RubyElementType::CallExpression
            | RubyElementType::ArrayExpression
            | RubyElementType::ParenExpression
            | RubyElementType::ParenthesizedExpression => {
                let expr = self.build_expression(node, source, offset)?;
                Ok(expr.map(StatementNode::Expression))
            }
            _ => Ok(None),
        }
    }

    fn split_method<S: Source + ?Sized>(&self, node: &GreenNode<RubyLanguage>, source: &S, base: usize) -> Result<(String, Vec<String>, Vec<StatementNode>), oak_core::OakError> {
        let mut name = String::new();
        let mut params = Vec::new();
        let mut in_params = false;
        let mut offset = base;
        let mut body_nodes = Vec::new();
        for child in node.children() {
            match child {
                GreenTree::Leaf(leaf) => {
                    let text = text_at(source, offset, leaf.length);
                    match leaf.kind {
                        RubyTokenType::Identifier | RubyTokenType::Constant if name.is_empty() && !in_params => name = text,
                        RubyTokenType::LeftParen => in_params = true,
                        RubyTokenType::RightParen => in_params = false,
                        RubyTokenType::Identifier | RubyTokenType::Constant if in_params => params.push(text),
                        _ => {}
                    }
                }
                GreenTree::Node(child_node) => body_nodes.push((child_node, offset)),
            }
            offset += child.len() as usize;
        }
        let mut body = Vec::new();
        for (child_node, child_offset) in body_nodes {
            if let Some(stmt) = self.build_statement(child_node, source, child_offset)? {
                body.push(stmt);
            }
        }
        Ok((name, params, body))
    }

    fn split_if<S: Source + ?Sized>(
        &self,
        node: &GreenNode<RubyLanguage>,
        source: &S,
        base: usize,
    ) -> Result<(ExpressionNode, Vec<StatementNode>, Option<Vec<StatementNode>>), oak_core::OakError> {
        // 修饰符形式 `body unless/if cond`：关键字出现在第一个表达式结点之后。
        let mut offset_scan = base;
        let mut saw_expr = false;
        let mut is_modifier = false;
        for child in node.children() {
            match child {
                GreenTree::Node(child_node)
                    if matches!(
                        child_node.kind,
                        RubyElementType::BinaryExpression
                            | RubyElementType::UnaryExpression
                            | RubyElementType::LiteralExpression
                            | RubyElementType::Identifier
                            | RubyElementType::CallExpression
                            | RubyElementType::ParenExpression
                            | RubyElementType::ArrayExpression
                    ) =>
                {
                    saw_expr = true;
                }
                GreenTree::Leaf(leaf)
                    if saw_expr
                        && matches!(leaf.kind, RubyTokenType::If | RubyTokenType::Unless) =>
                {
                    is_modifier = true;
                    break;
                }
                _ => {}
            }
            offset_scan += child.len() as usize;
        }

        let span = span_at(base, node.byte_length);
        if is_modifier {
            let mut body_expr = None;
            let mut condition = None;
            let mut offset = base;
            for child in node.children() {
                if let GreenTree::Node(child_node) = child {
                    if matches!(
                        child_node.kind,
                        RubyElementType::BinaryExpression
                            | RubyElementType::UnaryExpression
                            | RubyElementType::LiteralExpression
                            | RubyElementType::Identifier
                            | RubyElementType::CallExpression
                            | RubyElementType::ParenExpression
                            | RubyElementType::ArrayExpression
                    ) {
                        if body_expr.is_none() {
                            body_expr = self.build_expression(child_node, source, offset)?;
                        } else if condition.is_none() {
                            condition = self.build_expression(child_node, source, offset)?;
                        }
                    }
                }
                offset += child.len() as usize;
            }
            let condition = condition.unwrap_or(ExpressionNode::Literal(LiteralNode::Nil {
                span: span.clone(),
            }));
            let then_body = body_expr
                .map(|e| vec![StatementNode::Expression(e)])
                .unwrap_or_default();
            return Ok((condition, then_body, None));
        }

        let mut condition = None;
        let mut then_body = Vec::new();
        let mut else_body = Vec::new();
        let mut in_else = false;
        let mut offset = base;
        for child in node.children() {
            match child {
                GreenTree::Leaf(leaf) if leaf.kind == RubyTokenType::Else => in_else = true,
                GreenTree::Node(child_node) => {
                    if condition.is_none()
                        && matches!(
                            child_node.kind,
                            RubyElementType::BinaryExpression
                                | RubyElementType::UnaryExpression
                                | RubyElementType::LiteralExpression
                                | RubyElementType::Identifier
                                | RubyElementType::CallExpression
                                | RubyElementType::ParenExpression
                        )
                    {
                        condition = self.build_expression(child_node, source, offset)?;
                    } else if let Some(stmt) = self.build_statement(child_node, source, offset)? {
                        if in_else {
                            else_body.push(stmt);
                        } else {
                            then_body.push(stmt);
                        }
                    }
                }
                _ => {}
            }
            offset += child.len() as usize;
        }
        Ok((
            condition.unwrap_or(ExpressionNode::Literal(LiteralNode::Nil { span })),
            then_body,
            if else_body.is_empty() { None } else { Some(else_body) },
        ))
    }

    fn split_cond_body<S: Source + ?Sized>(&self, node: &GreenNode<RubyLanguage>, source: &S, base: usize) -> Result<(ExpressionNode, Vec<StatementNode>), oak_core::OakError> {
        let (condition, body, _) = self.split_if(node, source, base)?;
        Ok((condition, body))
    }

    fn first_expr<S: Source + ?Sized>(&self, node: &GreenNode<RubyLanguage>, source: &S, base: usize) -> Result<Option<ExpressionNode>, oak_core::OakError> {
        let mut offset = base;
        for child in node.children() {
            if let GreenTree::Node(child_node) = child {
                if let Some(expr) = self.build_expression(child_node, source, offset)? {
                    return Ok(Some(expr));
                }
            }
            offset += child.len() as usize;
        }
        Ok(None)
    }

    fn split_block<S: Source + ?Sized>(
        &self,
        node: &GreenNode<RubyLanguage>,
        source: &S,
        base: usize,
    ) -> Result<(Vec<String>, Vec<StatementNode>), oak_core::OakError> {
        let mut params = Vec::new();
        let mut in_params = false;
        let mut offset = base;
        let mut body = Vec::new();
        for child in node.children() {
            match child {
                GreenTree::Leaf(leaf) => {
                    if leaf.kind == RubyTokenType::BitOr {
                        in_params = !in_params;
                    } else if in_params
                        && matches!(
                            leaf.kind,
                            RubyTokenType::Identifier | RubyTokenType::Constant
                        )
                    {
                        let text = text_at(source, offset, leaf.length);
                        if !text.is_empty() {
                            params.push(text);
                        }
                    }
                }
                GreenTree::Node(child_node) => {
                    if child_node.kind == RubyElementType::BlockExpression {
                        let (p, b) = self.split_block(child_node, source, offset)?;
                        params.extend(p);
                        body.extend(b);
                    } else if let Some(stmt) = self.build_statement(child_node, source, offset)? {
                        body.push(stmt);
                    }
                }
            }
            offset += child.len() as usize;
        }
        Ok((params, body))
    }

    fn build_expression<S: Source + ?Sized>(&self, node: &GreenNode<RubyLanguage>, source: &S, offset: usize) -> Result<Option<ExpressionNode>, oak_core::OakError> {
        let span = span_at(offset, node.byte_length);
        match node.kind {
            RubyElementType::Identifier => {
                let name = first_token_text(node, source, offset, |_| true).unwrap_or_default();
                Ok(Some(ExpressionNode::Identifier { name, span }))
            }
            RubyElementType::LiteralExpression => {
                // `self` 是标识符语义，不是 nil。
                if first_token_kind(node) == Some(RubyTokenType::Self_) {
                    Ok(Some(ExpressionNode::Identifier {
                        name: "self".into(),
                        span,
                    }))
                } else {
                    Ok(Some(ExpressionNode::Literal(self.build_literal(node, source, offset))))
                }
            }
            RubyElementType::ParenExpression | RubyElementType::ParenthesizedExpression => self.first_expr(node, source, offset),
            RubyElementType::UnaryExpression => {
                let operator = first_token_text(node, source, offset, |kind| is_operator(kind)).unwrap_or_else(|| "!".into());
                let operand = self.first_expr(node, source, offset)?.unwrap_or(ExpressionNode::Literal(LiteralNode::Nil { span: span.clone() }));
                Ok(Some(ExpressionNode::UnaryOp {
                    operator,
                    operand: Box::new(operand),
                    span,
                }))
            }
            RubyElementType::BinaryExpression => {
                let mut left = None;
                let mut operator = String::new();
                let mut right = None;
                let mut child_offset = offset;
                for child in node.children() {
                    match child {
                        GreenTree::Leaf(leaf) if is_operator(leaf.kind) && operator.is_empty() => {
                            operator = token_operator(leaf.kind).unwrap_or_else(|| text_at(source, child_offset, leaf.length));
                        }
                        GreenTree::Node(child_node) => {
                            if let Some(expr) = self.build_expression(child_node, source, child_offset)? {
                                if left.is_none() {
                                    left = Some(expr);
                                } else {
                                    right = Some(expr);
                                }
                            }
                        }
                        _ => {}
                    }
                    child_offset += child.len() as usize;
                }
                match (left, right) {
                    (Some(left), Some(right)) => Ok(Some(ExpressionNode::BinaryOp {
                        left: Box::new(left),
                        operator,
                        right: Box::new(right),
                        span,
                    })),
                    (Some(expr), None) | (None, Some(expr)) => Ok(Some(expr)),
                    _ => Ok(None),
                }
            }
            RubyElementType::CallExpression => {
                // `foo(a)` / `recv.foo(a)` / `A::B` / `recv[a]` / `loop do` / `.each{|x|}`
                let mut receiver = None;
                let mut method = String::new();
                let mut args = Vec::new();
                let mut saw_dot = false;
                let mut saw_colon = false;
                let mut saw_bracket = false;
                let mut saw_assign = false;
                let mut saw_paren = false;
                let mut exprs = Vec::new();
                let mut block_params = Vec::new();
                let mut block_body = None;
                let mut child_offset = offset;
                for child in node.children() {
                    match child {
                        GreenTree::Leaf(leaf) => {
                            if leaf.kind == RubyTokenType::Dot {
                                saw_dot = true;
                            } else if leaf.kind == RubyTokenType::DoubleColon {
                                saw_colon = true;
                            } else if leaf.kind == RubyTokenType::LeftBracket {
                                saw_bracket = true;
                            } else if leaf.kind == RubyTokenType::LeftParen {
                                saw_paren = true;
                            } else if leaf.kind == RubyTokenType::Assign {
                                saw_assign = true;
                            } else if matches!(
                                leaf.kind,
                                RubyTokenType::Identifier | RubyTokenType::Constant
                            ) {
                                let text = text_at(source, child_offset, leaf.length);
                                if saw_dot || saw_colon {
                                    if method.is_empty() || saw_dot || saw_colon {
                                        if saw_colon && !method.is_empty() {
                                            // 已在下方用路径拼接处理
                                        }
                                        method = text;
                                    }
                                } else if method.is_empty() {
                                    method = text;
                                }
                            }
                        }
                        GreenTree::Node(child_node) => {
                            if child_node.kind == RubyElementType::BlockExpression {
                                let (params, body) =
                                    self.split_block(child_node, source, child_offset)?;
                                block_params = params;
                                block_body = Some(body);
                            } else if let Some(expr) =
                                self.build_expression(child_node, source, child_offset)?
                            {
                                exprs.push(expr);
                            }
                        }
                    }
                    child_offset += child.len() as usize;
                }
                if saw_assign && saw_dot {
                    method = format!("{method}=");
                }
                if saw_colon {
                    // `A::B` / `A::B(x)`：合成接收者路径或扁平名。
                    // 子节点可能是 Identifier 结点（非纯 Leaf）。
                    let mut parts = Vec::new();
                    let mut path_offset = offset;
                    for child in node.children() {
                        match child {
                            GreenTree::Leaf(leaf) => {
                                if matches!(
                                    leaf.kind,
                                    RubyTokenType::Identifier | RubyTokenType::Constant
                                ) {
                                    parts.push(text_at(source, path_offset, leaf.length));
                                } else if leaf.kind == RubyTokenType::DoubleColon {
                                    // 路径分隔，继续。
                                } else if leaf.kind == RubyTokenType::Dot {
                                    // `A::B.method`：路径到此结束，方法名由外层 saw_dot 处理。
                                    break;
                                } else if !leaf.kind.is_ignored()
                                    && leaf.kind != RubyTokenType::LeftParen
                                    && leaf.kind != RubyTokenType::RightParen
                                    && leaf.kind != RubyTokenType::Comma
                                {
                                    break;
                                }
                            }
                            GreenTree::Node(child_node) => {
                                if child_node.kind == RubyElementType::Identifier
                                    || child_node.kind == RubyElementType::LiteralExpression
                                {
                                    if let Some(ExpressionNode::Identifier { name, .. }) =
                                        self.build_expression(child_node, source, path_offset)?
                                    {
                                        parts.push(name);
                                    } else {
                                        break;
                                    }
                                } else {
                                    // 参数 / 嵌套调用：路径结束。
                                    break;
                                }
                            }
                        }
                        path_offset += child.len() as usize;
                    }
                    // 纯 `RPG::Cache`（无括号参数、无尾随 `.method`）→ 标识符。
                    // 注意：首遍可能把路径 Identifier 推进 exprs，不能靠 exprs.is_empty()。
                    if parts.len() >= 2 && !saw_paren && block_body.is_none() && !saw_dot {
                        let full = parts.join("::");
                        return Ok(Some(ExpressionNode::Identifier {
                            name: full,
                            span,
                        }));
                    }
                    // `RPG::Cache(x)`：路径前缀为接收者，末段为方法；exprs 里的路径 Identifier 丢掉。
                    if parts.len() >= 2 && !saw_dot && !saw_bracket {
                        let path = parts[..parts.len() - 1].join("::");
                        method = parts.last().cloned().unwrap_or_default();
                        receiver = Some(ExpressionNode::Identifier {
                            name: path,
                            span: span.clone(),
                        });
                        // 括号内实参：去掉前缀路径 Identifier。
                        let mut real = std::mem::take(&mut exprs);
                        if real.len() >= parts.len() {
                            real.drain(0..parts.len() - 1);
                            // 再去掉与 method 同名的接收段（若存在）
                            if let Some(ExpressionNode::Identifier { name, .. }) = real.first() {
                                if name == &method {
                                    real.remove(0);
                                }
                            }
                        } else {
                            real.clear();
                        }
                        args = real;
                    }
                }
                if saw_bracket && !saw_dot {
                    method = "[]".into();
                    let mut iter = std::mem::take(&mut exprs).into_iter();
                    receiver = iter.next();
                    args = iter.collect();
                } else if saw_dot {
                    // `recv.method` / `RPG::Cache.title`（内层已是路径 Identifier）。
                    let mut iter = std::mem::take(&mut exprs).into_iter();
                    receiver = iter.next();
                    args = iter.collect();
                } else if !saw_colon {
                    // `loop do ... end`：exprs 可能空，method 来自首标识符；块在 block_body。
                    let exprs = std::mem::take(&mut exprs);
                    if method.is_empty() {
                        if let Some(ExpressionNode::Identifier { name, .. }) = exprs.first() {
                            method = name.clone();
                            args = exprs.into_iter().skip(1).collect();
                        } else {
                            args = exprs;
                        }
                    } else {
                        args = exprs;
                    }
                } else if receiver.is_none() {
                    // `::` 路径未吃掉 exprs 时的回退。
                    args = std::mem::take(&mut exprs);
                }
                Ok(Some(ExpressionNode::MethodCall {
                    receiver: receiver.map(Box::new),
                    method,
                    args,
                    block_params,
                    block_body,
                    span,
                }))
            }
            RubyElementType::ArrayExpression => {
                let mut elements = Vec::new();
                let mut child_offset = offset;
                for child in node.children() {
                    if let GreenTree::Node(child_node) = child {
                        if let Some(expr) = self.build_expression(child_node, source, child_offset)? {
                            elements.push(expr);
                        }
                    }
                    child_offset += child.len() as usize;
                }
                Ok(Some(ExpressionNode::Array { elements, span }))
            }
            _ => Ok(None),
        }
    }

    fn build_literal<S: Source + ?Sized>(&self, node: &GreenNode<RubyLanguage>, source: &S, offset: usize) -> LiteralNode {
        let mut child_offset = offset;
        for child in node.children() {
            if let GreenTree::Leaf(leaf) = child {
                let span = span_at(child_offset, leaf.length);
                let text = text_at(source, child_offset, leaf.length);
                return match leaf.kind {
                    RubyTokenType::IntegerLiteral => LiteralNode::Integer {
                        value: parse_ruby_int(&text),
                        span,
                    },
                    RubyTokenType::FloatLiteral => LiteralNode::Float {
                        value: text.parse().unwrap_or(0.0),
                        span,
                    },
                    RubyTokenType::StringLiteral => LiteralNode::String {
                        value: unquote(&text),
                        span,
                    },
                    RubyTokenType::True => LiteralNode::Boolean { value: true, span },
                    RubyTokenType::False => LiteralNode::Boolean { value: false, span },
                    RubyTokenType::Nil | RubyTokenType::Self_ => LiteralNode::Nil { span },
                    RubyTokenType::Symbol => LiteralNode::Symbol {
                        value: text.trim_start_matches(':').to_string(),
                        span,
                    },
                    _ => LiteralNode::Nil { span },
                };
            }
            child_offset += child.len() as usize;
        }
        LiteralNode::Nil {
            span: span_at(offset, node.byte_length),
        }
    }
}

fn span_at(start: usize, len: u32) -> Range<usize> {
    Range {
        start,
        end: start + len as usize,
    }
}

fn text_at<S: Source + ?Sized>(source: &S, start: usize, len: u32) -> String {
    source
        .get_text_in(oak_core::Range {
            start,
            end: start + len as usize,
        })
        .trim()
        .to_string()
}

fn first_token_text<S: Source + ?Sized>(node: &GreenNode<RubyLanguage>, source: &S, base: usize, pred: impl Fn(RubyTokenType) -> bool) -> Option<String> {
    let mut offset = base;
    for child in node.children() {
        if let GreenTree::Leaf(leaf) = child {
            if pred(leaf.kind) && !leaf.kind.is_ignored() {
                let text = text_at(source, offset, leaf.length);
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        offset += child.len() as usize;
    }
    None
}

fn first_token_kind(node: &GreenNode<RubyLanguage>) -> Option<RubyTokenType> {
    for child in node.children() {
        if let GreenTree::Leaf(leaf) = child {
            if !leaf.kind.is_ignored() {
                return Some(leaf.kind);
            }
        }
    }
    None
}

/// 在 class/module 节点里找类型名（跳过 `class`/`module` 关键字与空白）。
fn class_or_module_name<S: Source + ?Sized>(node: &GreenNode<RubyLanguage>, source: &S, base: usize) -> Option<String> {
    let mut offset = base;
    let mut saw_keyword = false;
    for child in node.children() {
        match child {
            GreenTree::Leaf(leaf) => {
                if leaf.kind.is_ignored() {
                    offset += child.len() as usize;
                    continue;
                }
                if matches!(leaf.kind, RubyTokenType::Class | RubyTokenType::Module) {
                    saw_keyword = true;
                    offset += child.len() as usize;
                    continue;
                }
                if saw_keyword
                    && matches!(
                        leaf.kind,
                        RubyTokenType::Constant | RubyTokenType::Identifier
                    )
                {
                    let text = text_at(source, offset, leaf.length);
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
            GreenTree::Node(child_node) => {
                // 个别情况下常量被包进 Identifier 节点。
                if saw_keyword {
                    if let Some(text) = first_token_text(child_node, source, offset, |kind| {
                        matches!(kind, RubyTokenType::Constant | RubyTokenType::Identifier)
                    }) {
                        if !text.is_empty() && text != "class" && text != "module" {
                            return Some(text);
                        }
                    }
                }
            }
        }
        offset += child.len() as usize;
    }
    None
}

fn is_operator(kind: RubyTokenType) -> bool {
    token_operator(kind).is_some()
}

fn token_operator(kind: RubyTokenType) -> Option<String> {
    let text = match kind {
        RubyTokenType::Plus => "+",
        RubyTokenType::Minus => "-",
        RubyTokenType::Multiply => "*",
        RubyTokenType::Divide => "/",
        RubyTokenType::Modulo => "%",
        RubyTokenType::EqualEqual => "==",
        RubyTokenType::NotEqual => "!=",
        RubyTokenType::Less => "<",
        RubyTokenType::Greater => ">",
        RubyTokenType::LessEqual => "<=",
        RubyTokenType::GreaterEqual => ">=",
        RubyTokenType::AndAnd | RubyTokenType::And => "&&",
        RubyTokenType::OrOr | RubyTokenType::Or => "||",
        RubyTokenType::Power => "**",
        RubyTokenType::Not | RubyTokenType::LogicalNot => "!",
        RubyTokenType::DotDot => "..",
        RubyTokenType::DotDotDot => "...",
        _ => return None,
    };
    Some(text.into())
}

fn unquote(text: &str) -> String {
    let bytes = text.as_bytes();
    if bytes.len() >= 2 && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"') || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'')) {
        text[1..text.len() - 1].to_string()
    } else {
        text.to_string()
    }
}

fn parse_ruby_int(text: &str) -> i64 {
    let t = text.replace('_', "");
    if let Some(rest) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return i64::from_str_radix(rest, 16).unwrap_or(0);
    }
    if let Some(rest) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
        return i64::from_str_radix(rest, 2).unwrap_or(0);
    }
    if t.len() > 1 && t.starts_with('0') && t.chars().all(|c| c.is_ascii_digit()) {
        return i64::from_str_radix(&t[1..], 8).unwrap_or_else(|_| t.parse().unwrap_or(0));
    }
    t.parse().unwrap_or(0)
}
