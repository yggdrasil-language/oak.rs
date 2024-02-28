#![doc = include_str!("readme.md")]
use oak_core::{Language, LanguageCategory};

/// Configuration for the Markdown language features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarkdownLanguage {
    /// Enable math formulas.
    ///
    /// Example: `$a^2 + b^2 = c^2$` or `$$E = mc^2$$`
    pub allow_math: bool,
    /// Enable tables.
    ///
    /// Example:
    /// | Header |
    /// | ------ |
    /// | Cell   |
    pub allow_tables: bool,
    /// Enable task lists.
    ///
    /// Example: `- [ ] Task` or `- [x] Done`
    pub allow_task_lists: bool,
    /// Enable strikethrough.
    ///
    /// Example: `~~deleted~~`
    pub allow_strikethrough: bool,
    /// Enable footnotes.
    ///
    /// Example: `[^1]` and `[^1]: Note`
    pub allow_footnotes: bool,
    /// Enable front matter (YAML/TOML/JSON).
    ///
    /// Example:
    /// ---
    /// title: Hello
    /// ---
    pub allow_front_matter: bool,
    /// Enable definition lists.
    ///
    /// Example:
    /// Term
    /// : Definition
    pub allow_definition_lists: bool,
    /// Enable superscript and subscript.
    ///
    /// Example: `^sup^` or `~sub~`
    pub allow_subscript: bool,
    /// Enable CommonMark autolinks.
    ///
    /// Example: `<https://example.com>` or `<foo@bar.com>`
    pub allow_autolinks: bool,
    /// Enable bare links in plain text.
    ///
    /// Example: `https://example.com`
    pub allow_barelinks: bool,
    /// Enable abbreviations.
    ///
    /// Example: `*[HTML]: HyperText Markup Language`
    pub allow_abbreviations: bool,
    /// Enable indented code blocks.
    ///
    /// Example:
    ///     code block
    pub allow_indented_code_blocks: bool,
    /// Enable inline HTML tags.
    ///
    /// Example: `<div>` or `<!-- comment -->`
    pub allow_html: bool,
    /// Enable hard line breaks.
    ///
    /// Example: Two spaces at the end of a line or a backslash.
    pub allow_hard_line_breaks: bool,
    /// Enable ATX headings.
    ///
    /// Example: `# Heading`
    pub allow_headings: bool,
    /// Enable lists.
    ///
    /// Example: `- Item` or `1. Item`
    pub allow_lists: bool,
    /// Enable blockquotes.
    ///
    /// Example: `> Quote`
    pub allow_blockquotes: bool,
    /// Enable fenced code blocks.
    ///
    /// Example: ` ```rust `
    pub allow_fenced_code_blocks: bool,
    /// Enable horizontal rules.
    ///
    /// Example: `---` or `***`
    pub allow_horizontal_rules: bool,
    /// Enable Setext headings.
    ///
    /// Example:
    /// Heading
    /// =======
    pub allow_setext_headings: bool,
    /// Filter certain HTML tags like `<script>`.
    pub allow_html_tag_filter: bool,
    /// Enable XML/TSX syntax.
    ///
    /// Example: `<Component />`
    pub allow_xml: bool,
    /// Enable MDX syntax (Markdown + JSX).
    ///
    /// This enables:
    /// - JSX components: `<Component prop="value" />`
    /// - Import statements: `import { Component } from 'module'`
    /// - Export statements: `export default Component`
    /// - JSX expressions: `{variable}`
    pub allow_mdx: bool,
}

impl MarkdownLanguage {
    /// Creates a Markdown language configuration with every feature flag set to `enable`.
    pub fn new(enable: bool) -> Self {
        Self {
            allow_math: enable,
            allow_tables: enable,
            allow_task_lists: enable,
            allow_strikethrough: enable,
            allow_footnotes: enable,
            allow_front_matter: enable,
            allow_definition_lists: enable,
            allow_subscript: enable,
            allow_autolinks: enable,
            allow_barelinks: enable,
            allow_abbreviations: enable,
            allow_indented_code_blocks: enable,
            allow_html: enable,
            allow_hard_line_breaks: enable,
            allow_headings: enable,
            allow_lists: enable,
            allow_blockquotes: enable,
            allow_fenced_code_blocks: enable,
            allow_horizontal_rules: enable,
            allow_setext_headings: enable,
            allow_html_tag_filter: enable,
            allow_xml: enable,
            allow_mdx: enable,
        }
    }

    /// CommonMark baseline: core block and inline syntax without optional extensions.
    pub fn common_mark() -> Self {
        Self {
            allow_autolinks: true,
            allow_indented_code_blocks: true,
            allow_html: true,
            allow_hard_line_breaks: true,
            allow_headings: true,
            allow_lists: true,
            allow_blockquotes: true,
            allow_fenced_code_blocks: true,
            allow_horizontal_rules: true,
            allow_setext_headings: true,
            ..Self::new(false)
        }
    }

    /// [`common_mark`] plus tables, task lists, strikethrough, bare links, and HTML tag filtering.
    pub fn github_flavored() -> Self {
        Self { allow_tables: true, allow_task_lists: true, allow_strikethrough: true, allow_barelinks: true, allow_html_tag_filter: true, ..Self::common_mark() }
    }
}

impl Language for MarkdownLanguage {
    const NAME: &'static str = "markdown";
    const CATEGORY: LanguageCategory = LanguageCategory::Markup;

    type TokenType = crate::lexer::token_type::MarkdownTokenType;
    type ElementType = crate::parser::element_type::MarkdownElementType;
    type TypedRoot = crate::ast::MarkdownRoot;
}

impl Default for MarkdownLanguage {
    fn default() -> Self {
        Self::new(false)
    }
}
