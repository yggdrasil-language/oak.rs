use oak_core::{
    OakError, OakErrorKind, Parser, SourceText,
    parser::session::ParseSession,
    tree::{RedNode, RedTree},
};
use oak_dot::{DotElementType, DotLanguage, DotParser, DotTokenType};

/// Parses `source` and hands the red tree plus diagnostics to `check`.
fn parse<R>(source: &str, check: impl FnOnce(&RedNode<DotLanguage>, &[OakError]) -> R) -> R {
    let text = SourceText::new(source);
    let language = DotLanguage::default();
    let parser = DotParser::new(&language);
    let mut cache = ParseSession::<DotLanguage>::default();

    let output = parser.parse(&text, &[], &mut cache);
    let diagnostics = output.diagnostics;
    let green_tree = output.result.expect("the DOT source should still produce a green tree");
    let root = RedNode::new(&green_tree, 0);

    check(&root, &diagnostics)
}

/// Returns the composite children of a node.
fn child_nodes<'a>(node: &RedNode<'a, DotLanguage>) -> Vec<RedNode<'a, DotLanguage>> {
    node.children().filter_map(|child| child.as_node()).collect()
}

/// Returns the first node with the given element type, depth first.
fn find<'a>(node: &RedNode<'a, DotLanguage>, kind: DotElementType) -> Option<RedNode<'a, DotLanguage>> {
    let mut found = Vec::new();
    collect(node, kind, &mut found);
    found.into_iter().next()
}

/// Collects every node with the given element type, depth first.
fn collect<'a>(node: &RedNode<'a, DotLanguage>, kind: DotElementType, found: &mut Vec<RedNode<'a, DotLanguage>>) {
    for child in child_nodes(node) {
        if child.green.kind == kind {
            found.push(child);
        }
        collect(&child, kind, found);
    }
}

/// Counts leaves with the given token type below a node.
fn leaf_count(node: &RedNode<DotLanguage>, kind: DotTokenType) -> usize {
    node.children()
        .map(|child| match child {
            RedTree::Leaf(leaf) if leaf.kind == kind => 1,
            RedTree::Leaf(_) => 0,
            RedTree::Node(child) => leaf_count(&child, kind),
        })
        .sum()
}

/// Returns `true` if a syntax error message mentions `word`.
fn has_syntax_error(diagnostics: &[OakError], word: &str) -> bool {
    diagnostics.iter().any(|error| match error.kind() {
        OakErrorKind::SyntaxError { message, .. } => message.contains(word),
        _ => false,
    })
}

/// Returns `true` if a stray token was reported.
fn has_unexpected_token(diagnostics: &[OakError], token: &str) -> bool {
    diagnostics.iter().any(|error| matches!(error.kind(), OakErrorKind::UnexpectedToken { token: reported, .. } if reported == token))
}

#[test]
fn test_digraph_with_statements() {
    let source = "digraph G {\n  rankdir=LR;\n  A [label=\"Start\", shape=diamond];\n  A -> B [label=\"go\"];\n}\n";

    parse(source, |root, diagnostics| {
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        assert_eq!(root.green.kind, DotElementType::Root);

        let graphs = child_nodes(root);
        assert_eq!(graphs.len(), 1);

        let graph = graphs[0];
        assert_eq!(graph.green.kind, DotElementType::GraphDefinition);
        assert_eq!(leaf_count(&graph, DotTokenType::Digraph), 1);
        assert_eq!(graph.children().filter(|child| matches!(child, RedTree::Leaf(leaf) if leaf.kind == DotTokenType::Identifier)).count(), 1);

        let statements = child_nodes(&graph);
        let kinds: Vec<_> = statements.iter().map(|statement| statement.green.kind).collect();
        assert_eq!(kinds, [DotElementType::GraphAssignment, DotElementType::GraphNode, DotElementType::GraphEdge]);
    });
}

#[test]
fn test_node_statement_span_matches_source() {
    let source = "digraph { A [label=\"Start\", shape=diamond]; }";

    parse(source, |root, diagnostics| {
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

        let node = find(root, DotElementType::GraphNode).expect("the node statement should exist");
        let span = node.span();
        assert!(span.end > span.start);
        assert_eq!(&source[span.start..span.end], "A [label=\"Start\", shape=diamond]");

        let node_id = find(&node, DotElementType::NodeId).expect("the node id should exist");
        assert_eq!(&source[node_id.span().start..node_id.span().end], "A");
    });
}

#[test]
fn test_edge_chain_with_ports() {
    let source = "digraph { A:s -> B:n -> C }";

    parse(source, |root, diagnostics| {
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

        let edge = find(root, DotElementType::GraphEdge).expect("the edge statement should exist");
        let nodes = child_nodes(&edge);
        assert_eq!(nodes.len(), 3);
        assert!(nodes.iter().all(|node| node.green.kind == DotElementType::NodeId));
        assert_eq!(leaf_count(&edge, DotTokenType::Arrow), 2);

        let ports = find(&edge, DotElementType::Port).expect("the port should exist");
        assert_eq!(&source[ports.span().start..ports.span().end], ":s");
        assert_eq!(leaf_count(&ports, DotTokenType::Colon), 1);
    });
}

#[test]
fn test_attribute_statements() {
    let source = "graph { graph [bgcolor=gray] node [shape=box] edge [color=red] }";

    parse(source, |root, diagnostics| {
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

        let graph = find(root, DotElementType::GraphAttribute).expect("a graph attribute statement should exist");
        assert_eq!(leaf_count(&graph, DotTokenType::Graph), 1);
        assert_eq!(collect_size(root, DotElementType::GraphAttribute), 3);

        let attributes = find(root, DotElementType::AttributeList).expect("an attribute list should exist");
        assert_eq!(leaf_count(&attributes, DotTokenType::LeftBracket), 1);
        assert_eq!(leaf_count(&attributes, DotTokenType::RightBracket), 1);
    });
}

#[test]
fn test_subgraph_and_undirected_edge() {
    let source = "graph { subgraph cluster_x { A -- B } }";

    parse(source, |root, diagnostics| {
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

        let subgraph = find(root, DotElementType::GraphSubgraph).expect("the subgraph should exist");
        assert_eq!(&source[subgraph.span().start..subgraph.span().end], "subgraph cluster_x { A -- B }");
        assert_eq!(leaf_count(&subgraph, DotTokenType::Subgraph), 1);
        assert_eq!(leaf_count(&subgraph, DotTokenType::Line), 1);

        let inner = child_nodes(&subgraph);
        assert_eq!(inner.len(), 1);
        assert_eq!(inner[0].green.kind, DotElementType::GraphEdge);
    });
}

#[test]
fn test_anonymous_subgraph_operand() {
    let source = "digraph { { A B } -> C }";

    parse(source, |root, diagnostics| {
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

        let edge = find(root, DotElementType::GraphEdge).expect("the edge statement should exist");
        let operands = child_nodes(&edge);
        assert_eq!(operands[0].green.kind, DotElementType::GraphSubgraph);
        assert_eq!(operands[1].green.kind, DotElementType::NodeId);
        assert_eq!(leaf_count(&edge, DotTokenType::Arrow), 1);
    });
}

#[test]
fn test_assignments_and_comments() {
    let source = "// leading comment\nstrict digraph { rankdir = LR /* trailing */ a=1 }";

    parse(source, |root, diagnostics| {
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

        let graph = find(root, DotElementType::GraphDefinition).expect("the graph should exist");
        assert_eq!(leaf_count(&graph, DotTokenType::Strict), 1);
        assert_eq!(collect_size(&graph, DotElementType::GraphAssignment), 2);
    });
}

/// Counts nodes of the given element type.
fn collect_size(node: &RedNode<DotLanguage>, kind: DotElementType) -> usize {
    let mut found = Vec::new();
    collect(node, kind, &mut found);
    found.len()
}

#[test]
fn test_reports_unterminated_graph() {
    let source = "digraph { A -> B";

    parse(source, |root, diagnostics| {
        assert_eq!(root.green.kind, DotElementType::Root);
        assert!(diagnostics.iter().any(|error| matches!(error.kind(), OakErrorKind::UnexpectedEof { .. })), "expected an EOF diagnostic, got {diagnostics:?}");
    });
}

#[test]
fn test_reports_dangling_edge_operator() {
    let source = "digraph { A -> }";

    parse(source, |root, diagnostics| {
        assert!(root.green.kind == DotElementType::Root);
        assert!(has_syntax_error(diagnostics, "edge operator"), "expected an edge operator diagnostic, got {diagnostics:?}");
    });
}

#[test]
fn test_reports_stray_right_brace() {
    let source = "digraph { A } }";

    parse(source, |root, diagnostics| {
        assert_eq!(root.green.kind, DotElementType::Root);
        assert!(has_unexpected_token(diagnostics, "}"), "expected a stray brace diagnostic, got {diagnostics:?}");
    });
}

#[test]
fn test_reports_too_many_port_segments() {
    let source = "digraph { a:b:c:d }";

    parse(source, |root, diagnostics| {
        assert_eq!(root.green.kind, DotElementType::Root);
        assert!(has_syntax_error(diagnostics, "port"), "expected a port diagnostic, got {diagnostics:?}");

        let node = find(root, DotElementType::NodeId).expect("the node id should exist");
        assert_eq!(&source[node.span().start..node.span().end], "a:b:c:d");
    });
}

#[test]
fn test_reports_unterminated_string() {
    let source = "digraph { A [label=\"oops";

    parse(source, |root, diagnostics| {
        assert_eq!(root.green.kind, DotElementType::Root);
        assert!(has_syntax_error(diagnostics, "unterminated string"), "expected an unterminated string diagnostic, got {diagnostics:?}");
    });
}
