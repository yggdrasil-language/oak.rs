use oak_typescript::formatter::{FormatOptions, format_source};

fn format(source: &str) -> String {
    let options = FormatOptions::default();
    let output = format_source(source, &options).expect("format");
    assert_eq!(format_source(&output, &options).expect("idempotent"), output, "not idempotent: {source:?}");
    output
}

#[test]
fn inequality_operators_keep_the_space_before_them() {
    for (source, expected) in
        [("const value = a != null;", "const value = a != null;"), ("const value = a!==null;", "const value = a !== null;"), ("if (left != right) { run(); }", "if (left != right) { run(); }"), ("const ok = !done && a != b;", "const ok = !done && a != b;")]
    {
        assert_eq!(format(source), expected, "source={source:?}");
    }
}

#[test]
fn nested_generic_close_stays_one_token_before_assignment() {
    for (source, expected) in
        [("const map: Record<string, Record<string, unknown>> = {};", "const map: Record<string, Record<string, unknown>> = {};"), ("const deep: Map<string, Set<number>> = new Map();", "const deep: Map<string, Set<number>> = new Map();")]
    {
        assert_eq!(format(source), expected, "source={source:?}");
    }
}

#[test]
fn numeric_separators_stay_inside_one_literal() {
    for (source, expected) in [
        ("const deadline = Date.now() + 60_000;", "const deadline = Date.now() + 60_000;"),
        ("const big = 1_000_000;", "const big = 1_000_000;"),
        ("const mask = 0xFF_FF;", "const mask = 0xFF_FF;"),
        ("const bytes = 0b1010_1010;", "const bytes = 0b1010_1010;"),
        ("const mode = 0o7_5_5;", "const mode = 0o7_5_5;"),
    ] {
        assert_eq!(format(source), expected, "source={source:?}");
    }
}

#[test]
fn regex_literals_start_after_logical_operators() {
    for source in [r"const a = /x/.test(v) || /\.\.\.]/.test(v);", r"const b = /^a$/.test(v) && /[\]]/.test(v);", r"const c = /a/.test(v) ?? /b/.test(v);", r#"if (/\bfrom\s+['"]\.\/(?!\.\.)/.test(js)) { fail(); }"#] {
        assert_eq!(format(source), source, "source={source:?}");
    }
}
