use oak_typescript::formatter::{FormatOptions, format_source};

fn format(source: &str) -> String {
    let options = FormatOptions::default();
    let output = format_source(source, &options).expect("format");
    assert_eq!(format_source(&output, &options).expect("idempotent"), output, "not idempotent: {source:?}");
    output
}

#[test]
fn reserved_words_as_property_names_keep_call_parens_attached() {
    for source in [
        "obj.catch(arg)",
        "obj?.catch(arg)",
        "const p = a.then(f).catch(g)",
        "obj.delete(arg)",
        "obj.new(arg)",
        "obj.typeof(arg)",
        "obj.switch(arg)",
        "obj.throw(arg)",
        "obj.in(arg)",
        "obj.instanceof(arg)",
        "obj.as(arg)",
        "obj.void(arg)",
        "obj.await(arg)",
        "obj.yield(arg)",
        "obj.if(arg)",
        "obj.for(arg)",
        "obj.while(arg)",
        "obj.with(arg)",
        "obj.return(arg)",
    ] {
        assert_eq!(format(source), source, "property call gained a space: {source:?}");
    }
}

#[test]
fn reserved_words_in_member_modifier_position_keep_call_parens_attached() {
    for source in ["class A { static catch(arg) {} }", "class A { async catch(arg) {} }", "class A { get catch() { return 1 } }", "const o = { x: 1, catch(arg) { return arg } }"] {
        assert_eq!(format(source), source, "member gained a space: {source:?}");
    }
}

#[test]
fn keywords_in_statement_position_keep_their_space() {
    for (source, expected) in [
        ("try { f() } catch(e) { g(e) }", "try { f() } catch (e) { g(e) }"),
        ("function f() { if(a) {} }", "function f() { if (a) {} }"),
        ("a(); if(b) {}", "a(); if (b) {}"),
        ("function f() { return(a) }", "function f() { return (a) }"),
        ("const x = new(Foo)()", "const x = new (Foo)()"),
    ] {
        assert_eq!(format(source), expected, "keyword lost its space: {source:?}");
    }
}
