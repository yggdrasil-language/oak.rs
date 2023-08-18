use crate::ast::{Identifier, InterpolationSegment, NamePath, Span, StringSegment, TermExpression, TextSegment};

/// Locale 参数标记字符 (Gwot, U+07DF)
const LOCALE_MARKER: char = '\u{07DF}';

/// 解析字符串内容为片段列表。
///
/// 该函数处理以下功能：
/// - 插值解析：解析 `{terms}` 格式的插值表达式
/// - 转义处理：处理 `\{` 和 `\}` 转义序列
/// - Locale 标记：识别 `߷` (Gwot, U+07DF) 符号标记的本地化参数
/// - Raw String 处理：前缀为 `r` 时不解析插值
///
/// # 参数
///
/// * `content` - 字符串内容（不含引号）
/// * `span_start` - 内容在源码中的起始位置
/// * `is_raw` - 是否为 raw string（raw string 不解析插值）
///
/// # 返回
///
/// 返回解析后的字符串片段列表
///
/// # 示例
///
/// ```
/// use oak_valkyrie::parser::parse_string_segments;
///
/// // 普通文本
/// let segments = parse_string_segments("hello", 0, false);
/// assert_eq!(segments.len(), 1);
///
/// // 插值表达式
/// let segments = parse_string_segments("hello {name}!", 0, false);
/// assert_eq!(segments.len(), 3);
///
/// // Raw string 不解析插值
/// let segments = parse_string_segments("hello {name}!", 0, true);
/// assert_eq!(segments.len(), 1);
/// ```
pub fn parse_string_segments(content: &str, span_start: usize, is_raw: bool) -> Vec<StringSegment> {
    if is_raw {
        return vec![StringSegment::Text(Box::new(TextSegment { content: content.to_string(), span: Span { start: span_start, end: span_start + content.len() } }))];
    }

    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut text_start = span_start;
    let mut chars = content.char_indices().peekable();
    let content_len = content.len();

    while let Some((idx, ch)) = chars.next() {
        match ch {
            '\\' => {
                if let Some((_, next_ch)) = chars.peek() {
                    if *next_ch == '{' || *next_ch == '}' {
                        current_text.push(*next_ch);
                        chars.next();
                        continue;
                    }
                }
                current_text.push(ch);
            }
            '{' => {
                if !current_text.is_empty() {
                    segments.push(StringSegment::Text(Box::new(TextSegment { content: current_text.clone(), span: Span { start: text_start, end: span_start + idx } })));
                    current_text.clear();
                }

                let is_locale = if let Some((_, next_ch)) = chars.peek() { *next_ch == LOCALE_MARKER } else { false };

                if is_locale {
                    chars.next();
                }

                let expr_start = span_start + idx;
                let mut expr_content = String::new();
                let mut brace_count = 1;
                let mut expr_end = span_start + idx + 1;

                while let Some((inner_idx, inner_ch)) = chars.next() {
                    match inner_ch {
                        '{' => {
                            brace_count += 1;
                            expr_content.push(inner_ch);
                        }
                        '}' => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                expr_end = span_start + inner_idx;
                                break;
                            }
                            expr_content.push(inner_ch);
                        }
                        _ => {
                            expr_content.push(inner_ch);
                        }
                    }
                }

                let trimmed_expr = expr_content.trim();
                segments.push(StringSegment::Interpolation(Box::new(InterpolationSegment {
                    expr: TermExpression::NamePath(Box::new(NamePath { parts: vec![Identifier { name: trimmed_expr.to_string(), span: Span { start: expr_start, end: expr_end } }], span: Span { start: expr_start, end: expr_end } })),
                    is_locale,
                    span: Span { start: expr_start, end: expr_end + 1 },
                })));

                text_start = expr_end + 1;
            }
            _ => {
                current_text.push(ch);
            }
        }
    }

    if !current_text.is_empty() {
        segments.push(StringSegment::Text(Box::new(TextSegment { content: current_text, span: Span { start: text_start, end: span_start + content_len } })));
    }

    segments
}
