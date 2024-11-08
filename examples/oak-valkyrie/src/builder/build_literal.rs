use crate::{
    ValkyrieLanguage,
    ast::*,
    builder::{ValkyrieBuilder, text},
    lexer::{keywords::ValkyrieKeywords, token_type::ValkyrieTokenType},
};
use oak_core::{OakError, RedNode, RedTree, Source};

/// Extracts the raw content of a quote capture by removing both delimiter runs.
fn strip_delimiters(raw: &str, quote_count: usize) -> String {
    let end = raw.len().saturating_sub(quote_count);
    if quote_count >= end { String::new() } else { raw[quote_count..end].to_string() }
}

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_literal<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut prefix: Option<Identifier> = None;
        let mut quote_capture: Option<(QuoteKind, oak_core::Range<usize>)> = None;

        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    // 紧邻的标识符前缀（`r`、`b` 等）只是捕获事实的携带者，语义由编译器解释。
                    ValkyrieTokenType::Identifier => {
                        let prefix_text = text(source, t.span);
                        prefix = Some(Identifier { name: prefix_text, span: t.span });
                    }
                    ValkyrieTokenType::StringLiteral => {
                        quote_capture = Some((QuoteKind::Double, t.span));
                    }
                    ValkyrieTokenType::CharLiteral => {
                        quote_capture = Some((QuoteKind::Single, t.span));
                    }
                    ValkyrieTokenType::IntegerLiteral => {
                        return Ok(TermExpression::IntegerLiteral { value: text(source, t.span), span });
                    }
                    ValkyrieTokenType::FloatLiteral => {
                        return Ok(TermExpression::FloatLiteral { value: text(source, t.span), span });
                    }
                    ValkyrieTokenType::Keyword(ValkyrieKeywords::Null) => {
                        let name = Identifier { name: "null".to_string(), span: t.span };
                        return Ok(TermExpression::NamePath(Box::new(NamePath { parts: vec![name], span: t.span })));
                    }
                    _ => {}
                }
            }
        }

        let Some((quote, token_span)) = quote_capture
        else {
            return Err(source.syntax_error("Missing literal value".to_string(), span.start));
        };

        // 单双引号走同一条捕获路径：只记录定界形状与原始内容，
        // 不区分引号种类做语义分流，也不解释前缀或转义。
        let raw_text = text(source, token_span);
        let quote_char = quote.character();
        let quote_count = raw_text.chars().take_while(|c| *c == quote_char).count();
        let content = strip_delimiters(&raw_text, quote_count);
        let content_span = Span { start: token_span.start + quote_count, end: token_span.end - quote_count };

        Ok(TermExpression::QuoteLiteral(QuoteLiteral { prefix, quote, quote_count: quote_count as u8, content, content_span, span }))
    }

    pub(crate) fn build_bool_literal<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut value = false;
        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                if t.kind == ValkyrieTokenType::BoolLiteral {
                    let text_val = text(source, t.span);
                    value = text_val == "true";
                }
            }
        }
        Ok(TermExpression::BooleanLiteral { value, span })
    }

    pub(crate) fn build_identifier_expr<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let identifier = crate::builder::utils::build_identifier(&node, source, span)?;
        Ok(TermExpression::NamePath(Box::new(NamePath { parts: vec![identifier.clone()], span: identifier.span })))
    }

    pub(crate) fn build_path_expr<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let path = crate::builder::utils::build_name_path(&node, source)?;
        Ok(TermExpression::NamePath(Box::new(path)))
    }

    pub(crate) fn build_name_path<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<NamePath, OakError> {
        crate::builder::utils::build_name_path(&node, source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ValkyrieBuilder, ValkyrieLanguage};
    use oak_core::{Builder, SourceText, parser::ParseSession};

    fn build(source: &str) -> oak_core::OakDiagnostics<ValkyrieRoot> {
        let language = ValkyrieLanguage::default();
        let builder = ValkyrieBuilder::new(&language);
        let text = SourceText::new(source);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        builder.build(&text, &[], &mut session)
    }

    fn literal_of(source: &str) -> QuoteLiteral {
        let output = build(source);
        let root = output.result.expect("source builds");
        let StatementNode::Let(let_node) = &root.items[0]
        else {
            panic!("let statement expected")
        };
        match let_node.expr.as_ref().expect("initializer") {
            TermExpression::QuoteLiteral(literal) => literal.clone(),
            other => panic!("quote literal expected, got {other:?}"),
        }
    }

    /// 单双引号产出同一节点形状：只有定界种类不同，内容一律未解码。
    #[test]
    fn both_quote_kinds_produce_the_unified_capture_node() {
        let double = literal_of("let a = \"x\\ny\"");
        assert_eq!(double.quote, QuoteKind::Double);
        assert_eq!(double.quote_count, 1);
        assert_eq!(double.content, "x\\ny");
        assert_eq!(double.content_span, Span { start: 9, end: 13 });

        let single = literal_of("let a = 'x\\ny'");
        assert_eq!(single.quote, QuoteKind::Single);
        assert_eq!(single.quote_count, 1);
        assert_eq!(single.content, "x\\ny");
    }

    /// 反斜杠不救定界符：`"\""` 在第二个引号处闭合，悬挂引号是结构化错误。
    #[test]
    fn backslash_does_not_protect_the_delimiter() {
        let output = build("let a = \"\\\"\"");
        assert!(!output.diagnostics.is_empty(), "dangling quote must produce a structured error");
    }

    /// 前缀只是携带者：builder 不识别 `r`，不解析模板。
    #[test]
    fn prefix_is_carried_without_semantics() {
        let raw = literal_of("let a = r\"{x}\"");
        assert_eq!(raw.prefix.as_ref().expect("prefix").name, "r");
        assert_eq!(raw.content, "{x}");
    }

    /// 多引号只改变定界形状，捕获后内容与单引号同形。
    #[test]
    fn multi_quote_run_only_changes_the_delimiter() {
        let triple = literal_of("let a = \"\"\"x\"\"\"");
        assert_eq!(triple.quote_count, 3);
        assert_eq!(triple.content, "x");

        let empty = literal_of("let a = \"\"");
        assert_eq!(empty.quote_count, 2);
        assert_eq!(empty.content, "");
    }
}
