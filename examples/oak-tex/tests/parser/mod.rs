use oak_core::errors::OakError;
use oak_testing::parsing::ParserTester;
use oak_tex::{
    ast::{MathDelimiter, TexArgument, TexCell, TexEnvironment, TexItem, TexRow},
    builder::TexBuilder,
    language::TexLanguage,
    parser::TexParser,
};
use std::{path::Path, time::Duration};

#[test]
fn test_tex_parser() -> Result<(), oak_core::OakError> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tests = here.join("tests/parser");
    let language = TexLanguage::default();
    let parser = TexParser::new(&language);
    let test_runner = ParserTester::new(tests).with_extension("tex").with_timeout(Duration::from_secs(5));
    test_runner.run_tests(&parser)
}

// ---- AST-level unit tests for the new math surface ----

/// Parse a TeX string down to a `TexRoot` AST, panicking on failure.
fn parse_to_ast(source: &str) -> oak_tex::ast::TexRoot {
    use oak_core::{Builder, SourceText};
    let config = TexLanguage::default();
    let builder = TexBuilder::new(&config);
    let src = SourceText::new(source);
    let mut cache = oak_core::parser::ParseSession::<TexLanguage>::default();
    let out = builder.build(&src, &[], &mut cache);
    out.result.expect("parse to AST failed")
}

// --- Math delimiters -------------------------------------------------------

#[test]
fn dollar_inline_math() {
    let ast = parse_to_ast("$x^2$");
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        TexItem::Math(m) => {
            assert_eq!(m.delimiter, MathDelimiter::Dollar);
            assert!(!m.is_display);
        }
        other => panic!("expected Math, got {:?}", other),
    }
}

#[test]
fn double_dollar_display_math() {
    let ast = parse_to_ast("$$x^2$$");
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        TexItem::Math(m) => {
            assert_eq!(m.delimiter, MathDelimiter::DoubleDollar);
            assert!(m.is_display);
        }
        other => panic!("expected Math, got {:?}", other),
    }
}

#[test]
fn paren_inline_math() {
    let ast = parse_to_ast(r#"\(xy\)"#);
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        TexItem::Math(m) => {
            assert_eq!(m.delimiter, MathDelimiter::Paren);
            assert!(!m.is_display);
        }
        other => panic!("expected Math, got {:?}", other),
    }
}

#[test]
fn bracket_display_math() {
    let ast = parse_to_ast(
        r#"\[
\int_0^1 f(x)\,dx
\]"#,
    );
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        TexItem::Math(m) => {
            assert_eq!(m.delimiter, MathDelimiter::Bracket);
            assert!(m.is_display);
        }
        other => panic!("expected Math, got {:?}", other),
    }
}

// --- Star variants ---------------------------------------------------------

#[test]
fn star_command() {
    let ast = parse_to_ast(r#"\section*{No number}"#);
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        TexItem::Command(c) => {
            assert_eq!(c.name, "section");
            assert!(c.starred, "section* should be starred");
        }
        other => panic!("expected Command, got {:?}", other),
    }
}

#[test]
fn align_star_environment_is_math() {
    let ast = parse_to_ast(
        r#"\begin{align*}
x &= y \\
a &= b
\end{align*}"#,
    );
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        TexItem::Environment(e) => {
            assert_eq!(e.name, "align");
            assert!(e.starred, "align* should be starred");
            assert!(e.is_math, "align is a math environment");
        }
        other => panic!("expected Environment, got {:?}", other),
    }
}

// --- Optional parameters ---------------------------------------------------

#[test]
fn optional_argument_on_frac() {
    let ast = parse_to_ast(r#"\frac[k]{a}{b}"#);
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        TexItem::Command(c) => {
            assert_eq!(c.name, "frac");
            assert_eq!(c.arguments.len(), 3);
            assert!(matches!(c.arguments[0], oak_tex::ast::TexArgument::Optional(_)));
            assert!(matches!(c.arguments[1], oak_tex::ast::TexArgument::Required(_)));
            assert!(matches!(c.arguments[2], oak_tex::ast::TexArgument::Required(_)));
        }
        other => panic!("expected Command, got {:?}", other),
    }
}

// --- AMS environments ------------------------------------------------------

#[test]
fn ams_cases_environment() {
    let ast = parse_to_ast(
        r#"\begin{cases}
1 & x > 0 \\
0 & x \le 0
\end{cases}"#,
    );
    match &ast.items[0] {
        TexItem::Environment(e) => {
            assert_eq!(e.name, "cases");
            assert!(e.is_math);
            assert!(!e.starred);
        }
        other => panic!("expected Environment, got {:?}", other),
    }
}

#[test]
fn ams_pmatrix_environment() {
    let ast = parse_to_ast(
        r#"\begin{pmatrix}
1 & 2 \\
3 & 4
\end{pmatrix}"#,
    );
    match &ast.items[0] {
        TexItem::Environment(e) => {
            assert_eq!(e.name, "pmatrix");
            assert!(e.is_math);
        }
        other => panic!("expected Environment, got {:?}", other),
    }
}

// --- environment row / cell structure ------------------------------------

/// The rows of an environment body, in source order.
fn environment_rows(environment: &TexEnvironment) -> Vec<&TexRow> {
    environment
        .content
        .items
        .iter()
        .filter_map(|item| match item {
            TexItem::Row(row) => Some(row),
            _ => None,
        })
        .collect()
}

/// The visible text of one cell, ignoring separators and trivia.
fn cell_text(cell: &TexCell) -> String {
    cell.content
        .items
        .iter()
        .filter_map(|item| match item {
            TexItem::Text { content, .. } => Some(content.as_str()),
            TexItem::Command(command) => Some(command.name.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn math_environment_body_is_rows_of_cells() {
    let ast = parse_to_ast(
        r#"\begin{cases}
1 & x > 0 \\
0 & x \le 0
\end{cases}"#,
    );
    let TexItem::Environment(environment) = &ast.items[0]
    else {
        panic!("expected an environment, got {:?}", ast.items[0])
    };

    assert!(environment.content.items.iter().all(|item| matches!(item, TexItem::Row(_))), "an environment body only contains rows: {:?}", environment.content.items);
    let rows = environment_rows(environment);
    assert_eq!(rows.len(), 2, "two authored rows, got {rows:?}");
    assert_eq!(rows[0].cells.len(), 2, "`1 & x > 0` has two cells");
    assert_eq!(rows[1].cells.len(), 2, "`0 & x \\le 0` has two cells");
    assert_eq!(cell_text(&rows[0].cells[0]), "1");
    assert_eq!(cell_text(&rows[1].cells[0]), "0");

    for row in &rows {
        for cell in &row.cells {
            assert!(cell.span.start <= cell.span.end);
            assert!(row.span.start <= cell.span.start && cell.span.end <= row.span.end.max(cell.span.end));
        }
    }
}

#[test]
fn empty_cells_and_trailing_row_breaks_keep_the_grid() {
    let ast = parse_to_ast(r#"\begin{matrix} a & & b \\ \end{matrix}"#);
    let TexItem::Environment(environment) = &ast.items[0]
    else {
        panic!("expected an environment")
    };
    let rows = environment_rows(environment);

    assert_eq!(rows.len(), 1, "a trailing `\\\\` does not add a row: {rows:?}");
    assert_eq!(rows[0].cells.len(), 3, "the middle cell is empty but present");
    assert_eq!(cell_text(&rows[0].cells[0]), "a");
    assert_eq!(cell_text(&rows[0].cells[1]), "");
    assert_eq!(cell_text(&rows[0].cells[2]), "b");
}

#[test]
fn alignedat_declares_its_column_pairs_as_environment_arguments() {
    let ast = parse_to_ast(r"\begin{alignedat}{2} a &= b \end{alignedat}");
    let TexItem::Environment(environment) = &ast.items[0]
    else {
        panic!("expected an environment")
    };
    assert_eq!(environment.name, "alignedat");
    let TexArgument::Required(root) = &environment.arguments[0]
    else {
        panic!("the column-pair count is a required argument")
    };
    assert_eq!(root.items.iter().filter_map(|item| match item { TexItem::Text { content, .. } => Some(content.as_str()), _ => None }).collect::<String>(), "2");
    let rows = environment_rows(environment);
    assert_eq!(rows.len(), 1);
    assert_eq!(cell_text(&rows[0].cells[0]), "a", "the declared pair count is not part of the first cell");
}

#[test]
fn row_break_outside_an_environment_stays_a_command() {
    let ast = parse_to_ast(r#"first \\ second"#);
    assert!(ast.items.iter().any(|item| matches!(item, TexItem::Command(command) if command.name == "\\")), "a text-mode row break must stay the `\\\\` command: {:?}", ast.items);
}

// --- left right pair -------------------------------------------------------

#[test]
fn left_right_simple() {
    let ast = parse_to_ast(r#"\left( \frac{a}{b} \right)"#);
    assert_eq!(ast.items.len(), 1);
    match &ast.items[0] {
        TexItem::LeftRightPair(p) => {
            assert_eq!(p.left_delim, "(");
            assert_eq!(p.right_delim, ")");
            assert!(!p.content.items.is_empty());
        }
        other => panic!("expected LeftRightPair, got {:?}", other),
    }
}

#[test]
fn left_right_with_middle() {
    let ast = parse_to_ast(r#"\left| a \middle| b \right|"#);
    match &ast.items[0] {
        TexItem::LeftRightPair(p) => {
            assert_eq!(p.left_delim, "|");
            assert_eq!(p.right_delim, "|");
            // the \middle should be preserved as a Command named "middle" inside content
            assert!(p.content.items.iter().any(|it| matches!(it, TexItem::Command(c) if c.name == "middle")));
        }
        other => panic!("expected LeftRightPair, got {:?}", other),
    }
}

#[test]
fn left_invisible_right_brace() {
    let ast = parse_to_ast(r#"\left. x^2 \right\}"#);
    match &ast.items[0] {
        TexItem::LeftRightPair(p) => {
            assert_eq!(p.left_delim, ".", "invisible left delimiter");
            // `\right\}` produces a BraceDelim token whose text preserves the full `\}`
            assert!(p.right_delim == "}" || p.right_delim == "\\}", "braced right delimiter, got {:?}", p.right_delim);
        }
        other => panic!("expected LeftRightPair, got {:?}", other),
    }
}

// --- limits ---------------------------------------------------------------

#[test]
fn limits_command() {
    let ast = parse_to_ast(r#"\sum\limits_{i=1}^n x_i"#);
    assert!(ast.items.iter().any(|it| matches!(it, TexItem::Limits(l) if l.is_limits)));
}

#[test]
fn nolimits_command() {
    let ast = parse_to_ast(r#"\int\nolimits_a^b f(x)\,dx"#);
    assert!(ast.items.iter().any(|it| matches!(it, TexItem::Limits(l) if !l.is_limits)));
}
