use crate::{SqlElementType, SqlLanguage, ast::*, builder::SqlBuilder, lexer::token_type::SqlTokenType};
use oak_core::{OakError, Range, RedNode, RedTree, SourceText, TokenType};
use std::sync::Arc;

impl<'config> SqlBuilder<'config> {
    /// Builds a typed expression from an untyped expression node.
    pub(crate) fn build_expression<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Expression, OakError> {
        self.build_expression_with_precedence(node, source, 0)
    }

    /// Internal recursive method to build expressions while respecting operator precedence.
    ///
    /// It uses a "precedence climbing" algorithm to correctly structure binary operations
    /// when the input CST is flat or partially structured.
    fn build_expression_with_precedence<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText, min_precedence: u8) -> Result<Expression, OakError> {
        let children = node.children().collect::<Vec<_>>();

        // Handle parentheses — only unwrap when there is a single inner expression.
        // Multiple comma-separated expressions (e.g. IN lists `(1,2,3)`) must fall
        // through to build_primary_expression which detects LeftParen + expr_children > 1
        // and routes to build_vector.
        if children.len() >= 3 {
            if let (RedTree::Leaf(l), RedTree::Leaf(r)) = (&children[0], &children[children.len() - 1]) {
                if l.kind == SqlTokenType::LeftParen && r.kind == SqlTokenType::RightParen {
                    let expr_count = children[1..children.len() - 1].iter().filter(|c| matches!(c, RedTree::Node(n) if n.green.kind == SqlElementType::Expression)).count();
                    if expr_count == 1 {
                        for child in &children[1..children.len() - 1] {
                            if let RedTree::Node(n) = child {
                                if n.green.kind == SqlElementType::Expression {
                                    return self.build_expression_with_precedence(n.clone(), source, 0);
                                }
                            }
                        }
                    }
                }
            }
        }

        // CASE WHEN ... THEN ... [ELSE ...] END and CAST(expr AS type) are parsed
        // as SqlElementType::Expression nodes whose first leaf is the CASE/CAST
        // keyword. Recognise them before the usual precedence-climbing so we don't
        // try to interpret their WHEN/THEN/ELSE tokens as binary operators.
        let first_non_ws = children.iter().position(|c| !matches!(c, RedTree::Leaf(l) if l.kind == SqlTokenType::Whitespace));
        if let Some(pos) = first_non_ws {
            if let RedTree::Leaf(leaf) = &children[pos] {
                let full_span = node.span();
                match leaf.kind {
                    SqlTokenType::Case => return self.build_case_expression(node, source),
                    SqlTokenType::Cast => return self.build_cast_expression(node, source),
                    SqlTokenType::Exists => return self.build_exists_expression(node, source),
                    _ => {}
                }
                let _ = full_span;
            }
        }

        // Prefix unary operators (`-3`, `+1`, `NOT flag`) keep the operator as a
        // leading leaf, so it must be consumed before the primary expression.
        // Otherwise the binary scan would treat it as a minus without a left operand.
        let mut prefix_end = 0usize;
        let mut prefix_start: Option<usize> = None;
        let mut negate = false;
        let mut logical_not = false;
        while prefix_end < children.len() {
            match &children[prefix_end] {
                RedTree::Leaf(leaf) if leaf.kind == SqlTokenType::Whitespace => prefix_end += 1,
                RedTree::Leaf(leaf) if matches!(leaf.kind, SqlTokenType::Minus | SqlTokenType::Plus) => {
                    if leaf.kind == SqlTokenType::Minus {
                        negate = !negate;
                    }
                    prefix_start.get_or_insert(leaf.span.start);
                    prefix_end += 1;
                }
                RedTree::Leaf(leaf) if leaf.kind == SqlTokenType::Not => {
                    logical_not = !logical_not;
                    prefix_start.get_or_insert(leaf.span.start);
                    prefix_end += 1;
                }
                _ => break,
            }
        }

        let mut left = self.build_primary_expression(node.clone(), source)?;
        if let Some(start) = prefix_start {
            if negate {
                let end = left.span().end;
                left = Expression::Unary { op: UnaryOperator::Minus, expr: Box::new(left), span: Range { start, end } };
            }
            if logical_not {
                let end = left.span().end;
                // `NOT EXISTS (...)` is one predicate, so the negation belongs to
                // the `EXISTS` node instead of wrapping it in a unary `NOT`.
                left = match left {
                    Expression::Exists { query, negated: false, .. } => Expression::Exists { query, negated: true, span: Range { start, end } },
                    other => Expression::Unary { op: UnaryOperator::Not, expr: Box::new(other), span: Range { start, end } },
                };
            }
        }

        // Precedence climbing
        // This is tricky because the CST might already be structured or flat.
        // If it's flat, we iterate through children.
        // If it's already structured (e.g. nested Expression nodes), we might need to handle it differently.

        let mut i = prefix_end;
        // A `NOT` leaf that precedes `IN`, `BETWEEN` or `LIKE` belongs to the
        // operator. The Pratt parser keeps it as a sibling of the operator, so the
        // builder has to carry the flag until the operator branch consumes it.
        let mut negated_operator = false;
        while i < children.len() {
            let child = &children[i];
            if let RedTree::Leaf(op_token) = child {
                let kind = op_token.kind;
                match kind {
                    SqlTokenType::Not => {
                        negated_operator = true;
                    }
                    // === BETWEEN low AND high ===
                    SqlTokenType::Between => {
                        let precedence = 5;
                        if precedence >= min_precedence {
                            i += 1;
                            while matches!(children.get(i), Some(RedTree::Leaf(leaf)) if leaf.kind == SqlTokenType::Whitespace) {
                                i += 1;
                            }
                            if let Some(RedTree::Node(low_node)) = children.get(i) {
                                let low = self.build_expression_with_precedence(low_node.clone(), source, 6)?;
                                i += 1;
                                while matches!(children.get(i), Some(RedTree::Leaf(leaf)) if leaf.kind == SqlTokenType::Whitespace) {
                                    i += 1;
                                }
                                let has_and = matches!(children.get(i), Some(RedTree::Leaf(leaf)) if leaf.kind == SqlTokenType::And);
                                if has_and {
                                    i += 1;
                                    while matches!(children.get(i), Some(RedTree::Leaf(leaf)) if leaf.kind == SqlTokenType::Whitespace) {
                                        i += 1;
                                    }
                                    if let Some(RedTree::Node(high_node)) = children.get(i) {
                                        let high = Box::new(self.build_expression_with_precedence(high_node.clone(), source, 6)?);
                                        let span = Range { start: left.span().start, end: high.span().end };
                                        left = Expression::Between { expr: Box::new(left), low: Box::new(low), high, negated: negated_operator, span };
                                        negated_operator = false;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    // === IS [NOT] NULL / IS [NOT] TRUE / IS [NOT] FALSE ===
                    SqlTokenType::Is => {
                        let precedence = 5;
                        if precedence >= min_precedence {
                            i += 1;
                            let mut negated = false;
                            while let Some(child) = children.get(i) {
                                match child {
                                    RedTree::Leaf(leaf) if leaf.kind == SqlTokenType::Whitespace => i += 1,
                                    RedTree::Leaf(leaf) if leaf.kind == SqlTokenType::Not => {
                                        negated = true;
                                        i += 1;
                                    }
                                    _ => break,
                                }
                            }
                            let Some(RedTree::Node(n)) = children.get(i)
                            else {
                                return Err(OakError::custom_error("`IS` requires `NULL`, `TRUE`, or `FALSE`"));
                            };
                            let right = self.build_expression_with_precedence(n.clone(), source, 0)?;
                            let span = Range { start: left.span().start, end: right.span().end };
                            let expr = Box::new(left);
                            // The predicate keeps its own node: folding it into a
                            // comparison would turn `NULL IS NULL` (TRUE) into
                            // `NULL = NULL` (NULL) and lose three-valued logic.
                            left = match &right {
                                Expression::Literal(Literal::Null(_)) => Expression::IsNull { expr, negated, span },
                                Expression::Literal(Literal::Boolean(value, _)) => Expression::IsTruth { expr, value: *value, negated, span },
                                // `IS UNKNOWN` is not lowered yet, and folding it into a
                                // comparison would silently change three-valued logic.
                                Expression::Identifier(identifier) if identifier.name.eq_ignore_ascii_case("UNKNOWN") => return Err(OakError::custom_error("`IS UNKNOWN` is not supported")),
                                _ => return Err(OakError::custom_error("`IS` requires `NULL`, `TRUE`, or `FALSE`")),
                            };
                            continue;
                        }
                    }
                    // === IN (list) / IN (subquery) ===
                    SqlTokenType::In => {
                        let precedence = 5;
                        if precedence >= min_precedence {
                            i += 1;
                            while matches!(children.get(i), Some(RedTree::Leaf(leaf)) if leaf.kind == SqlTokenType::Whitespace) {
                                i += 1;
                            }
                            if let Some(RedTree::Node(n)) = children.get(i) {
                                let span = Range { start: left.span().start, end: n.span().end };
                                if n.green.kind == SqlElementType::Subquery {
                                    let Expression::Subquery { query, .. } = self.build_subquery(n.clone(), source)?
                                    else {
                                        return Err(OakError::custom_error("IN expected a subquery"));
                                    };
                                    left = Expression::InSubquery { expr: Box::new(left), query, negated: negated_operator, span };
                                }
                                else {
                                    let mut list = Vec::new();
                                    self.collect_expressions(n.clone(), source, &mut list)?;
                                    left = Expression::InList { expr: Box::new(left), list, negated: negated_operator, span };
                                }
                                negated_operator = false;
                                continue;
                            }
                        }
                    }
                    _ => {
                        if let Some(op) = self.map_binary_op(kind) {
                            let precedence = self.get_precedence(kind);
                            if precedence >= min_precedence {
                                i += 1;
                                // Find the next operand
                                let mut next_operand = None;
                                while i < children.len() {
                                    match &children[i] {
                                        RedTree::Node(n) => {
                                            if n.green.kind == SqlElementType::Expression || n.green.kind == SqlElementType::Identifier {
                                                next_operand = Some(self.build_expression_with_precedence(n.clone(), source, precedence + 1)?);
                                                break;
                                            }
                                        }
                                        RedTree::Leaf(t) => {
                                            if matches!(t.kind, SqlTokenType::NumberLiteral | SqlTokenType::StringLiteral | SqlTokenType::True | SqlTokenType::False | SqlTokenType::Null) {
                                                next_operand = Some(self.build_primary_expression_from_leaf(t, source)?);
                                                break;
                                            }
                                        }
                                    }
                                    i += 1;
                                }

                                if let Some(right) = next_operand {
                                    let span = Range { start: left.span().start, end: right.span().end };
                                    let binary = Expression::Binary { left: Box::new(left), op, right: Box::new(right), span };
                                    left = if negated_operator && op == BinaryOperator::Like {
                                        let span = binary.span().clone();
                                        Expression::Unary { op: UnaryOperator::Not, expr: Box::new(binary), span }
                                    }
                                    else {
                                        binary
                                    };
                                    negated_operator = false;
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
            i += 1;
        }

        Ok(left)
    }

    fn build_primary_expression<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Expression, OakError> {
        // Dispatch on node kind first — FunctionCall nodes have Identifier children
        // that the child loop below would otherwise pick up as the primary.
        if node.green.kind == SqlElementType::FunctionCall {
            return self.build_function_call(node, source);
        }
        // Identifier nodes may span several leaves (`table.column` is merged into one
        // node), so reading the first leaf would silently truncate qualified names.
        if node.green.kind == SqlElementType::Identifier {
            return Ok(Expression::Identifier(self.build_identifier(node, source)?));
        }

        // A LeftParen with multiple comma-separated children is a Vector, a single
        // Expression child is a plain grouped expression.
        let expr_children = node.children().filter(|c| matches!(c, RedTree::Node(n) if n.green.kind == SqlElementType::Expression)).count();
        for child in node.children() {
            match child {
                RedTree::Leaf(t) => {
                    if t.kind == SqlTokenType::LeftBracket {
                        return self.build_vector(node.clone(), source);
                    }
                    if t.kind == SqlTokenType::LeftParen && expr_children > 1 {
                        return self.build_vector(node.clone(), source);
                    }
                    if let Ok(expr) = self.build_primary_expression_from_leaf(&t, source) {
                        return Ok(expr);
                    }
                }
                RedTree::Node(n) => {
                    if n.green.kind == SqlElementType::Expression {
                        return self.build_expression(n, source);
                    }
                    else if n.green.kind == SqlElementType::Identifier {
                        return Ok(Expression::Identifier(self.build_identifier(n, source)?));
                    }
                    else if n.green.kind == SqlElementType::FunctionCall {
                        return self.build_function_call(n, source);
                    }
                    else if n.green.kind == SqlElementType::Subquery {
                        return self.build_subquery(n, source);
                    }
                }
            }
        }
        Err(OakError::custom_error("Failed to build primary expression"))
    }

    fn build_primary_expression_from_leaf(&self, t: &oak_core::RedLeaf<SqlLanguage>, source: &SourceText) -> Result<Expression, OakError> {
        match t.kind {
            SqlTokenType::NumberLiteral | SqlTokenType::FloatLiteral => {
                let text = self.get_text(t.span.clone(), source);
                let trimmed = text.trim();
                let value = if trimmed.len() == text.len() { text } else { Arc::from(trimmed) };
                Ok(Expression::Literal(Literal::Number(value, t.span.clone())))
            }
            SqlTokenType::StringLiteral => {
                let text = self.get_text(t.span.clone(), source);
                let trimmed = text.trim();
                let content = if (trimmed.starts_with('\'') && trimmed.ends_with('\'')) || (trimmed.starts_with('"') && trimmed.ends_with('"')) { if trimmed.len() >= 2 { &trimmed[1..trimmed.len() - 1] } else { "" } } else { trimmed };
                Ok(Expression::Literal(Literal::String(Arc::from(content), t.span.clone())))
            }
            SqlTokenType::True | SqlTokenType::False => Ok(Expression::Literal(Literal::Boolean(t.kind == SqlTokenType::True, t.span.clone()))),
            SqlTokenType::Null => Ok(Expression::Literal(Literal::Null(t.span.clone()))),
            SqlTokenType::Identifier_ => Ok(Expression::Identifier(Identifier { name: self.get_text(t.span.clone(), source), span: t.span.clone() })),
            _ => Err(OakError::custom_error("Not a primary expression leaf")),
        }
    }

    /// Build a `CASE WHEN ... THEN ... [ELSE ...] END` expression.
    ///
    /// The parser packages the whole CASE as a single SqlElementType::Expression
    /// node whose children interleave keyword leaves (CASE/WHEN/THEN/ELSE/END)
    /// with nested expression nodes.
    fn build_case_expression<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Expression, OakError> {
        let full_span = node.span();
        let children: Vec<_> = node.children().collect();
        let mut iter = children.iter().peekable();

        // Skip leading whitespace + the CASE keyword itself.
        while let Some(RedTree::Leaf(l)) = iter.peek() {
            if l.kind == SqlTokenType::Whitespace || l.kind == SqlTokenType::Case {
                iter.next();
            }
            else {
                break;
            }
        }

        let mut branches: Vec<(Box<Expression>, Box<Expression>)> = Vec::new();
        let mut else_branch: Option<Box<Expression>> = None;

        loop {
            // Skip whitespace and locate the first non-ws token.
            while let Some(RedTree::Leaf(l)) = iter.peek() {
                if l.kind == SqlTokenType::Whitespace {
                    iter.next();
                }
                else {
                    break;
                }
            }
            match iter.peek() {
                Some(RedTree::Leaf(l)) if l.kind == SqlTokenType::When => {
                    iter.next(); // consume WHEN
                    let when_expr = self.consume_expression_child(&mut iter, source)?;
                    // Skip whitespace until THEN
                    while let Some(RedTree::Leaf(l)) = iter.peek() {
                        if l.kind == SqlTokenType::Whitespace || l.kind == SqlTokenType::Then {
                            iter.next();
                        }
                        else {
                            break;
                        }
                    }
                    let then_expr = self.consume_expression_child(&mut iter, source)?;
                    branches.push((Box::new(when_expr), Box::new(then_expr)));
                }
                Some(RedTree::Leaf(l)) if l.kind == SqlTokenType::Else => {
                    iter.next(); // consume ELSE
                    let els = self.consume_expression_child(&mut iter, source)?;
                    else_branch = Some(Box::new(els));
                    break;
                }
                Some(RedTree::Leaf(l)) if l.kind == SqlTokenType::End => {
                    iter.next();
                    break;
                }
                _ => break,
            }
        }

        Ok(Expression::Case { branches, else_branch, span: full_span })
    }

    /// Build a `CAST(expr AS type_name)` expression.
    fn build_cast_expression<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Expression, OakError> {
        let full_span = node.span();
        let children: Vec<_> = node.children().collect();
        let mut iter = children.iter().peekable();

        // Skip whitespace + CAST + (
        while let Some(RedTree::Leaf(l)) = iter.peek() {
            if l.kind == SqlTokenType::Whitespace || l.kind == SqlTokenType::Cast || l.kind == SqlTokenType::LeftParen {
                iter.next();
            }
            else {
                break;
            }
        }

        let expr = self.consume_expression_child(&mut iter, source)?;

        // Skip whitespace + AS
        while let Some(RedTree::Leaf(l)) = iter.peek() {
            if l.kind == SqlTokenType::Whitespace || l.kind == SqlTokenType::As {
                iter.next();
            }
            else {
                break;
            }
        }

        // Collect type_name: everything until )
        let mut type_start: Option<usize> = None;
        let mut type_end: Option<usize> = None;
        while let Some(item) = iter.peek() {
            match item {
                RedTree::Leaf(l) if l.kind == SqlTokenType::Whitespace => {
                    iter.next();
                }
                RedTree::Leaf(l) if l.kind == SqlTokenType::RightParen => {
                    iter.next();
                    break;
                }
                RedTree::Leaf(l) => {
                    type_start = Some(type_start.unwrap_or(l.span.start));
                    type_end = Some(l.span.end);
                    iter.next();
                }
                _ => {
                    iter.next();
                }
            }
        }

        let target = match (type_start, type_end) {
            (Some(s), Some(e)) => self.get_text(Range { start: s, end: e }, source),
            _ => Arc::from("TEXT"),
        };

        Ok(Expression::Cast { expr: Box::new(expr), target, span: full_span })
    }

    /// Advances the peekable iterator over a single expression child (either a
    /// nested SqlElementType::Expression node or, as a fallback, builds a leaf
    /// literal/identifier).
    fn consume_expression_child<'a, I>(&self, iter: &mut std::iter::Peekable<I>, source: &SourceText) -> Result<Expression, OakError>
    where
        I: Iterator<Item = &'a RedTree<'a, SqlLanguage>>,
    {
        // Skip whitespace leaves.
        while let Some(RedTree::Leaf(l)) = iter.peek() {
            if l.kind == SqlTokenType::Whitespace {
                iter.next();
            }
            else {
                break;
            }
        }
        let Some(item) = iter.next()
        else {
            return Err(OakError::custom_error("Expected expression in CASE/CAST"));
        };
        match item {
            RedTree::Leaf(l) => {
                // Bare literal / identifier leaf — wrap it directly.
                match l.kind {
                    SqlTokenType::NumberLiteral => Ok(Expression::Literal(Literal::Number(self.get_text(l.span.clone(), source), l.span.clone()))),
                    SqlTokenType::StringLiteral => {
                        let text = self.get_text(l.span.clone(), source);
                        let content = text.trim_matches('\'');
                        Ok(Expression::Literal(Literal::String(Arc::from(content), l.span.clone())))
                    }
                    SqlTokenType::True => Ok(Expression::Literal(Literal::Boolean(true, l.span.clone()))),
                    SqlTokenType::False => Ok(Expression::Literal(Literal::Boolean(false, l.span.clone()))),
                    SqlTokenType::Null => Ok(Expression::Literal(Literal::Null(l.span.clone()))),
                    SqlTokenType::Identifier_ => Ok(Expression::Identifier(Identifier { name: self.get_text(l.span.clone(), source), span: l.span.clone() })),
                    _ => Err(OakError::custom_error(format!("Expected expression in CASE/CAST, got {:?}", l.kind))),
                }
            }
            RedTree::Node(child) => self.build_expression(*child, source),
        }
    }

    fn build_literal<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Literal, OakError> {
        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                match t.kind {
                    SqlTokenType::NumberLiteral => return Ok(Literal::Number(self.get_text(t.span.clone(), source), t.span.clone())),
                    SqlTokenType::StringLiteral => {
                        let text = self.get_text(t.span.clone(), source);
                        let content = text.trim_matches('\'');
                        return Ok(Literal::String(Arc::from(content), t.span.clone()));
                    }
                    SqlTokenType::True => return Ok(Literal::Boolean(true, t.span.clone())),
                    SqlTokenType::False => return Ok(Literal::Boolean(false, t.span.clone())),
                    SqlTokenType::Null => return Ok(Literal::Null(t.span.clone())),
                    _ => {}
                }
            }
        }
        Err(OakError::custom_error("Missing literal content"))
    }

    /// Maps a token kind to its operator precedence level.
    fn get_precedence(&self, kind: SqlTokenType) -> u8 {
        match kind {
            SqlTokenType::Or => 1,
            SqlTokenType::And => 2,
            SqlTokenType::Equal | SqlTokenType::NotEqual | SqlTokenType::Less | SqlTokenType::Greater | SqlTokenType::LessEqual | SqlTokenType::GreaterEqual | SqlTokenType::Like => 4,
            SqlTokenType::Between | SqlTokenType::In | SqlTokenType::Is => 5,
            SqlTokenType::Concat => 6,
            SqlTokenType::Plus | SqlTokenType::Minus => 10,
            SqlTokenType::Star | SqlTokenType::Slash | SqlTokenType::Percent => 11,
            _ => 0,
        }
    }

    fn build_function_call<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Expression, OakError> {
        let mut name = None;
        let mut args = Vec::new();
        let mut distinct = false;
        let mut window = None;

        for child in node.children() {
            match child {
                RedTree::Node(n) => {
                    if n.green.kind == SqlElementType::Identifier {
                        // The first Identifier child is the function name. Later ones
                        // come from bare identifier arguments such as `COUNT(age)`.
                        if name.is_none() {
                            name = Some(self.build_identifier(n, source)?);
                        }
                        else {
                            args.push(Expression::Identifier(self.build_identifier(n, source)?));
                        }
                    }
                    else if n.green.kind == SqlElementType::Expression {
                        args.push(self.build_expression(n, source)?);
                    }
                    // A nested call such as `SUM(LENGTH(email))` is its own
                    // `FunctionCall` child, so it must be lowered as an argument
                    // instead of being skipped. Dropping it would leave the outer
                    // call with zero arguments.
                    else if n.green.kind == SqlElementType::FunctionCall {
                        args.push(self.build_function_call(n, source)?);
                    }
                    else if n.green.kind == SqlElementType::WindowSpec {
                        window = Some(self.build_window_spec(n, source)?);
                    }
                }
                // `COUNT(*)` keeps the wildcard as a bare Star leaf. Dropping it
                // would turn the call into `COUNT()` on the way back to source.
                RedTree::Leaf(t) if t.kind == SqlTokenType::Star => {
                    args.push(Expression::Wildcard { span: t.span.clone() });
                }
                // `DISTINCT` quantifier consumed by the FunctionCall parser
                // (otherwise it would be rejected as a stray keyword). Mark the
                // call so downstream binders can project it onto aggregates.
                RedTree::Leaf(t) if t.kind == SqlTokenType::Distinct => {
                    distinct = true;
                }
                _ => {}
            }
        }

        Ok(Expression::FunctionCall { name: name.ok_or_else(|| OakError::custom_error("Missing function name"))?, args, distinct, window, span: node.span() })
    }

    /// Lowers an `OVER (...)` node into a [`WindowSpec`].
    ///
    /// `PARTITION BY` and `ORDER BY` accept only column references, a window must
    /// name at least one clause, and the frame is limited to the forms the AST
    /// models. Rejecting here keeps unsupported windows out of an analyzer as
    /// valid calls.
    fn build_window_spec<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<WindowSpec, OakError> {
        let mut partition_by: Vec<Identifier> = Vec::new();
        let mut order_by: Vec<WindowOrderItem> = Vec::new();
        let mut frame = None;
        let mut partition_clauses = 0usize;

        for child in node.children() {
            let RedTree::Node(n) = child
            else {
                continue;
            };
            match n.green.kind {
                SqlElementType::WindowPartition => {
                    partition_clauses += 1;
                    for item in n.children() {
                        let RedTree::Node(column) = item
                        else {
                            continue;
                        };
                        partition_by.push(self.build_window_column(column, source, "`PARTITION BY`")?);
                    }
                }
                SqlElementType::WindowOrderBy => order_by = self.build_window_order_by(n, source)?,
                SqlElementType::WindowFrame => frame = Some(self.build_window_frame(n, source)?),
                _ => {}
            }
        }

        if partition_clauses > 1 {
            return Err(OakError::custom_error("Duplicate `PARTITION BY` in window specification"));
        }
        if partition_by.is_empty() && order_by.is_empty() && frame.is_none() {
            return Err(OakError::custom_error("`OVER` requires `PARTITION BY`, `ORDER BY`, or a window frame"));
        }

        Ok(WindowSpec { partition_by, order_by, frame, span: node.span() })
    }

    /// Builds one window column, rejecting anything that is not a column reference.
    fn build_window_column<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText, clause: &str) -> Result<Identifier, OakError> {
        match self.build_expression(node.clone(), source)? {
            Expression::Identifier(identifier) => Ok(identifier),
            _ => Err(OakError::custom_error(format!("{clause} in a window accepts only column references"))),
        }
    }

    /// Builds the `ORDER BY` items of a window specification.
    fn build_window_order_by<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Vec<WindowOrderItem>, OakError> {
        let mut items: Vec<WindowOrderItem> = Vec::new();
        let mut current: Option<WindowOrderItem> = None;

        for child in node.children() {
            match child {
                RedTree::Node(n) => {
                    if matches!(n.green.kind, SqlElementType::Expression | SqlElementType::Identifier) {
                        if let Some(item) = current.take() {
                            items.push(item);
                        }
                        let column = self.build_window_column(n.clone(), source, "`ORDER BY`")?;
                        current = Some(WindowOrderItem { column, direction: OrderDirection::Asc, nulls: None, span: n.span() });
                    }
                }
                RedTree::Leaf(t) => match t.kind {
                    SqlTokenType::Asc => {
                        if let Some(item) = current.as_mut() {
                            item.direction = OrderDirection::Asc;
                            item.span.end = t.span.end;
                        }
                    }
                    SqlTokenType::Desc => {
                        if let Some(item) = current.as_mut() {
                            item.direction = OrderDirection::Desc;
                            item.span.end = t.span.end;
                        }
                    }
                    SqlTokenType::First => {
                        if let Some(item) = current.as_mut() {
                            item.nulls = Some(NullsOrder::First);
                            item.span.end = t.span.end;
                        }
                    }
                    SqlTokenType::Last => {
                        if let Some(item) = current.as_mut() {
                            item.nulls = Some(NullsOrder::Last);
                            item.span.end = t.span.end;
                        }
                    }
                    SqlTokenType::Comma => {
                        if let Some(item) = current.take() {
                            items.push(item);
                        }
                    }
                    _ => {}
                },
            }
        }

        if let Some(item) = current {
            items.push(item);
        }
        if items.is_empty() {
            return Err(OakError::custom_error("`ORDER BY` in a window requires at least one column"));
        }
        Ok(items)
    }

    /// Builds a `ROWS` / `RANGE` frame from its lowered leaves.
    fn build_window_frame<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<WindowFrame, OakError> {
        let mut leaves: Vec<(SqlTokenType, String)> = Vec::new();
        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                // Trivia stays a leaf child of the frame node, so whitespace must be
                // skipped or every bound token is read at the wrong index.
                if t.kind.is_ignored() {
                    continue;
                }
                leaves.push((t.kind, self.get_text(t.span.clone(), source).to_ascii_uppercase()));
            }
        }

        let unit = match leaves.first().map(|(_, text)| text.as_str()) {
            Some("ROWS") => WindowFrameUnit::Rows,
            Some("RANGE") => WindowFrameUnit::Range,
            _ => return Err(OakError::custom_error("A window frame must start with `ROWS` or `RANGE`")),
        };

        let mut index = 1usize;
        let (start, end) = if leaves.get(index).is_some_and(|(_, text)| text == "BETWEEN") {
            index += 1;
            let start = window_frame_bound(&leaves, &mut index)?;
            if !leaves.get(index).is_some_and(|(_, text)| text == "AND") {
                return Err(OakError::custom_error("A window frame `BETWEEN` requires `AND`"));
            }
            index += 1;
            let end = window_frame_bound(&leaves, &mut index)?;
            (start, Some(end))
        }
        else {
            (window_frame_bound(&leaves, &mut index)?, None)
        };

        if index != leaves.len() {
            return Err(OakError::custom_error("Unsupported window frame"));
        }
        if matches!(start, WindowFrameBound::UnboundedFollowing) {
            return Err(OakError::custom_error("`UNBOUNDED FOLLOWING` cannot start a window frame"));
        }
        if let Some(end) = &end {
            if start.is_after(end) {
                return Err(OakError::custom_error("A window frame cannot start after its end bound"));
            }
        }

        Ok(WindowFrame { unit, start, end, span: node.span() })
    }

    /// Builds an `[NOT] EXISTS (SELECT ...)` predicate.
    ///
    /// The parser keeps the parenthesized query as a `Subquery` child, so the
    /// predicate reuses the subquery lowering and only tags the node as `Exists`.
    fn build_exists_expression<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Expression, OakError> {
        let span = node.span();
        for child in node.children() {
            if let RedTree::Node(n) = child {
                if n.green.kind == SqlElementType::Subquery {
                    let Expression::Subquery { query, .. } = self.build_subquery(n, source)?
                    else {
                        return Err(OakError::custom_error("EXISTS expected a subquery"));
                    };
                    return Ok(Expression::Exists { query, negated: false, span });
                }
            }
        }
        Err(OakError::custom_error("EXISTS expected a subquery"))
    }

    fn build_subquery<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Expression, OakError> {
        for child in node.children() {
            if let RedTree::Node(n) = child {
                if n.green.kind == SqlElementType::SelectStatement {
                    return Ok(Expression::Subquery { query: Box::new(self.build_select_statement(n, source)?), span: node.span() });
                }
            }
        }
        Err(OakError::custom_error("Missing select statement in subquery"))
    }

    fn build_vector<'a>(&self, node: RedNode<'a, SqlLanguage>, source: &SourceText) -> Result<Expression, OakError> {
        let mut elements = Vec::new();
        for child in node.children() {
            if let RedTree::Node(n) = child {
                if n.green.kind == SqlElementType::Expression {
                    elements.push(self.build_expression(n, source)?);
                }
            }
        }
        Ok(Expression::Vector { elements, span: node.span() })
    }
}

/// Reads one bound of a window frame from its upper-cased frame leaves.
///
/// The parser already validated the token sequence, so a mismatch here means the
/// frame is not one of the supported forms rather than a syntax error.
fn window_frame_bound(leaves: &[(SqlTokenType, String)], index: &mut usize) -> Result<WindowFrameBound, OakError> {
    let Some((kind, text)) = leaves.get(*index)
    else {
        return Err(OakError::custom_error("Missing window frame bound"));
    };
    match text.as_str() {
        "UNBOUNDED" => {
            *index += 1;
            match leaves.get(*index).map(|(_, text)| text.as_str()) {
                Some("PRECEDING") => {
                    *index += 1;
                    Ok(WindowFrameBound::UnboundedPreceding)
                }
                Some("FOLLOWING") => {
                    *index += 1;
                    Ok(WindowFrameBound::UnboundedFollowing)
                }
                _ => Err(OakError::custom_error("`UNBOUNDED` must be followed by `PRECEDING` or `FOLLOWING`")),
            }
        }
        "CURRENT" => {
            *index += 1;
            if leaves.get(*index).map(|(kind, _)| *kind) != Some(SqlTokenType::Row) {
                return Err(OakError::custom_error("`CURRENT` must be followed by `ROW`"));
            }
            *index += 1;
            Ok(WindowFrameBound::CurrentRow)
        }
        _ if *kind == SqlTokenType::NumberLiteral => {
            let offset = text.parse::<u64>().map_err(|_| OakError::custom_error("A window frame offset must be a non-negative integer"))?;
            *index += 1;
            match leaves.get(*index).map(|(_, text)| text.as_str()) {
                Some("PRECEDING") => {
                    *index += 1;
                    Ok(WindowFrameBound::Preceding(offset))
                }
                Some("FOLLOWING") => {
                    *index += 1;
                    Ok(WindowFrameBound::Following(offset))
                }
                _ => Err(OakError::custom_error("A window frame offset must be followed by `PRECEDING` or `FOLLOWING`")),
            }
        }
        _ => Err(OakError::custom_error("Unsupported window frame bound")),
    }
}
