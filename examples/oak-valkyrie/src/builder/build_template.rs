use crate::{
    ValkyrieLanguage, ValkyrieParser,
    ast::{Statement, StatementNode, TemplateIf, TemplateIfArm, TemplateLoop, TemplateMatch, TemplateMatchArm, TemplateNode, TemplateTokenStream},
    builder::ValkyrieBuilder,
    lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType},
    parser::{
        element_type::ValkyrieElementType,
        template_directive::{directive_first_keyword, directive_payload_tokens, is_else_if_directive, parse_template_directive},
    },
};
use oak_core::{OakError, Parser, RedNode, RedTree, Source, SourceText, Token, TokenType, parser::session::ParseSession};

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_template_item<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<StatementNode, OakError> {
        Ok(StatementNode::Template(Box::new(self.build_template_node(node, source)?)))
    }

    pub(crate) fn build_template_statement<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Statement, OakError> {
        Ok(Statement::Template(Box::new(self.build_template_node(node, source)?)))
    }

    fn build_template_node<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TemplateNode, OakError> {
        match node.green.kind {
            ValkyrieElementType::TemplateIfStatement => {
                let mut arms = Vec::new();
                for child in node.children() {
                    if let RedTree::Node(arm_node) = child
                        && arm_node.green.kind == ValkyrieElementType::TemplateIfFragment
                    {
                        arms.push(self.build_template_if_arm(arm_node, source)?);
                    }
                }
                Ok(TemplateNode::If(TemplateIf { arms, span: node.span() }))
            }
            ValkyrieElementType::TemplateLoop => {
                let header = fragment_header_tokens(&node, ValkyrieElementType::TemplateLoopFragment);
                let body = self.build_template_body_items(&node, source)?;
                Ok(TemplateNode::Loop(TemplateLoop { header, body, span: node.span() }))
            }
            ValkyrieElementType::TemplateMatch => {
                let header = fragment_header_tokens(&node, ValkyrieElementType::TemplateMatchFragment);
                let mut arms = Vec::new();
                for child in node.children() {
                    if let RedTree::Node(arm_node) = child {
                        match arm_node.green.kind {
                            ValkyrieElementType::TemplateCaseFragment => {
                                arms.push(self.build_template_case_arm(arm_node, source)?);
                            }
                            ValkyrieElementType::TemplateElseFragment => {
                                arms.push(self.build_template_else_arm(arm_node, source)?);
                            }
                            _ => {}
                        }
                    }
                }
                Ok(TemplateNode::Match(TemplateMatch { header, arms, span: node.span() }))
            }
            ValkyrieElementType::TemplateStatement => Ok(TemplateNode::Fragment { tokens: directive_tokens_from_node(&node), span: node.span() }),
            _ => Err(source.syntax_error(format!("Unexpected template node: {:?}", node.green.kind), node.span().start)),
        }
    }

    fn build_template_if_arm<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TemplateIfArm, OakError> {
        let directive = parse_template_directive(&directive_tokens_from_node(&node));
        let header = if directive_first_keyword(&directive.tokens) == Some(ValkyrieKeywords::Else) && !is_else_if_directive(&directive.tokens) { Vec::new() } else { directive_payload_tokens(&directive) };
        let body = self.build_template_body_items(&node, source)?;
        Ok(TemplateIfArm { header, body, span: node.span() })
    }

    fn build_template_case_arm<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TemplateMatchArm, OakError> {
        let directive = parse_template_directive(&directive_tokens_from_node(&node));
        let pattern = Some(directive_payload_tokens(&directive));
        let body = self.build_template_body_items(&node, source)?;
        Ok(TemplateMatchArm { pattern, body, span: node.span() })
    }

    fn build_template_else_arm<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TemplateMatchArm, OakError> {
        let body = self.build_template_body_items(&node, source)?;
        Ok(TemplateMatchArm { pattern: None, body, span: node.span() })
    }

    fn build_template_body_items<S: Source + ?Sized>(&self, node: &RedNode<ValkyrieLanguage>, source: &S) -> Result<Vec<StatementNode>, OakError> {
        let mut items = Vec::new();
        for child in node.children() {
            if let RedTree::Node(child_node) = child {
                if matches!(child_node.green.kind, ValkyrieElementType::TemplateIfStatement | ValkyrieElementType::TemplateLoop | ValkyrieElementType::TemplateMatch | ValkyrieElementType::TemplateStatement) {
                    items.push(self.build_template_item(child_node, source)?);
                    continue;
                }
                if child_node.green.kind == ValkyrieElementType::TemplateText {
                    items.extend(self.build_items_from_template_text(child_node, source)?);
                }
            }
        }
        Ok(items)
    }

    fn build_items_from_template_text<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<Vec<StatementNode>, OakError> {
        let text = crate::builder::text(source, node.span());
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        let nested_source = SourceText::new(text);
        let parser = ValkyrieParser::new(self.config);
        let mut parse_cache = ParseSession::<ValkyrieLanguage>::default();
        let parse_output = parser.parse(&nested_source, &[], &mut parse_cache);
        let green_tree = parse_output.result?;
        self.build_root(green_tree, &nested_source).map(|root| root.items)
    }
}

fn fragment_header_tokens(node: &RedNode<ValkyrieLanguage>, fragment_kind: ValkyrieElementType) -> TemplateTokenStream {
    for child in node.children() {
        if let RedTree::Node(fragment) = child
            && fragment.green.kind == fragment_kind
        {
            let directive = parse_template_directive(&directive_tokens_from_node(&fragment));
            return directive_payload_tokens(&directive);
        }
    }
    Vec::new()
}

fn directive_tokens_from_node(node: &RedNode<ValkyrieLanguage>) -> TemplateTokenStream {
    let mut in_directive = false;
    let mut tokens = Vec::new();
    for child in node.children() {
        if let RedTree::Leaf(token) = child {
            if token.kind == ValkyrieTokenType::TemplateL {
                in_directive = true;
                continue;
            }
            if token.kind == ValkyrieTokenType::TemplateR {
                break;
            }
            if in_directive && !token.kind.is_ignored() {
                tokens.push(Token { kind: token.kind, span: token.span });
            }
        }
    }
    tokens
}
