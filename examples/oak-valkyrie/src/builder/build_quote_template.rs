//! 引号捕获区间的有边界模板语法解析。
//!
//! 这是 Oak 的语法能力，由消费方在需要解释模板时显式请求：
//!
//! - 文本段保留原始字节，转义候选（`\{`、`\}`）只用于保护结构边界，
//!   不在此处解码任何转义——`\n` 在这里仍然是反斜杠加字母 n。
//! - 插值区间用真实词法器定界（注释、内层引号捕获与嵌套括号按 token
//!   结构处理，不是数花括号），再用普通表达式解析器生成真实表达式树，
//!   源区间为文件绝对偏移。
//! - 空表达式、缺结束括号、裸结束括号、表达式残余 token 一律结构化报错，
//!   不生成半成品成功 AST。

use crate::{
    ValkyrieLanguage, ValkyrieParser,
    ast::{InterpolationSegment, StatementNode, StringSegment, TermExpression, TextSegment},
    builder::ValkyrieBuilder,
    lexer::{ValkyrieLexer, token_type::ValkyrieTokenType},
};
use oak_core::{Lexer, NoLexerCache, OakError, Parser, SourceText, parser::session::ParseSession};

/// Locale 参数标记字符 (Gwot, U+07DF)
const LOCALE_MARKER: char = '\u{07DF}';

/// 以空格填充到绝对偏移后解析：token 与 AST 区间天然保持文件绝对位置。
fn padded_source(text: &str, absolute_start: usize) -> SourceText {
    let mut padded = String::with_capacity(absolute_start + text.len());
    for _ in 0..absolute_start {
        padded.push(' ');
    }
    padded.push_str(text);
    SourceText::new(padded)
}

impl<'config> ValkyrieBuilder<'config> {
    /// 对已捕获的引号字面量区间做结构分段。
    ///
    /// `content` 是两侧定界串之间的原始文本（未解码），`content_start` 是其
    /// 首字节在文件中的绝对偏移。返回的文本段内容与源码逐字节一致；插值段
    /// 携带真实表达式树与绝对区间。本函数不解释前缀：`r` 等前缀是否进入
    /// 模板模式由消费方决定。
    pub fn parse_quote_template(&self, content: &str, content_start: usize) -> Result<Vec<StringSegment>, OakError> {
        let mut segments = Vec::new();
        let mut text_start = 0usize;
        let mut index = 0usize;

        while index < content.len() {
            let character = content[index..].chars().next().expect("index sits on a char boundary");
            match character {
                '\\' => {
                    // 转义候选：反斜杠与后继字符成对消费，后继字符因此不能开启结构；
                    // 两个字节都原样留在文本段里，解码属于消费方。
                    let next_len = content[index + 1..].chars().next().map_or(0, |next| next.len_utf8());
                    index += 1 + next_len;
                }
                '{' => {
                    if index > text_start {
                        segments.push(StringSegment::Text(Box::new(TextSegment { content: content[text_start..index].to_string(), span: crate::ast::Span { start: content_start + text_start, end: content_start + index } })));
                    }
                    let after_brace = index + 1;
                    let is_locale = content[after_brace..].starts_with(LOCALE_MARKER);
                    let expr_start = if is_locale { after_brace + LOCALE_MARKER.len_utf8() } else { after_brace };
                    let expr_end = self.scan_interpolation_end(content, content_start, expr_start)?;
                    let expr = self.parse_interpolation_expression(&content[expr_start..expr_end], content_start + expr_start)?;
                    segments.push(StringSegment::Interpolation(Box::new(InterpolationSegment { expr, is_locale, span: crate::ast::Span { start: content_start + index, end: content_start + expr_end + 1 } })));
                    index = expr_end + 1;
                    text_start = index;
                }
                '}' => {
                    return Err(OakError::syntax_error("bare `}` in quoted template: write `\\}` for a literal closing brace".to_string(), content_start + index, None));
                }
                _ => index += character.len_utf8(),
            }
        }

        if text_start < content.len() {
            segments.push(StringSegment::Text(Box::new(TextSegment { content: content[text_start..].to_string(), span: crate::ast::Span { start: content_start + text_start, end: content_start + content.len() } })));
        }

        Ok(segments)
    }

    /// 用真实词法器为插值区间定界：从表达式起点扫描到深度归零的 `}`。
    ///
    /// 内层字符串、注释与嵌套括号都是单个 token 或成对深度，不数花括号。
    /// 返回匹配 `}` 在 `content` 中的相对字节偏移（不含 `}` 本身）。
    fn scan_interpolation_end(&self, content: &str, content_start: usize, expr_start: usize) -> Result<usize, OakError> {
        let absolute_start = content_start + expr_start;
        let source = padded_source(&content[expr_start..], absolute_start);
        let lexer = ValkyrieLexer::new(self.config);
        let mut cache = NoLexerCache;
        let output = lexer.lex(&source, &[], &mut cache);
        let tokens = output.result?;

        let mut depth = 1usize;
        for token in tokens.iter() {
            if token.span.end <= absolute_start {
                continue;
            }
            match token.kind {
                ValkyrieTokenType::BraceL | ValkyrieTokenType::ParenthesisL | ValkyrieTokenType::BracketL => depth += 1,
                ValkyrieTokenType::BraceR => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(token.span.start - content_start);
                    }
                }
                ValkyrieTokenType::ParenthesisR | ValkyrieTokenType::BracketR => {
                    depth -= 1;
                    if depth == 0 {
                        return Err(OakError::syntax_error(format!("unbalanced closing `{:?}` in interpolation expression", token.kind), token.span.start, None));
                    }
                }
                ValkyrieTokenType::Error => {
                    return Err(OakError::syntax_error("illegal token inside interpolation expression", token.span.start, None));
                }
                _ => {}
            }
        }

        Err(OakError::syntax_error("missing closing `}` for interpolation expression", absolute_start, None))
    }

    /// 用普通表达式解析器解析插值区间，产出绝对区间的真实表达式树。
    fn parse_interpolation_expression(&self, expr_text: &str, absolute_start: usize) -> Result<TermExpression, OakError> {
        if expr_text.trim().is_empty() {
            return Err(OakError::syntax_error("empty interpolation expression", absolute_start, None));
        }
        let source = padded_source(expr_text, absolute_start);
        let parser = ValkyrieParser::new(self.config);
        let mut session = ParseSession::<ValkyrieLanguage>::default();
        let parse_output = parser.parse(&source, &[], &mut session);
        let green_tree = match parse_output.result {
            Ok(green_tree) => green_tree,
            Err(error) => return Err(error),
        };
        if let Some(error) = parse_output.diagnostics.into_iter().next() {
            return Err(error);
        }
        let root = self.build_root(green_tree, &source)?;
        let pending = std::mem::take(&mut *self.recoverable.borrow_mut());
        if let Some(error) = pending.into_iter().next() {
            return Err(error);
        }
        match root.items.as_slice() {
            [StatementNode::ExprStmt(stmt)] => Ok(stmt.expr.clone()),
            [] => Err(OakError::syntax_error("interpolation expression did not parse to a term expression", absolute_start, None)),
            _ => Err(OakError::syntax_error("residual tokens after interpolation expression", absolute_start, None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ValkyrieLanguage,
        ast::{QuoteKind, TermExpression},
        builder::ValkyrieBuilder,
    };

    fn segments(content: &str, start: usize) -> Result<Vec<StringSegment>, OakError> {
        let language = ValkyrieLanguage::default();
        let builder = ValkyrieBuilder::new(&language);
        builder.parse_quote_template(content, start)
    }

    /// 插值必须是真实表达式树：`slot + 1` 成 Binary，区间为绝对偏移。
    #[test]
    fn interpolation_is_a_real_expression_tree_with_absolute_spans() {
        let segs = segments("x{slot + 1}y", 10).expect("template parses");
        assert_eq!(segs.len(), 3);
        match (&segs[0], &segs[2]) {
            (StringSegment::Text(open), StringSegment::Text(close)) => {
                assert_eq!((open.content.as_str(), close.content.as_str()), ("x", "y"));
                assert_eq!((open.span.start, open.span.end), (10, 11));
            }
            _ => panic!("text segments expected around the interpolation"),
        }
        match &segs[1] {
            StringSegment::Interpolation(interp) => {
                assert_eq!((interp.span.start, interp.span.end), (11, 21));
                match &interp.expr {
                    TermExpression::Binary(node) => {
                        assert_eq!((node.span.start, node.span.end), (12, 20));
                        match &node.lhs {
                            TermExpression::NamePath(path) => {
                                assert_eq!(path.parts[0].name, "slot");
                                assert_eq!((path.span.start, path.span.end), (12, 16));
                            }
                            other => panic!("lhs should be a name path, got {other:?}"),
                        }
                    }
                    other => panic!("interpolation should be a binary expression, got {other:?}"),
                }
            }
            _ => panic!("interpolation segment expected"),
        }
    }

    /// 文本段逐字节保留原始内容：`\{`、`\n`、`\}` 都不在这里解码。
    #[test]
    fn text_segments_keep_escape_candidates_verbatim() {
        let segs = segments("a\\{b\\}c\\n", 0).expect("template parses");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            StringSegment::Text(text) => assert_eq!(text.content, "a\\{b\\}c\\n"),
            _ => panic!("single raw text segment expected"),
        }
    }

    /// 内层引号捕获是单个 token：串里的 `}` 不能给插值定界。
    #[test]
    fn inner_quote_capture_shields_braces_from_the_boundary_scan() {
        let segs = segments("{\"}\"}", 0).expect("template parses");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            StringSegment::Interpolation(interp) => match &interp.expr {
                TermExpression::QuoteLiteral(literal) => {
                    assert_eq!(literal.quote, QuoteKind::Double);
                    assert_eq!(literal.content, "}");
                }
                other => panic!("inner literal should stay a quote capture, got {other:?}"),
            },
            _ => panic!("interpolation segment expected"),
        }
    }

    /// 注释里的 `}` 不参与定界。
    #[test]
    fn comment_brace_does_not_close_the_interpolation() {
        let segs = segments("{alpha <#}#> + beta}", 0).expect("template parses");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            StringSegment::Interpolation(interp) => assert!(matches!(&interp.expr, TermExpression::Binary(_))),
            _ => panic!("interpolation segment expected"),
        }
    }

    /// locale 标记 `{߷name}` 仍是普通表达式，并携带 `is_locale`。
    #[test]
    fn locale_marker_is_recognized() {
        let segs = segments("{\u{07df}name}", 0).expect("template parses");
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            StringSegment::Interpolation(interp) => {
                assert!(interp.is_locale);
                match &interp.expr {
                    TermExpression::NamePath(path) => assert_eq!(path.parts[0].name, "name"),
                    other => panic!("locale should bind a name path, got {other:?}"),
                }
            }
            _ => panic!("interpolation segment expected"),
        }
    }

    #[test]
    fn bare_closing_brace_is_a_structured_error() {
        let error = segments("a}b", 0).expect_err("bare `}` must fail");
        assert_eq!(error.source_offset(), Some(1));
    }

    #[test]
    fn empty_interpolation_is_a_structured_error() {
        assert!(segments("{ }", 0).is_err());
    }

    #[test]
    fn missing_closing_brace_is_a_structured_error() {
        assert!(segments("{alpha", 0).is_err());
    }

    #[test]
    fn residual_tokens_after_the_expression_are_a_structured_error() {
        assert!(segments("{alpha beta}", 0).is_err());
    }
}
