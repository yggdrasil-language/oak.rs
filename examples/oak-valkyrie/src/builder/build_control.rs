use crate::{
    ValkyrieLanguage,
    ast::{
        pattern_nodes::{ArrayPattern, ClassPattern, ElsePattern, LiteralPattern, TypePattern, VariablePattern, WildcardPattern},
        term_nodes::{Break, Continue, Raise, Resume, Return},
        *,
    },
    builder::ValkyrieBuilder,
    lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType},
};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_if<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut pattern = None;
        let mut condition = None;
        let mut then_branch = None;
        let mut else_branch = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::Pattern => {
                        pattern = Some(self.build_pattern(n, source)?);
                    }
                    crate::parser::element_type::ValkyrieElementType::BlockExpression => {
                        if then_branch.is_none() {
                            then_branch = Some(self.build_block(n, source)?);
                        }
                        else {
                            else_branch = Some(self.build_block(n, source)?);
                        }
                    }
                    // `else if` 在 CST 里是嵌套的 `IfExpression` 子节点（parser 的
                    // `parse_if_expression` 在 `else` 后直接递归解析 if）。它不是
                    // `BlockExpression`，若落进 `_` 分支会被当作第二个条件丢弃，
                    // 外层 if 因此得到 `else_branch: None`，`else if` 整条链在
                    // HIR 之前就消失了（表现为语义缺失/循环回边读到旧值而死循环）。
                    // 这里把它包成只含一个尾表达式语句的块，恢复 `else { if … }`
                    // 的等价 AST 形状，交由 HIR 正常降低。
                    crate::parser::element_type::ValkyrieElementType::IfExpression => {
                        let nested_span = n.span();
                        let nested = self.build_if(n, source)?;
                        else_branch = Some(Block { statements: vec![Statement::Expression(Box::new(ExprStmt { expr: nested, semi: false, annotations: Vec::new(), span: nested_span }))], span: nested_span });
                    }
                    _ => {
                        if condition.is_none() {
                            condition = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
            }
        }

        let condition = condition.ok_or_else(|| source.syntax_error("Missing if condition".to_string(), span.start))?;
        let then_branch = then_branch.ok_or_else(|| source.syntax_error("Missing if then branch".to_string(), span.start))?;

        Ok(TermExpression::If { pattern, condition, then_branch, else_branch, span })
    }

    pub(crate) fn build_match<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut scrutinee = None;
        let mut arms = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::MatchArm => {
                        if let Some(arm) = self.recover_member(self.build_match_arm(n, source)) {
                            arms.push(arm);
                        }
                    }
                    _ => {
                        if scrutinee.is_none() {
                            scrutinee = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
            }
        }

        let scrutinee = scrutinee.ok_or_else(|| source.syntax_error("Missing match scrutinee".to_string(), span.start))?;

        Ok(TermExpression::Match { scrutinee, arms, span })
    }

    pub(crate) fn build_match_arm<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<MatchArm, OakError> {
        let span = node.span();
        let mut pattern = None;
        let mut else_span = None;
        let mut expressions = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Else) => else_span = Some(t.span.clone()),
                    // `default` 臂与 `else` 同形：标识符叶节点文本判定。
                    ValkyrieTokenType::Identifier if crate::builder::text(source, t.span.clone()) == "default" => else_span = Some(t.span.clone()),
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::Pattern => {
                        pattern = Some(self.build_pattern(n, source)?);
                    }
                    _ => expressions.push(self.build_expr(n, source)?),
                },
            }
        }

        // `else` / `default` 臂不产 Pattern 子节点（只有关键字叶），按 AST 既有
        // `Pattern::Else` 合成；缺关键字又缺模式才报缺模式。
        let pattern = match pattern {
            Some(pattern) => pattern,
            None => match else_span {
                Some(span) => Pattern::Else(Box::new(ElsePattern { span })),
                None => return Err(source.syntax_error("Missing match arm pattern".to_string(), span.start)),
            },
        };
        let (guard, body) = match expressions.len() {
            0 => {
                return Err(source.syntax_error("Missing match arm body".to_string(), span.start));
            }
            1 => (None, expressions.remove(0)),
            2 => (Some(expressions.remove(0)), expressions.remove(0)),
            _ => {
                return Err(source.syntax_error("Match arm has too many expressions".to_string(), span.start));
            }
        };

        Ok(MatchArm { pattern, guard, body, span })
    }

    pub(crate) fn build_loop<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut kind = LoopKind::default();
        let mut label = None;
        let mut pattern = None;
        let mut condition = None;
        let mut body = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Label => {
                        label = Some(crate::builder::text(source, t.span));
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::For) => {
                        kind = LoopKind::For;
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Loop) => {
                        kind = LoopKind::Loop;
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::Pattern => {
                        pattern = Some(self.build_pattern(n, source)?);
                    }
                    crate::parser::element_type::ValkyrieElementType::BlockExpression => {
                        body = Some(self.build_block(n, source)?);
                    }
                    _ => {
                        if condition.is_none() {
                            condition = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
            }
        }

        let body = body.ok_or_else(|| source.syntax_error("Missing loop body".to_string(), span.start))?;

        Ok(TermExpression::Loop { kind, label, pattern, condition, body, span })
    }

    pub(crate) fn build_return<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut expr = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    _ => {
                        if expr.is_none() {
                            expr = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
            }
        }

        Ok(TermExpression::Return(Box::new(Return { base: expr.map(|e| *e), span })))
    }

    pub(crate) fn build_break<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut label = None;
        let mut expr = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Label => {
                        label = Some(crate::builder::text(source, t.span));
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    _ => {
                        if expr.is_none() {
                            expr = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
            }
        }

        Ok(TermExpression::Break(Box::new(Break { label, base: expr.map(|e| *e), span })))
    }

    pub(crate) fn build_continue<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut label = None;

        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Label => {
                        label = Some(crate::builder::text(source, t.span));
                    }
                    _ => {}
                }
            }
        }

        Ok(TermExpression::Continue(Box::new(Continue { label, span })))
    }

    pub(crate) fn build_yield<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut expr = None;
        let yield_from = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    _ => {
                        if expr.is_none() {
                            expr = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
            }
        }

        Ok(TermExpression::Yield { expr, yield_from, span })
    }

    pub(crate) fn build_raise<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut expr = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    _ => {
                        if expr.is_none() {
                            expr = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
            }
        }

        let expr = expr.ok_or_else(|| source.syntax_error("Missing raise expression".to_string(), span.start))?;

        Ok(TermExpression::Raise(Box::new(Raise { base: Some(*expr), span })))
    }

    pub(crate) fn build_resume<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut expr = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    _ => {
                        if expr.is_none() {
                            expr = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
            }
        }

        let expr = expr.ok_or_else(|| source.syntax_error("Missing resume expression".to_string(), span.start))?;

        Ok(TermExpression::Resume(Box::new(Resume { base: Some(*expr), span })))
    }

    pub(crate) fn build_catch<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut expr = None;
        let mut arms = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::MatchArm => arms.push(self.build_match_arm(n, source)?),
                    _ => {
                        if expr.is_none() {
                            expr = Some(Box::new(self.build_expr(n, source)?));
                        }
                    }
                },
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {}
                },
            }
        }

        let expr = expr.ok_or_else(|| source.syntax_error("Missing catch expression".to_string(), span.start))?;

        Ok(TermExpression::Catch { expr, arms, span })
    }

    pub(crate) fn build_pattern<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Pattern, OakError> {
        let span = node.span();
        let mut name_path: Option<NamePath> = None;
        let mut fields: Option<Vec<(Identifier, Option<Pattern>)>> = None;
        let mut nested_patterns: Vec<Pattern> = Vec::new();
        let mut lone_identifier: Option<(Identifier, Span)> = None;
        // 数组形态：`[prefix, ..., ...rest, suffix]`，子模式落在嵌套 Pattern 节点里，
        // `...` 词法形态是 DotDot + Dot 两个 leaf，以 DotDot 为剩余绑定标记。
        let mut saw_bracket = false;
        let mut rest_pending = false;
        let mut array_prefix: Vec<Pattern> = Vec::new();
        let mut array_rest: Option<Box<Pattern>> = None;
        let mut array_suffix: Vec<Pattern> = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::BracketL => {
                        saw_bracket = true;
                    }
                    ValkyrieTokenType::DotDot => {
                        if saw_bracket && array_rest.is_none() {
                            rest_pending = true;
                        }
                    }
                    ValkyrieTokenType::Underscore => {
                        if name_path.is_none() && fields.is_none() && nested_patterns.is_empty() && lone_identifier.is_none() && !saw_bracket {
                            return Ok(Pattern::Wildcard(Box::new(WildcardPattern { span: t.span })));
                        }
                    }
                    kind if kind.is_soft_identifier() => {
                        if name_path.is_none() && fields.is_none() && nested_patterns.is_empty() && lone_identifier.is_none() && !saw_bracket {
                            let name = crate::builder::text(source, t.span);
                            lone_identifier = Some((Identifier { name, span: t.span }, t.span));
                        }
                    }
                    ValkyrieTokenType::IntegerLiteral | ValkyrieTokenType::FloatLiteral | ValkyrieTokenType::BoolLiteral | ValkyrieTokenType::StringLiteral | ValkyrieTokenType::CharLiteral => {
                        // 字面量可以是完整模式（`case 60`），也可以是构造/数组模式的
                        // 位置子模式（`case Some(60)` / `case [head, 0]`）：前者整臂即
                        // Literal，后者按位置落入嵌套/数组序列，不得丢弃成 Wildcard。
                        let value = crate::builder::text(source, t.span);
                        let literal = Pattern::Literal(Box::new(LiteralPattern { value, span: t.span }));
                        if name_path.is_none() && fields.is_none() && nested_patterns.is_empty() && lone_identifier.is_none() && !saw_bracket {
                            return Ok(literal);
                        }
                        if saw_bracket {
                            if rest_pending {
                                array_rest = Some(Box::new(literal));
                                rest_pending = false;
                            }
                            else if array_rest.is_some() {
                                array_suffix.push(literal);
                            }
                            else {
                                array_prefix.push(literal);
                            }
                        }
                        else {
                            nested_patterns.push(literal);
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::NamePath => {
                        if name_path.is_none() {
                            name_path = Some(self.build_name_path(n, source)?);
                        }
                    }
                    crate::parser::element_type::ValkyrieElementType::BlockExpression => {
                        if name_path.is_some() && fields.is_none() {
                            fields = Some(self.build_pattern_fields(&n, source)?);
                        }
                    }
                    crate::parser::element_type::ValkyrieElementType::Pattern => {
                        let pattern = self.build_pattern(n, source)?;
                        if saw_bracket {
                            if rest_pending {
                                array_rest = Some(Box::new(pattern));
                                rest_pending = false;
                            }
                            else if array_rest.is_some() {
                                array_suffix.push(pattern);
                            }
                            else {
                                array_prefix.push(pattern);
                            }
                        }
                        else {
                            nested_patterns.push(pattern);
                        }
                    }
                    _ => {}
                },
            }
        }

        if saw_bracket {
            return Ok(Pattern::Array(Box::new(ArrayPattern { prefix: array_prefix, rest: array_rest, suffix: array_suffix, span })));
        }

        if let Some(name) = name_path {
            if let Some(fields) = fields {
                return Ok(Pattern::Class(Box::new(ClassPattern { name, fields, span })));
            }
            if !nested_patterns.is_empty() {
                return Ok(Pattern::Class(Box::new(ClassPattern { name, fields: Self::positional_variant_fields(&nested_patterns, span), span })));
            }
            return Ok(Pattern::Type(Box::new(TypePattern { name, span })));
        }

        if let Some((identifier, id_span)) = lone_identifier {
            if !nested_patterns.is_empty() {
                let name = NamePath { parts: vec![identifier], span: id_span };
                return Ok(Pattern::Class(Box::new(ClassPattern { name, fields: Self::positional_variant_fields(&nested_patterns, span), span })));
            }
            if identifier.name.chars().next().is_some_and(|ch| ch.is_uppercase()) {
                return Ok(Pattern::Type(Box::new(TypePattern { name: NamePath { parts: vec![identifier], span: id_span }, span })));
            }
            return Ok(Pattern::Variable(Box::new(VariablePattern { name: identifier, span: id_span })));
        }

        // 无法识别的叶子不再合成通配符：通配符只能来自源码里真实写出的 `_`。
        Err(source.syntax_error(format!("Missing pattern at {span:?}"), span.start))
    }

    fn positional_variant_fields(nested_patterns: &[Pattern], fallback_span: Span) -> Vec<(Identifier, Option<Pattern>)> {
        nested_patterns
            .iter()
            .enumerate()
            .map(|(index, pattern)| match pattern {
                Pattern::Variable(variable) => (variable.name.clone(), None),
                _ => (Identifier { name: format!("_{index}"), span: fallback_span }, Some(pattern.clone())),
            })
            .collect()
    }

    /// Builds pattern fields from a block expression.
    ///
    /// Supports both new syntax (`:` separator) and deprecated syntax (`=` separator).
    /// When the deprecated `=` syntax is detected, a warning is logged.
    fn build_pattern_fields<S: Source + ?Sized>(&self, node: &RedNode<ValkyrieLanguage>, source: &S) -> Result<Vec<(Identifier, Option<Pattern>)>, OakError> {
        let mut fields = Vec::new();

        for child in node.children() {
            if let RedTree::Node(stmt_n) = child {
                match stmt_n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::ExprStatement | crate::parser::element_type::ValkyrieElementType::BinaryExpression => {
                        if let Some(field) = self.extract_pattern_field(&stmt_n, source)? {
                            fields.push(field);
                        }
                    }
                    crate::parser::element_type::ValkyrieElementType::IdentifierExpression => {
                        if let Some(expr) = self.recover_member(self.build_identifier_expr(stmt_n.clone(), source)) {
                            if let TermExpression::NamePath(path) = expr {
                                if let Some(ident) = path.parts.first().cloned() {
                                    fields.push((ident, None));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(fields)
    }

    /// Extracts a pattern field from a binary expression or statement.
    ///
    /// Supports both new syntax (`:` separator) and deprecated syntax (`=` separator).
    fn extract_pattern_field<S: Source + ?Sized>(&self, node: &RedNode<ValkyrieLanguage>, source: &S) -> Result<Option<(Identifier, Option<Pattern>)>, OakError> {
        let mut field_name = None;
        let mut field_pattern = None;
        let mut separator_found = false;
        let mut uses_deprecated_syntax = false;

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::Eq => {
                        if !separator_found {
                            separator_found = true;
                            uses_deprecated_syntax = true;
                        }
                    }
                    ValkyrieTokenType::Colon => {
                        if !separator_found {
                            separator_found = true;
                        }
                    }
                    _ => {}
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    crate::parser::element_type::ValkyrieElementType::IdentifierExpression => {
                        if field_name.is_none() {
                            if let Some(expr) = self.recover_member(self.build_identifier_expr(n.clone(), source)) {
                                if let TermExpression::NamePath(path) = expr {
                                    if let Some(ident) = path.parts.first().cloned() {
                                        field_name = Some(ident);
                                    }
                                }
                            }
                        }
                    }
                    crate::parser::element_type::ValkyrieElementType::Pattern => {
                        if field_name.is_some() && field_pattern.is_none() {
                            field_pattern = Some(self.build_pattern(n, source)?);
                        }
                    }
                    _ => {
                        if field_name.is_some() && field_pattern.is_none() {
                            // 子模式构建失败时保留字段名，只丢子模式，并记录诊断。
                            field_pattern = self.recover_member(self.build_pattern(n, source));
                        }
                    }
                },
            }
        }

        if uses_deprecated_syntax {
            if let Some(ref name) = field_name {
                eprintln!("Warning: Use of deprecated '=' syntax in pattern field at offset {}. Use ':' instead. Field: '{}'", name.span.start, name.name);
            }
        }

        if let Some(name) = field_name { Ok(Some((name, field_pattern))) } else { Ok(None) }
    }
}

#[cfg(test)]
mod soft_identifier_pattern_tests {
    use crate::{
        ValkyrieBuilder, ValkyrieLanguage, ValkyrieRoot,
        ast::{Pattern, StatementNode},
    };
    use oak_core::{Builder, OakDiagnostics, SourceText, parser::ParseSession};

    fn build(source: &str) -> OakDiagnostics<ValkyrieRoot> {
        let language = ValkyrieLanguage::default();
        let builder = ValkyrieBuilder::new(&language);
        let text = SourceText::new(source);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        builder.build(&text, &[], &mut session)
    }

    fn first_let_pattern(root: &ValkyrieRoot) -> &Pattern {
        root.items
            .iter()
            .find_map(|item| match item {
                StatementNode::Let(item) => Some(&item.pattern),
                _ => None,
            })
            .expect("expected a let item")
    }

    /// 正例：软关键字 `by` 在绑定名位置按标识符处理，`let by` 必须产出
    /// `VariablePattern`，不能掉进“不认识的叶子”兜底。
    #[test]
    fn soft_keyword_binding_builds_variable_pattern() {
        let output = build("let by: i32 = 1; by");
        assert!(output.diagnostics.is_empty(), "diagnostics: {:?}", output.diagnostics);
        let root = output.result.unwrap_or_else(|error| panic!("let must build, got {error:?}"));
        assert!(matches!(first_let_pattern(&root), Pattern::Variable(variable) if variable.name.name == "by"));
    }

    /// 正例：`get` / `set` 是普通标识符，绑定名同样走 `VariablePattern`。
    #[test]
    fn accessor_name_binding_builds_variable_pattern() {
        let output = build("let set: i32 = 1; set");
        assert!(output.diagnostics.is_empty(), "diagnostics: {:?}", output.diagnostics);
        let root = output.result.unwrap_or_else(|error| panic!("let must build, got {error:?}"));
        assert!(matches!(first_let_pattern(&root), Pattern::Variable(variable) if variable.name.name == "set"));
    }
}
