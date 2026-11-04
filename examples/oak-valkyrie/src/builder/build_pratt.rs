use crate::{
    ValkyrieLanguage,
    ast::{term_nodes::TermBinaryNode as Binary, *},
    builder::ValkyrieBuilder,
    lexer::token_type::ValkyrieTokenType,
};
use oak_core::{OakError, RedNode, RedTree, Source};

impl<'config> ValkyrieBuilder<'config> {
    /// 使用 Pratt 解析器构建表达式
    /// 这是一个更高效的表达式解析方法，特别适合处理运算符优先级
    pub(crate) fn build_expr_pratt<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        // 对于简单的表达式，直接使用现有的构建方法
        // 对于复杂的二元表达式，使用 Pratt 解析器
        match node.green.kind {
            crate::parser::element_type::ValkyrieElementType::BinaryExpression => self.build_binary_pratt(node, source),
            _ => {
                // 其他类型的表达式仍然使用原有的构建方法
                self.build_expr(node, source)
            }
        }
    }

    /// 使用 Pratt 解析器构建二元表达式
    fn build_binary_pratt<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();

        // 收集所有的操作数和运算符
        let mut operands = Vec::new();
        let mut operators = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(t) => match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    _ => {
                        // 这是一个运算符
                        operators.push((t.kind, t.span));
                    }
                },
                RedTree::Node(n) => match n.green.kind {
                    crate::parser::element_type::ValkyrieElementType::Whitespace
                    | crate::parser::element_type::ValkyrieElementType::Newline
                    | crate::parser::element_type::ValkyrieElementType::LineComment
                    | crate::parser::element_type::ValkyrieElementType::BlockComment => continue,
                    _ => {
                        // 这是一个操作数
                        operands.push(self.build_expr(n, source)?);
                    }
                },
            }
        }

        // 确保操作数和运算符的数量匹配
        if operands.len() != operators.len() + 1 {
            return Err(source.syntax_error("Invalid binary expression".to_string(), span.start));
        }

        // 使用 Pratt 解析器算法构建表达式树
        self.pratt_parse::<S>(operands, operators, span)
    }

    /// Pratt 解析器的核心算法
    fn pratt_parse<S: Source + ?Sized>(&self, operands: Vec<TermExpression>, operators: Vec<(ValkyrieTokenType, oak_core::Range<usize>)>, span: oak_core::Range<usize>) -> Result<TermExpression, OakError> {
        // 运算符优先级表
        let precedence = |op: &ValkyrieTokenType| -> u8 {
            match op {
                ValkyrieTokenType::Star | ValkyrieTokenType::Slash | ValkyrieTokenType::Percent => 10,
                ValkyrieTokenType::Plus | ValkyrieTokenType::Minus => 9,
                ValkyrieTokenType::LessThan | ValkyrieTokenType::GreaterThan | ValkyrieTokenType::LessEq | ValkyrieTokenType::GreaterEq | ValkyrieTokenType::LessEqGreater => 8,
                ValkyrieTokenType::EqEq | ValkyrieTokenType::NotEq => 7,
                ValkyrieTokenType::AndAnd => 6,
                ValkyrieTokenType::OrOr => 5,
                _ => 0,
            }
        };

        // 左结合性
        let left_associative = |op: &ValkyrieTokenType| -> bool {
            match op {
                ValkyrieTokenType::Star
                | ValkyrieTokenType::Slash
                | ValkyrieTokenType::Percent
                | ValkyrieTokenType::Plus
                | ValkyrieTokenType::Minus
                | ValkyrieTokenType::LessThan
                | ValkyrieTokenType::GreaterThan
                | ValkyrieTokenType::LessEq
                | ValkyrieTokenType::GreaterEq
                | ValkyrieTokenType::LessEqGreater
                | ValkyrieTokenType::EqEq
                | ValkyrieTokenType::NotEq
                | ValkyrieTokenType::AndAnd
                | ValkyrieTokenType::OrOr => true,
                _ => false,
            }
        };

        // 实现 Pratt 解析器算法
        fn parse_expression(
            operands: &[TermExpression],
            operators: &[(ValkyrieTokenType, oak_core::Range<usize>)],
            precedence: &dyn Fn(&ValkyrieTokenType) -> u8,
            left_associative: &dyn Fn(&ValkyrieTokenType) -> bool,
            min_prec: u8,
        ) -> (TermExpression, usize) {
            let mut i = 0;
            let mut expr = operands[i].clone();
            i += 1;

            while i <= operators.len() {
                if i == operators.len() {
                    break;
                }

                let (op, op_span) = &operators[i - 1];
                let prec = precedence(op);

                if prec < min_prec {
                    break;
                }

                let next_min_prec = if left_associative(op) { prec + 1 } else { prec };
                let (right_expr, consumed) = parse_expression(&operands[i..], &operators[i..], precedence, left_associative, next_min_prec);

                // 创建二元表达式
                let left_span = expr.span();
                let right_span = right_expr.span();
                expr = TermExpression::Binary(Box::new(Binary { lhs: expr, operator: op.clone(), rhs: right_expr, span: oak_core::Range { start: left_span.start, end: right_span.end } }));

                i += consumed + 1;
            }

            (expr, i - 1)
        }

        // 开始解析
        let (result, _) = parse_expression(&operands, &operators, &precedence, &left_associative, 0);
        Ok(result)
    }
}
