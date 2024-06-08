use oak_typescript::formatter::{FormatOptions, format_source};

fn format(source: &str) -> String {
    let options = FormatOptions::default();
    let output = format_source(source, &options).expect("format");
    assert_eq!(format_source(&output, &options).expect("idempotent"), output, "not idempotent: {source:?}");
    output
}

// Regression: `foo?(x: number): void;` used to become `foo ? (x: number) : void;`
// (spaces around the optional marker and before the return-type colon).
#[test]
fn optional_method_signatures_keep_marker_and_return_type_attached() {
    for source in [
        "class A { foo?(x: number): void; }",
        "class A { foo?(this: A, ...args: unknown[]): void; }",
        "class A { delete?(k: string): void; }",
        "declare class A { foo?<T>(x: T): void; }",
        "interface Store { setMicroHostHandler?(handler: MicroHostHandler | undefined): void; }",
        "interface Store {\n    delete(key: string): Promise<void>;\n    setMicroHostHandler?(handler: MicroHostHandler | undefined): void;\n}\n",
    ] {
        assert_eq!(format(source), source, "optional method signature changed: {source:?}");
    }
}

#[test]
fn optional_properties_and_parameters_stay_tight() {
    for source in ["class A { foo?: number; }", "interface A { foo?: string }", "function f(x?: number): void {}", "type A = { foo?: () => void };"] {
        assert_eq!(format(source), source, "optional property or parameter changed: {source:?}");
    }
}

#[test]
fn required_return_type_keeps_the_space_after_the_colon() {
    for source in ["class A { foo(x: number): void; }", "interface Store { read(key: string): Promise<void>; }", "function f(x: number): string { return String(x); }"] {
        assert_eq!(format(source), source, "return type spacing changed: {source:?}");
    }
}

// Regression: `delete(key: string): void;` used to become `delete (key: string)`.
#[test]
fn reserved_member_methods_keep_their_parens_attached() {
    for source in [
        "interface Store { delete(key: string): Promise<void>; }",
        "interface Store { delete(key: string): void; new(k: string): void; in(k: string): void; default(k: string): void; }",
        "class Store { delete(key: string): void {} }",
        "class Store { static delete(key: string): void {} }",
        "class Store { async delete(key: string): Promise<void> { this.native.delete(key); } }",
        "class Store extends Base<string, number> { delete(key: string): void {} }",
        "interface Store<T, U> { delete(key: string): void; }",
        "const handlers = { delete(key: string): void {} };",
    ] {
        assert_eq!(format(source), source, "reserved member name gained a space: {source:?}");
    }
}

#[test]
fn reserved_words_as_property_names_stay_attached() {
    for source in ["obj.delete(key);", "obj?.delete(key);", "obj.in(key);", "obj.default(key);", "class A { delete = 1; }"] {
        assert_eq!(format(source), source, "reserved property name changed: {source:?}");
    }
}

#[test]
fn reserved_words_in_expression_position_keep_their_space() {
    for (source, expected) in [
        ("function f() { delete (obj.key); }", "function f() { delete (obj.key); }"),
        ("function f() { return(a); }", "function f() { return (a); }"),
        ("const x = new(Foo)();", "const x = new (Foo)();"),
        ("function f() { if(a) {} }", "function f() { if (a) {} }"),
        ("function f() { try {} catch(e) {} }", "function f() { try {} catch (e) {} }"),
    ] {
        assert_eq!(format(source), expected, "keyword lost its space: {source:?}");
    }
}

#[test]
fn ternaries_with_parenthesized_branches_keep_their_spaces() {
    for (source, expected) in [("const x = flag ? (a) : b;", "const x = flag ? (a) : b;"), ("const x = flag?(a):b;", "const x = flag ? (a) : b;"), ("const x = flag ? (a + b) : c;", "const x = flag ? (a + b) : c;")] {
        assert_eq!(format(source), expected, "ternary spacing changed: {source:?}");
    }
}

// Counterexamples: a `?` inside an expression is never an optional marker.
#[test]
fn expression_question_marks_keep_their_spacing() {
    for (source, expected) in [
        ("const x = cond ? a : b;", "const x = cond ? a : b;"),
        ("const value = cond?1:2;", "const value = cond ? 1 : 2;"),
        ("const x = flag ? (a) : b;", "const x = flag ? (a) : b;"),
        ("const found = items.find((item) => item ? item.id : null);", "const found = items.find((item) => item ? item.id : null);"),
    ] {
        assert_eq!(format(source), expected, "expression `?` spacing changed: {source:?}");
    }
}

// Regression: a `{` after a signature with a return type annotation is the
// function body, not a type-literal member body, so `if (` keeps its space.
#[test]
fn function_bodies_with_return_types_are_not_member_bodies() {
    for (source, expected) in [
        ("function f(x: number): void { if (x) {} }", "function f(x: number): void { if (x) {} }"),
        ("function f(x: number): void { for (const y of x) {} }", "function f(x: number): void { for (const y of x) {} }"),
        ("function f(x: number): void { while (x) {} }", "function f(x: number): void { while (x) {} }"),
        ("function f(x: number): void { switch (x) { case 1: break; } }", "function f(x: number): void { switch (x) { case 1: break; } }"),
        ("function f(): void { while (x) {} }", "function f(): void { while (x) {} }"),
        ("const f = (x: number): void => { if (x) {} };", "const f = (x: number): void => { if (x) {} };"),
        ("class A { m(x: number): void { if (x) {} } }", "class A { m(x: number): void { if (x) {} } }"),
        ("const o = { m(x: number): void { if (x) {} } };", "const o = { m(x: number): void { if (x) {} } };"),
    ] {
        assert_eq!(format(source), expected, "function body treated as member body: {source:?}");
    }
}

#[test]
fn optional_chaining_stays_attached() {
    for source in ["const v = a?.b;", "const v = a?.b?.c;", "const v = a?.delete?.(k);", "const v = a?.[k];", "obj?.delete(key);"] {
        assert_eq!(format(source), source, "optional chaining changed: {source:?}");
    }
}

// Regression: commas inside a return type's type arguments are type separators,
// not member separators, so the brace still opens a function body.
#[test]
fn generic_return_types_with_commas_keep_function_bodies() {
    for (source, expected) in [
        ("function f(x: number): Record<string, unknown> { if (x) {} }", "function f(x: number): Record<string, unknown> { if (x) {} }"),
        ("function f(x: number): Promise<Record<string, unknown>> { if (x) {} }", "function f(x: number): Promise<Record<string, unknown>> { if (x) {} }"),
        ("function f(x: number): Map<string, GithubAuthor> { return (x); }", "function f(x: number): Map<string, GithubAuthor> { return (x); }"),
        ("function f(x: number): Record<string, string> { for (const y of x) {} }", "function f(x: number): Record<string, string> { for (const y of x) {} }"),
        ("function f(x: number): Promise<Record<string, unknown>> { switch (x) { case 1: break; } }", "function f(x: number): Promise<Record<string, unknown>> { switch (x) { case 1: break; } }"),
        ("function f(x: number): Promise<Record<string, unknown>> { return (x); }", "function f(x: number): Promise<Record<string, unknown>> { return (x); }"),
        ("class A { m(x: number): Record<string, unknown> { if (x) {} } }", "class A { m(x: number): Record<string, unknown> { if (x) {} } }"),
        ("const f = (x: number): Promise<Record<string, unknown>> => { if (x) {} };", "const f = (x: number): Promise<Record<string, unknown>> => { if (x) {} };"),
    ] {
        assert_eq!(format(source), expected, "type argument comma broke the function body: {source:?}");
    }
}
