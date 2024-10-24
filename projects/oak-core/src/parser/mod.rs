#![doc = include_str!("readme.md")]

/// Content-based caching for parsed results.
pub mod cache;
/// Pratt parser implementation for operator precedence parsing.
pub mod pratt;
/// Parser memory pool management.
pub mod session;
/// Internal parser state and checkpointing.
pub mod state;

pub use self::{
    cache::{CachingParseSession, ContentCache},
    pratt::{Associativity, OperatorInfo, Pratt, PrattParser, binary, postfix, unary},
    session::{ParseCache, ParseSession},
    state::{ParserState, deep_clone_node},
};

pub use triomphe::Arc;

pub use crate::{
    Language, Lexer,
    errors::{OakDiagnostics, OakError},
    source::{Source, TextEdit},
    tree::GreenNode,
};

/// Computes the fingerprint a [`ParseCache`] stores to recognise its document.
fn source_fingerprint<S: Source + ?Sized>(text: &S) -> u64 {
    use std::hash::Hasher;
    let mut hasher = twox_hash::XxHash64::default();
    hasher.write(text.get_text_from(0).as_bytes());
    hasher.finish()
}

/// The output of a parsing operation, containing the result and diagnostics.
pub type ParseOutput<'a, L: Language> = OakDiagnostics<&'a GreenNode<'a, L>>;

/// Core parser trait that defines the interface for language parsers.
///
/// This trait is responsible for converting a stream of tokens into a green tree
/// (a lossless, immutable syntax tree). It supports incremental parsing by
/// taking previous edits and using a cache for reuse.
///
/// # Usage Scenario
///
/// The `Parser` is typically used after lexical analysis to:
/// 1. Take a sequence of tokens produced by a [`Lexer`].
/// 2. Build a [`GreenNode`] tree representing the hierarchical structure of the source.
/// 3. Handle incremental updates by reusing nodes from a previous [`GreenNode`] tree.
///
/// # Incremental Parsing
///
/// The `parse` method should ideally be able to reuse nodes from a previous
/// parse if the source has only changed partially. This is facilitated by
/// the [`ParseCache`] and the provided [`TextEdit`]s.
pub trait Parser<L: Language + Send + Sync>
where
    L::ElementType: From<L::TokenType>,
{
    /// The core parsing entry point for converting tokens into a syntax tree.
    ///
    /// This method orchestrates the parsing process. It performs lexical analysis
    /// (if not already cached) and then builds a green tree structure. It handles
    /// incremental reuse automatically if the cache contains a previous tree.
    ///
    /// # Arguments
    ///
    /// * `text` - The source text to parse.
    /// * `edits` - A slice of [`TextEdit`]s representing changes since the last parse.
    ///             Used for incremental parsing.
    /// * `cache` - The [`ParseCache`] for resources, incremental reuse, and diagnostics.
    ///
    /// # Returns
    ///
    /// A [`ParseOutput`] containing the root [`GreenNode`] and any diagnostics.
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<L>) -> ParseOutput<'a, L>;
}

/// Standalone parsing function that coordinates lexing and parsing.
///
/// This is a convenience function for performing a complete parse (lexing + parsing)
/// in one call.
pub fn parse<'a, L, P, Lex, S>(parser: &P, _lexer: &Lex, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<L>) -> ParseOutput<'a, L>
where
    L: Language + Send + Sync,
    L::ElementType: From<L::TokenType>,
    P: Parser<L>,
    Lex: Lexer<L>,
    S: Source + ?Sized,
{
    parser.parse(text, edits, cache)
}

/// Standalone parsing function that performs a complete parse without incremental reuse.
///
/// This is a convenience function for parsing a source from scratch.
pub fn parse_one_pass<'a, L, P, S>(parser: &P, text: &'a S, cache: &'a mut impl ParseCache<L>) -> ParseOutput<'a, L>
where
    L: Language + Send + Sync,
    L::ElementType: From<L::TokenType>,
    P: Parser<L>,
    S: Source + ?Sized,
{
    parser.parse(text, &[], cache)
}

/// This function handles the boilerplate of preparing the cache, ensuring lexing is performed,
/// setting up the parser state, and committing the result.
///
/// The cached lex output and the previous tree are only reused for the same
/// document: a [`Source`] fingerprint is recorded in the [`ParseCache`] and any
/// text mismatch drops the cached tokens and skips the incremental base. Passing
/// a different document to an existing cache therefore always yields a tree for
/// that document, while re-parsing an unchanged document still hits the cache.
pub fn parse_with_lexer<'a, L, S, Lex>(lexer: &Lex, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<L>, run: impl FnOnce(&mut ParserState<'a, L, S>) -> Result<&'a GreenNode<'a, L>, OakError>) -> ParseOutput<'a, L>
where
    L: Language + Send + Sync,
    L::ElementType: From<L::TokenType>,
    S: Source + ?Sized,
    Lex: Lexer<L>,
{
    // 1. Check that the cache belongs to this document. Reusing the token
    //    stream or the tree of another text would silently produce a tree with
    //    the wrong content, so everything cached is gated on the fingerprint.
    let fingerprint = source_fingerprint(text);
    let same_source = cache.source_fingerprint() == Some(fingerprint);
    if !same_source {
        cache.clear_lex_output();
    }

    // 2. Get Lexing Result (Auto-lex if missing)
    let lex_out = match cache.lex_output().cloned() {
        Some(out) => out,
        None => {
            let out = lexer.lex(text, edits, cache);
            cache.set_lex_output(out.clone());
            out
        }
    };
    cache.set_source_fingerprint(fingerprint);

    let capacity_hint = cache.old_tree().map(|old| old.children.len().max(1024)).unwrap_or(1024);

    // 3. Initialize Parser State
    // Safety: We transmute the arena and old tree to 'a to satisfy the borrow checker.
    // The ParseCache guarantees that the arena and old tree live long enough.
    let arena: &'a crate::memory::arena::SyntaxArena = unsafe { std::mem::transmute(cache.arena()) };
    let mut st = ParserState::new(arena, lex_out, text, capacity_hint);

    if let Some(old) = cache.old_tree() {
        // The old tree belongs to the text of the previous generation. With an
        // empty edit list it may only be reused when the text is unchanged;
        // otherwise the caller has to describe the change through `edits`.
        if same_source || !edits.is_empty() {
            let old: &'a GreenNode<'a, L> = unsafe { std::mem::transmute(old) };
            st.set_incremental(old, edits);
        }
    }

    // 4. Run Parser Logic
    let result = run(&mut st);
    let output = st.finish(result);

    // 5. Commit Generation
    if let Ok(root) = output.result {
        cache.commit_generation(root);
    }

    output
}
