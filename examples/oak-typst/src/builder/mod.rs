use crate::{
    ast::*,
    language::TypstLanguage,
    lexer::token_type::TypstTokenType,
    parser::{TypstParser, element_type::TypstElementType},
};
use oak_core::{
    Builder, BuilderCache, GreenNode, OakDiagnostics, OakError, Parser, RedNode, RedTree,
    source::{Source, TextEdit},
};

/// AST builder for Typst language
#[derive(Clone)]
pub struct TypstBuilder<'config> {
    config: &'config TypstLanguage,
}

impl<'config> TypstBuilder<'config> {
    /// Creates a new TypstBuilder with the given language configuration.
    pub fn new(config: &'config TypstLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<TypstLanguage> for TypstBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(&self, source: &'a S, edits: &[TextEdit], cache: &'a mut impl BuilderCache<TypstLanguage>) -> OakDiagnostics<TypstRoot> {
        let parser = TypstParser::new(self.config);

        let parse_result = parser.parse(source, edits, cache);

        match parse_result.result {
            Ok(green_tree) => match self.build_root(green_tree, source) {
                Ok(ast_root) => OakDiagnostics { result: Ok(ast_root), diagnostics: parse_result.diagnostics },
                Err(build_error) => {
                    let mut diagnostics = parse_result.diagnostics;
                    diagnostics.push(build_error.clone());
                    OakDiagnostics { result: Err(build_error), diagnostics }
                }
            },
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics: parse_result.diagnostics },
        }
    }
}

impl<'config> TypstBuilder<'config> {
    pub(crate) fn build_root<S: Source + ?Sized>(&self, green_tree: &GreenNode<TypstLanguage>, source: &S) -> Result<TypstRoot, OakError> {
        let red_root = RedNode::new(green_tree, 0);
        let mut root = TypstRoot::new(red_root.span());

        for child in red_root.children() {
            if let Some(item) = self.build_tree(child, source)? {
                root.items.push(item);
            }
        }

        Ok(root)
    }

    fn build_tree<S: Source + ?Sized>(&self, tree: RedTree<TypstLanguage>, source: &S) -> Result<Option<TypstItem>, OakError> {
        match tree {
            RedTree::Node(node) => self.build_item(node, source),
            RedTree::Leaf(leaf) => match leaf.kind {
                TypstTokenType::Whitespace | TypstTokenType::Newline => Ok(Some(TypstItem::Space)),
                TypstTokenType::Escape => {
                    // 转义条目只暴露被保护的字符，反斜杠本身不进入正文。
                    let raw = source.get_text_in(leaf.span);
                    Ok(Some(TypstItem::Text(raw.get(1..).unwrap_or_default().to_string())))
                }
                _ => Ok(Some(TypstItem::Text(source.get_text_in(leaf.span).to_string()))),
            },
        }
    }

    fn build_item<S: Source + ?Sized>(&self, node: RedNode<TypstLanguage>, source: &S) -> Result<Option<TypstItem>, OakError> {
        match node.kind::<TypstElementType>() {
            TypstElementType::Heading => {
                let text = source.get_text_in(node.span());
                let mut level = 0;
                for ch in text.chars() {
                    if ch == '=' {
                        level += 1;
                    }
                    else {
                        break;
                    }
                }
                let content_text = text.trim_start_matches('=').trim_start().to_string();
                let mut content = TypstRoot::new(node.span());
                content.items.push(TypstItem::Text(content_text));

                Ok(Some(TypstItem::Heading(TypstHeading { level, content })))
            }
            TypstElementType::Math => {
                let mut root = TypstRoot::new(node.span());
                for child in node.children() {
                    if let Some(item) = self.build_tree(child, source)? {
                        root.items.push(item);
                    }
                }
                Ok(Some(TypstItem::Math(root)))
            }
            TypstElementType::Strong => {
                let mut root = TypstRoot::new(node.span());
                for child in node.children() {
                    if let Some(item) = self.build_tree(child, source)? {
                        root.items.push(item);
                    }
                }
                Ok(Some(TypstItem::Strong(root)))
            }
            TypstElementType::Emphasis => {
                let mut root = TypstRoot::new(node.span().into());
                for child in node.children() {
                    if let Some(item) = self.build_tree(child, source)? {
                        root.items.push(item);
                    }
                }
                Ok(Some(TypstItem::Emphasis(root)))
            }
            TypstElementType::Quote => {
                let mut root = TypstRoot::new(node.span().into());
                for child in node.children() {
                    if let Some(item) = self.build_tree(child, source)? {
                        root.items.push(item);
                    }
                }
                Ok(Some(TypstItem::Quote(root)))
            }
            TypstElementType::ListItem => {
                let mut root = TypstRoot::new(node.span().into());
                for child in node.children() {
                    if let Some(item) = self.build_tree(child, source)? {
                        root.items.push(item);
                    }
                }
                Ok(Some(TypstItem::ListItem(root)))
            }
            TypstElementType::EnumItem => {
                let mut root = TypstRoot::new(node.span().into());
                for child in node.children() {
                    if let Some(item) = self.build_tree(child, source)? {
                        root.items.push(item);
                    }
                }
                Ok(Some(TypstItem::EnumItem(root)))
            }
            TypstElementType::Raw => Ok(Some(TypstItem::Raw(source.get_text_in(node.span()).to_string()))),
            _ => Ok(None),
        }
    }
}
