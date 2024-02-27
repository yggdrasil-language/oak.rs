use oak_core::{Builder, ParseSession, SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage, ast::StatementNode};

const TEMPLATE: &str = r#"<% match arch %>
<% case "wasm32" %>
[main] micro main() -> i32 { return 23 }
<% else %>
[main] micro main() -> i32 { return 0 }
<% end %>"#;

#[test]
fn module_template_accepts_resolver_style_trailing_newline() {
    let language = ValkyrieLanguage { support_t_grammar: true, ..ValkyrieLanguage::default() };
    let source = format!("{TEMPLATE}\n");
    let builder = ValkyrieBuilder::new(&language);
    let text = SourceText::new(source);
    let mut session = ParseSession::<ValkyrieLanguage>::default();
    let output = builder.build(&text, &[], &mut session);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let root = output.result.expect("parse");
    assert_eq!(root.items.len(), 1);
    assert!(matches!(root.items[0], StatementNode::Template(_)));
}
