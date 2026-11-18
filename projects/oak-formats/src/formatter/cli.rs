//! CLI 共用：批量格式化参数、执行与报告。

use std::path::PathBuf;

use super::{AWSL_EXTENSIONS, FormatBatchOptions, FormatBatchReport, FormatError, FormatOptions, format_paths};

/// 宿主 CLI `fmt` / `format` 子命令共用选项。
#[derive(Debug, Clone)]
pub struct FormatCliOptions {
    /// 要格式化的文件或目录（空则当前目录）。
    pub paths: Vec<PathBuf>,
    /// 仅检查、不写盘。
    pub check: bool,
    /// 从各文件路径的 `.editorconfig` 解析风格（默认开启）。
    pub use_editorconfig: bool,
    /// 打印每个被检查/改写的文件。
    pub verbose: bool,
    /// 有解析跳过项时以非 0 退出。
    pub fail_on_skip: bool,
    /// `use_editorconfig = false` 时使用的风格。
    pub format: FormatOptions,
    /// 仅处理这些扩展名（不含点）；`None` 表示所有支持的语言。
    pub extensions: Option<Vec<String>>,
}

impl Default for FormatCliOptions {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            check: false,
            use_editorconfig: true,
            verbose: false,
            fail_on_skip: false,
            format: FormatOptions::default(),
            extensions: None,
        }
    }
}

impl FormatCliOptions {
    /// 仅格式化 AWSL（`.awsl` 规范拼写与同义缩写 `.w`，取自单一事实源 [`AWSL_EXTENSIONS`]）。
    pub fn awsl_only(mut self) -> Self {
        self.extensions = Some(AWSL_EXTENSIONS.iter().map(|extension| (*extension).to_string()).collect());
        self
    }

    fn resolved_paths(&self) -> Vec<PathBuf> {
        if self.paths.is_empty() { vec![PathBuf::from(".")] } else { self.paths.clone() }
    }

    fn batch_options(&self) -> FormatBatchOptions {
        FormatBatchOptions {
            format: self.format.clone(),
            use_editorconfig: self.use_editorconfig,
            check: self.check,
            extensions: self.extensions.clone(),
            ..FormatBatchOptions::default()
        }
    }
}

/// 执行格式化（写回或 `--check`）。
pub fn run_format_cli_with(options: &FormatCliOptions) -> Result<FormatBatchReport, FormatError> {
    format_paths(&options.resolved_paths(), &options.batch_options())
}

/// 兼容旧 API。
pub fn run_format_cli(paths: &[PathBuf], check: bool) -> Result<FormatBatchReport, FormatError> {
    run_format_cli_with(&FormatCliOptions { paths: paths.to_vec(), check, ..FormatCliOptions::default() })
}

/// 将批量报告打印到 stdout/stderr；返回是否应以非 0 退出。
pub fn report_format_cli_with(report: &FormatBatchReport, options: &FormatCliOptions) -> bool {
    for message in &report.skip_messages {
        eprintln!("{message}");
    }

    if options.verbose && report.changed_paths.is_empty() && report.checked > 0 && !options.check {
        println!("已检查 {} 个文件，无需改写", report.checked);
    }

    for path in &report.changed_paths {
        if options.check {
            println!("需要格式化：{}", path.display());
        }
        else if options.verbose {
            println!("已格式化：{}", path.display());
        }
    }

    if !options.check && !options.verbose && report.changed > 0 {
        for path in &report.changed_paths {
            println!("已格式化：{}", path.display());
        }
    }

    let mut failed = false;
    if options.check && report.changed > 0 {
        println!("检查失败：{} 个文件需要格式化（共检查 {}）", report.changed, report.checked);
        failed = true;
    }
    else if options.check {
        println!("检查通过：{} 个文件已格式化", report.checked);
    }
    else {
        println!("格式化完成，共 {} 个文件（改写 {}，跳过 {}）", report.checked, report.changed, report.skipped);
    }

    if options.fail_on_skip && report.skipped > 0 {
        eprintln!("失败：{} 个文件因解析错误被跳过", report.skipped);
        failed = true;
    }

    failed
}

/// 兼容旧 API。
pub fn report_format_cli(report: &FormatBatchReport, check: bool) -> bool {
    report_format_cli_with(report, &FormatCliOptions { check, ..FormatCliOptions::default() })
}

/// 由 `--indent-width` 等 CLI 标志构造 [`FormatOptions`]。
pub fn format_options_from_cli(indent_width: Option<usize>, max_width: Option<usize>) -> FormatOptions {
    let mut options = FormatOptions::default();
    if let Some(width) = indent_width {
        options.indent_width = width.max(1);
        options.tab_size = width.max(1) as u32;
    }
    if let Some(width) = max_width {
        options.max_width = width;
    }
    options
}

/// 解析 `--ext` 列表（`awsl` / `.awsl` 均可）。
pub fn normalize_extensions(values: &[String]) -> Vec<String> {
    values.iter().map(|value| value.trim_start_matches('.').to_ascii_lowercase()).collect()
}
