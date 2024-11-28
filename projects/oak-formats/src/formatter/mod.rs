//! LSP / CLI **格式化注册表**（契约见 `crate::contract::SourceFormatterRegistry`）。
//!
//! 本模块只提供平台侧注册表与扩展名表：具体语言的 formatter 由前端 crate 在启动时调用
//! [`register_source_provider`] 注册（方向是前端 → 平台），平台不反向依赖任何前端。
//! 语言前端 provider 由前端装配入口注册；VON / AWSL 由本 crate 直接注册。合法 formatter
//! 只能来自上游 `oak-<language>/src/formatter/`（见 `oak-typescript::formatter`），
//! 不得在此新增 CST token-gap 排版规则。
//!
//! 与 [`crate::printer`]（AST pretty print）严格分离，对齐 Oak 的 `formatter/` vs `printer/`；
//! 早期把 printer API 与 deprecated `FormatBuffer` 从本模块重导出的混合接口已删除，
//! 消费者必须直接使用 `crate::printer`。
//!
//! - **SourceFormatter**：源码 → CST token-gap → 文本（保留 trivia）
//! - **配置**：默认从各文件路径的 `.editorconfig` 解析（[`FormatBatchOptions::use_editorconfig`]）

mod batch;
mod cli;

use std::{
    path::{Path, PathBuf},
    sync::{OnceLock, RwLock, RwLockReadGuard},
};

pub use batch::{FormatBatchOptions, FormatBatchReport, format_paths};
pub use cli::{
    FormatCliOptions, format_options_from_cli, normalize_extensions, report_format_cli, report_format_cli_with, run_format_cli,
    run_format_cli_with,
};
pub use crate::contract::{
    ByteRange, Document, FormatConfigLoader, FormatConfigResolution, FormatError, FormatOptions, FormattedOutput, PrintStyle, Printer,
    PrinterProvider, PrinterRegistry, SourceFormatter, SourceFormatterProvider, SourceFormatterRegistry, SourceMap,
};

/// VON 配置/清单。
pub const VON_EXTENSIONS: &[&str] = &["von"];

/// 平台拥有的模板语言面（AWSL）的扩展名集合——Asgard 源码后缀的单一事实源。
///
/// `.awsl` 为规范拼写，`.w` 为其同义缩写：同 parser、同 lowering、同身份，
/// 可接受、不要求迁移、不留双轨。其余站点（LSP、Asgard 管线、CLI、格式化）
/// 必须引用本集合或其判定函数 [`is_awsl_extension`]，不得各自维护字符串。
pub const AWSL_EXTENSIONS: &[&str] = &["awsl", "w"];

/// 扩展名是否属于 AWSL 源码集合（`.awsl` 规范拼写 / `.w` 同义缩写）。
pub fn is_awsl_extension(ext: &str) -> bool {
    AWSL_EXTENSIONS.contains(&ext)
}

/// 平台直接拥有 formatter 的源码种类（按文件扩展名）。
///
/// 其它语言的扩展名与语言种类由前端 provider 在启动时注册，平台不以枚举或常量
/// 携带这些事实。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// VON 数据：`.von`（`oak-von::formatter`）。
    Von,
    /// AWSL 模板：`.awsl`（规范拼写）或同义缩写 `.w`（`oak-awsl::formatter`）。
    Awsl,
}

impl SourceKind {
    /// 对应 `SourceFormatter::language_id`。
    pub fn language_id(self) -> &'static str {
        match self {
            Self::Von => "von",
            Self::Awsl => "awsl",
        }
    }

    /// 由文件扩展名推断种类（只含平台拥有的语言）。
    pub fn from_extension(ext: &str) -> Option<Self> {
        source_kind_from_extension(ext)
    }

    /// 由路径推断（只含平台拥有的语言）。
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension().and_then(|ext| ext.to_str()).and_then(Self::from_extension)
    }
}

/// 由扩展名推断平台拥有的 `SourceKind`。
///
/// 前端语言的扩展名由 [`source_registry`] 中的注册结果解析，不在本函数内判定。
pub fn source_kind_from_extension(ext: &str) -> Option<SourceKind> {
    if VON_EXTENSIONS.contains(&ext) {
        Some(SourceKind::Von)
    }
    else if AWSL_EXTENSIONS.contains(&ext) {
        Some(SourceKind::Awsl)
    }
    else {
        None
    }
}

/// 单文件格式化结果（不写盘）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatFileOutcome {
    /// 源文件路径。
    pub path: PathBuf,
    /// 格式化后的文本。
    pub formatted: String,
    /// 是否与原始内容不同。
    pub changed: bool,
}

struct VonSourceFormatter;
impl SourceFormatter for VonSourceFormatter {
    fn language_id(&self) -> &str {
        "von"
    }

    fn format_document(&self, source: &str, options: &FormatOptions) -> Result<FormattedOutput, FormatError> {
        crate::von::source_format::format_von_source(source, options)
    }
}

struct AwslSourceFormatter;
impl SourceFormatter for AwslSourceFormatter {
    fn language_id(&self) -> &str {
        "awsl"
    }

    fn format_document(&self, source: &str, options: &FormatOptions) -> Result<FormattedOutput, FormatError> {
        crate::awsl::source_format::format_awsl(source, options)
    }
}

/// 平台源码格式化注册表（启动时由各前端 provider 补全）。
///
/// 读取方应通过 [`format_source`] / [`format_source_for_extension`] / [`format_path`]
/// 间接访问，避免长时间持有写锁。扩展名 → formatter 的映射由平台注册（VON / AWSL）
/// 与前端 provider 注册共同构成；未注册的扩展名返回 [`FormatError::Unsupported`]。
///
/// 不要反向依赖具体前端；provider 由装配层（宿主 CLI、集成测试）注册。
pub fn source_registry() -> &'static RwLock<SourceFormatterRegistry> {
    static REGISTRY: OnceLock<RwLock<SourceFormatterRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut reg = SourceFormatterRegistry::new();
        reg.register(&["von"], VON_EXTENSIONS, Box::new(VonSourceFormatter));
        reg.register(&["awsl"], AWSL_EXTENSIONS, Box::new(AwslSourceFormatter));
        // JS/TS/JSON 与 Python 的源码排版归各自 Oak formatter（`oak-typescript::formatter` 等）；
        // 本 crate 不得再注册「只裁剪空白行」的第二套本地 formatter 冒充格式化。
        // 在这些 Oak formatter 接入前，相应扩展名按 unsupported 失败。
        RwLock::new(reg)
    })
}

/// 注册前端源码格式化 provider（线程安全，可重复调用；先注册者优先）。
pub fn register_source_provider(provider: &dyn SourceFormatterProvider) {
    source_registry().write().unwrap_or_else(|error| error.into_inner()).register_provider(provider);
}

fn source_registry_read() -> RwLockReadGuard<'static, SourceFormatterRegistry> {
    source_registry().read().unwrap_or_else(|error| error.into_inner())
}

/// 将源码格式化为规范文本。
pub fn format_source(kind: SourceKind, source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    source_registry_read().format(kind.language_id(), source, options)
}

/// 格式化选中字节范围。
pub fn format_source_range(kind: SourceKind, source: &str, range: ByteRange, options: &FormatOptions) -> Result<FormattedOutput, FormatError> {
    source_registry_read().format_range(kind.language_id(), source, range, options)
}

/// 按源码文件扩展名格式化。
///
/// 扩展名 → formatter 的映射来自注册表：平台注册 VON / AWSL，前端 provider 注册
/// 自己语言面的扩展名。
pub fn format_source_for_extension(extension: &str, source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let registry = source_registry_read();
    let language = registry.extension_language_id(extension).ok_or_else(|| FormatError::unsupported_language(extension))?;
    registry.format(language, source, options)
}

/// 读取路径并格式化（不写盘）。
pub fn format_path(path: &Path, options: &FormatOptions) -> Result<FormatFileOutcome, FormatError> {
    let extension = path.extension().and_then(|ext| ext.to_str()).ok_or_else(|| FormatError::unsupported_extension(path.to_path_buf()))?;
    let language = {
        let registry = source_registry_read();
        registry.extension_language_id(extension).map(str::to_owned)
    };
    let language = language.ok_or_else(|| FormatError::unsupported_extension(path.to_path_buf()))?;
    let source = std::fs::read_to_string(path).map_err(|error| FormatError::Io { path: path.to_path_buf(), source: error })?;
    let formatted = source_registry_read().format(&language, &source, options)?;
    Ok(FormatFileOutcome { path: path.to_path_buf(), changed: formatted != source, formatted })
}
