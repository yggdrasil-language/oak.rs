//! 批量发现与格式化路径。

use std::{
    collections::{HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use super::{FormatError, FormatOptions, format_path, source_registry_read};

/// 批量格式化选项。
#[derive(Debug, Clone)]
pub struct FormatBatchOptions {
    /// 格式化风格（`use_editorconfig = false` 时用于所有文件）。
    pub format: FormatOptions,
    /// 从各文件路径的 `.editorconfig` 解析选项（默认开启）。
    pub use_editorconfig: bool,
    /// 仅检查、不写盘。
    pub check: bool,
    /// 跳过的目录名。
    pub skip_dirs: Vec<&'static str>,
    /// 仅处理这些扩展名（不含点）；`None` 表示所有支持的语言。
    pub extensions: Option<Vec<String>>,
}

impl Default for FormatBatchOptions {
    fn default() -> Self {
        Self {
            format: FormatOptions::default(),
            use_editorconfig: true,
            check: false,
            skip_dirs: vec!["target", "dist", "build", "vendors", ".git", "node_modules", ".intellijPlatform", ".cursor", ".run"],
            extensions: None,
        }
    }
}

/// 批量格式化报告。
#[derive(Debug, Clone, Default)]
pub struct FormatBatchReport {
    /// 已检查文件数。
    pub checked: usize,
    /// 已改写（或 check 模式下需改写）的文件数。
    pub changed: usize,
    /// 跳过（解析失败等）文件数。
    pub skipped: usize,
    /// 跳过原因。
    pub skip_messages: Vec<String>,
    /// 已改写路径。
    pub changed_paths: Vec<PathBuf>,
}

/// 对路径列表执行格式化：文件直接处理，目录递归收集。
pub fn format_paths(paths: &[PathBuf], batch: &FormatBatchOptions) -> Result<FormatBatchReport, FormatError> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            if matches_extension_filter(path, batch.extensions.as_deref()) {
                files.push(path.clone());
            }
        }
        else if path.is_dir() {
            collect_files_iterative(path, batch, &mut files)?;
        }
        else {
            return Err(FormatError::Io { path: path.clone(), source: std::io::Error::new(std::io::ErrorKind::NotFound, "路径不存在") });
        }
    }
    files.sort();
    files.dedup();

    let mut report = FormatBatchReport::default();
    for file in files {
        let format_options =
            if batch.use_editorconfig { crate::contract::FormatConfigLoader::for_path(&file).options } else { batch.format.clone() };
        match format_path_large_stack(&file, &format_options) {
            Ok(outcome) => {
                report.checked += 1;
                if outcome.changed {
                    report.changed += 1;
                    report.changed_paths.push(outcome.path.clone());
                    if !batch.check {
                        fs::write(&outcome.path, &outcome.formatted)
                            .map_err(|error| FormatError::Io { path: outcome.path.clone(), source: error })?;
                    }
                }
            }
            Err(FormatError::Parse { message, .. }) => {
                report.skipped += 1;
                report.skip_messages.push(format!("跳过（解析错误）：{} — {message}", file.display()));
            }
            Err(error) => return Err(error),
        }
    }
    Ok(report)
}

fn format_path_large_stack(path: &Path, options: &FormatOptions) -> Result<super::FormatFileOutcome, FormatError> {
    // Some generated parsers / deeply-nested CST formatting can be stack-hungry on Windows.
    // Run formatting on a dedicated thread with a larger stack to avoid process abort.
    let path = path.to_path_buf();
    let options = options.clone();
    let path_for_thread = path.clone();
    let path_for_error = path;
    std::thread::Builder::new()
        .name("oak-format".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || format_path(&path_for_thread, &options))
        .map_err(|error| FormatError::Io {
            path: PathBuf::from("<format-thread>"),
            source: std::io::Error::new(std::io::ErrorKind::Other, error.to_string()),
        })?
        .join()
        .unwrap_or_else(|_| {
            Err(FormatError::Parse { path: Some(path_for_error), message: "格式化线程崩溃（可能是 stack overflow）".into() })
        })
}

fn matches_extension_filter(path: &Path, extensions: Option<&[String]>) -> bool {
    let Some(ext) = path.extension().and_then(|value| value.to_str())
    else {
        return false;
    };
    match extensions {
        Some(extensions) => extensions.iter().any(|allowed| allowed.eq_ignore_ascii_case(ext)),
        // 未指定过滤时，以注册表实际拥有的扩展名为准（平台与前端 provider 共同构成）。
        None => source_registry_read().supports_extension(ext),
    }
}

fn collect_files_iterative(root: &Path, batch: &FormatBatchOptions, out: &mut Vec<PathBuf>) -> Result<(), FormatError> {
    let skip_dirs = &batch.skip_dirs;
    let extensions = batch.extensions.as_deref();
    let mut queue: VecDeque<PathBuf> = VecDeque::new();
    queue.push_back(root.to_path_buf());
    let mut visited: HashSet<PathBuf> = HashSet::new();

    while let Some(dir) = queue.pop_front() {
        // Break directory cycles (Windows junctions/symlinks can create loops).
        let canonical = std::fs::canonicalize(&dir).unwrap_or_else(|_| dir.clone());
        if !visited.insert(canonical) {
            continue;
        }
        let entries = fs::read_dir(&dir).map_err(|error| FormatError::Io { path: dir.clone(), source: error })?;
        for entry in entries {
            let entry = entry.map_err(|error| FormatError::Io { path: dir.clone(), source: error })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|error| FormatError::Io { path: dir.clone(), source: error })?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if skip_dirs.iter().any(|s| *s == name) {
                    continue;
                }
                queue.push_back(path);
            }
            else if matches_extension_filter(&path, extensions) {
                out.push(path);
            }
        }
    }
    Ok(())
}
