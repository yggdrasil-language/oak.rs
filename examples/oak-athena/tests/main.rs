#![feature(new_range_api)]

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_athena::{AthenaLanguage, AthenaParser};
use oak_core::Parser;

#[test]
fn ready() {
    println!("oak-athena tests ready!");
}

#[test]
fn parse_power_sum() {
    let language = AthenaLanguage::default();
    let parser = AthenaParser::new(&language);
    let mut session = ParseSession::<AthenaLanguage>::default();
    let source = SourceText::new("x^2 + 1");
    let output = parser.parse(&source, &[], &mut session);
    assert!(output.result.is_ok(), "{:?}", output.result);
}

#[test]
fn parse_sin_call() {
    let language = AthenaLanguage::default();
    let parser = AthenaParser::new(&language);
    let mut session = ParseSession::<AthenaLanguage>::default();
    let source = SourceText::new("sin(x)^2 + cos(x)^2");
    let output = parser.parse(&source, &[], &mut session);
    assert!(output.result.is_ok(), "{:?}", output.result);
}

#[test]
fn parse_list_and_dict() {
    let language = AthenaLanguage::default();
    let parser = AthenaParser::new(&language);
    let mut session = ParseSession::<AthenaLanguage>::default();
    let source = SourceText::new(r#"[1, 2, x] + {a: 1, "b": [2, 3]}"#);
    let output = parser.parse(&source, &[], &mut session);
    assert!(output.result.is_ok(), "{:?}", output.result);
}
