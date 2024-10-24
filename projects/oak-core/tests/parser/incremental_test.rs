use core::range::Range;
use oak_core::{
    ElementType, Language, TokenType, UniversalElementRole, UniversalTokenRole,
    lexer::{LexOutput, Lexer, LexerCache, Token, Tokens},
    memory::arena::SyntaxArena,
    parser::{ParseCache, ParseOutput, Parser, parse_with_lexer, state::ParserState},
    source::{Source, SourceText, TextEdit},
    tree::{GreenNode, GreenTree},
};
use std::cell::Cell;
use triomphe::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum MockToken {
    Item,
    Whitespace,
    End,
}

impl TokenType for MockToken {
    const END_OF_STREAM: Self = MockToken::End;
    type Role = UniversalTokenRole;
    fn role(&self) -> Self::Role {
        UniversalTokenRole::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum MockElement {
    Root,
    Item,
}

impl ElementType for MockElement {
    type Role = UniversalElementRole;
    fn role(&self) -> Self::Role {
        UniversalElementRole::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MockLanguage;
impl Language for MockLanguage {
    const NAME: &'static str = "mock";
    type TokenType = MockToken;
    type ElementType = MockElement;
    type TypedRoot = ();
}

// Simple manual lexer for the mock language
fn mock_lex(text: &str) -> LexOutput<MockLanguage> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let bytes = text.as_bytes();

    while pos < bytes.len() {
        if bytes[pos] == b' ' {
            tokens.push(Token { kind: MockToken::Whitespace, span: Range { start: pos, end: pos + 1 } });
            pos += 1
        }
        else if text[pos..].starts_with("item") {
            let start = pos;
            pos += 4;
            while pos < bytes.len() && bytes[pos].is_ascii_alphanumeric() {
                pos += 1
            }
            tokens.push(Token { kind: MockToken::Item, span: Range { start, end: pos } })
        }
        else {
            pos += 1; // skip unknown
        }
    }

    tokens.push(Token { kind: MockToken::End, span: Range { start: pos, end: pos } });

    LexOutput::<MockLanguage> { result: Ok(Tokens::from(tokens)), diagnostics: Vec::new() }
}

impl From<MockToken> for MockElement {
    fn from(_: MockToken) -> Self {
        MockElement::Item
    }
}

thread_local! {
    /// Number of `Item` nodes the mock parser built from scratch, i.e. that
    /// could not be taken from the previous tree.
    static ITEM_BUILDS: Cell<usize> = const { Cell::new(0) };
}

// Simple parser for the mock language
fn parse<'a, S: Source + ?Sized>(state: &mut ParserState<'a, MockLanguage, S>) -> &'a GreenNode<'a, MockLanguage> {
    let cp_root = state.checkpoint();

    while state.not_at_end() && state.peek_kind() != Some(MockToken::End) {
        if state.peek_kind() == Some(MockToken::Item) {
            // Try to reuse Item node
            if !state.try_reuse(MockElement::Item) {
                ITEM_BUILDS.with(|count| count.set(count.get() + 1));
                let cp_item = state.checkpoint();
                state.bump(); // eat the Item token
                state.finish_at(cp_item, MockElement::Item);
            }
        }
        else {
            state.bump(); // skip whitespace etc
        }
    }

    state.finish_at(cp_root, MockElement::Root)
}

#[test]
fn test_incremental_reuse() {
    let arena = SyntaxArena::default();
    let text1_str = "item1 item2 item3";
    let source1 = oak_core::source::SourceText::new(text1_str);
    let lex1 = mock_lex(text1_str);
    let mut state1 = ParserState::new(&arena, lex1, &source1, 1024);
    let tree1 = parse(&mut state1);

    // text2 has "itemX" instead of "item2"
    let text2_str = "item1 itemX item3";
    let source2 = oak_core::source::SourceText::new(text2_str);
    let lex2 = mock_lex(text2_str);

    // The edit is from "item2" (index 6-11) to "itemX" (index 6-11)
    let edits = vec![TextEdit { span: Range { start: 6, end: 11 }, text: "itemX".to_string().into() }];

    let arena2 = SyntaxArena::default();
    let mut state2 = ParserState::new(&arena2, lex2, &source2, 1024);
    state2.set_incremental(tree1, &edits);

    let tree2 = parse(&mut state2);

    // Check reuse
    // children: Item1, WS, ItemX, WS, Item3
    // Wait, the mock parser above doesn't handle WS as nodes.
    // It should have: Root -> [Item1, ItemX, Item3] if WS are skipped.

    let mut item_nodes = Vec::new();
    for child in tree2.children {
        if let GreenTree::Node(n) = child {
            if n.kind == MockElement::Item {
                item_nodes.push(n)
            }
        }
    }

    assert_eq!(item_nodes.len(), 3);

    for (i, node) in item_nodes.iter().enumerate() {
        println!("Node {}: kind={:?}, len={}", i, node.kind, node.text_len())
    }

    // Item1 should be reused
    assert_eq!(item_nodes[0].kind, MockElement::Item);
    assert_eq!(item_nodes[0].text_len(), 5);

    // ItemX should be new
    assert_eq!(item_nodes[1].kind, MockElement::Item);
    assert_eq!(item_nodes[1].text_len(), 5);

    // Item3 should be reused
    assert_eq!(item_nodes[2].kind, MockElement::Item);
    assert_eq!(item_nodes[2].text_len(), 5)
}

/// Counts how often it is asked to lex, so tests can tell a cache hit from a
/// re-lex.
#[derive(Default)]
struct CountingLexer {
    calls: Cell<usize>,
}

impl Lexer<MockLanguage> for CountingLexer {
    fn lex<'a, S: Source + ?Sized>(&self, text: &S, _edits: &[TextEdit], _cache: &'a mut impl LexerCache<MockLanguage>) -> LexOutput<MockLanguage> {
        self.calls.set(self.calls.get() + 1);
        mock_lex(&text.get_text_from(0))
    }
}

/// Drives the mock parser through [`parse_with_lexer`], i.e. the same
/// lexing/parsing/caching orchestration real languages use.
struct MockParser<'a> {
    lexer: &'a CountingLexer,
}

impl Parser<MockLanguage> for MockParser<'_> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<MockLanguage>) -> ParseOutput<'a, MockLanguage> {
        parse_with_lexer(self.lexer, text, edits, cache, |state| Ok(parse(state)))
    }
}

fn item_nodes<'a>(root: &'a GreenNode<'a, MockLanguage>) -> Vec<&'a GreenNode<'a, MockLanguage>> {
    root.children
        .iter()
        .filter_map(|child| match *child {
            GreenTree::Node(n) if n.kind == MockElement::Item => Some(n),
            _ => None,
        })
        .collect()
}

/// The same document parsed twice with the same session must hit the cached
/// lex output instead of lexing again.
#[test]
fn test_unchanged_document_hits_the_cached_lex_output() {
    let lexer = CountingLexer::default();
    let parser = MockParser { lexer: &lexer };
    let mut session = oak_core::parser::ParseSession::<MockLanguage>::default();

    let source = SourceText::new("item1 item2");
    parser.parse(&source, &[], &mut session).result.expect("first parse");
    // A distinct source object with the same text is still the same document.
    let source_again = SourceText::new("item1 item2");
    parser.parse(&source_again, &[], &mut session).result.expect("second parse");

    assert_eq!(lexer.calls.get(), 1, "an unchanged document must not be lexed again");
}

/// A different document handed to the same session must never reuse the
/// previous document's tokens or tree.
#[test]
fn test_different_document_does_not_reuse_the_previous_tree() {
    let lexer = CountingLexer::default();
    let parser = MockParser { lexer: &lexer };
    let mut session = oak_core::parser::ParseSession::<MockLanguage>::default();

    let source1 = SourceText::new("item1 item2");
    parser.parse(&source1, &[], &mut session).result.expect("first parse");

    let source2 = SourceText::new("nothing");
    let tree2 = parser.parse(&source2, &[], &mut session).result.expect("second parse");

    assert!(item_nodes(tree2).is_empty(), "the second tree must describe the second document");
    assert_eq!(lexer.calls.get(), 2, "a different document must be lexed again");
}

/// A new generation with a different document and no edits must skip the old
/// tree instead of incrementally reusing it.
#[test]
fn test_different_document_without_edits_skips_the_old_tree() {
    let lexer = CountingLexer::default();
    let parser = MockParser { lexer: &lexer };
    let mut session = oak_core::parser::ParseSession::<MockLanguage>::default();

    let source1 = SourceText::new("item1 item2");
    parser.parse(&source1, &[], &mut session).result.expect("first parse");

    session.prepare_generation();

    let source2 = SourceText::new("nothing");
    let tree2 = parser.parse(&source2, &[], &mut session).result.expect("second parse");

    assert!(item_nodes(tree2).is_empty(), "the old tree of another document must not be reused");
    assert_eq!(lexer.calls.get(), 2);
}

/// Edits still take the incremental path: unchanged nodes are reused.
#[test]
fn test_edits_reuse_nodes_from_the_previous_tree() {
    let lexer = CountingLexer::default();
    let parser = MockParser { lexer: &lexer };
    let mut session = oak_core::parser::ParseSession::<MockLanguage>::default();

    let first_builds = {
        let source1 = SourceText::new("item1 item2 item3");
        let tree1 = parser.parse(&source1, &[], &mut session).result.expect("first parse");
        assert_eq!(item_nodes(tree1).len(), 3);
        ITEM_BUILDS.with(|count| count.get())
    };
    assert_eq!(first_builds, 3, "all items are built when there is no previous tree");

    session.prepare_generation();
    ITEM_BUILDS.with(|count| count.set(0));

    let source2 = SourceText::new("item1 itemX item3");
    let edits = [TextEdit { span: Range { start: 6, end: 11 }, text: "itemX".to_string().into() }];
    let tree2 = parser.parse(&source2, &edits, &mut session).result.expect("incremental parse");

    let items2 = item_nodes(tree2);
    assert_eq!(items2.len(), 3, "the parsed tree must still contain all items");
    assert!(ITEM_BUILDS.with(|count| count.get()) < 3, "at least one item must be reused from the previous tree");
}
