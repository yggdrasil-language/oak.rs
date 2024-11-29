//! 高亮语义种类（对齐 C# `Nyar.Analyzer.Highlight.HighlightKind`）。

/// 词法/语义着色种类。
///
/// 语言插件将自身 token / node 映射到这些统一种类；
/// 呈现层（文档 HTML、IDE、LSP semantic tokens）再映射到 CSS class 或编辑器属性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HighlightKind {
    /// 未分类。
    #[default]
    None,
    /// 关键字。
    Keyword,
    /// 控制流关键字。
    ControlKeyword,
    /// 字符串字面量。
    String,
    /// 数字字面量。
    Number,
    /// 注释。
    Comment,
    /// 运算符。
    Operator,
    /// 标点。
    Punctuation,
    /// 普通标识符。
    Identifier,
    /// 类型名。
    TypeIdentifier,
    /// 枚举/联合变体标识符（如 `Some` / `None` / `Fine` / `Fail`）。
    VariantIdentifier,
    /// 函数名。
    FunctionIdentifier,
    /// 参数。
    Parameter,
    /// 属性。
    Property,
    /// 字段。
    Field,
    /// 局部变量。
    Variable,
    /// 常量。
    Constant,
    /// 命名空间。
    Namespace,
    /// 模块。
    Module,
    /// 装饰器 / 注解。
    Decorator,
    /// 正则。
    Regex,
    /// 转义序列。
    Escape,
    /// 分隔符。
    Delimiter,
    /// 字符串插值。
    Interpolation,
}

impl HighlightKind {
    /// 文档 / API 页使用的 CSS class（`hl-*`）。
    pub fn css_class(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Keyword | Self::ControlKeyword => Some("hl-keyword"),
            Self::String => Some("hl-string"),
            Self::Number => Some("hl-number"),
            Self::Comment => Some("hl-comment"),
            Self::Operator => Some("hl-operator"),
            Self::Punctuation | Self::Delimiter => Some("hl-punctuation"),
            Self::Identifier | Self::Variable => Some("hl-identifier"),
            Self::TypeIdentifier => Some("hl-type"),
            Self::VariantIdentifier => Some("hl-variant"),
            Self::FunctionIdentifier => Some("hl-function"),
            Self::Parameter => Some("hl-parameter"),
            Self::Property | Self::Field => Some("hl-property"),
            Self::Constant => Some("hl-constant"),
            Self::Namespace | Self::Module => Some("hl-namespace"),
            Self::Decorator => Some("hl-decorator"),
            Self::Regex => Some("hl-regex"),
            Self::Escape => Some("hl-escape"),
            Self::Interpolation => Some("hl-interpolation"),
        }
    }
}
