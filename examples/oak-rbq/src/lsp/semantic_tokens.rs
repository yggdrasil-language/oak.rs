use crate::{language::RbqLanguage, lexer::token_type::RbqTokenType, parser::element_type::RbqElementType};
use oak_core::{
    TokenType,
    language::UniversalTokenRole,
    source::Source,
    tree::{RedNode, RedTree},
};
use oak_semantic_tokens::{SemanticToken, SemanticTokensProvider};
use oak_lsp::LineMap;

/// RBQ semantic tokens provider.
pub struct RbqSemanticTokensProvider;

impl SemanticTokensProvider<RbqLanguage> for RbqSemanticTokensProvider {
    fn semantic_tokens(&self, root: &RedNode<RbqLanguage>, source: &dyn Source, line_map: &LineMap) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        let mut last_line = 0;
        let mut last_start = 0;
        self.collect_semantic_tokens(root, source, line_map, &mut tokens, &mut last_line, &mut last_start, None);
        tokens
    }
}

impl RbqSemanticTokensProvider {
    fn collect_semantic_tokens<S: Source + ?Sized>(&self, node: &RedNode<RbqLanguage>, source: &S, line_map: &LineMap, tokens: &mut Vec<SemanticToken>, last_line: &mut u32, last_start: &mut u32, parent_kind: Option<RbqElementType>) {
        for child in node.children() {
            match child {
                RedTree::Leaf(leaf) => {
                    if leaf.kind == RbqTokenType::Whitespace || leaf.kind == RbqTokenType::Newline {
                        continue;
                    }

                    let role = leaf.kind.role();
                    let mut token_type = match role {
                        UniversalTokenRole::Keyword => 4, // keyword
                        UniversalTokenRole::Literal => {
                            match leaf.kind {
                                RbqTokenType::StringLiteral => 5, // string
                                RbqTokenType::NumberLiteral => 6, // number
                                _ => 5,
                            }
                        }
                        UniversalTokenRole::Comment => 15, // comment
                        UniversalTokenRole::Operator => 7, // operator
                        UniversalTokenRole::Name => 2,     // variable
                        _ => {
                            if leaf.kind == RbqTokenType::Ident {
                                2 // variable
                            }
                            else {
                                continue;
                            }
                        }
                    };

                    // Contextual overrides for identifiers
                    if leaf.kind == RbqTokenType::Ident {
                        if let Some(pk) = parent_kind {
                            token_type = match pk {
                                RbqElementType::NAMESPACE_DEFINITION => 8, // namespace
                                RbqElementType::STRUCT_DEFINITION => 9,    // struct
                                RbqElementType::ENUM_DEFINITION => 10,     // enum
                                RbqElementType::TYPE_REFERENCE => 12,      // type
                                RbqElementType::ANNOTATION => 13,          // decorator
                                RbqElementType::FIELD_DEFINITION => 2,     // variable
                                RbqElementType::ENUM_VARIANT => 3,         // function (using function for enum variant for now)
                                _ => 2,
                            };
                        }
                    }

                    let span = leaf.span;
                    let (line, start) = line_map.offset_to_line_col_utf16(source, span.start);

                    let delta_line = line - *last_line;
                    let delta_start = if delta_line == 0 { start - *last_start } else { start };

                    tokens.push(SemanticToken { delta_line, delta_start, length: (span.end - span.start) as u32, token_type, token_modifiers_bitset: 0 });

                    *last_line = line;
                    *last_start = start;
                }
                RedTree::Node(n) => {
                    let current_kind = n.green.kind;
                    self.collect_semantic_tokens(&n, source, line_map, tokens, last_line, last_start, Some(current_kind));
                }
            }
        }
    }
}
