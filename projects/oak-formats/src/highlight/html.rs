//! 将 `HighlightSpan` 渲染为带 `hl-*` class 的 HTML 片段。

use super::HighlightSpan;

/// 将源文本与高亮区间合并为 HTML（未覆盖区间仅 HTML 转义）。
pub fn render_spans_html(source: &str, spans: &[HighlightSpan]) -> String {
    let mut spans: Vec<&HighlightSpan> = spans.iter().filter(|s| s.range.start < s.range.end && s.range.end <= source.len()).collect();
    spans.sort_by_key(|s| s.range.start);

    let mut html = String::new();
    let mut last_end = 0usize;
    for span in spans {
        if span.range.start < last_end {
            continue;
        }
        if span.range.start > last_end {
            html.push_str(&escape_html(&source[last_end..span.range.start]));
        }
        let text = &source[span.range.start..span.range.end];
        match span.kind.css_class() {
            Some(class) => {
                html.push_str("<span class=\"");
                html.push_str(class);
                html.push_str("\">");
                html.push_str(&escape_html(text));
                html.push_str("</span>");
            }
            None => html.push_str(&escape_html(text)),
        }
        last_end = span.range.end;
    }
    if last_end < source.len() {
        html.push_str(&escape_html(&source[last_end..]));
    }
    html
}

/// HTML 转义。
pub fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}
