use crate::{
    MapleElementType, MapleLanguage, MapleParser, MapleTokenType,
    ast::{MapleExpression, MapleExpressionKind, MapleOperator, MapleRoot, MapleSpan},
};
use oak_core::{Builder, BuilderCache, OakDiagnostics, OakError, Parser, RedNode, RedTree, Source, TextEdit, TokenType, builder::BuildOutput};

/// Builds owned Maple syntax from Oak's lossless tree.
#[derive(Debug, Clone)]
pub struct MapleBuilder<'config> {
    config: &'config MapleLanguage,
}
impl<'config> MapleBuilder<'config> {
    /// Creates a syntax builder.
    pub fn new(config: &'config MapleLanguage) -> Self {
        Self { config }
    }
}

impl Builder<MapleLanguage> for MapleBuilder<'_> {
    fn build<'a, SourceType: Source + ?Sized>(&self, source: &SourceType, edits: &[TextEdit], cache: &'a mut impl BuilderCache<MapleLanguage>) -> BuildOutput<MapleLanguage> {
        let output = MapleParser::new(self.config).parse(source, edits, cache);
        let diagnostics = output.diagnostics;
        if let Some(error) = diagnostics.first() {
            return OakDiagnostics { result: Err(error.clone()), diagnostics };
        }
        let result = output.result.and_then(|green| {
            let root = RedNode::new(green, 0);
            let expressions = root
                .children()
                .filter_map(|child| match child {
                    RedTree::Node(node) => Some(build_expression(node, source)),
                    _ => None,
                })
                .collect::<Result<Vec<_>, _>>()?;
            let span = root.span();
            Ok(MapleRoot { expressions, span: MapleSpan { start: span.start, end: span.end } })
        });
        OakDiagnostics { result, diagnostics }
    }
}

fn build_expression<SourceType: Source + ?Sized>(node: RedNode<'_, MapleLanguage>, source: &SourceType) -> Result<MapleExpression, OakError> {
    let range = node.span();
    let mut expression_span = None;
    let invalid = || source.syntax_error("Invalid Maple syntax tree".into(), range.start);
    let mut expressions = Vec::new();
    let mut operator = None;
    let mut spelling = None;
    let mut arguments = Vec::new();
    for child in node.children() {
        match child {
            RedTree::Node(child) if child.element_type() == MapleElementType::Arguments => {
                for argument in child.children() {
                    match argument {
                        RedTree::Node(argument) => {
                            let argument = build_expression(argument, source)?;
                            extend_span(&mut expression_span, argument.span.start, argument.span.end);
                            arguments.push(argument);
                        }
                        RedTree::Leaf(token) if !token.kind().is_ignored() => {
                            let range = token.span();
                            extend_span(&mut expression_span, range.start, range.end);
                        }
                        _ => {}
                    }
                }
            }
            RedTree::Node(child) => {
                let child = build_expression(child, source)?;
                extend_span(&mut expression_span, child.span.start, child.span.end);
                expressions.push(child);
            }
            RedTree::Leaf(token) if !token.kind().is_ignored() => {
                let range = token.span();
                extend_span(&mut expression_span, range.start, range.end);
                match token.kind() {
                    MapleTokenType::Identifier | MapleTokenType::Integer | MapleTokenType::Float => spelling = Some((token.kind(), source.get_text_in(token.span()).to_string())),
                    MapleTokenType::Plus => operator = Some(MapleOperator::Add),
                    MapleTokenType::Minus => operator = Some(MapleOperator::Subtract),
                    MapleTokenType::Times => operator = Some(MapleOperator::Multiply),
                    MapleTokenType::Divide => operator = Some(MapleOperator::Divide),
                    MapleTokenType::Power => operator = Some(MapleOperator::Power),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    let span = expression_span.ok_or_else(invalid)?;
    let kind = match node.element_type() {
        MapleElementType::Symbol => MapleExpressionKind::Symbol(spelling.ok_or_else(invalid)?.1),
        MapleElementType::Literal => {
            let (token, text) = spelling.ok_or_else(invalid)?;
            if token == MapleTokenType::Integer { MapleExpressionKind::Integer(text) } else { MapleExpressionKind::Decimal(text) }
        }
        MapleElementType::Call => MapleExpressionKind::Call { name: spelling.ok_or_else(invalid)?.1, arguments },
        MapleElementType::List => MapleExpressionKind::List(expressions),
        MapleElementType::Expression if expressions.len() == 1 => MapleExpressionKind::Grouped(Box::new(expressions.remove(0))),
        MapleElementType::PrefixExpr if expressions.len() == 1 => MapleExpressionKind::Unary { operator: operator.ok_or_else(invalid)?, operand: Box::new(expressions.remove(0)) },
        MapleElementType::BinaryExpr if expressions.len() == 2 => {
            let right = expressions.pop().ok_or_else(invalid)?;
            let left = expressions.pop().ok_or_else(invalid)?;
            MapleExpressionKind::Binary { operator: operator.ok_or_else(invalid)?, left: Box::new(left), right: Box::new(right) }
        }
        _ => return Err(invalid()),
    };
    Ok(MapleExpression { kind, span })
}

fn extend_span(span: &mut Option<MapleSpan>, start: usize, end: usize) {
    match span {
        Some(span) => {
            span.start = span.start.min(start);
            span.end = span.end.max(end);
        }
        None => *span = Some(MapleSpan { start, end }),
    }
}
