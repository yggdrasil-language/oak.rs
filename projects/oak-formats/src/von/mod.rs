//! VON 文本格式化。

pub(crate) mod source_format;
mod value_format;
#[cfg(feature = "serde")]
pub use value_format::{to_string, to_string_pretty};
