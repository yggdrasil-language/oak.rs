//! AWSL 单文件组件契约：每个 `.awsl` 对应一个 widget。

use super::{AwslParseError, AwslRoot};

/// 从 `.awsl` 文件名 stem 推导 **snake_case** widget 名（与 `<widget>` 标签一致）。
///
/// 同时处理三种命名风格到 snake_case 的归一化：
/// - kebab-case：`interactive-col-plot` → `interactive_col_plot`
/// - snake_case：`todo_item` → `todo_item`（不变）
/// - PascalCase：`BenchReport` → `bench_report`，`TodoItem` → `todo_item`
///
/// `[slug]` 形式的围栏会被剥离为 `slug`。
pub fn widget_name_from_stem(stem: &str) -> String {
    let base = stem.trim().trim_matches(['[', ']']);
    if base.is_empty() {
        return "component".into();
    }
    // PascalCase → snake_case：在每个大写字母前插入下划线（首字符除外），再统一小写。
    let mut out = String::new();
    for ch in base.chars() {
        if ch.is_uppercase() {
            if !out.is_empty() {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        }
        else {
            out.push(ch);
        }
    }
    out.replace('-', "_").to_lowercase()
}

/// PascalCase 模板组件标签 → snake_case `.awsl` stem（`TodoItem` → `todo_item`）。
pub fn awsl_stem_from_component_tag(tag: &str) -> String {
    let mut out = String::new();
    for ch in tag.chars() {
        if ch.is_uppercase() {
            if !out.is_empty() {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        }
        else {
            out.push(ch);
        }
    }
    out
}

/// 解析 widget 名（文件名 stem 为唯一真相来源）。
pub fn resolve_widget_name(file_stem: &str) -> String {
    widget_name_from_stem(file_stem)
}

/// 校验「一文件 = 一 widget」契约。
pub fn validate_component_contract(root: &AwslRoot, file_stem: &str) -> Result<(), AwslParseError> {
    if !root.has_widget_shell {
        return Err(AwslParseError {
            message: "each .awsl file must define exactly one top-level <widget> block".into(),
            span: root.span.clone(),
        });
    }
    let expected = widget_name_from_stem(file_stem);
    if let Some(tag) = root.widget_name.as_deref().filter(|name| !name.is_empty()) {
        if tag != expected {
            return Err(AwslParseError {
                message: format!("<widget {tag}> does not match file stem `{file_stem}` (expected <widget {expected}>)"),
                span: root.span.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_to_widget_name_examples() {
        assert_eq!(widget_name_from_stem("counter"), "counter");
        assert_eq!(widget_name_from_stem("todo_item"), "todo_item");
        assert_eq!(widget_name_from_stem("interactive-col-plot"), "interactive_col_plot");
        assert_eq!(widget_name_from_stem("[slug]"), "slug");
        assert_eq!(widget_name_from_stem("chart-status"), "chart_status");
        assert_eq!(widget_name_from_stem("BenchReport"), "bench_report");
        assert_eq!(widget_name_from_stem("TodoItem"), "todo_item");
        assert_eq!(widget_name_from_stem("CoverageReport"), "coverage_report");
    }
}
