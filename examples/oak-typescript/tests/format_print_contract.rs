//! Shared format contract fixtures for VMZ/Nifty capability matrix row
//! `oak-typescript::format::format_source`.
//!
//! Covers supported syntax, idempotence, and documented gaps. AST-only trivia behavior
//! is tested in the private `print` module.

use oak_typescript::{FormatOptions, format_source};

struct FormatCase {
    name: &'static str,
    input: &'static str,
    /// Expected formatted output when format succeeds.
    expect_output: Option<&'static str>,
    /// When true, `format_source` must return `Err` (coverage gap).
    expect_err: bool,
}

const SUPPORTED: &[FormatCase] = &[
    FormatCase {
        name: "const_spacing",
        input: "const  x=1",
        expect_output: Some("const x = 1"),
        expect_err: false,
    },
    FormatCase {
        name: "ternary_string_literals",
        input: r#"const v = error ? "true" : "false""#,
        expect_output: Some("const v = error ? 'true' : 'false'"),
        expect_err: false,
    },
    FormatCase {
        name: "scoped_named_import",
        input: "import { foo } from 'pkg'",
        expect_output: Some("import { foo } from 'pkg';"),
        expect_err: false,
    },
    FormatCase {
        name: "jsx_element",
        input: r#"const el = <div className="foo">bar</div>"#,
        expect_output: None,
        expect_err: false,
    },
    FormatCase {
        name: "jsx_self_closing",
        input: "const el = <br />",
        expect_output: Some("const el = <br />"),
        expect_err: false,
    },
    FormatCase {
        name: "jsx_fragment",
        input: "const el = <>hello</>",
        expect_output: Some("const el = <>hello</>"),
        expect_err: false,
    },
];

const UNSUPPORTED: &[FormatCase] = &[FormatCase {
    name: "class_declaration",
    input: "class Foo {}",
    expect_output: None,
    expect_err: true,
}];

#[test]
fn supported_cases_format_and_idempotent() {
    for case in SUPPORTED {
        let out = format_source(case.input, &FormatOptions::default())
            .unwrap_or_else(|err| panic!("{}: oak format failed: {err}", case.name));
        if let Some(expected) = case.expect_output {
            assert_eq!(out, expected, "{}", case.name);
        } else if case.name == "jsx_element" {
            assert!(
                out.contains("<div className='foo'>bar</div>"),
                "{}: unexpected {:?}",
                case.name,
                out
            );
        }
        let again = format_source(&out, &FormatOptions::default())
            .unwrap_or_else(|err| panic!("{}: second pass failed: {err}", case.name));
        assert_eq!(out, again, "{}: not idempotent", case.name);
    }
}

#[test]
fn unsupported_syntax_returns_err() {
    for case in UNSUPPORTED {
        let result = format_source(case.input, &FormatOptions::default());
        assert!(result.is_err(), "{}: expected Err, got {:?}", case.name, result);
    }
}

#[test]
fn leading_line_comment_is_preserved() {
    let input = "// keep\nconst x = 1";
    let out = format_source(input, &FormatOptions::default()).expect("format");
    assert_eq!(out, "// keep\nconst x = 1");
}
