use oak_awsl::Lexer;

#[test]
fn peek_top_level_template_open_recognizes_widget_and_template() {
    let widget = Lexer::new("<widget todo>");
    assert_eq!(widget.peek_top_level_template_open(), Some("widget"));

    let template = Lexer::new("<template App>");
    assert_eq!(template.peek_top_level_template_open(), Some("template"));

    let div = Lexer::new("<div>");
    assert_eq!(div.peek_top_level_template_open(), None);
}

#[test]
fn read_name_reads_tag_and_attr_names() {
    let mut lexer = Lexer::new("TodoItem on:click");
    assert_eq!(lexer.read_name(), "TodoItem");
    lexer.skip_trivia();
    assert_eq!(lexer.read_name_with_colon(), "on:click");
}

#[test]
fn read_quoted_string_supports_single_and_double_quotes() {
    let mut single = Lexer::new("'all'");
    assert_eq!(single.read_quoted_string().unwrap(), "all");

    let mut double = Lexer::new("\"active\"");
    assert_eq!(double.read_quoted_string().unwrap(), "active");
}

#[test]
fn read_braced_expr_handles_nested_braces() {
    let mut lexer = Lexer::new("{filter == 'all' ? 'active' : ''}");
    assert_eq!(lexer.read_braced_expr().unwrap(), "filter == 'all' ? 'active' : ''");
}

#[test]
fn read_raw_expr_until_gt_stops_at_tag_close() {
    let mut lexer = Lexer::new("todos() key=\"item.id\"");
    assert_eq!(lexer.read_raw_expr_until_gt(), "todos() key=\"item.id\"");
}
