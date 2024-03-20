use core::range::Range;

use oak_core::{GreenNode, GreenTree, OakError, Source, SourceText, TokenType};

use crate::{
    ast::{VosBinaryOp, VosExpr, VosFieldInit, VosFnDecl, VosFnKind, VosLambda, VosLet, VosLiteral, VosParameter, VosPathSep, VosProgram, VosProjItem, VosRoot, VosStmt, VosSyntaxElement, VosSyntaxSlice, VosTypeSyntax, VosUnaryOp},
    language::VosLanguage,
    lexer::VosTokenType,
};

use super::VosBuilder;

/// A significant source token addressed by its span.
type Token = (VosTokenType, Range<usize>);

/// Two-character operators that the VOS lexer emits as single characters.
const MERGED_OPERATORS: [&str; 9] = ["==", "!=", "<=", ">=", "&&", "||", "::", "=>", "->"];

impl VosBuilder {
    /// Collects significant leaves of an Oak green tree in source order.
    fn collect_green_tokens(tree: &GreenTree<'_, VosLanguage>, offset: &mut usize, tokens: &mut Vec<Token>) {
        match tree {
            GreenTree::Node(node) => {
                for child in node.children {
                    Self::collect_green_tokens(child, offset, tokens);
                }
            }
            GreenTree::Leaf(leaf) => {
                let start = *offset;
                *offset += leaf.length as usize;
                if !leaf.kind.is_ignored() {
                    tokens.push((leaf.kind, (start..*offset).into()));
                }
            }
        }
    }

    /// Collects significant leaves of an Oak green node in source order.
    fn collect_green_node(node: &GreenNode<'_, VosLanguage>, offset: &mut usize, tokens: &mut Vec<Token>) {
        for child in node.children {
            Self::collect_green_tokens(child, offset, tokens);
        }
    }

    /// Collects significant tokens of an Oak-built syntax element in source order.
    fn collect_syntax_tokens(element: &VosSyntaxElement, tokens: &mut Vec<Token>) {
        match element {
            VosSyntaxElement::Node(node) => {
                for child in &node.children {
                    Self::collect_syntax_tokens(child, tokens);
                }
            }
            VosSyntaxElement::Token(token) => {
                if !token.kind.is_ignored() {
                    tokens.push((token.kind, token.span.clone()));
                }
            }
        }
    }

    /// Merges adjacent single-character tokens that spell one VOS operator.
    fn merge_operators(tokens: &mut Vec<Token>, source: &SourceText) {
        let mut merged: Vec<Token> = Vec::with_capacity(tokens.len());
        for token in tokens.drain(..) {
            if let Some(previous) = merged.last_mut() {
                if previous.1.end == token.1.start {
                    let span: Range<usize> = (previous.1.start..token.1.end).into();
                    if MERGED_OPERATORS.contains(&source.get_text_in(span.clone().into()).as_ref()) {
                        *previous = (previous.0, span);
                        continue;
                    }
                }
            }
            merged.push(token);
        }
        *tokens = merged;
    }

    /// Projects a balanced body block into an expression program.
    ///
    /// Returns `None` when the body does not follow the VOS expression grammar.
    pub(super) fn build_block_program(&self, node: &GreenNode<'_, VosLanguage>, offset: usize, source: &SourceText) -> Option<VosProgram> {
        let mut tokens = Vec::new();
        let mut cursor = offset;
        Self::collect_green_node(node, &mut cursor, &mut tokens);
        Self::merge_operators(&mut tokens, source);
        let span_start = tokens.first().map(|token| token.1.start)?;
        let span_end = tokens.last().map(|token| token.1.end)?;
        let inner = match tokens.first().map(|token| token.0) {
            Some(VosTokenType::LeftBrace) if tokens.last().map(|token| token.0) == Some(VosTokenType::RightBrace) => tokens[1..tokens.len() - 1].to_vec(),
            _ => tokens,
        };
        ProgramParser::new(self, source, inner, span_start, span_end).parse_program(false).ok()
    }

    /// Projects a whole Oak root into an expression program.
    pub fn project_program(&self, root: &VosRoot) -> Result<VosProgram, OakError> {
        let source = SourceText::new(root.source.clone());
        let mut tokens = Vec::new();
        for child in &root.syntax.children {
            Self::collect_syntax_tokens(child, &mut tokens);
        }
        Self::merge_operators(&mut tokens, &source);
        let start = 0;
        let end = tokens.last().map(|token| token.1.end).unwrap_or(0);
        ProgramParser::new(self, &source, tokens, start, end).parse_program(false)
    }
}

/// Recursive descent parser over Oak-provided tokens for VOS expression programs.
struct ProgramParser<'a, 'b> {
    builder: &'b VosBuilder,
    source: &'a SourceText,
    tokens: Vec<Token>,
    index: usize,
    span_start: usize,
    span_end: usize,
}

impl<'a, 'b> ProgramParser<'a, 'b> {
    fn new(builder: &'b VosBuilder, source: &'a SourceText, tokens: Vec<Token>, span_start: usize, span_end: usize) -> Self {
        Self { builder, source, tokens, index: 0, span_start, span_end }
    }

    // --- token access ---

    fn at_end(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn kind_at(&self, index: usize) -> Option<VosTokenType> {
        self.tokens.get(index).map(|token| token.0)
    }

    fn at(&self, kind: VosTokenType) -> bool {
        self.kind_at(self.index) == Some(kind)
    }

    fn text_at(&self, index: usize) -> Option<std::borrow::Cow<'a, str>> {
        self.tokens.get(index).map(|token| self.source.get_text_in(token.1.clone().into()))
    }

    fn at_text(&self, expected: &str) -> bool {
        self.text_at(self.index).is_some_and(|text| text == expected)
    }

    fn eat(&mut self, kind: VosTokenType) -> bool {
        if self.at(kind) {
            self.index += 1;
            return true;
        }
        false
    }

    fn eat_text(&mut self, expected: &str) -> bool {
        if self.at_text(expected) {
            self.index += 1;
            return true;
        }
        false
    }

    fn current_start(&self) -> usize {
        self.tokens.get(self.index).map(|token| token.1.start).unwrap_or(self.span_end)
    }

    fn last_end(&self) -> usize {
        if self.index == 0 { self.span_end } else { self.tokens[self.index - 1].1.end }
    }

    fn unexpected(&self, message: &str) -> OakError {
        OakError::expected_token(message, self.current_start(), None)
    }

    fn unexpected_eof(&self) -> OakError {
        OakError::unexpected_eof(self.span_end, None)
    }

    fn expect(&mut self, kind: VosTokenType, message: &str) -> Result<(), OakError> {
        if self.eat(kind) {
            return Ok(());
        }
        if self.at_end() {
            return Err(self.unexpected_eof());
        }
        Err(self.unexpected(message))
    }

    fn is_name(kind: VosTokenType) -> bool {
        matches!(
            kind,
            VosTokenType::Identifier
                | VosTokenType::Namespace
                | VosTokenType::Table
                | VosTokenType::Class
                | VosTokenType::Enums
                | VosTokenType::Flags
                | VosTokenType::Obsolete
                | VosTokenType::Using
                | VosTokenType::Const
                | VosTokenType::Service
                | VosTokenType::Micro
                | VosTokenType::Macro
                | VosTokenType::Query
                | VosTokenType::Udf
                | VosTokenType::Let
                | VosTokenType::Return
        )
    }

    fn expect_name(&mut self, message: &str) -> Result<(String, Range<usize>), OakError> {
        let Some(kind) = self.kind_at(self.index)
        else {
            return Err(self.unexpected_eof());
        };
        if !Self::is_name(kind) {
            return Err(self.unexpected(message));
        }
        let token = self.tokens[self.index].clone();
        self.index += 1;
        let text = self.source.get_text_in(token.1.clone().into()).into_owned();
        Ok((text, token.1))
    }

    fn slice_tokens(&self, start: usize, end: usize) -> VosSyntaxSlice {
        let span: Range<usize> = (self.tokens[start].1.start..self.tokens[end - 1].1.end).into();
        let text = self.source.get_text_in(span.clone().into()).into_owned();
        VosSyntaxSlice { text, span }
    }

    // --- program ---

    fn parse_program(&mut self, stop_at_rbrace: bool) -> Result<VosProgram, OakError> {
        let mut micros = Vec::new();
        let mut statements = Vec::new();
        let mut result = None;
        loop {
            if self.at_end() {
                break;
            }
            if stop_at_rbrace && self.at(VosTokenType::RightBrace) {
                break;
            }
            if self.at(VosTokenType::Micro) {
                micros.push(self.parse_fn_decl(VosFnKind::Micro)?);
                continue;
            }
            if self.at(VosTokenType::Macro) {
                micros.push(self.parse_fn_decl(VosFnKind::Macro)?);
                continue;
            }
            if self.at(VosTokenType::Let) {
                statements.push(VosStmt::Let(self.parse_let()?));
                continue;
            }
            let expr = self.parse_expression()?;
            if self.at_end() || (stop_at_rbrace && self.at(VosTokenType::RightBrace)) {
                result = Some(expr);
                break;
            }
            statements.push(VosStmt::Expr(expr));
        }
        Ok(VosProgram { micros, statements, result, span: (self.span_start..self.span_end).into() })
    }

    fn parse_fn_decl(&mut self, kind: VosFnKind) -> Result<VosFnDecl, OakError> {
        let span_start = self.current_start();
        self.index += 1;
        let (name, name_span) = self.expect_name("VOS function name")?;
        self.expect(VosTokenType::LeftParen, "expected `(` after VOS function name")?;
        let mut parameters = Vec::new();
        loop {
            if self.at_end() {
                return Err(self.unexpected_eof());
            }
            if self.eat(VosTokenType::RightParen) {
                break;
            }
            let parameter_start = self.current_start();
            let (parameter_name, parameter_name_span) = self.expect_name("VOS parameter name")?;
            self.expect(VosTokenType::Colon, "expected `:` after VOS parameter name")?;
            let type_index = self.index;
            let type_expr = self.parse_type()?;
            let type_syntax = self.slice_tokens(type_index, self.index);
            parameters.push(VosParameter { name: parameter_name, name_span: parameter_name_span, type_syntax, type_expr, span: (parameter_start..self.last_end()).into() });
            if self.eat(VosTokenType::Comma) {
                continue;
            }
            if !self.at(VosTokenType::RightParen) {
                return Err(self.unexpected("expected `,` or `)` after VOS parameter"));
            }
        }
        let return_type = if self.eat_text("->") { Some(self.parse_type()?) } else { None };
        let body_start = self.current_start();
        self.expect(VosTokenType::LeftBrace, "expected `{` before VOS function body")?;
        let mut body = self.parse_program(true)?;
        self.expect(VosTokenType::RightBrace, "expected `}` after VOS function body")?;
        body.span = (body_start..self.last_end()).into();
        Ok(VosFnDecl { kind, name, name_span, parameters, return_type, body, span: (span_start..self.last_end()).into() })
    }

    fn parse_let(&mut self) -> Result<VosLet, OakError> {
        let span_start = self.current_start();
        self.index += 1;
        let (name, name_span) = self.expect_name("VOS binding name")?;
        let type_expr = if self.eat(VosTokenType::Colon) { Some(self.parse_type()?) } else { None };
        self.expect(VosTokenType::Equal, "expected `=` in VOS let binding")?;
        let value = self.parse_expression()?;
        Ok(VosLet { name, name_span, type_expr, value, span: (span_start..self.last_end()).into() })
    }

    fn parse_type(&mut self) -> Result<VosTypeSyntax, OakError> {
        if self.at_end() {
            return Err(self.unexpected_eof());
        }
        let mut index = self.index;
        let tokens = &self.tokens;
        let source = self.source;
        let result = self.builder.parse_type_tokens(tokens, &mut index, source)?;
        self.index = index;
        Ok(result)
    }

    // --- expressions ---

    fn parse_expression(&mut self) -> Result<VosExpr, OakError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<VosExpr, OakError> {
        let mut left = self.parse_and()?;
        while self.eat_text("||") {
            let right = self.parse_and()?;
            let span_start = Self::expr_start(&left);
            left = VosExpr::Binary { op: VosBinaryOp::Or, left: Box::new(left), right: Box::new(right), span: (span_start..self.last_end()).into() };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<VosExpr, OakError> {
        let mut left = self.parse_cmp()?;
        while self.eat_text("&&") {
            let right = self.parse_cmp()?;
            let span_start = Self::expr_start(&left);
            left = VosExpr::Binary { op: VosBinaryOp::And, left: Box::new(left), right: Box::new(right), span: (span_start..self.last_end()).into() };
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<VosExpr, OakError> {
        let mut left = self.parse_add()?;
        loop {
            let op = if self.eat_text("==") {
                Some(VosBinaryOp::Eq)
            }
            else if self.eat_text("!=") {
                Some(VosBinaryOp::Ne)
            }
            else if self.eat_text("<=") {
                Some(VosBinaryOp::Le)
            }
            else if self.eat_text(">=") {
                Some(VosBinaryOp::Ge)
            }
            else if self.eat_text("<") {
                Some(VosBinaryOp::Lt)
            }
            else if self.eat_text(">") {
                Some(VosBinaryOp::Gt)
            }
            else {
                None
            };
            let Some(op) = op
            else {
                break;
            };
            let right = self.parse_add()?;
            let span_start = Self::expr_start(&left);
            left = VosExpr::Binary { op, left: Box::new(left), right: Box::new(right), span: (span_start..self.last_end()).into() };
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<VosExpr, OakError> {
        let mut left = self.parse_mul()?;
        loop {
            let op = if self.eat_text("+") {
                Some(VosBinaryOp::Add)
            }
            else if self.eat_text("-") {
                Some(VosBinaryOp::Sub)
            }
            else {
                None
            };
            let Some(op) = op
            else {
                break;
            };
            let right = self.parse_mul()?;
            let span_start = Self::expr_start(&left);
            left = VosExpr::Binary { op, left: Box::new(left), right: Box::new(right), span: (span_start..self.last_end()).into() };
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<VosExpr, OakError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = if self.eat_text("*") {
                Some(VosBinaryOp::Mul)
            }
            else if self.eat_text("/") {
                Some(VosBinaryOp::Div)
            }
            else {
                None
            };
            let Some(op) = op
            else {
                break;
            };
            let right = self.parse_unary()?;
            let span_start = Self::expr_start(&left);
            left = VosExpr::Binary { op, left: Box::new(left), right: Box::new(right), span: (span_start..self.last_end()).into() };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<VosExpr, OakError> {
        if self.at_text("!") {
            let span_start = self.current_start();
            self.index += 1;
            let expr = self.parse_unary()?;
            return Ok(VosExpr::Unary { op: VosUnaryOp::Not, expr: Box::new(expr), span: (span_start..self.last_end()).into() });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<VosExpr, OakError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.eat(VosTokenType::Question) {
                let span_start = Self::expr_start(&expr);
                expr = VosExpr::Try { expr: Box::new(expr), span: (span_start..self.last_end()).into() };
                continue;
            }
            if self.eat(VosTokenType::Dot) {
                if self.eat_text("*") {
                    let span_start = Self::expr_start(&expr);
                    expr = VosExpr::StarProj { receiver: Box::new(expr), span: (span_start..self.last_end()).into() };
                    continue;
                }
                if self.at(VosTokenType::LeftBrace) {
                    let span_start = Self::expr_start(&expr);
                    let items = self.parse_proj_items_block()?;
                    expr = VosExpr::StructProj { receiver: Box::new(expr), items, span: (span_start..self.last_end()).into() };
                    continue;
                }
                expr = self.finish_member(expr, VosPathSep::Dot)?;
                continue;
            }
            if self.eat_text("::") {
                expr = self.finish_member(expr, VosPathSep::ColonColon)?;
                continue;
            }
            if self.at(VosTokenType::LeftParen) {
                let span_start = Self::expr_start(&expr);
                let args = self.parse_arg_list()?;
                expr = VosExpr::Call { callee: Box::new(expr), args, span: (span_start..self.last_end()).into() };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn finish_member(&mut self, expr: VosExpr, sep: VosPathSep) -> Result<VosExpr, OakError> {
        let (name, name_span) = self.expect_name("expected VOS member name")?;
        let mut expr = VosExpr::Member { object: Box::new(expr), name, name_span: name_span.clone(), sep, span: name_span.clone() };
        if self.at(VosTokenType::LeftParen) {
            let args = self.parse_arg_list()?;
            expr = VosExpr::Call { callee: Box::new(expr), args, span: (name_span.start..self.last_end()).into() };
        }
        else if sep == VosPathSep::Dot && self.at(VosTokenType::LeftBrace) {
            let patch = self.parse_anon_object()?;
            expr = VosExpr::Call { callee: Box::new(expr), args: vec![patch], span: (name_span.start..self.last_end()).into() };
        }
        Ok(expr)
    }

    fn expr_start(expr: &VosExpr) -> usize {
        match expr {
            VosExpr::Literal(_) => 0,
            other => other.span().start,
        }
    }

    fn parse_primary(&mut self) -> Result<VosExpr, OakError> {
        let Some(kind) = self.kind_at(self.index)
        else {
            return Err(self.unexpected_eof());
        };
        match kind {
            VosTokenType::LeftParen => self.parse_parenthesized(),
            VosTokenType::LeftBracket => self.parse_list(),
            VosTokenType::LeftBrace => self.parse_anon_object(),
            VosTokenType::NullLiteral | VosTokenType::BooleanLiteral | VosTokenType::StringLiteral | VosTokenType::NumberLiteral => self.parse_literal(),
            _ if Self::is_name(kind) => self.parse_name_expression(),
            _ => Err(self.unexpected("expected VOS expression")),
        }
    }

    fn parse_parenthesized(&mut self) -> Result<VosExpr, OakError> {
        let span_start = self.current_start();
        self.index += 1;
        let checkpoint = self.index;
        if let Some(lambda) = self.try_paren_lambda(span_start) {
            return Ok(lambda);
        }
        self.index = checkpoint;
        let expr = self.parse_expression()?;
        self.expect(VosTokenType::RightParen, "expected `)` after VOS expression")?;
        Ok(expr)
    }

    fn try_paren_lambda(&mut self, span_start: usize) -> Option<VosExpr> {
        let checkpoint = self.index;
        let mut params = Vec::new();
        let mut param_spans = Vec::new();
        if !self.eat(VosTokenType::RightParen) {
            loop {
                let Ok((name, span)) = self.expect_name("expected VOS lambda parameter")
                else {
                    self.index = checkpoint;
                    return None;
                };
                params.push(name);
                param_spans.push(span);
                if self.eat(VosTokenType::RightParen) {
                    break;
                }
                if !self.eat(VosTokenType::Comma) {
                    self.index = checkpoint;
                    return None;
                }
            }
        }
        if !self.eat_text("=>") {
            self.index = checkpoint;
            return None;
        }
        match self.parse_lambda_body() {
            Ok(body) => Some(VosExpr::Lambda(VosLambda { params, param_spans, body: Box::new(body), span: (span_start..self.last_end()).into() })),
            Err(_) => {
                self.index = checkpoint;
                None
            }
        }
    }

    fn parse_name_expression(&mut self) -> Result<VosExpr, OakError> {
        let (name, name_span) = self.expect_name("expected VOS expression")?;
        if self.eat_text("=>") {
            let body = self.parse_lambda_body()?;
            return Ok(VosExpr::Lambda(VosLambda { params: vec![name], param_spans: vec![name_span.clone()], body: Box::new(body), span: (name_span.start..self.last_end()).into() }));
        }
        if self.at(VosTokenType::LeftBrace) {
            let fields = self.parse_field_init_block()?;
            return Ok(VosExpr::TypedObject { ty: name, ty_span: name_span.clone(), fields, span: (name_span.start..self.last_end()).into() });
        }
        Ok(VosExpr::Name { name, span: name_span })
    }

    fn parse_lambda_body(&mut self) -> Result<VosExpr, OakError> {
        if self.at(VosTokenType::LeftBrace) { self.parse_anon_object() } else { self.parse_expression() }
    }

    fn parse_literal(&mut self) -> Result<VosExpr, OakError> {
        let token = self.tokens[self.index].clone();
        self.index += 1;
        let text = self.source.get_text_in(token.1.clone().into()).into_owned();
        let value = match token.0 {
            VosTokenType::NullLiteral => VosLiteral::Null,
            VosTokenType::BooleanLiteral => VosLiteral::Bool(text == "true"),
            VosTokenType::StringLiteral => {
                let inner = text.strip_prefix('"').and_then(|value| value.strip_suffix('"')).or_else(|| text.strip_prefix('\'').and_then(|value| value.strip_suffix('\'')));
                VosLiteral::String(inner.unwrap_or(text.as_str()).to_owned())
            }
            _ => {
                if text.contains('.') {
                    VosLiteral::Float(text)
                }
                else {
                    VosLiteral::Int(text)
                }
            }
        };
        Ok(VosExpr::Literal(value))
    }

    fn parse_list(&mut self) -> Result<VosExpr, OakError> {
        let span_start = self.current_start();
        self.index += 1;
        let mut items = Vec::new();
        loop {
            if self.at_end() {
                return Err(self.unexpected_eof());
            }
            if self.eat(VosTokenType::RightBracket) {
                break;
            }
            items.push(self.parse_expression()?);
            if self.eat(VosTokenType::Comma) {
                continue;
            }
            self.expect(VosTokenType::RightBracket, "expected `,` or `]` in VOS list")?;
            break;
        }
        Ok(VosExpr::List { items, span: (span_start..self.last_end()).into() })
    }

    fn parse_anon_object(&mut self) -> Result<VosExpr, OakError> {
        let span_start = self.current_start();
        let fields = self.parse_field_init_block()?;
        Ok(VosExpr::AnonObject { fields, span: (span_start..self.last_end()).into() })
    }

    fn parse_field_init_block(&mut self) -> Result<Vec<VosFieldInit>, OakError> {
        self.expect(VosTokenType::LeftBrace, "expected `{` before VOS field list")?;
        let mut fields = Vec::new();
        loop {
            if self.at_end() {
                return Err(self.unexpected_eof());
            }
            if self.eat(VosTokenType::RightBrace) {
                break;
            }
            fields.push(self.parse_field_init()?);
            let _ = self.eat(VosTokenType::Comma);
        }
        Ok(fields)
    }

    fn parse_proj_items_block(&mut self) -> Result<Vec<VosProjItem>, OakError> {
        self.expect(VosTokenType::LeftBrace, "expected `{` before VOS projection items")?;
        let mut items = Vec::new();
        loop {
            if self.at_end() {
                return Err(self.unexpected_eof());
            }
            if self.eat(VosTokenType::RightBrace) {
                break;
            }
            if self.eat_text("*") {
                items.push(VosProjItem::Star { span: (self.last_end() - 1..self.last_end()).into() });
                let _ = self.eat(VosTokenType::Comma);
                continue;
            }
            items.push(VosProjItem::Field(self.parse_field_init()?));
            let _ = self.eat(VosTokenType::Comma);
        }
        Ok(items)
    }

    fn parse_field_init(&mut self) -> Result<VosFieldInit, OakError> {
        let (name, name_span) = self.expect_name("expected VOS field name")?;
        if self.eat(VosTokenType::Colon) {
            let value = self.parse_expression()?;
            Ok(VosFieldInit { name, name_span, value: Some(value), span: (name_span.start..self.last_end()).into() })
        }
        else {
            Ok(VosFieldInit { name, name_span: name_span.clone(), value: None, span: name_span })
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<VosExpr>, OakError> {
        self.expect(VosTokenType::LeftParen, "expected `(` before VOS arguments")?;
        let mut args = Vec::new();
        loop {
            if self.at_end() {
                return Err(self.unexpected_eof());
            }
            if self.eat(VosTokenType::RightParen) {
                break;
            }
            args.push(self.parse_expression()?);
            if self.eat(VosTokenType::Comma) {
                continue;
            }
            self.expect(VosTokenType::RightParen, "expected `,` or `)` in VOS arguments")?;
            break;
        }
        Ok(args)
    }
}
