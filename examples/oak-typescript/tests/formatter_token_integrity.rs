use oak_typescript::formatter::{FormatOptions, QuoteStyle, format_source};

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

#[test]
fn ternary_colons_after_braced_types_keep_the_space() {
    let braced = "const sources = condition ? (table.profiles[table.default] as { sources?: unknown } | undefined)?.sources : null;";
    let callback = "const shopLayout = Array.isArray(layouts) ? layouts.find((l: { bump?: () => void }) => typeof l?.bump === 'function') : null;";
    for source in [braced, callback] {
        assert_eq!(format(source), source, "source={source:?}");
    }
}

#[test]
fn member_calls_and_private_names_keep_their_boundaries() {
    for source in ["async #render(): Promise<void> { this.#run(); }", "const sync = EditorView.updateListener.of((u) => { run(u); });", "class Store { static #count = 0; readonly items: readonly [string, string][] = []; }"] {
        assert_eq!(format(source), source, "source={source:?}");
    }
}

#[test]
fn operators_and_type_arguments_stay_separated() {
    for source in [
        "const ok = guard++ < 64 && i-- > 0;",
        "return cur ^ rhs;",
        "const owned = new Map<string, Set<string>>();",
        "export type Bag = Record<string, Array<(detail?: unknown) => void>>;",
        "export type Locale = { locales?: Array<{ id: string } | string>; };",
        "type Metas = ReturnType<typeof buildMeta>[];",
        "export type Def = { circles?: readonly Readonly<{ cx: number; cy: number }>[]; };",
    ] {
        assert_eq!(format(source), source, "source={source:?}");
    }
}

// --- Bug 1: generics get spaces inside ---
#[test]
fn deeply_nested_generics_with_bracket_access_keep_tight() {
    for source in ["type Open = ReturnType<ReturnType<typeof Object>['valueOf']>;", "async query<T = Record<string, unknown>>(source: string): Promise<T[]> {}", "Promise<{ version: number; document: string } | null>"] {
        assert_eq!(format(source), source, "generics gained spaces: {source:?}");
    }
}

// --- Bug 2: delete reserved-word method gets space ---
#[test]
fn async_delete_method_stays_attached() {
    for source in ["async delete(key: string): Promise<void> { this.native.delete(key); }", "class Store { async delete(key: string): Promise<void> {} }"] {
        assert_eq!(format(source), source, "delete method gained a space: {source:?}");
    }
}

// --- Bug 3: bitwise shift loses spacing ---
#[test]
fn bitwise_shift_operators_keep_spaces() {
    for source in ["return (value >> 8) & 0xff;", "pushU32(out, version >>> 0);", "out.push(value & 0xff, (value >> 8) & 0xff, (value >> 16) & 0xff, (value >> 24) & 0xff);"] {
        assert_eq!(format(source), source, "shift lost spaces: {source:?}");
    }
}

// --- Bug 5: `<` / `>` comparisons misread as type arguments ---
#[test]
fn comparison_operators_are_not_type_arguments() {
    for (source, expected) in [
        ("if (parsed < minimum || parsed > 65535) { run(); }", "if (parsed < minimum || parsed > 65535) { run(); }"),
        (
            "if (\n    !Number.isInteger(parsed) ||\n    parsed < minimum ||\n    parsed > (arg === \"-n\" || arg === \"--db\" ? 15 : 65535)\n) {\n    fail();\n}\n",
            "if (\n    !Number.isInteger(parsed) ||\n    parsed < minimum ||\n    parsed > (arg === \"-n\" || arg === \"--db\" ? 15 : 65535)\n) {\n    fail();\n}\n",
        ),
        ("const picked = index < limit && index > floor;", "const picked = index < limit && index > floor;"),
        ("const clamped = value < low ? low : value;", "const clamped = value < low ? low : value;"),
    ] {
        assert_eq!(format(source), expected, "comparison read as type arguments: {source:?}");
    }
}

#[test]
fn type_arguments_stay_tight_next_to_comparisons() {
    for source in ["const items = make<Item>(value);", "function read<T = unknown>(file: string): T { return load(file) as T; }", "type Fixed = Box<-1>;", "const pair = a < b && c > d;"] {
        assert_eq!(format(source), source, "type arguments lost their shape: {source:?}");
    }
}

// --- Bug 6: `!` gained a space before a regex literal ---
#[test]
fn logical_not_binds_to_regex_literals() {
    for source in [r"if (!/^-?\d+$/.test(line)) { fail(); }", r"if (!/^(?:-1|\d+)$/.test(line)) { fail(); }", r"const ok = !/x/.test(value);"] {
        assert_eq!(format(source), source, "regex literal detached from `!`: {source:?}");
    }
}

// --- Bug 7: unary signs separated from their operand after a comparison ---
#[test]
fn unary_signs_after_comparisons_bind_to_their_operands() {
    for (source, expected) in [
        ("if (length < -1) { fail(); }", "if (length < -1) { fail(); }"),
        ("if (length<-1) { fail(); }", "if (length < -1) { fail(); }"),
        ("if (count > +1) { fail(); }", "if (count > +1) { fail(); }"),
        ("if (eq === -1) { reset(); }", "if (eq === -1) { reset(); }"),
        ("const shifted = Box<number> - 1;", "const shifted = Box<number> - 1;"),
    ] {
        assert_eq!(format(source), expected, "unary sign spacing changed: {source:?}");
    }
}

// --- Bug 8: `quoteStyle` rejected instead of accepted by the gap formatter ---
#[test]
fn quote_style_options_keep_literal_text_intact() {
    for quote_style in [QuoteStyle::Single, QuoteStyle::Double, QuoteStyle::Preserve] {
        let options = FormatOptions { quote_style, ..FormatOptions::default() };
        for (source, expected) in [
            ("const a = 'x';", "const a = 'x';"),
            ("const a = \"x\";", "const a = \"x\";"),
            ("const a = 'it\\'s';", "const a = 'it\\'s';"),
            ("const a = `keep '${value}'`;", "const a = `keep '${value}'`;"),
            ("const a = /'x'/;", "const a = /'x'/;"),
        ] {
            assert_eq!(format_source(source, &options).expect("format"), expected, "quote literal rewritten under {quote_style:?}: {source:?}");
        }
    }
}

#[test]
fn non_null_assertion_stays_separate_from_inequality() {
    // 真正的 discussion case：两个相邻的 ! — non-null assertion + inequality 前导 !
    // 输入里 body[n]!!== 应该变成 body[n]! !==（中间加空格把两个 ! token 分开）
    let source = "function read(body: Uint8Array, n: number) { return body[n]!!== 0; }";
    let expected = "function read(body: Uint8Array, n: number) { return body[n]! !== 0; }";
    assert_eq!(format(source), expected);
}
