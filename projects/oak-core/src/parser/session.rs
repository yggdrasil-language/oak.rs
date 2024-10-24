use crate::{
    Language,
    lexer::{LexOutput, LexerCache, Token},
    memory::arena::SyntaxArena,
    tree::GreenNode,
};
use std::{any::Any, cell::Cell, collections::HashMap, ptr::NonNull};

/// Trait for providing resources and caching for a parsing session.
pub trait ParseCache<L: Language>: LexerCache<L> {
    /// Returns the arena for allocating nodes in the current generation.
    fn arena(&self) -> &SyntaxArena;

    /// Returns the root of the previous tree for incremental parsing.
    fn old_tree(&self) -> Option<&GreenNode<'_, L>>;

    /// Returns the output of the lexing phase.
    fn lex_output(&self) -> Option<&LexOutput<L>>;

    /// Returns the fingerprint of the source text whose lex output and tree are
    /// currently cached.
    ///
    /// [`parse_with_lexer`](super::parse_with_lexer) compares it with the text it
    /// is asked to parse: when they differ the cache belongs to another document
    /// and must not be reused. Implementors that cannot provide a fingerprint
    /// simply keep the default and every parse re-lexes from scratch.
    fn source_fingerprint(&self) -> Option<u64> {
        None
    }

    /// Records the fingerprint of the source text being parsed.
    fn set_source_fingerprint(&mut self, _fingerprint: u64) {}

    /// Drops the cached lex output so the next parse lexes the document in full.
    ///
    /// This is called when a different document is handed to the same cache.
    fn clear_lex_output(&mut self) {}

    /// Prepares for a new parsing generation (e.g. by swapping arenas).
    fn prepare_generation(&mut self);

    /// Commits the result of a parsing generation.
    fn commit_generation(&self, root: &GreenNode<L>);
}

/// A memory pool that manages the lifecycle of parsing generations.
///
/// `ParseSession` handles the double-buffering of memory arenas (Active vs Old)
/// to support efficient incremental reuse. It implements `ParseCache`.
pub struct ParseSession<L: Language + Send + Sync> {
    /// The arena holding the most recently parsed tree (or currently building).
    arena_active: SyntaxArena,
    /// The arena holding the previous tree (used for reuse).
    arena_old: Option<SyntaxArena>,
    /// Pointer to the root of the last parsed tree.
    last_root: Cell<Option<NonNull<()>>>,
    /// Full output from the last lexing pass.
    last_lex: Option<LexOutput<L>>,
    /// Fingerprint of the source text that produced `last_lex` and `last_root`.
    last_source_fingerprint: Option<u64>,
    /// Cached typed nodes for incremental building.
    pub(crate) typed_nodes: HashMap<NonNull<()>, Box<dyn Any + Send + Sync>>,
}

unsafe impl<L: Language + Send + Sync> Send for ParseSession<L> {}
unsafe impl<L: Language + Send + Sync> Sync for ParseSession<L> {}

impl<L: Language + Send + Sync> Default for ParseSession<L> {
    fn default() -> Self {
        Self::new(16)
    }
}

impl<L: Language + Send + Sync> ParseSession<L> {
    /// Creates a new parse session.
    pub fn new(capacity: usize) -> Self {
        Self { arena_active: SyntaxArena::new(capacity), arena_old: None, last_root: Cell::new(None), last_lex: None, last_source_fingerprint: None, typed_nodes: HashMap::new() }
    }

    /// Returns the root of the last parsed tree.
    pub fn last_root(&self) -> Option<&GreenNode<'_, L>> {
        let ptr = self.last_root.get()?;
        // Safety: last_root is guaranteed to be in arena_active after commit_generation
        unsafe { Some(&*(ptr.as_ptr() as *const GreenNode<'_, L>)) }
    }

    pub(crate) fn get_typed_node<T: Any + Clone>(&self, node: &GreenNode<L>) -> Option<T> {
        let ptr = NonNull::from(node).cast::<()>();
        self.typed_nodes.get(&ptr).and_then(|any| any.downcast_ref::<T>()).cloned()
    }

    pub(crate) fn set_typed_node<T: Any + Send + Sync>(&mut self, node: &GreenNode<L>, value: T) {
        let ptr = NonNull::from(node).cast::<()>();
        self.typed_nodes.insert(ptr, Box::new(value));
    }
}

impl<L: Language + Send + Sync> ParseCache<L> for ParseSession<L> {
    fn arena(&self) -> &SyntaxArena {
        &self.arena_active
    }

    fn old_tree(&self) -> Option<&GreenNode<'_, L>> {
        let ptr = self.last_root.get()?;
        if self.arena_old.is_some() {
            // Safety: last_root is guaranteed to be in arena_old after swap_generations
            unsafe { Some(&*(ptr.as_ptr() as *const GreenNode<'_, L>)) }
        }
        else {
            None
        }
    }

    fn lex_output(&self) -> Option<&LexOutput<L>> {
        self.last_lex.as_ref()
    }

    fn source_fingerprint(&self) -> Option<u64> {
        self.last_source_fingerprint
    }

    fn set_source_fingerprint(&mut self, fingerprint: u64) {
        self.last_source_fingerprint = Some(fingerprint)
    }

    fn clear_lex_output(&mut self) {
        self.last_lex = None
    }

    fn prepare_generation(&mut self) {
        // 1. Move active to old. Oldest is dropped.
        self.arena_old = Some(std::mem::replace(&mut self.arena_active, SyntaxArena::new(16)));
        // 2. Clear caches
        self.last_lex = None;
        self.last_source_fingerprint = None;
        self.typed_nodes.clear();
    }

    fn commit_generation(&self, root: &GreenNode<L>) {
        let ptr = NonNull::from(root).cast::<()>();
        self.last_root.set(Some(ptr));
    }
}

impl<L: Language + Send + Sync> LexerCache<L> for ParseSession<L> {
    fn set_lex_output(&mut self, output: LexOutput<L>) {
        self.last_lex = Some(output)
    }

    fn get_token(&self, index: usize) -> Option<Token<L::TokenType>> {
        self.last_lex.as_ref()?.result.as_ref().ok()?.get(index).cloned()
    }

    fn count_tokens(&self) -> usize {
        self.last_lex.as_ref().and_then(|out| out.result.as_ref().ok()).map(|tokens| tokens.len()).unwrap_or(0)
    }

    fn has_tokens(&self) -> bool {
        self.last_lex.as_ref().and_then(|out| out.result.as_ref().ok()).map(|tokens| !tokens.is_empty()).unwrap_or(false)
    }

    fn get_tokens(&self) -> Option<&[Token<L::TokenType>]> {
        self.last_lex.as_ref()?.result.as_ref().ok().map(|tokens| &**tokens)
    }
}

impl<'a, L: Language, C: ParseCache<L> + ?Sized> ParseCache<L> for &'a mut C {
    fn arena(&self) -> &SyntaxArena {
        (**self).arena()
    }

    fn old_tree(&self) -> Option<&GreenNode<'_, L>> {
        (**self).old_tree()
    }

    fn lex_output(&self) -> Option<&LexOutput<L>> {
        (**self).lex_output()
    }

    fn source_fingerprint(&self) -> Option<u64> {
        (**self).source_fingerprint()
    }

    fn set_source_fingerprint(&mut self, fingerprint: u64) {
        (**self).set_source_fingerprint(fingerprint)
    }

    fn clear_lex_output(&mut self) {
        (**self).clear_lex_output()
    }

    fn prepare_generation(&mut self) {
        (**self).prepare_generation()
    }

    fn commit_generation(&self, root: &GreenNode<L>) {
        (**self).commit_generation(root)
    }
}
