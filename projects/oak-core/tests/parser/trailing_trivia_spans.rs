//! Nodes must end exactly at their last token: whitespace and comments that follow
//! the token belong to the surrounding node, not to the node that was just closed.

use core::range::Range;
use oak_core::{
    ElementType, Language, RedTree, TokenType, UniversalElementRole, UniversalTokenRole,
    lexer::{LexOutput, Token, Tokens},
    memory::arena::SyntaxArena,
    parser::state::ParserState,
    tree::GreenNode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WordToken {
    Word,
    Space,
    End,
}

impl TokenType for WordToken {
    const END_OF_STREAM: Self = WordToken::End;
    type Role = UniversalTokenRole;
    fn role(&self) -> Self::Role {
        match self {
            WordToken::Space => UniversalTokenRole::Whitespace,
            _ => UniversalTokenRole::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WordElement {
    Root,
    Word,
}

impl ElementType for WordElement {
    type Role = UniversalElementRole;
    fn role(&self) -> Self::Role {
        match self {
            WordElement::Root => UniversalElementRole::Root,
            WordElement::Word => UniversalElementRole::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WordLanguage;

impl Language for WordLanguage {
    const NAME: &'static str = "word";
    type TokenType = WordToken;
    type ElementType = WordElement;
    type TypedRoot = ();
}

impl From<WordToken> for WordElement {
    fn from(_: WordToken) -> Self {
        WordElement::Word
    }
}

fn lex_words(text: &str) -> LexOutput<WordLanguage> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let start = pos;
        if bytes[pos] == b' ' {
            while pos < bytes.len() && bytes[pos] == b' ' {
                pos += 1
            }
            tokens.push(Token { kind: WordToken::Space, span: Range { start, end: pos } });
        }
        else {
            while pos < bytes.len() && bytes[pos] != b' ' {
                pos += 1
            }
            tokens.push(Token { kind: WordToken::Word, span: Range { start, end: pos } });
        }
    }
    tokens.push(Token { kind: WordToken::End, span: Range { start: pos, end: pos } });
    LexOutput::<WordLanguage> { result: Ok(Tokens::from(tokens)), diagnostics: Vec::new() }
}

fn parse_words<'a>(state: &mut ParserState<'a, WordLanguage>) -> &'a GreenNode<'a, WordLanguage> {
    let root = state.checkpoint();
    while state.not_at_end() && state.peek_kind() != Some(WordToken::End) {
        if state.peek_kind() == Some(WordToken::Word) {
            let word = state.checkpoint();
            state.bump();
            state.finish_at(word, WordElement::Word);
        }
        else {
            state.bump();
        }
    }
    state.finish_at(root, WordElement::Root)
}

#[test]
fn word_nodes_end_at_their_last_byte() {
    let text = "alpha beta ";
    let source = oak_core::source::SourceText::new(text);
    let arena = SyntaxArena::default();
    let mut state = ParserState::new(&arena, lex_words(text), &source, 64);
    let root = parse_words(&mut state);

    let tree = RedTree::new(root);
    assert_eq!((tree.span().start, tree.span().end), (0, text.len()));

    let mut words = Vec::new();
    for child in tree.children() {
        if let RedTree::Node(node) = child {
            assert_eq!(node.green.kind, WordElement::Word);
            words.push((node.span().start, node.span().end));
        }
    }
    assert_eq!(words, vec![(0, 5), (6, 10)]);
}
