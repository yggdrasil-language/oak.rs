use oak_core::{Builder, ParseSession, Parser, SourceText};
use oak_vue::{VueBuilder, VueLanguage, VueParser};

#[test]
fn test_simple_vue() {
    let source = SourceText::new("<template><div>{{ msg }}</div></template>");
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();

    let result = parser.parse(&source, &[], &mut session);
    assert!(result.diagnostics.is_empty(), "Parser should not return errors");

    println!("Parse successful!");
}

#[test]
fn dynamic_directive_args_parse_and_build_without_hang() {
    let source = SourceText::new(
        "<template><button :[attrName]=\"val\" @[eventName]=\"onEv\">x</button></template>",
    );
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);

    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    assert!(built.result.is_ok(), "build failed: {:?}", built.diagnostics);
}

#[test]
fn keyword_attr_name_for_does_not_hang() {
    // `for` lexes as VueTokenType::For, not Identifier. Old parse_attribute never bumped
    // and the attribute loop allocated forever.
    let source = SourceText::new(
        r#"<template><label :for="controlId" class="x">{{ label }}</label></template>"#,
    );
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);

    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    assert!(built.result.is_ok(), "build failed: {:?}", built.diagnostics);
}

#[test]
fn autocomplete_like_template_parses_without_hang() {
    let source = SourceText::new(
        r#"<template>
  <div data-vmz-ui="autocomplete" :data-invalid="invalidAttr" class="vmz-ui-autocomplete">
    <label v-if="label" :for="controlId" class="vmz-ui-autocomplete__label">{{ label }}</label>
    <input :id="controlId" type="text" :aria-invalid='error ? "true" : "false"' :value="value" />
  </div>
</template>"#,
    );
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);
}

#[test]
fn mismatched_closing_tag_reports_diagnostic() {
    let source = SourceText::new("<template><div><span></div></template>");
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(
        parsed.diagnostics.iter().any(|d| d.to_string().contains("mismatched closing tag")),
        "expected mismatch diagnostic, got: {:?}",
        parsed.diagnostics
    );
}

#[test]
fn minimal_sfc_with_script_and_style_does_not_hang() {
    let source = SourceText::new(
        r#"<template>
  <p>{{ msg }}</p>
</template>
<script>
export default class Page {
  msg = 'hi';
}
</script>
<style>
.p { color: red; }
</style>"#,
    );
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);

    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    assert!(built.result.is_ok(), "build failed: {:?}", built.diagnostics);
}

#[test]
fn ternary_in_interpolation_parses() {
    let source = SourceText::new(
        r#"<template><button>{{ armed ? "ON" : "OFF" }}</button></template>"#,
    );
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);
}

#[test]
fn boolean_null_literals_in_bind_expr_parse() {
    let source = SourceText::new(
        r#"<template><div :a="true" :b="false" :c="null">x</div></template>"#,
    );
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);
}

#[test]
fn pascal_case_link_component_parses_children() {
    let source = SourceText::new(
        "<template><main><Link to=\"IndexPage\">Home</Link></main></template>",
    );
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);

    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    assert!(built.result.is_ok(), "build failed: {:?}", built.diagnostics);
}
