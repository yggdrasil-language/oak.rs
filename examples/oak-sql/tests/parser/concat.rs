use oak_core::{Builder, Lexer, ParseSession, SourceText, TokenType, source::ToSource};
#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::AsDocument;
use oak_sql::{
    SqlBuilder, SqlLanguage, SqlLexer, SqlTokenType,
    ast::{BinaryOperator, Expression, Identifier, Literal, SelectItem, SqlStatement},
};

fn build_expression(sql: &str) -> Expression {
    let language = SqlLanguage::default();
    let builder = SqlBuilder::new(&language);
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new(sql.to_string()), &[], &mut session);
    assert!(result.diagnostics.is_empty(), "{sql}: {:?}", result.diagnostics);
    let root = result.result.unwrap_or_else(|error| panic!("{sql}: {error:?}"));
    let SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("expected SELECT statement for {sql}, got {:?}", root.statements);
    };
    let SelectItem::Expression { expr, .. } = &select.items[0]
    else {
        panic!("expected an expression select item for {sql}, got {:?}", select.items[0]);
    };
    expr.clone()
}

fn span_of(expression: &Expression) -> (usize, usize) {
    let range = match expression {
        Expression::Binary { span, .. } | Expression::Unary { span, .. } | Expression::FunctionCall { span, .. } | Expression::Vector { span, .. } => span,
        Expression::Literal(Literal::Number(_, span) | Literal::String(_, span) | Literal::Boolean(_, span) | Literal::Null(span)) => span,
        Expression::Identifier(Identifier { span, .. }) => span,
        other => panic!("no span available for {other:?}"),
    };
    (range.start, range.end)
}

fn expect_concat(expression: &Expression) -> (&Expression, &Expression, (usize, usize)) {
    let Expression::Binary { op: BinaryOperator::Concat, left, right, .. } = expression
    else {
        panic!("expected a `||` binary expression, got {expression:?}");
    };
    (left, right, span_of(expression))
}

fn expect_number(expression: &Expression, expected: &str, expected_span: (usize, usize)) {
    let Expression::Literal(Literal::Number(value, _)) = expression
    else {
        panic!("expected number literal {expected}, got {expression:?}");
    };
    assert_eq!(value.as_ref(), expected);
    assert_eq!(span_of(expression), expected_span);
}

fn expect_string(expression: &Expression, expected: &str, expected_span: (usize, usize)) {
    let Expression::Literal(Literal::String(value, _)) = expression
    else {
        panic!("expected string literal {expected}, got {expression:?}");
    };
    assert_eq!(value.as_ref(), expected);
    assert_eq!(span_of(expression), expected_span);
}

fn expect_identifier(expression: &Expression, expected: &str, expected_span: (usize, usize)) {
    let Expression::Identifier(Identifier { name, .. }) = expression
    else {
        panic!("expected identifier {expected}, got {expression:?}");
    };
    assert_eq!(name.as_ref(), expected);
    assert_eq!(span_of(expression), expected_span);
}

fn expect_op(expression: &Expression, expected: BinaryOperator) {
    let Expression::Binary { op, .. } = expression
    else {
        panic!("expected a binary expression with {expected:?}, got {expression:?}");
    };
    assert_eq!(*op, expected);
}

fn binary_parts(expression: &Expression) -> (&Expression, &Expression) {
    let Expression::Binary { left, right, .. } = expression
    else {
        panic!("expected a binary expression, got {expression:?}");
    };
    (left, right)
}

#[test]
fn test_concat_builds_binary_node() {
    let expr = build_expression("select 1 || 'x'");
    let (left, right, span) = expect_concat(&expr);
    assert_eq!(span, (7, 15));
    expect_number(left, "1", (7, 8));
    expect_string(right, "x", (12, 15));
}

#[test]
fn test_concat_is_left_associative() {
    let expr = build_expression("select 'a' || 'b' || 'c'");
    let (left, right, span) = expect_concat(&expr);
    assert_eq!(span, (7, 24), "the second `||` should produce the outer node");
    expect_string(right, "c", (21, 24));

    let (inner_left, inner_right, inner_span) = expect_concat(left);
    assert_eq!(inner_span, (7, 17));
    expect_string(inner_left, "a", (7, 10));
    expect_string(inner_right, "b", (14, 17));

    let expr = build_expression("select a || 'x' || 1");
    let (left, right, span) = expect_concat(&expr);
    assert_eq!(span, (7, 20));
    expect_number(right, "1", (19, 20));
    let (inner_left, inner_right, inner_span) = expect_concat(left);
    assert_eq!(inner_span, (7, 15));
    expect_identifier(inner_left, "a", (7, 8));
    expect_string(inner_right, "x", (12, 15));
}

/// SQLite binds `||` tighter than `*`/`/`/`%`, which in turn bind tighter than
/// `+`/`-`, and all of them bind tighter than comparisons. Verified against
/// SQLite: `select 1 || 2 * 3` -> 36 (`(1 || 2) * 3`), `select 1 || 2 + 3` ->
/// 15 (`(1 || 2) + 3`), `select 'a' || 'b' = 'ab'` -> 1.
#[test]
fn test_concat_precedence_matches_sqlite() {
    let expr = build_expression("select 1 || 2 * 3");
    expect_op(&expr, BinaryOperator::Star);
    let (left, _) = binary_parts(&expr);
    let (concat_left, _, _) = expect_concat(left);
    expect_number(concat_left, "1", (7, 8));

    let expr = build_expression("select 1 || 2 + 3");
    expect_op(&expr, BinaryOperator::Plus);
    let (left, _) = binary_parts(&expr);
    expect_concat(left);

    let expr = build_expression("select 3 + 4 || 5");
    expect_op(&expr, BinaryOperator::Plus);
    let (_, right) = binary_parts(&expr);
    expect_concat(right);

    let expr = build_expression("select 'a' || 'b' = 'ab'");
    expect_op(&expr, BinaryOperator::Equal);
    let (left, _) = binary_parts(&expr);
    expect_concat(left);
}

#[test]
fn test_concat_keeps_operator_in_source() {
    let language = SqlLanguage::default();
    let builder = SqlBuilder::new(&language);
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("select 1 || 'x'".to_string()), &[], &mut session);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let source = result.result.unwrap().to_source_string();
    assert!(source.contains("||"), "{source}");
}

#[cfg(feature = "oak-pretty-print")]
fn shape(expr: &Expression) -> String {
    match expr {
        Expression::Binary { left, op, right, .. } => format!("({} {op:?} {})", shape(left), shape(right)),
        Expression::Literal(Literal::Number(value, _)) => format!("number:{value}"),
        Expression::Literal(Literal::String(value, _)) => format!("string:{value}"),
        Expression::Identifier(Identifier { name, .. }) => format!("identifier:{name}"),
        other => format!("{other:?}"),
    }
}

#[cfg(feature = "oak-pretty-print")]
#[test]
fn test_concat_pretty_round_trip() {
    for sql in ["select 1 || 'x'", "select 'a' || 'b' || 'c'", "select a || b from t", "select a || 'x' || 1", "select 1 || 2 * 3", "select 3 + 4 || 5", "select (a + b) || c", "select a || (b || c)", "select (a || b) * 2", "select 'a' || 'b' = 'ab'"] {
        let expression = build_expression(sql);
        let rendered = expression.as_document(&()).render();
        assert!(rendered.contains("||"), "{sql} rendered without `||`: {rendered}");

        let reparsed = build_expression(&format!("select {rendered}"));
        assert_eq!(shape(&expression), shape(&reparsed), "{sql} pretty-printed as {rendered}");
    }
}

#[test]
fn test_concat_and_unsupported_operator_tokens() {
    let language = SqlLanguage::default();
    let lexer = SqlLexer::new(&language);

    let source = SourceText::new("a || b".to_string());
    let mut cache = ParseSession::<SqlLanguage>::default();
    let output = lexer.lex(&source, &[], &mut cache);
    let kinds: Vec<_> = output.result.unwrap().iter().filter(|token| !token.kind.is_ignored()).map(|token| token.kind).collect();
    assert!(kinds.contains(&SqlTokenType::Concat), "{kinds:?}");

    // Unsupported SQLite operators must stay visible as `Error` tokens instead
    // of being decomposed into comparisons / arithmetic.
    for sql in ["1 | 2", "1 & 2", "1 << 2", "1 >> 2", "1 -> 2", "1 ->> 2"] {
        let source = SourceText::new(sql.to_string());
        let mut cache = ParseSession::<SqlLanguage>::default();
        let output = lexer.lex(&source, &[], &mut cache);
        let kinds: Vec<_> = output.result.unwrap().iter().filter(|token| !token.kind.is_ignored()).map(|token| token.kind).collect();
        assert!(kinds.contains(&SqlTokenType::Error), "{sql}: {kinds:?}");
    }
}
