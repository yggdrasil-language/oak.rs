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

#[test]
fn keyword_prefixes_keep_the_space_before_them() {
    for (source, expected) in [
        ("const f = function(a) { return a; };", "const f = function (a) { return a; };"),
        ("function run() { return !!done; }", "function run() { return !!done; }"),
        ("function pick() { return ~mask; }", "function pick() { return ~mask; }"),
        ("const x = obj.function();", "const x = obj.function();"),
    ] {
        assert_eq!(format(source), expected, "source={source:?}");
    }
}

#[test]
fn division_operators_keep_the_space_on_both_sides() {
    for (source, expected) in [("const x = a / b;", "const x = a / b;"), ("const pct = Math.round((i + 1) /total * 100);", "const pct = Math.round((i + 1) / total * 100);"), ("const ratio = (a) /b;", "const ratio = (a) / b;")] {
        assert_eq!(format(source), expected, "source={source:?}");
    }
}

#[test]
fn nested_template_interpolations_stay_one_token() {
    for source in ["const message = `outer ${ok ? `inner ${value}` : \"\"}`;", "const path = `${base}/${name}`;", "const nested = `a ${ { b: `c ${d}` } } e`;"] {
        assert_eq!(format(source), source, "source={source:?}");
    }
}

#[test]
fn statement_and_operator_spacing_stays_canonical() {
    for source in [
        "for (const id of [...new Set(chain)].reverse()) { run(id); }",
        "export function* stream(node) { yield* walk(node); }",
        "async function* pages() { yield* source; }",
        "const seq = ++mountSeq;",
        "const offset = value >>> 0;",
        "const shifted = left << 2;",
        "if (eq === -1) { reset(); }",
        "const negated = flag !== -1;",
    ] {
        assert_eq!(format(source), source, "source={source:?}");
    }
}
#[test]
fn generic_defaults_and_statement_bodies_stay_readable() {
    for (source, expected) in [
        ("export function readAsync<T = unknown>(file: string): Promise<T> { return load(file) as Promise<T>; }", "export function readAsync<T = unknown>(file: string): Promise<T> { return load(file) as Promise<T>; }"),
        ("if (ready)(handler)();", "if (ready) (handler)();"),
        ("const el = <div class=\"x\">text</div>;", "const el = <div class=\"x\">text</div>;"),
    ] {
        assert_eq!(format(source), expected, "source={source:?}");
    }
}

#[test]
fn triple_nested_type_arguments_stay_tight() {
    for source in ["const params: Record<string, Array<Record<string, string>>> = {};", "class Store { staticParams?: Record<string, Array<Record<string, string>>>; }"] {
        assert_eq!(format(source), source, "source={source:?}");
    }
}
