//! `.editorconfig` to [`FormatOptions`] parsing (project-wide formatting config source).

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use super::FormatOptions;

/// 解析结果：含最终选项与 editorconfig 显式声明的键（LSP 不得覆盖后者）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatConfigResolution {
    /// 合并后的格式化选项。
    pub options: FormatOptions,
    /// `.editorconfig` 中显式出现的属性名（如 `indent_size`）。
    pub editorconfig_keys: HashSet<String>,
}

/// 从 `.editorconfig` 加载 [`FormatOptions`]。
#[derive(Debug, Clone, Default)]
pub struct FormatConfigLoader;

impl FormatConfigLoader {
    /// 按文件路径解析（自文件所在目录向上查找 `.editorconfig`）。
    pub fn for_path(path: &Path) -> FormatConfigResolution {
        let mut options = FormatOptions::default();
        let mut keys = HashSet::new();

        let Some(file_name) = path.file_name().and_then(|n| n.to_str())
        else {
            return FormatConfigResolution { options, editorconfig_keys: keys };
        };

        let mut dir = path.parent().map(Path::to_path_buf);
        let mut config_files = Vec::new();

        while let Some(ref mut current) = dir {
            let config_path = current.join(".editorconfig");
            if config_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    let root = parse_root_flag(&content);
                    config_files.push((current.clone(), content));
                    if root {
                        break;
                    }
                }
            }
            if !current.pop() {
                break;
            }
        }

        config_files.reverse();
        for (base, content) in config_files {
            apply_editorconfig_file(&base, file_name, &content, &mut options, &mut keys);
        }

        FormatConfigResolution { options, editorconfig_keys: keys }
    }

    /// 将 LSP `FormattingOptions` 合并进解析结果（仅填充 editorconfig 未声明的项）。
    pub fn merge_lsp(
        mut resolution: FormatConfigResolution,
        tab_size: u32,
        insert_spaces: bool,
        insert_final_newline: Option<bool>,
    ) -> FormatOptions {
        if !resolution.editorconfig_keys.contains("indent_size") {
            resolution.options.indent_width = tab_size.max(1) as usize;
            resolution.options.tab_size = tab_size.max(1);
        }
        if !resolution.editorconfig_keys.contains("indent_style") {
            resolution.options.insert_spaces = insert_spaces;
        }
        if !resolution.editorconfig_keys.contains("insert_final_newline") {
            if let Some(v) = insert_final_newline {
                resolution.options.ensure_trailing_newline = v;
            }
        }
        resolution.options
    }

    /// 从 `file://` URI 解析本地路径并加载配置。
    pub fn for_uri(uri: &str) -> FormatConfigResolution {
        let path = uri_to_path(uri);
        Self::for_path(&path)
    }
}

fn uri_to_path(uri: &str) -> PathBuf {
    if let Some(rest) = uri.strip_prefix("file://") {
        let path = rest.trim_start_matches('/');
        if rest.len() > 2 && rest.as_bytes().get(1) == Some(&b':') {
            PathBuf::from(format!("{}:{}", &rest[..1], &rest[2..]))
        }
        else {
            PathBuf::from(path)
        }
    }
    else {
        PathBuf::from(uri)
    }
}

fn parse_root_flag(content: &str) -> bool {
    for line in content.lines() {
        let line = line.split('#').next().unwrap_or(line).trim();
        if line.is_empty() || line.starts_with('[') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if key.trim().eq_ignore_ascii_case("root") && value.trim().eq_ignore_ascii_case("true") {
                return true;
            }
        }
    }
    false
}

fn apply_editorconfig_file(base: &Path, file_name: &str, content: &str, options: &mut FormatOptions, keys: &mut HashSet<String>) {
    let mut current_glob: Option<String> = None;
    for line in content.lines() {
        let line = line.split('#').next().unwrap_or(line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_glob = Some(line[1..line.len() - 1].trim().to_string());
            continue;
        }
        let Some(glob) = current_glob.as_deref()
        else {
            continue;
        };
        if !glob_matches(glob, file_name) {
            continue;
        }
        let Some((key, value)) = line.split_once('=')
        else {
            continue;
        };
        apply_property(key.trim(), value.trim(), options, keys);
        let _ = base;
    }
}

fn apply_property(key: &str, value: &str, options: &mut FormatOptions, keys: &mut HashSet<String>) {
    let key_lower = key.to_ascii_lowercase();
    match key_lower.as_str() {
        "indent_size" => {
            if let Ok(n) = value.parse::<usize>() {
                options.indent_width = n.max(1);
                options.tab_size = n.max(1) as u32;
                keys.insert("indent_size".into());
            }
        }
        "tab_width" => {
            if let Ok(n) = value.parse::<u32>() {
                options.tab_size = n.max(1);
                keys.insert("tab_width".into());
            }
        }
        "indent_style" => {
            options.insert_spaces = !value.eq_ignore_ascii_case("tab");
            keys.insert("indent_style".into());
        }
        "max_line_length" => {
            if let Ok(n) = value.parse::<usize>() {
                options.max_width = n;
                keys.insert("max_line_length".into());
            }
        }
        "insert_final_newline" => {
            options.ensure_trailing_newline = value.eq_ignore_ascii_case("true");
            keys.insert("insert_final_newline".into());
        }
        _ => {}
    }
}

fn glob_matches(glob: &str, file_name: &str) -> bool {
    if glob == "*" {
        return true;
    }
    simple_glob_match(glob, file_name)
}

fn simple_glob_match(glob: &str, text: &str) -> bool {
    let parts: Vec<&str> = glob.split('*').collect();
    if parts.len() == 1 {
        return glob == text;
    }
    let mut rest = text;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !rest.starts_with(part) {
                return false;
            }
            rest = &rest[part.len()..];
        }
        else if i == parts.len() - 1 {
            if !rest.ends_with(part) {
                return false;
            }
        }
        else if let Some(idx) = rest.find(part) {
            rest = &rest[idx + part.len()..];
        }
        else {
            return false;
        }
    }
    true
}
