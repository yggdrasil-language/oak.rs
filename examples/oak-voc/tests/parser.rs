use oak_awsl::{AwslAttributeValue, AwslDirectiveKind, AwslElement, AwslParser, AwslTemplateNode, validate_component_contract};

fn expect_element<'a>(node: &'a AwslTemplateNode) -> &'a AwslElement {
    match node {
        AwslTemplateNode::Element(element) => element,
        other => panic!("expected element, got {other:?}"),
    }
}

fn non_empty_element_children(element: &AwslElement) -> Vec<&AwslElement> {
    element
        .children
        .iter()
        .filter_map(|node| match node {
            AwslTemplateNode::Element(child) => Some(child),
            AwslTemplateNode::Text { content, .. } if content.trim().is_empty() => None,
            other => panic!("expected element child, got {other:?}"),
        })
        .collect()
}

#[test]
fn parse_multiline_widget_whitespace() {
    AwslParser::parse_root("<widget><div>\n<button>x</button>\n</div></widget>").expect("inner newlines");
    AwslParser::parse_root("<widget>\n<div><button>x</button></div>\n</widget>").expect("outer newlines");
}

#[test]
fn parse_widget_with_script_style_and_template() {
    let source = r#"<script>
[property] let label = ""
</script>
<style>.btn { }</style>
<widget button>
<div>{label}</div>
</widget>"#;
    let root = AwslParser::parse_root(source).expect("parse widget shell");
    assert!(root.script.is_some());
    assert!(root.style.is_some());
    assert!(!root.template.is_empty());
}

#[test]
fn parse_control_flow_tags() {
    let source = "<widget>\n\
        <loop item in todos()><span /></loop>\n\
        <if !empty><span /></if>\n\
    </widget>";
    let root = AwslParser::parse_root(source).expect("control flow");
    assert!(!root.template.is_empty());
}

#[test]
fn widget_and_template_are_equivalent_top_level_containers() {
    let widget_source = "<widget todo>\n\
        <div class=\"app\"><span>{label}</span></div>\n\
    </widget>";
    let template_source = "<template todo>\n\
        <div class=\"app\"><span>{label}</span></div>\n\
    </template>";

    let widget_root = AwslParser::parse_root(widget_source).expect("widget");
    let template_root = AwslParser::parse_root(template_source).expect("template");

    assert_eq!(widget_root.widget_name, Some("todo".into()));
    assert_eq!(template_root.widget_name, Some("todo".into()));
    assert_eq!(widget_root.template.len(), template_root.template.len());

    let widget_div = expect_element(&widget_root.template[0]);
    let template_div = expect_element(&template_root.template[0]);
    assert_eq!(widget_div.tag, "div");
    assert_eq!(template_div.tag, "div");
    assert_eq!(widget_div.children.len(), template_div.children.len());
}

#[test]
fn nested_element_tree_preserves_parent_child_relationships() {
    let source = "<widget><div class=\"wrap\"><button @click=\"handler\">{label}</button></div></widget>";
    let root = AwslParser::parse_root(source).expect("nested tree");

    assert_eq!(root.template.len(), 1);
    let div = expect_element(&root.template[0]);
    assert_eq!(div.tag, "div");
    assert_eq!(div.attributes.len(), 1);
    assert_eq!(non_empty_element_children(div).len(), 1);

    let button = non_empty_element_children(div)[0];
    assert_eq!(button.tag, "button");
    assert!(button.children.iter().any(|child| matches!(child, AwslTemplateNode::Interpolation { expr, .. } if expr == "label")));
}

#[test]
fn rejects_nested_script_style_or_template_sections() {
    for source in [
        "<widget todo><script>let x = 1</script></widget>",
        "<widget todo><style>.x {}</style></widget>",
        "<widget todo><template><div /></template></widget>",
        "<widget todo><script>let x = 1</script><style>.x {}</style><template><div /></template></widget>",
    ] {
        let error = AwslParser::parse_root(source).expect_err("nested section should fail");
        assert!(error.message.contains("top-level sibling"), "unexpected error for `{source}`: {}", error.message);
    }
}

#[test]
fn loop_and_if_children_are_nested_under_control_flow_tags() {
    let source = r#"<template>
<loop item in todos()>
<TodoItem :id="item.id" />
</loop>
<if !todos()>
<div class="empty" />
</if>
</template>"#;
    let root = AwslParser::parse_root(source).expect("control flow nesting");

    assert_eq!(root.template.len(), 2);
    let loop_el = expect_element(&root.template[0]);
    assert_eq!(loop_el.tag, "loop");
    assert_eq!(non_empty_element_children(loop_el).len(), 1);
    assert_eq!(non_empty_element_children(loop_el)[0].tag, "TodoItem");

    let if_el = expect_element(&root.template[1]);
    assert_eq!(if_el.tag, "if");
    assert_eq!(non_empty_element_children(if_el).len(), 1);
    assert_eq!(non_empty_element_children(if_el)[0].tag, "div");
}

#[test]
fn widget_with_script_style_as_sibling_sections() {
    let source = "<widget todo>\n\
        <div class=\"todo-app\" />\n\
    </widget>\n\
    <script>let x = 1</script>\n\
    <style>.x {}</style>";
    let root = AwslParser::parse_root(source).expect("sections");

    assert_eq!(root.widget_name, Some("todo".into()));
    assert!(root.script.as_ref().is_some_and(|s| s.contains("let x = 1")));
    assert!(root.style.as_ref().is_some_and(|s| s.contains(".x")));
    assert_eq!(root.template.len(), 1);
    assert_eq!(expect_element(&root.template[0]).tag, "div");
}

#[test]
fn rejects_script_only_without_widget_shell() {
    let root = AwslParser::parse_root("<script>let x = 1</script>").unwrap();
    assert!(!root.has_widget_shell);
    let contract = oak_awsl::validate_component_contract(&root, "x");
    assert!(contract.is_err());
}

#[test]
fn widget_name_must_match_file_stem_when_declared() {
    let source = "<widget foo><div /></widget>";
    let root = AwslParser::parse_root(source).expect("parse");
    let error = oak_awsl::validate_component_contract(&root, "bar").expect_err("mismatch");
    assert!(error.message.contains("expected <widget bar>"));
}

#[test]
fn rejects_script_or_style_nested_in_markup() {
    let source = "<widget><div><script>let x = 1</script></div></widget>";
    let error = AwslParser::parse_root(source).expect_err("nested script in markup should fail");
    assert!(error.message.contains("top-level sibling"));
}

#[test]
fn widget_accepts_multiple_top_level_tags() {
    let source = "<widget>\n\
        <div class=\"a\" />\n\
        <div class=\"b\" />\n\
    </widget>";
    let root = AwslParser::parse_root(source).expect("multi-root widget");
    assert!(root.has_widget_shell);
    assert_eq!(root.template.len(), 2);
}

#[test]
fn html_void_elements_parse_without_close_tag() {
    let source = "<widget><div><br><img src=\"/x.png\"><input type=\"text\"></div></widget>";
    let root = AwslParser::parse_root(source).expect("void html");
    let div = expect_element(&root.template[0]);
    assert_eq!(div.children.len(), 3);
    assert!(div.children.iter().all(|node| matches!(node, AwslTemplateNode::Element(el) if el.self_closing)));
}

#[test]
fn static_class_attribute_is_quoted_string_literal() {
    let source = "<widget><text class=\"counter\">hi</text></widget>";
    let root = AwslParser::parse_root(source).expect("parse class");
    let text = expect_element(&root.template[0]);
    let class_attr = text.attributes.iter().find(|a| a.name == "class").expect("class attr");
    assert!(matches!(&class_attr.value, AwslAttributeValue::Literal(v) if v == "counter"));
}

#[test]
fn rejects_bare_literal_attribute_values() {
    let error = AwslParser::parse_root("<widget><div class=wrap /></widget>").expect_err("bare class");
    assert!(error.message.contains("quoted"));
}

#[test]
fn accepts_quoted_static_attribute_values() {
    AwslParser::parse_root("<widget><div class=\"wrap\" /></widget>").expect("quoted class");
}

#[test]
fn rejects_braced_attribute_values() {
    let error = AwslParser::parse_root("<widget><Text title={title} /></widget>").expect_err("braced attr");
    assert!(error.message.contains("quoted") || error.message.contains("static") || error.message.contains("braced"), "unexpected error: {}", error.message);
}

#[test]
fn click_directive_uses_quoted_handler() {
    let source = "<widget><button @click=\"increment\">+1</button></widget>";
    let root = AwslParser::parse_root(source).expect("parse @click");
    let button = expect_element(&root.template[0]);
    assert!(button.directives.iter().any(|d| d.value.as_deref() == Some("increment")));
}

#[test]
fn colon_bind_uses_quoted_expression() {
    let source = "<widget><input :bind=\"name\" /></widget>";
    let root = AwslParser::parse_root(source).expect("parse :bind");
    let input = expect_element(&root.template[0]);
    assert_eq!(input.attributes[0].name, "bind");
    assert!(matches!(&input.attributes[0].value, AwslAttributeValue::Expression(v) if v == "name"));
}

#[test]
fn style_directive_accepts_tailwind_string() {
    let source = "<widget><div @style=\"flex w-4 h-4 rounded bg-blue-500\">x</div></widget>";
    let root = AwslParser::parse_root(source).expect("parse @style");
    let div = expect_element(&root.template[0]);
    assert!(div.directives.iter().any(|d| matches!(&d.kind, AwslDirectiveKind::Style)));
}

#[test]
fn loop_directive_parses_item_in_expr() {
    let source = "<widget><ul @loop=\"item in items()\"><li>{item}</li></ul></widget>";
    let root = AwslParser::parse_root(source).expect("parse @loop");
    let ul = expect_element(&root.template[0]);
    assert!(ul.directives.iter().any(|d| d.value.as_deref() == Some("item in items()")));
}

#[test]
fn rejects_bare_click_directive() {
    let error = AwslParser::parse_root("<widget><button @click=increment /></widget>").expect_err("bare @click");
    assert!(error.message.contains("quoted"));
}

#[test]
fn rejects_on_colon_click_attribute() {
    let error = AwslParser::parse_root("<widget><button on:click=\"handler\" /></widget>").expect_err("on:click");
    assert!(error.message.contains("@click"));
}

#[test]
fn rejects_nested_widget_or_template_elements() {
    let source = "<widget><div><widget /></div></widget>";
    let error = AwslParser::parse_root(source).expect_err("nested widget should fail");
    assert!(error.message.contains("top-level template container"));
}
