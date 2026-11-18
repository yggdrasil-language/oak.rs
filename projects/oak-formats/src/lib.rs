//! 跨语言文本格式化与 AST pretty printer 的平台注册表。
//!
//! 本 crate 拥有与语言无关的格式化契约（选项、错误、`Document` 布局、CST 视图、`SourceMap`、
//! formatter/printer 注册表）以及 VON / AWSL 的委托实现；具体格式的书写器由拥有该格式的 crate
//! 提供（例如 CLR 的 MSIL 文本、WebAssembly 的 WAT/WIT 文本），通过注册表接入，不在这里重建。
//!
//! 编译、链接与产物发射不属本 crate：格式层只处理文本，不参与语义降低。

#![deny(missing_docs)]

pub mod awsl;
pub mod contract;
pub mod formatter;
pub mod highlight;
pub mod printer;
pub mod von;
