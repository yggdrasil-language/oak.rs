use crate::{
    Language,
    lexer::{LexOutput, LexerCache, Token},
    memory::arena::SyntaxArena,
    parser::session::ParseCache,
    source::Source,
    tree::GreenNode,
};
use std::{collections::HashMap, hash::Hasher, sync::Arc};
use twox_hash::XxHash64;

/// A content-based cache for parsed results.
///
/// This cache stores parsed results based on the hash of the source content,
/// allowing for efficient reuse of parsing results when processing the same content multiple times.
pub struct ContentCache<L: Language + Send + Sync + 'static> {
    /// Cache entries mapping content hash to parsed results.
    entries: HashMap<u64, CacheEntry<L>>,
    /// Maximum number of entries to keep in the cache.
    max_entries: usize,
    /// Current number of entries in the cache.
    entry_count: usize,
}

/// A single entry in the content cache.
struct CacheEntry<L: Language + Send + Sync + 'static> {
    /// The parsed green node.
    root: Arc<GreenNode<'static, L>>,
    /// The lex output associated with this entry.
    lex_output: LexOutput<L>,
}

impl<L: Language + Send + Sync + 'static> ContentCache<L> {
    /// Creates a new content cache with the specified maximum size.
    pub fn new(max_entries: usize) -> Self {
        Self { entries: HashMap::new(), max_entries, entry_count: 0 }
    }

    /// Computes a hash for the given source content.
    fn hash_content<S: Source + ?Sized>(source: &S) -> u64 {
        let mut hasher = XxHash64::default();
        let text = source.get_text_from(0);
        hasher.write(text.as_bytes());
        hasher.finish()
    }

    /// Gets a cached entry for the given source content if it exists.
    pub fn get<S: Source + ?Sized>(&self, source: &S) -> Option<(&GreenNode<'_, L>, &LexOutput<L>)> {
        let hash = Self::hash_content(source);
        self.entries.get(&hash).map(|entry| {
            let root: &GreenNode<'_, L> = unsafe { std::mem::transmute(&*entry.root) };
            (root, &entry.lex_output)
        })
    }

    /// Inserts a new entry into the cache.
    pub fn insert<S: Source + ?Sized>(&mut self, source: &S, root: &GreenNode<'_, L>, lex_output: LexOutput<L>) {
        let hash = Self::hash_content(source);

        // Remove existing entry if it exists
        if self.entries.contains_key(&hash) {
            self.entries.remove(&hash);
            self.entry_count -= 1;
        }

        // Evict oldest entries if cache is full
        while self.entry_count >= self.max_entries && !self.entries.is_empty() {
            // Remove the first entry (FIFO eviction)
            if let Some(key) = self.entries.keys().next().cloned() {
                self.entries.remove(&key);
                self.entry_count -= 1;
            }
        }

        // Insert new entry
        let root: Arc<GreenNode<'static, L>> = unsafe { Arc::from_raw(Arc::into_raw(Arc::from(root)) as *const GreenNode<'static, L>) };

        self.entries.insert(hash, CacheEntry { root, lex_output });
        self.entry_count += 1;
    }

    /// Clears all entries from the cache.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.entry_count = 0;
    }

    /// Returns the current number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entry_count
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entry_count == 0
    }
}

impl<L: Language + Send + Sync + 'static> Default for ContentCache<L> {
    fn default() -> Self {
        Self::new(100) // Default to 100 entries
    }
}

/// A ParseCache implementation that wraps another ParseCache and adds content-based caching.
pub struct CachingParseSession<L: Language + Send + Sync + 'static, C: ParseCache<L>> {
    /// The underlying parse session.
    inner: C,
    /// The content cache for storing parsed results.
    content_cache: ContentCache<L>,
}

impl<L: Language + Send + Sync + 'static, C: ParseCache<L>> CachingParseSession<L, C> {
    /// Creates a new caching parse session.
    pub fn new(inner: C, max_cache_entries: usize) -> Self {
        Self { inner, content_cache: ContentCache::new(max_cache_entries) }
    }

    /// Returns a reference to the content cache.
    pub fn content_cache(&self) -> &ContentCache<L> {
        &self.content_cache
    }

    /// Returns a mutable reference to the content cache.
    pub fn content_cache_mut(&mut self) -> &mut ContentCache<L> {
        &mut self.content_cache
    }

    /// Returns a reference to the inner parse session.
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Returns a mutable reference to the inner parse session.
    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.inner
    }
}

impl<L: Language + Send + Sync + 'static, C: ParseCache<L>> ParseCache<L> for CachingParseSession<L, C> {
    fn arena(&self) -> &SyntaxArena {
        self.inner.arena()
    }

    fn old_tree(&self) -> Option<&GreenNode<'_, L>> {
        self.inner.old_tree()
    }

    fn lex_output(&self) -> Option<&LexOutput<L>> {
        self.inner.lex_output()
    }

    fn source_fingerprint(&self) -> Option<u64> {
        self.inner.source_fingerprint()
    }

    fn set_source_fingerprint(&mut self, fingerprint: u64) {
        self.inner.set_source_fingerprint(fingerprint)
    }

    fn clear_lex_output(&mut self) {
        self.inner.clear_lex_output()
    }

    fn prepare_generation(&mut self) {
        self.inner.prepare_generation()
    }

    fn commit_generation(&self, root: &GreenNode<L>) {
        self.inner.commit_generation(root)
    }
}

impl<L: Language + Send + Sync + 'static, C: ParseCache<L>> LexerCache<L> for CachingParseSession<L, C> {
    fn set_lex_output(&mut self, output: LexOutput<L>) {
        self.inner.set_lex_output(output)
    }

    fn get_token(&self, index: usize) -> Option<Token<L::TokenType>> {
        self.inner.get_token(index)
    }

    fn count_tokens(&self) -> usize {
        self.inner.count_tokens()
    }

    fn has_tokens(&self) -> bool {
        self.inner.has_tokens()
    }

    fn get_tokens(&self) -> Option<&[Token<L::TokenType>]> {
        self.inner.get_tokens()
    }
}

/// Implementation of BuilderCache for CachingParseSession.
impl<L: Language + Send + Sync + 'static, C: ParseCache<L> + crate::builder::BuilderCache<L>> crate::builder::BuilderCache<L> for CachingParseSession<L, C> {
    fn get_typed_node<T: std::any::Any + Clone + Send + Sync>(&self, node: &GreenNode<L>) -> Option<T> {
        self.inner.get_typed_node(node)
    }

    fn set_typed_node<T: std::any::Any + Send + Sync>(&mut self, node: &GreenNode<L>, value: T) {
        self.inner.set_typed_node(node, value)
    }
}
