use crate::{language::TypeScriptLanguage, lexer::token_type::TypeScriptTokenType};
use core::range::Range;
use oak_core::{RedNode, RedTree, Source, SourceText, TokenType};

/// A pattern token: its kind and span inside the authoritative source.
struct PatternToken {
    kind: TypeScriptTokenType,
    span: Range<usize>,
}

/// Collects the binding identifiers introduced by a `BindingPattern` node.
///
/// The parser records the pattern as a flat token run, so the pattern grammar
/// (`{ a, b: c, ...rest }` / `[x, , ...ys]`, nested patterns and defaults) is
/// walked here over the CST tokens instead of re-reading declarator source text.
/// Each entry pairs the bound name with the source span of its token.
pub(crate) fn binding_pattern_names(node: &RedNode<TypeScriptLanguage>, source: &SourceText) -> Vec<(String, Range<usize>)> {
    let mut tokens = Vec::new();
    collect_pattern_tokens(node, &mut tokens);
    let mut names = Vec::new();
    let mut cursor = 0usize;
    parse_binding_target(&tokens, &mut cursor, source, &mut names);
    names
}

/// Reads the binding identifier text and token span from an `IdentifierName`
/// node. The span is zero-width at the node start when no token is found.
pub(crate) fn binding_identifier_text(node: &RedNode<TypeScriptLanguage>, source: &SourceText) -> (String, Range<usize>) {
    for child in node.children() {
        if let RedTree::Leaf(leaf) = child {
            if !leaf.kind.is_ignored() {
                let span: Range<usize> = leaf.span.into();
                return (source.get_text_in(span.clone()).trim().to_string(), span);
            }
        }
    }
    let node_span: Range<usize> = node.span().into();
    (String::new(), Range { start: node_span.start, end: node_span.start })
}

fn collect_pattern_tokens(node: &RedNode<TypeScriptLanguage>, tokens: &mut Vec<PatternToken>) {
    for child in node.children() {
        match child {
            RedTree::Leaf(leaf) => {
                if !leaf.kind.is_ignored() {
                    tokens.push(PatternToken { kind: leaf.kind, span: leaf.span.into() });
                }
            }
            RedTree::Node(child_node) => collect_pattern_tokens(&child_node, tokens),
        }
    }
}

fn token_text(token: &PatternToken, source: &SourceText) -> String {
    source.get_text_in(token.span.clone()).trim().to_string()
}

fn is_binding_identifier(token: &PatternToken, source: &SourceText) -> bool {
    token.kind == TypeScriptTokenType::IdentifierName || TypeScriptTokenType::from_keyword(&token_text(token, source)).is_some()
}

fn kind_at(tokens: &[PatternToken], cursor: usize) -> Option<TypeScriptTokenType> {
    tokens.get(cursor).map(|token| token.kind)
}

/// Parses one binding target: a plain identifier, an object pattern or an
/// array pattern. The cursor advances past the whole target.
fn parse_binding_target(tokens: &[PatternToken], cursor: &mut usize, source: &SourceText, names: &mut Vec<(String, Range<usize>)>) {
    match kind_at(tokens, *cursor) {
        Some(TypeScriptTokenType::LeftBrace) => parse_object_pattern(tokens, cursor, source, names),
        Some(TypeScriptTokenType::LeftBracket) => parse_array_pattern(tokens, cursor, source, names),
        Some(_) => {
            if let Some(token) = tokens.get(*cursor) {
                if is_binding_identifier(token, source) {
                    names.push((token_text(token, source), token.span.clone()));
                }
            }
            *cursor += 1;
        }
        None => {}
    }
}

fn parse_object_pattern(tokens: &[PatternToken], cursor: &mut usize, source: &SourceText, names: &mut Vec<(String, Range<usize>)>) {
    *cursor += 1; // `{`
    while let Some(kind) = kind_at(tokens, *cursor) {
        match kind {
            TypeScriptTokenType::Comma => *cursor += 1,
            TypeScriptTokenType::RightBrace => {
                *cursor += 1;
                return;
            }
            TypeScriptTokenType::DotDotDot => {
                *cursor += 1;
                parse_binding_target(tokens, cursor, source, names);
                skip_default_value(tokens, cursor);
            }
            TypeScriptTokenType::LeftBracket => {
                skip_balanced(tokens, cursor, TypeScriptTokenType::LeftBracket, TypeScriptTokenType::RightBracket);
                if kind_at(tokens, *cursor) == Some(TypeScriptTokenType::Colon) {
                    *cursor += 1;
                    parse_binding_target(tokens, cursor, source, names);
                }
                skip_default_value(tokens, cursor);
            }
            _ => {
                let is_identifier = tokens.get(*cursor).is_some_and(|token| is_binding_identifier(token, source));
                let key = tokens.get(*cursor).map(|token| token_text(token, source)).unwrap_or_default();
                let key_span = tokens.get(*cursor).map(|token| token.span.clone());
                *cursor += 1;
                if kind_at(tokens, *cursor) == Some(TypeScriptTokenType::Colon) {
                    *cursor += 1;
                    parse_binding_target(tokens, cursor, source, names);
                }
                else if is_identifier {
                    if let Some(span) = key_span {
                        names.push((key, span));
                    }
                }
                skip_default_value(tokens, cursor);
            }
        }
    }
}

fn parse_array_pattern(tokens: &[PatternToken], cursor: &mut usize, source: &SourceText, names: &mut Vec<(String, Range<usize>)>) {
    *cursor += 1; // `[`
    while let Some(kind) = kind_at(tokens, *cursor) {
        match kind {
            TypeScriptTokenType::Comma => *cursor += 1,
            TypeScriptTokenType::RightBracket => {
                *cursor += 1;
                return;
            }
            TypeScriptTokenType::DotDotDot => {
                *cursor += 1;
                parse_binding_target(tokens, cursor, source, names);
                skip_default_value(tokens, cursor);
            }
            _ => {
                parse_binding_target(tokens, cursor, source, names);
                skip_default_value(tokens, cursor);
            }
        }
    }
}

/// Skips a `= <expression>` default value up to the enclosing pattern's
/// separator, tracking nested brackets and parentheses.
fn skip_default_value(tokens: &[PatternToken], cursor: &mut usize) {
    if kind_at(tokens, *cursor) != Some(TypeScriptTokenType::Equal) {
        return;
    }
    *cursor += 1;
    let mut depth = 0usize;
    while let Some(kind) = kind_at(tokens, *cursor) {
        match kind {
            TypeScriptTokenType::LeftParen | TypeScriptTokenType::LeftBracket | TypeScriptTokenType::LeftBrace => depth += 1,
            TypeScriptTokenType::RightParen | TypeScriptTokenType::RightBracket | TypeScriptTokenType::RightBrace => {
                if depth == 0 {
                    return;
                }
                depth -= 1;
            }
            TypeScriptTokenType::Comma if depth == 0 => return,
            _ => {}
        }
        *cursor += 1;
    }
}

/// Skips a balanced `open` … `close` run including the opening token.
fn skip_balanced(tokens: &[PatternToken], cursor: &mut usize, open: TypeScriptTokenType, close: TypeScriptTokenType) {
    if kind_at(tokens, *cursor) != Some(open) {
        return;
    }
    let mut depth = 0usize;
    while let Some(kind) = kind_at(tokens, *cursor) {
        *cursor += 1;
        if kind == open {
            depth += 1;
        }
        else if kind == close {
            depth -= 1;
            if depth == 0 {
                return;
            }
        }
    }
}
