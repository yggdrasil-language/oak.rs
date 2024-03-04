//! vcc-data ABI index / template / symbols tests.

use crate::{AwslParser, ComponentAbiIndex, TemplateBindingKind, collect_abi_references, collect_template_bindings, extract_component_abi_from_script};

#[test]
fn template_bindings_on_child_component() {
    let source = r#"<widget parent>
    <Switch :checked="value" @change="on_change" />
</widget>
<script>
[property] let value: bool = false
[event] micro change(value: bool) {}
</script>"#;
    let root = AwslParser::parse_root(source).expect("parse");
    let bindings = collect_template_bindings(&root);
    assert_eq!(bindings.len(), 2);
    assert!(bindings.iter().any(|b| b.name == "checked" && b.kind == TemplateBindingKind::Property));
    assert!(bindings.iter().any(|b| b.name == "change" && b.kind == TemplateBindingKind::Event));
}

#[test]
fn abi_index_validates_missing_property() {
    let child_script = "[property] let checked: bool = false";
    let child_abi = extract_component_abi_from_script(child_script, "switch");
    let index = ComponentAbiIndex::from_entries([("switch".into(), child_abi.abi)]);
    let parent = r#"<widget parent><Switch :missing="x" /></widget><script></script>"#;
    let root = AwslParser::parse_root(parent).expect("parse");
    let issues = index.validate_awsl_root(&root);
    assert!(issues.iter().any(|i| i.message.contains("missing")));
}

#[test]
fn collect_abi_references_includes_template_and_script() {
    let source = r#"<widget demo><Child :theme="theme" /></widget>
<script>
[property] let theme: utf8 = ""
</script>"#;
    let root = AwslParser::parse_root(source).expect("parse");
    let abi = extract_component_abi_from_script(root.script.as_deref().unwrap_or(""), "demo").abi;
    let refs = collect_abi_references(&abi, "", &root);
    assert!(refs.iter().any(|r| r.name == "theme"));
}
