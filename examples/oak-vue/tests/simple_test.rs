use oak_core::{Builder, ParseSession, Parser, Source, SourceText};
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
    let source = SourceText::new("<template><button :[attrName]=\"val\" @[eventName]=\"onEv\">x</button></template>");
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
    let source = SourceText::new(r#"<template><label :for="controlId" class="x">{{ label }}</label></template>"#);
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
    assert!(parsed.diagnostics.iter().any(|d| d.to_string().contains("mismatched closing tag")), "expected mismatch diagnostic, got: {:?}", parsed.diagnostics);
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
    let source = SourceText::new(r#"<template><button>{{ armed ? "ON" : "OFF" }}</button></template>"#);
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);
}

#[test]
fn boolean_null_literals_in_bind_expr_parse() {
    let source = SourceText::new(r#"<template><div :a="true" :b="false" :c="null">x</div></template>"#);
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);
}

#[test]
fn pascal_case_link_component_parses_children() {
    let source = SourceText::new("<template><main><Link to=\"IndexPage\">Home</Link></main></template>");
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);

    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    let root = built.result.expect("build failed");
    let template = root.blocks.iter().find(|b| source.get_text_in(b.name.clone()) == "template").expect("template");
    let main = match &template.children[0] {
        oak_vue::VueNode::Element(el) => el,
        other => panic!("expected main element, got {other:?}"),
    };
    let link = match &main.children[0] {
        oak_vue::VueNode::Element(el) => el,
        other => panic!("expected Link element, got {other:?}"),
    };
    assert_eq!(source.get_text_in(link.tag_name.clone()), "Link");
    let oak_vue::VueAttribute::Attribute(attr) = &link.attributes[0]
    else {
        panic!("expected static attribute, got {:?}", link.attributes);
    };
    assert_eq!(source.get_text_in(attr.name.clone()), "to");
    let value = attr.value.as_ref().expect("value");
    assert_eq!(source.get_text_in(value.span.clone()), "IndexPage");
}

#[test]
fn text_before_interpolation_is_retained() {
    let source = SourceText::new("<template><p>Hi {{ name }}</p></template>");
    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    let root = built.result.expect("build failed");
    let template = root.blocks.iter().find(|b| source.get_text_in(b.name.clone()) == "template").expect("template");
    let p = match &template.children[0] {
        oak_vue::VueNode::Element(el) => el,
        other => panic!("expected p element, got {other:?}"),
    };
    assert!(p.children.iter().any(|c| matches!(c, oak_vue::VueNode::Text(t) if source.get_text_in(t.span.clone()).contains("Hi"))), "expected text child with Hi, got {:?}", p.children);
    assert!(p.children.iter().any(|c| matches!(c, oak_vue::VueNode::Interpolation(_))), "expected interpolation child");
}

#[test]
fn nested_template_hash_slot_builds_usable_ast() {
    let source = SourceText::new(r#"<template><Comp><template #title>T</template></Comp></template>"#);
    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    assert!(built.result.is_ok(), "build failed: {:?}", built.diagnostics);
    let root = built.result.unwrap();
    let template = root.blocks.iter().find(|b| source.get_text_in(b.name.clone()) == "template").expect("template");
    let comp = match &template.children[0] {
        oak_vue::VueNode::Element(el) => el,
        other => panic!("expected Comp, got {other:?}"),
    };
    assert_eq!(source.get_text_in(comp.tag_name.clone()), "Comp");
    let slot_tpl = match &comp.children[0] {
        oak_vue::VueNode::Element(el) => el,
        other => panic!("expected nested template element, got {other:?}"),
    };
    let tag_text = source.get_text_in(slot_tpl.tag_name.clone());
    let span_text = source.get_text_in(slot_tpl.span.clone());
    assert_eq!(tag_text, "template", "nested <template> tag_name broken (TemplateStart leaf?). tag_range={:?} span_text={span_text:?} attrs={:?}", slot_tpl.tag_name, slot_tpl.attributes);
    assert!(!slot_tpl.attributes.is_empty(), "expected #title directive attr, got none");
    let oak_vue::VueAttribute::Directive(dir) = &slot_tpl.attributes[0]
    else {
        panic!("expected Directive for #title, got {:?}", slot_tpl.attributes);
    };
    let dir_span = source.get_text_in(dir.span.clone());
    assert_eq!(dir_span, "#title", "directive span misaligned: {dir_span:?}");
    assert!(slot_tpl.children.iter().any(|c| matches!(c, oak_vue::VueNode::Text(t) if source.get_text_in(t.span.clone()).contains('T'))), "expected text child T, got {:?}", slot_tpl.children);
}

#[test]
fn vmz_agreed_matrix_builds_without_hang() {
    let cases = [
        ("text_interp", r#"<p>Hi {{ name }}</p>"#),
        ("void_input", r#"<input type="text" />"#),
        ("bind_on", r#"<button :disabled="busy" @click="go">Go</button>"#),
        ("v_if_else", r#"<p v-if="a">A</p><p v-else>B</p>"#),
        ("v_if_elseif_else", r#"<p v-if="a">A</p><p v-else-if="b">B</p><p v-else>C</p>"#),
        ("v_for_key", r#"<li v-for="tag in tags" :key="tag.id">{{ tag.label }}</li>"#),
        ("slot_outlet", r#"<slot name="footer"><p>fallback</p></slot>"#),
        ("v_model", r#"<input v-model="q" />"#),
        ("component_link", r#"<Link to="Home">Go</Link>"#),
        ("dynamic_bind_on", r#"<button :[attrName]="val" @[eventName]="onEv">x</button>"#),
        ("ternary_interp", r#"<span>{{ ok ? 'y' : 'n' }}</span>"#),
        ("hash_slot", r#"<Comp><template #title>T</template></Comp>"#),
        ("v_slot", r#"<Comp><template v-slot:footer>F</template></Comp>"#),
        ("class_style_plans", r#"<div class="a b" :class="extra" style="color:red" :style="dyn">x</div>"#),
        ("on_modifier", r#"<button @click.stop.prevent="go">Go</button>"#),
        ("v_for_aliases", r#"<li v-for="(item, index) in items" :key="index">{{ item }}</li>"#),
        ("v_model_arg", r#"<input v-model:title="doc.title" />"#),
        ("bind_object", r#"<div v-bind="attrs">x</div>"#),
        ("comment_in_if_chain", r#"<p v-if="a">A</p><!-- between --><p v-else>B</p>"#),
        ("nested_elements", r#"<main><section><p>{{ t }}</p></section></main>"#),
        ("counter_option_value", r#"<option value>—</option>"#),
    ];
    for (name, body) in cases {
        let shell = format!("<template>{body}</template>");
        let source = SourceText::new(shell.as_str());
        let builder = VueBuilder::new();
        let mut cache = ParseSession::default();
        let built = Builder::build(&builder, &source, &[], &mut cache);
        assert!(built.result.is_ok(), "{name}: build failed: {:?}", built.diagnostics);
    }
}

#[test]
fn counter_button_cst_parse_does_not_hang() {
    let source = SourceText::new(
        r#"<template>
    <div data-testid="counter-root">
        <label>
            Note
            <input type="text" data-testid="counter-note" :value="note" @input="((e) => note = e.target.value)" />
        </label>
        <button type="button" aria-label="Increment counter" data-testid="counter-inc" @click="(() => count++)">
            count:
            {{ count }}
        </button>
        <label>
            Plan
            <select data-testid="counter-plan" :value="plan" @change="((e) => plan = e.target.value)">
                <option value>
                    —
                </option>
                <option value="ops">
                    Ops
                </option>
                <option value="dev">
                    Dev
                </option>
            </select>
        </label>
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
fn event_shell_event_button_template_builds_without_hang() {
    let source = SourceText::new(
        r#"<template>
    <button type="button" @click="(() => armed = !armed)">
        {{ label }}
        :
        {{ armed ? "ON" : "OFF" }}
    </button>
</template>"#,
    );
    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    assert!(built.result.is_ok(), "build failed: {:?}", built.diagnostics);
}

#[test]
fn event_shell_index_client_event_attr_builds_without_hang() {
    let source = SourceText::new(
        r#"<template>
    <article>
        <EventButton client:event label="Go" />
    </article>
</template>"#,
    );
    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    assert!(built.result.is_ok(), "build failed: {:?}", built.diagnostics);
}

#[test]
fn nested_template_v_slot_directive_builds() {
    let source = SourceText::new(r#"<template><Card><template v-slot:footer>f</template></Card></template>"#);
    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    let root = built.result.expect("build failed");
    let template = root.blocks.iter().find(|b| source.get_text_in(b.name.clone()) == "template").expect("template");
    let card = match &template.children[0] {
        oak_vue::VueNode::Element(el) => el,
        other => panic!("expected Card, got {other:?}"),
    };
    let slot_tpl = match &card.children[0] {
        oak_vue::VueNode::Element(el) => el,
        other => panic!("expected nested template, got {other:?}"),
    };
    assert_eq!(source.get_text_in(slot_tpl.tag_name.clone()), "template");
    let oak_vue::VueAttribute::Directive(dir) = &slot_tpl.attributes[0]
    else {
        panic!("expected Directive, got {:?}", slot_tpl.attributes);
    };
    assert_eq!(source.get_text_in(dir.span.clone()), "v-slot:footer");
}
