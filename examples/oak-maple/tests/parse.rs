use oak_core::{ParseSession, parser::Parser, source::SourceText};
use oak_maple::{MapleLanguage, MapleParser};

#[test]
fn parses_maple_minimal_expression() {
    let language = MapleLanguage::default();
    let parser = MapleParser::new(&language);
    let mut cache = ParseSession::<MapleLanguage>::default();
    let source = SourceText::new("f(x) + [1, 2]^2");
    let output = parser.parse(&source, &[], &mut cache);
    assert!(!output.has_errors(), "parse errors: {:?}", output.diagnostics);
}

fn build(input: &str) -> oak_core::OakDiagnostics<oak_maple::ast::MapleRoot> {
    use oak_core::Builder;
    let language = MapleLanguage::default();
    let source = SourceText::new(input);
    let mut cache = ParseSession::<MapleLanguage>::default();
    oak_maple::MapleBuilder::new(&language).build(&source, &[], &mut cache)
}

#[test]
fn rejects_malformed_or_unsupported_syntax() {
    for input in ["f(", "[1", "(1", "f(1,)", "[1,]", "1+", "1 2", "x:=1", "x=1", "x[1]", "@", "()", "[,]", "f(,1)", "1)", "1;;", "1:", "λ"] {
        let output = build(input);
        assert!(output.result.is_err(), "unexpected acceptance: {input}");
        assert!(!output.diagnostics.is_empty(), "missing diagnostic: {input}");
        assert!(output.diagnostics.iter().any(|error| error.source_offset().is_some()), "missing location: {input}");
    }
}

#[test]
fn preserves_owned_numeric_spelling_and_source_spans() {
    use oak_maple::ast::MapleExpressionKind;
    let output = build(
        " # 注释
[123456789012345678901234567890, 1.25]; f()",
    );
    let root = output.result.unwrap();
    assert_eq!(root.expressions.len(), 2);
    let MapleExpressionKind::List(elements) = &root.expressions[0].kind
    else {
        panic!("expected list")
    };
    assert_eq!(elements[0].kind, MapleExpressionKind::Integer("123456789012345678901234567890".into()));
    assert_eq!(elements[1].kind, MapleExpressionKind::Decimal("1.25".into()));
    assert_eq!(
        &" # 注释
[123456789012345678901234567890, 1.25]; f()"[elements[1].span.start..elements[1].span.end],
        "1.25"
    );
    assert!(matches!(&root.expressions[1].kind, MapleExpressionKind::Call { name, arguments } if name == "f" && arguments.is_empty()));
}

#[test]
fn binds_unary_below_power_and_multiplication() {
    use oak_maple::ast::{MapleExpressionKind as Kind, MapleOperator};
    let root = build("-x^2*3").result.unwrap();
    let Kind::Unary { operator: MapleOperator::Subtract, operand } = &root.expressions[0].kind
    else {
        panic!("expected negation")
    };
    let Kind::Binary { operator: MapleOperator::Multiply, left, .. } = &operand.kind
    else {
        panic!("expected multiplication")
    };
    assert!(matches!(left.kind, Kind::Binary { operator: MapleOperator::Power, .. }));
}

#[test]
fn preserves_explicit_power_grouping_and_left_associative_division() {
    use oak_maple::ast::{MapleExpressionKind as Kind, MapleOperator};
    assert!(build("x^y^z").has_errors());
    assert!(build("2^3^4").has_errors());
    for input in ["x^(y^z)", "(x^y)^z", "2^-3"] {
        assert!(!build(input).has_errors(), "{input}");
    }
    let root = build("8/4/2").result.unwrap();
    let Kind::Binary { operator: MapleOperator::Divide, left, .. } = &root.expressions[0].kind
    else {
        panic!("expected division")
    };
    assert!(matches!(left.kind, Kind::Binary { operator: MapleOperator::Divide, .. }));
}

#[test]
fn parses_nested_calls_lists_and_grouping() {
    let output = build("f([1, g(2)], (3+4)); []; +1");
    assert!(!output.has_errors(), "{:?}", output.diagnostics);
    assert_eq!(output.result.unwrap().expressions.len(), 3);
}

#[test]
fn builder_owns_syntax_after_source_and_cache_are_dropped() {
    use oak_maple::ast::MapleExpressionKind;
    let root = build("f(12345678901234567890)").result.unwrap();
    assert!(matches!(&root.expressions[0].kind, MapleExpressionKind::Call { name, arguments } if name == "f" && arguments.len() == 1));
}

#[test]
fn reusable_cache_does_not_leak_previous_source() {
    use oak_core::Builder;
    let language = MapleLanguage::default();
    let builder = oak_maple::MapleBuilder::new(&language);
    let mut cache = ParseSession::<MapleLanguage>::default();
    for input in [
        "1",
        "f(2)",
        "[3, 4]",
        "-x^2",
        "(x^y)^z",
        "",
        " # 注释
 ",
    ] {
        let source = SourceText::new(input);
        let output = builder.build(&source, &[], &mut cache);
        let root = output.result.unwrap();
        assert_eq!(root, build(input).result.unwrap(), "stale syntax for {input}");
        assert_eq!(root.span.start, 0);
        assert_eq!(root.span.end, input.len());
    }
}

#[test]
fn arithmetic_subset_is_lossless_with_trivia() {
    use oak_maple::ast::MapleExpressionKind as Kind;
    let input = "  f # 调用
 ( 1 # 数字
 , [ 2 ] ) ;  ";
    let root = build(input).result.unwrap();
    assert_eq!(root.span.end, input.len());
    let Kind::Call { name, arguments } = &root.expressions[0].kind
    else {
        panic!("expected call")
    };
    assert_eq!(name, "f");
    assert_eq!(arguments[0].kind, Kind::Integer("1".into()));
}

#[test]
fn rejects_reserved_words_in_the_expression_subset() {
    for keyword in [
        "and",
        "assuming",
        "break",
        "by",
        "catch",
        "description",
        "do",
        "done",
        "elif",
        "else",
        "end",
        "error",
        "export",
        "fi",
        "finally",
        "for",
        "from",
        "global",
        "if",
        "implies",
        "in",
        "intersect",
        "local",
        "minus",
        "mod",
        "module",
        "next",
        "not",
        "od",
        "option",
        "options",
        "or",
        "proc",
        "quit",
        "read",
        "return",
        "save",
        "stop",
        "subset",
        "then",
        "to",
        "try",
        "union",
        "until",
        "use",
        "uses",
        "while",
        "xor",
    ] {
        assert!(build(keyword).has_errors(), "unsupported keyword accepted as a symbol: {keyword}");
    }
    for symbol in ["android", "If", "local_name", "x1", "_name"] {
        assert!(!build(symbol).has_errors(), "valid identifier rejected: {symbol}");
    }
}

#[test]
fn expression_spans_exclude_trivia_but_keep_source_delimiters() {
    use oak_maple::ast::MapleExpressionKind;
    let input = " # 前导\n [ 11 , f( 22 ) , ( 33 ) , - 44 ] # 尾部\n ;";
    let root = build(input).result.unwrap();
    assert_eq!(root.span.start, 0);
    assert_eq!(root.span.end, input.len());
    let outer = &root.expressions[0];
    assert_eq!(&input[outer.span.start..outer.span.end], "[ 11 , f( 22 ) , ( 33 ) , - 44 ]");
    let MapleExpressionKind::List(items) = &outer.kind
    else {
        panic!("list")
    };
    for (item, expected) in items.iter().zip(["11", "f( 22 )", "( 33 )", "- 44"]) {
        assert_eq!(&input[item.span.start..item.span.end], expected);
    }
    let MapleExpressionKind::Call { arguments, .. } = &items[1].kind
    else {
        panic!("call")
    };
    assert_eq!(&input[arguments[0].span.start..arguments[0].span.end], "22");
    let MapleExpressionKind::Grouped(inner) = &items[2].kind
    else {
        panic!("grouped")
    };
    assert_eq!(&input[inner.span.start..inner.span.end], "33");
    let MapleExpressionKind::Unary { operand, .. } = &items[3].kind
    else {
        panic!("unary")
    };
    assert_eq!(&input[operand.span.start..operand.span.end], "44");
}

#[test]
fn recovery_preserves_following_statements_without_publishing_ast() {
    use oak_core::RedNode;
    let input = "x^y^z; g(2); [1,]; 3";
    let source = SourceText::new(input);
    let language = MapleLanguage::default();
    let mut cache = ParseSession::<MapleLanguage>::default();
    let output = MapleParser::new(&language).parse(&source, &[], &mut cache);
    assert!(output.has_errors());
    let green = output.result.unwrap();
    assert_eq!(RedNode::new(green, 0).span().end, input.len());
    assert!(build(input).result.is_err());
}
