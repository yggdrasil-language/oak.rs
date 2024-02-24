//! AWSL AST 节点定义。

use std::ops::Range;

/// AWSL 文件根节点。
#[derive(Debug, Clone, PartialEq)]
pub struct AwslRoot {
    /// 是否包含顶层 `<widget>` / `<template>` 容器。
    pub has_widget_shell: bool,
    /// 组件名（snake_case，来自 `<widget name>`；未写时由文件名 stem 推导）。
    pub widget_name: Option<String>,
    /// 模板根节点列表。
    pub template: Vec<AwslTemplateNode>,
    /// `<script>` 块原始文本。
    pub script: Option<String>,
    /// `<style>` 块原始文本。
    pub style: Option<String>,
    /// 导入声明。
    pub imports: Vec<AwslImport>,
    /// 根节点 span。
    pub span: Range<usize>,
}

/// 模板节点。
#[derive(Debug, Clone, PartialEq)]
pub enum AwslTemplateNode {
    /// 元素节点。
    Element(AwslElement),
    /// 纯文本。
    Text {
        /// 文本内容。
        content: String,
        /// span。
        span: Range<usize>,
    },
    /// 插值表达式 `{expr}` 或 `{{expr}}`。
    Interpolation {
        /// 表达式源码。
        expr: String,
        /// span。
        span: Range<usize>,
    },
}

/// 元素节点。
#[derive(Debug, Clone, PartialEq)]
pub struct AwslElement {
    /// 标签名。
    pub tag: String,
    /// 普通属性。
    pub attributes: Vec<AwslAttribute>,
    /// 编译器指令（`@` 前缀属性）。
    pub directives: Vec<AwslDirective>,
    /// 子节点。
    pub children: Vec<AwslTemplateNode>,
    /// 是否自闭合。
    pub self_closing: bool,
    /// span。
    pub span: Range<usize>,
}

/// 元素属性。
#[derive(Debug, Clone, PartialEq)]
pub struct AwslAttribute {
    /// 属性名。
    pub name: String,
    /// 属性值。
    pub value: AwslAttributeValue,
    /// span。
    pub span: Range<usize>,
}

/// 属性值形式。
#[derive(Debug, Clone, PartialEq)]
pub enum AwslAttributeValue {
    /// 字面量字符串。
    Literal(String),
    /// 表达式 `{expr}`。
    Expression(String),
    /// 混合文本与插值。
    Mixed(Vec<AwslTextPart>),
}

/// 文本片段（用于混合属性值）。
#[derive(Debug, Clone, PartialEq)]
pub enum AwslTextPart {
    /// 静态文本。
    Text(String),
    /// 插值表达式。
    Expr(String),
}

/// 编译器指令。
#[derive(Debug, Clone, PartialEq)]
pub struct AwslDirective {
    /// 指令种类。
    pub kind: AwslDirectiveKind,
    /// 指令值表达式（若有）。
    pub value: Option<String>,
    /// span。
    pub span: Range<usize>,
}

/// 指令种类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AwslDirectiveKind {
    /// `@if`
    If,
    /// `@bind`
    Bind,
    /// `@on` / `@click` 等事件。
    On(String),
    /// `@loop`
    Loop,
    /// `@style`（如 Tailwind atomic class 字符串）
    Style,
    /// `@ref`
    Ref,
    /// `@class`
    Class,
    /// 其他 `@` 指令。
    Other(String),
}

/// 组件导入。
#[derive(Debug, Clone, PartialEq)]
pub struct AwslImport {
    /// 组件名。
    pub name: String,
    /// 来源路径。
    pub from: String,
    /// span。
    pub span: Range<usize>,
}

impl AwslElement {
    /// 是否为 PascalCase 组件标签。
    pub fn is_component(&self) -> bool {
        self.tag.chars().next().is_some_and(|ch| ch.is_ascii_uppercase())
    }

    /// 是否为控制流标签（`<for>` / `<if>` / `<loop>`）。
    pub fn is_control_flow(&self) -> bool {
        matches!(self.tag.as_str(), "for" | "if" | "loop")
    }
}
