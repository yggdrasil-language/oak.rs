#![doc = include_str!("readme.md")]

use crate::{ast::*, language::DotLanguage, lexer::token_type::DotTokenType, parser::element_type::DotElementType};
use oak_core::{Builder, BuilderCache, GreenNode, OakDiagnostics, OakError, Parser, RedNode, RedTree, SourceText, TextEdit, source::Source};

/// Builder for converting a DOT syntax tree into the typed AST.
pub struct DotBuilder<'config> {
    /// The DOT language configuration.
    config: &'config DotLanguage,
}

impl<'config> DotBuilder<'config> {
    /// Creates a new `DotBuilder` with the given configuration.
    pub fn new(config: &'config DotLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<DotLanguage> for DotBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<DotLanguage>) -> OakDiagnostics<DotRoot> {
        let parser = crate::parser::DotParser::new(self.config);
        let mut cache = oak_core::parser::session::ParseSession::<DotLanguage>::default();
        let parse_result = parser.parse(source, edits, &mut cache);

        match parse_result.result {
            Ok(green_tree) => {
                let source_text = SourceText::new(source.get_text_in((0..source.length()).into()).into_owned());
                let mut diagnostics = parse_result.diagnostics;
                match self.build_root(green_tree, &source_text, &mut diagnostics) {
                    Ok(ast_root) => OakDiagnostics { result: Ok(ast_root), diagnostics },
                    Err(build_error) => {
                        diagnostics.push(build_error.clone());
                        OakDiagnostics { result: Err(build_error), diagnostics }
                    }
                }
            }
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics: parse_result.diagnostics },
        }
    }
}

impl DotBuilder<'_> {
    /// Builds the AST root from a green tree.
    pub(crate) fn build_root(&self, green_tree: &GreenNode<DotLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Result<DotRoot, OakError> {
        if green_tree.kind != DotElementType::Root {
            return Err(OakError::internal_error("the DOT syntax tree is missing its root element"));
        }

        let red_root = RedNode::new(green_tree, 0);
        let mut graphs = Vec::new();

        for child in red_root.children() {
            if let RedTree::Node(node) = child {
                if node.green.kind == DotElementType::GraphDefinition {
                    graphs.push(self.build_graph(node, source, diagnostics));
                }
                else {
                    diagnostics.push(OakError::syntax_error("unexpected element at the top level", node.span().start, source.source_id()));
                }
            }
        }

        Ok(DotRoot { graphs })
    }

    /// Builds one graph definition.
    fn build_graph(&self, node: RedNode<DotLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Graph {
        let span = node.span();
        let mut strict = false;
        let mut graph_type = GraphType::Graph;
        let mut id = None;
        let mut statements = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(leaf) => match leaf.kind {
                    DotTokenType::Strict => strict = true,
                    DotTokenType::Digraph => graph_type = GraphType::Digraph,
                    DotTokenType::Graph => graph_type = GraphType::Graph,
                    DotTokenType::Identifier | DotTokenType::String | DotTokenType::Number if id.is_none() => {
                        id = Some(Self::value(&leaf.text(source)));
                    }
                    _ => {}
                },
                RedTree::Node(child) => {
                    if let Some(statement) = self.build_statement(child, source, diagnostics) {
                        statements.push(statement);
                    }
                }
            }
        }

        Graph { strict, graph_type, id, statements, span }
    }

    /// Builds one statement, skipping malformed pieces with a diagnostic instead of panicking.
    fn build_statement(&self, node: RedNode<DotLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Option<Statement> {
        match node.green.kind {
            DotElementType::GraphNode => self.build_node_statement(node, source, diagnostics).map(Statement::Node),
            DotElementType::GraphEdge => self.build_edge_statement(node, source, diagnostics).map(Statement::Edge),
            DotElementType::GraphAttribute => self.build_attribute_statement(node, source).map(Statement::Attribute),
            DotElementType::GraphAssignment => self.build_assignment(node, source).map(Statement::Assignment),
            DotElementType::GraphSubgraph => Some(Statement::Subgraph(self.build_subgraph(node, source, diagnostics))),
            kind => {
                diagnostics.push(OakError::syntax_error(format!("unsupported statement element: {kind:?}"), node.span().start, source.source_id()));
                None
            }
        }
    }

    /// Builds a node statement.
    fn build_node_statement(&self, node: RedNode<DotLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Option<NodeStatement> {
        let span = node.span();
        let mut node_id = None;
        let mut attributes = Vec::new();

        for child in node.children() {
            if let RedTree::Node(child) = child {
                match child.green.kind {
                    DotElementType::NodeId if node_id.is_none() => node_id = Some(self.build_node_id(child, source, diagnostics)),
                    DotElementType::AttributeList => attributes.extend(self.build_attribute_list(child, source)),
                    _ => {}
                }
            }
        }

        let Some(node_id) = node_id
        else {
            diagnostics.push(OakError::syntax_error("a node statement is missing its identifier", span.start, source.source_id()));
            return None;
        };

        let record = self.build_record(&attributes, source, diagnostics);

        Some(NodeStatement { node_id, attributes, record, span })
    }

    /// Parses a `record` or `mrecord` label into fields and brace groups.
    ///
    /// Only node shapes named `record` or `mrecord` are structured. The raw
    /// `label` attribute is left untouched so callers can still read the
    /// original text.
    fn build_record(&self, attributes: &[Attribute], source: &SourceText, diagnostics: &mut Vec<OakError>) -> Option<RecordLabel> {
        let is_record = attributes.iter().any(|attribute| attribute.name.eq_ignore_ascii_case("shape") && matches!(attribute.value.as_deref(), Some(value) if value.eq_ignore_ascii_case("record") || value.eq_ignore_ascii_case("mrecord")));
        if !is_record {
            return None;
        }

        let label = attributes.iter().find(|attribute| attribute.name.eq_ignore_ascii_case("label"))?;
        let raw = source.get_text_in(label.span);
        let quote = raw.find('"')?;

        let mut escaped = false;
        let mut end = None;
        for (offset, ch) in raw[quote + 1..].char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => {
                    end = Some(quote + 1 + offset);
                    break;
                }
                _ => {}
            }
        }

        let Some(end) = end
        else {
            diagnostics.push(OakError::syntax_error("unterminated record label", label.span.start, source.source_id()));
            return None;
        };

        let content_start = label.span.start + quote + 1;
        let mut parser = RecordParser::new(&raw[quote + 1..end], content_start);
        let items = parser.parse_items(false);
        let span = core::range::Range { start: content_start, end: content_start + end - (quote + 1) };

        Some(RecordLabel { items, span })
    }

    /// Builds a node identifier with its optional port.
    fn build_node_id(&self, node: RedNode<DotLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> NodeId {
        let span = node.span();
        let mut id = None;
        let mut port = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(leaf) if id.is_none() && is_identifier_token(leaf.kind) => {
                    id = Some(Self::value(&leaf.text(source)));
                }
                RedTree::Node(child) if child.green.kind == DotElementType::Port => {
                    port = Some(self.build_port(child, source, diagnostics));
                }
                _ => {}
            }
        }

        NodeId { id: id.unwrap_or_default(), port, span }
    }

    /// Builds a port suffix, normalizing compass points and reporting unknown ones.
    fn build_port(&self, node: RedNode<DotLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Port {
        let span = node.span();
        let mut segments = Vec::new();

        for child in node.children() {
            if let RedTree::Leaf(leaf) = child {
                if is_identifier_token(leaf.kind) {
                    segments.push((leaf.text(source).into_owned(), leaf.span.start));
                }
            }
        }

        let mut id = None;
        let mut compass = None;

        match segments.as_slice() {
            [] => {}
            [(text, _)] => {
                if let Some(point) = compass_point(text) {
                    compass = Some(point);
                }
                else {
                    id = Some(Self::value(text));
                }
            }
            [(first, _), (second, offset), ..] => {
                id = Some(Self::value(first));
                if let Some(point) = compass_point(second) {
                    compass = Some(point);
                }
                else {
                    diagnostics.push(OakError::syntax_error(format!("`{}` is not a compass point", second.trim()), *offset, source.source_id()));
                }
            }
        }

        Port { id, compass, span }
    }

    /// Builds an edge statement, folding chained hops into ordered `(EdgeOp, operand)` pairs.
    fn build_edge_statement(&self, node: RedNode<DotLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Option<EdgeStatement> {
        let span = node.span();
        let mut left = None;
        let mut edges = Vec::new();
        let mut attributes = Vec::new();
        let mut pending = None;

        for child in node.children() {
            match child {
                RedTree::Leaf(leaf) => match leaf.kind {
                    DotTokenType::Arrow => pending = Some(EdgeOp::Directed),
                    DotTokenType::Line => pending = Some(EdgeOp::Undirected),
                    _ => {}
                },
                RedTree::Node(child) => match child.green.kind {
                    DotElementType::NodeId => {
                        let operand = EdgeOperand::Node(self.build_node_id(child, source, diagnostics));
                        Self::push_operand(&mut left, &mut edges, &mut pending, operand);
                    }
                    DotElementType::GraphSubgraph => {
                        let operand = EdgeOperand::Subgraph(self.build_subgraph(child, source, diagnostics));
                        Self::push_operand(&mut left, &mut edges, &mut pending, operand);
                    }
                    DotElementType::AttributeList => attributes.extend(self.build_attribute_list(child, source)),
                    _ => {}
                },
            }
        }

        if pending.is_some() {
            diagnostics.push(OakError::syntax_error("an edge operator has no right-hand operand", span.end, source.source_id()));
        }

        let Some(left) = left
        else {
            diagnostics.push(OakError::syntax_error("an edge statement is missing its left operand", span.start, source.source_id()));
            return None;
        };

        Some(EdgeStatement { left, edges, attributes, span })
    }

    /// Appends an edge operand, using the pending operator when one exists.
    fn push_operand(left: &mut Option<EdgeOperand>, edges: &mut Vec<(EdgeOp, EdgeOperand)>, pending: &mut Option<EdgeOp>, operand: EdgeOperand) {
        match pending.take() {
            Some(op) => edges.push((op, operand)),
            None => {
                if left.is_none() {
                    *left = Some(operand);
                }
            }
        }
    }

    /// Builds a `graph`/`node`/`edge` attribute statement.
    fn build_attribute_statement(&self, node: RedNode<DotLanguage>, source: &SourceText) -> Option<AttributeStatement> {
        let span = node.span();
        let mut target = None;
        let mut attributes = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(leaf) => match leaf.kind {
                    DotTokenType::Graph | DotTokenType::Digraph => target = Some(AttributeTarget::Graph),
                    DotTokenType::Node => target = Some(AttributeTarget::Node),
                    DotTokenType::Edge => target = Some(AttributeTarget::Edge),
                    _ => {}
                },
                RedTree::Node(child) if child.green.kind == DotElementType::AttributeList => {
                    attributes.extend(self.build_attribute_list(child, source));
                }
                _ => {}
            }
        }

        target.map(|target| AttributeStatement { target, attributes, span })
    }

    /// Builds a subgraph, including anonymous subgraphs used as edge operands.
    fn build_subgraph(&self, node: RedNode<DotLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> SubgraphStatement {
        let span = node.span();
        let mut id = None;
        let mut statements = Vec::new();

        for child in node.children() {
            match child {
                RedTree::Leaf(leaf) if id.is_none() && is_plain_identifier_token(leaf.kind) => {
                    id = Some(Self::value(&leaf.text(source)));
                }
                RedTree::Node(child) => {
                    if let Some(statement) = self.build_statement(child, source, diagnostics) {
                        statements.push(statement);
                    }
                }
                _ => {}
            }
        }

        SubgraphStatement { id, statements, span }
    }

    /// Builds a bare `key = value` assignment.
    fn build_assignment(&self, node: RedNode<DotLanguage>, source: &SourceText) -> Option<AssignmentStatement> {
        let span = node.span();
        let mut values = node.children().filter_map(|child| match child {
            RedTree::Leaf(leaf) if is_identifier_token(leaf.kind) => Some(Self::value(&leaf.text(source))),
            _ => None,
        });

        match (values.next(), values.next()) {
            (Some(id), Some(value)) => Some(AssignmentStatement { id, value, span }),
            _ => None,
        }
    }

    /// Builds every attribute in one bracketed attribute list.
    fn build_attribute_list(&self, node: RedNode<DotLanguage>, source: &SourceText) -> Vec<Attribute> {
        let mut attributes = Vec::new();
        let mut name: Option<(String, usize, usize)> = None;
        let mut value: Option<(String, usize)> = None;
        let mut expecting_value = false;

        for child in node.children() {
            let RedTree::Leaf(leaf) = child
            else {
                continue;
            };

            match leaf.kind {
                DotTokenType::Equal => expecting_value = true,
                DotTokenType::Comma | DotTokenType::Semicolon => {
                    flush_attribute(&mut attributes, &mut name, &mut value);
                    expecting_value = false;
                }
                kind if is_identifier_token(kind) => {
                    let span = leaf.span;
                    let text = Self::value(&leaf.text(source));

                    if expecting_value && name.is_some() {
                        value = Some((text, span.end));
                        expecting_value = false;
                    }
                    else {
                        flush_attribute(&mut attributes, &mut name, &mut value);
                        name = Some((text, span.start, span.end));
                    }
                }
                _ => {}
            }
        }

        flush_attribute(&mut attributes, &mut name, &mut value);
        attributes
    }

    /// Normalizes a leaf's text into an AST value: trims trivia and unquotes strings.
    fn value(text: &str) -> String {
        let text = text.trim();
        let inner = if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') { &text[1..text.len() - 1] } else { text };

        let mut result = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            }
            else {
                result.push(ch);
            }
        }
        result
    }
}

/// Returns `true` if the token can act as a DOT identifier.
fn is_identifier_token(kind: DotTokenType) -> bool {
    matches!(kind, DotTokenType::Identifier | DotTokenType::String | DotTokenType::Number | DotTokenType::Graph | DotTokenType::Digraph | DotTokenType::Subgraph | DotTokenType::Node | DotTokenType::Edge | DotTokenType::Strict)
}

/// Returns `true` if the token is an identifier that cannot be a keyword.
fn is_plain_identifier_token(kind: DotTokenType) -> bool {
    matches!(kind, DotTokenType::Identifier | DotTokenType::String | DotTokenType::Number)
}

/// Maps a case-insensitive compass keyword to its AST value.
fn compass_point(text: &str) -> Option<Compass> {
    Some(match text.trim().to_ascii_uppercase().as_str() {
        "N" => Compass::N,
        "NE" => Compass::NE,
        "E" => Compass::E,
        "SE" => Compass::SE,
        "S" => Compass::S,
        "SW" => Compass::SW,
        "W" => Compass::W,
        "NW" => Compass::NW,
        "C" => Compass::C,
        _ => return None,
    })
}

/// Emits the pending attribute, giving valueless attributes an exact name span.
fn flush_attribute(attributes: &mut Vec<Attribute>, name: &mut Option<(String, usize, usize)>, value: &mut Option<(String, usize)>) {
    if let Some((name, start, end)) = name.take() {
        let (value, end) = match value.take() {
            Some((value, end)) => (Some(value), end),
            None => (None, end),
        };
        attributes.push(Attribute { name, value, span: (start..end).into() });
    }
}

/// Splits record label text into fields and brace groups.
///
/// The grammar is a flat sequence of items separated by unescaped `|`. A
/// leading `{` opens a group whose own items are separated the same way, so
/// nesting is arbitrary. `<name>` before the field text is its port. A
/// backslash escapes the next character, which keeps `\|`, `\{` and `\<`
/// inside field text instead of acting as separators.
struct RecordParser<'text> {
    text: &'text str,
    position: usize,
    base: usize,
}

impl<'text> RecordParser<'text> {
    fn new(text: &'text str, base: usize) -> Self {
        Self { text, position: 0, base }
    }

    fn peek(&self) -> Option<char> {
        self.text[self.position..].chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    fn parse_items(&mut self, in_group: bool) -> Vec<RecordItem> {
        let mut items = Vec::new();

        loop {
            let item_start = self.position;
            let mut text = String::new();
            let mut port = None;
            let mut group = None;

            while let Some(ch) = self.peek() {
                match ch {
                    '|' | '}' => break,
                    '\\' => {
                        self.next();
                        if let Some(escaped) = self.next() {
                            text.push(escaped);
                        }
                    }
                    '{' if text.trim().is_empty() && port.is_none() => {
                        let group_start = self.position;
                        self.next();
                        let nested = self.parse_items(true);
                        group = Some(RecordGroup { items: nested, span: self.span_from(group_start) });
                    }
                    '<' if text.trim().is_empty() && port.is_none() => {
                        self.next();
                        let mut name = String::new();
                        while let Some(inner) = self.next() {
                            if inner == '>' {
                                break;
                            }
                            if inner == '\\' {
                                if let Some(escaped) = self.next() {
                                    name.push(escaped);
                                }
                                continue;
                            }
                            name.push(inner);
                        }
                        port = Some(name.trim().to_string());
                    }
                    _ => {
                        text.push(ch);
                        self.next();
                    }
                }
            }

            let end = self.position;
            let span = self.span_between(item_start, end);

            if let Some(group) = group {
                items.push(RecordItem::Group(group));
            }
            else if port.is_some() || !text.trim().is_empty() {
                items.push(RecordItem::Field(RecordField { port, text: text.trim().to_string(), span }));
            }

            match self.peek() {
                Some('|') => {
                    self.next();
                }
                Some('}') => {
                    self.next();
                    if in_group {
                        break;
                    }
                }
                _ => break,
            }
        }

        items
    }

    fn span_from(&self, start: usize) -> core::range::Range<usize> {
        self.span_between(start, self.position)
    }

    fn span_between(&self, start: usize, end: usize) -> core::range::Range<usize> {
        core::range::Range { start: self.base + start, end: self.base + end }
    }
}
