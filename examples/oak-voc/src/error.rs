//! AWSL 解析错误。

use std::ops::Range;

/// AWSL 解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwslParseError {
    /// 错误消息。
    pub message: String,
    /// 错误位置。
    pub span: Range<usize>,
}
