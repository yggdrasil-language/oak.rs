use crate::lexer::{MarkdownLexer, State, token_type::MarkdownTokenType};
use oak_core::Source;

impl<'config> MarkdownLexer<'config> {
    /// Handles list markers.
    pub fn lex_list_marker<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        let mut check_pos = start_pos;
        while check_pos > 0 {
            check_pos -= 1;
            if let Some(ch) = state.source().get_char_at(check_pos) {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                else if ch != ' ' && ch != '\t' {
                    return false;
                }
            }
        }

        if let Some(ch) = state.peek() {
            match ch {
                '-' | '*' | '+' => {
                    state.advance(1);
                    if let Some(next_ch) = state.peek() {
                        if next_ch == ' ' || next_ch == '\t' {
                            state.add_token(MarkdownTokenType::ListMarker, start_pos, state.get_position());
                            return true;
                        }
                    }
                    state.set_position(start_pos);
                    false
                }
                '0'..='9' => {
                    while let Some(digit) = state.peek() {
                        if digit.is_ascii_digit() { state.advance(1) } else { break }
                    }

                    if let Some('.') = state.peek() {
                        state.advance(1);
                        if let Some(next_ch) = state.peek() {
                            if next_ch == ' ' || next_ch == '\t' {
                                state.add_token(MarkdownTokenType::ListMarker, start_pos, state.get_position());
                                return true;
                            }
                        }
                    }

                    state.set_position(start_pos);
                    false
                }
                _ => false,
            }
        }
        else {
            false
        }
    }

    /// Handles task markers.
    ///
    /// A task marker only exists as the first inline content of a list item,
    /// where it directly follows a line-start list marker. Anywhere else the
    /// `[ ]` / `[x]` / `[X]` shape is an ordinary bracket, so inputs such as
    /// `[x](https://e.com)` keep their `Link` (or `Image`) shape.
    pub fn lex_task_marker<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if !Self::at_list_item_content(state) {
            return false;
        }

        if let Some('[') = state.peek() {
            state.advance(1);
            if let Some(ch) = state.peek() {
                if ch == ' ' || ch == 'x' || ch == 'X' {
                    state.advance(1);
                    if let Some(']') = state.peek() {
                        state.advance(1);
                        state.add_token(MarkdownTokenType::TaskMarker, start_pos, state.get_position());
                        return true;
                    }
                }
            }
            state.set_position(start_pos);
        }
        false
    }

    /// Checks whether the already emitted tokens end with a list marker, so the
    /// current position is the start of a list item's content.
    ///
    /// [`Self::lex_list_marker`] only emits `ListMarker` at the beginning of a
    /// line (after optional indentation), which makes this token check the
    /// structural test for "inside a list item" without any source lookahead.
    fn at_list_item_content<S: Source + ?Sized>(state: &State<'_, S>) -> bool {
        let tokens = state.get_tokens();
        let mut index = tokens.len();

        if index > 0 && tokens[index - 1].kind == MarkdownTokenType::Whitespace {
            index -= 1;
        }

        index > 0 && tokens[index - 1].kind == MarkdownTokenType::ListMarker
    }

    /// Lexes definition list descriptions.
    pub fn lex_definition_description<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        let mut check_pos = start_pos;
        while check_pos > 0 {
            check_pos -= 1;
            if let Some(ch) = state.source().get_char_at(check_pos) {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                else if ch != ' ' && ch != '\t' {
                    return false;
                }
            }
        }

        if let Some(':') = state.peek() {
            state.advance(1);
            if let Some(next_ch) = state.peek() {
                if next_ch == ' ' || next_ch == '\t' {
                    state.add_token(MarkdownTokenType::DefinitionDescription, start_pos, state.get_position());
                    return true;
                }
            }
            state.set_position(start_pos);
            false
        }
        else {
            false
        }
    }

    /// Lexes abbreviation definitions.
    pub fn lex_abbreviation<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        let mut check_pos = start_pos;
        while check_pos > 0 {
            check_pos -= 1;
            if let Some(ch) = state.source().get_char_at(check_pos) {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                else if ch != ' ' && ch != '\t' {
                    return false;
                }
            }
        }

        if let Some('*') = state.peek() {
            state.advance(1);
            if let Some('[') = state.peek() {
                state.advance(1);

                while let Some(ch) = state.peek() {
                    if ch == ']' {
                        state.advance(1);
                        if let Some(':') = state.peek() {
                            state.advance(1);
                            state.add_token(MarkdownTokenType::Abbreviation, start_pos, state.get_position());
                            return true;
                        }
                        break;
                    }
                    state.advance(ch.len_utf8());
                }
            }
            state.set_position(start_pos);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::Block,
        builder::MarkdownBuilder,
        language::MarkdownLanguage,
        lexer::{MarkdownLexer, token_type::MarkdownTokenType},
        parser::{MarkdownParser, element_type::MarkdownElementType},
    };
    use oak_core::{Builder, Parser, RedNode, RedTree, parser::ParseSession};

    fn lex_kinds(source: &str) -> Vec<MarkdownTokenType> {
        let language = MarkdownLanguage::github_flavored();
        let lexer = MarkdownLexer::new(&language);
        let output = lexer.lex_internal(source);
        output.result.expect("lexing should succeed").iter().map(|token| token.kind).collect()
    }

    fn collect_elements(node: RedNode<'_, MarkdownLanguage>, elements: &mut Vec<MarkdownElementType>) {
        elements.push(node.element_type());
        for child in node.children() {
            if let RedTree::Node(child_node) = child {
                collect_elements(child_node, elements);
            }
        }
    }

    fn parse_elements(source: &str) -> Vec<MarkdownElementType> {
        let language = MarkdownLanguage::github_flavored();
        let parser = MarkdownParser::new(&language);
        let mut session = ParseSession::<MarkdownLanguage>::default();
        let output = parser.parse(source, &[], &mut session);
        let tree = output.result.expect("parsing should succeed");

        let mut elements = Vec::new();
        collect_elements(RedNode::new(tree, 0), &mut elements);
        elements
    }

    fn build_blocks(source: &str) -> Vec<Block> {
        let language = MarkdownLanguage::github_flavored();
        let builder = MarkdownBuilder::new(&language);
        let mut session = ParseSession::<MarkdownLanguage>::default();
        let output = builder.build(source, &[], &mut session);
        output.result.expect("building should succeed").blocks
    }

    #[test]
    fn task_markers_lex_at_list_item_content_start() {
        for source in ["- [ ] task", "- [x] done", "* [X] done", "1. [x] done"] {
            let kinds = lex_kinds(source);
            assert!(kinds.contains(&MarkdownTokenType::TaskMarker), "`{source}` should keep its task marker");
        }
    }

    #[test]
    fn list_items_keep_task_state() {
        let blocks = build_blocks("- [ ] task\n- [x] done\n");
        let Block::List(list) = &blocks[0]
        else {
            panic!("`- [ ] task` should build a list block")
        };

        assert_eq!(list.items.len(), 2);
        assert!(list.items[0].is_task);
        assert_eq!(list.items[0].is_checked, Some(false));
        assert!(list.items[1].is_task);
        assert_eq!(list.items[1].is_checked, Some(true));
    }

    #[test]
    fn bracket_links_are_not_task_markers() {
        for source in ["[x](https://e.com)", "[X](https://e.com)", "[ ](https://e.com)"] {
            let kinds = lex_kinds(source);
            assert!(kinds.contains(&MarkdownTokenType::Link), "`{source}` should lex as a link");
            assert!(!kinds.contains(&MarkdownTokenType::TaskMarker), "`{source}` should not lex as a task marker");
        }

        let elements = parse_elements("[x](https://e.com)");
        assert!(elements.contains(&MarkdownElementType::Link), "`[x](https://e.com)` should parse as a link element");
        assert!(!elements.contains(&MarkdownElementType::TaskMarker), "`[x](https://e.com)` should not parse as a task marker");
    }

    #[test]
    fn table_cell_bracket_links_are_not_task_markers() {
        let source = "| a | b |\n| --- | --- |\n| c | [x](https://e.com) |\n";

        let kinds = lex_kinds(source);
        assert!(kinds.contains(&MarkdownTokenType::Link), "the table cell should lex a link");
        assert!(!kinds.contains(&MarkdownTokenType::TaskMarker), "the table cell should not lex a task marker");

        let elements = parse_elements(source);
        assert!(elements.contains(&MarkdownElementType::Link), "the table cell should keep a link element");
        assert!(!elements.contains(&MarkdownElementType::TaskMarker), "the table cell should not keep a task marker");
    }
}
