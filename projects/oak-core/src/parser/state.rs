use crate::{
    Language,
    errors::OakError,
    language::{ElementType, TokenType},
    lexer::{LexOutput, Token, Tokens},
    memory::arena::SyntaxArena,
    source::Source,
    tree::{Cursor, GreenLeaf, GreenNode, GreenTree, TokenProvenance},
};
use core::range::Range;

/// Helper function to raise deep clone of a node into an arena.
///
/// This is used to "promote" nodes from a previous generation's arena to the current one,
/// ensuring that the new tree is self-contained and has good memory locality.
pub fn deep_clone_node<'a, L: Language>(node: &GreenNode<'_, L>, arena: &'a SyntaxArena) -> &'a GreenNode<'a, L> {
    let children_iter = node.children.iter().map(|child| match child {
        GreenTree::Node(n) => GreenTree::Node(deep_clone_node(n, arena)),
        GreenTree::Leaf(l) => GreenTree::Leaf(*l),
    });

    let slice = arena.alloc_slice_fill_iter(node.children.len(), children_iter);
    let new_node = GreenNode::new(node.kind, slice);
    arena.alloc(new_node)
}

/// Provides tokens to the parser.
pub struct TokenSource<L: Language> {
    /// The tokens being consumed.
    tokens: Tokens<L>,
    /// The current index into the tokens.
    index: usize,
}

impl<L: Language> Clone for TokenSource<L> {
    fn clone(&self) -> Self {
        Self { tokens: self.tokens.clone(), index: self.index }
    }
}

impl<L: Language> TokenSource<L> {
    /// Creates a new token source.
    pub fn new(tokens: Tokens<L>) -> Self {
        Self { tokens, index: 0 }
    }

    /// Gets the current token.
    #[inline]
    pub fn current(&self) -> Option<&Token<L::TokenType>> {
        self.tokens.get(self.index)
    }

    /// Peeks at a token at the specified offset from the current position.
    #[inline]
    pub fn peek_at(&self, offset: usize) -> Option<&Token<L::TokenType>> {
        self.tokens.get(self.index + offset)
    }

    /// Advances the current position to the next token.
    #[inline]
    pub fn advance(&mut self) {
        self.index += 1;
    }

    /// Checks if the token source has reached the end.
    #[inline]
    pub fn is_end(&self) -> bool {
        self.index >= self.tokens.len()
    }

    /// Gets the current index.
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Sets the current index.
    #[inline]
    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

/// Collects the results of the parsing process.
pub struct TreeSink<'a, L: Language> {
    /// The arena to allocate syntax nodes in.
    arena: &'a SyntaxArena,
    /// The list of children for the current node being constructed.
    children: Vec<GreenTree<'a, L>>,
}

impl<'a, L: Language> TreeSink<'a, L> {
    /// Creates a new tree sink.
    pub fn new(arena: &'a SyntaxArena, capacity_hint: usize) -> Self {
        // Use a larger default capacity to reduce reallocations
        let capacity = usize::max(capacity_hint, 1024);
        Self { arena, children: Vec::with_capacity(capacity) }
    }

    /// Pushes a leaf node (token) to the current list of children.
    #[inline(always)]
    pub fn push_leaf(&mut self, kind: L::TokenType, len: usize) {
        self.children.push(GreenTree::Leaf(GreenLeaf::new(kind, len as u32)));
    }

    /// Pushes a leaf node (token) with provenance metadata to the current list of children.
    #[inline(always)]
    pub fn push_leaf_with_metadata(&mut self, kind: L::TokenType, len: usize, provenance: TokenProvenance) {
        let index = self.arena.add_metadata(provenance);
        self.children.push(GreenTree::Leaf(GreenLeaf::with_metadata(kind, len as u32, Some(index))));
    }

    /// Pushes an existing node to the current list of children.
    #[inline(always)]
    pub fn push_node(&mut self, node: &'a GreenNode<'a, L>) {
        self.children.push(GreenTree::Node(node));
    }

    /// Returns the current number of children, serving as a checkpoint for finishing a node later.
    #[inline(always)]
    pub fn checkpoint(&self) -> usize {
        self.children.len()
    }

    /// Returns the syntax arena used by this sink.
    #[inline(always)]
    pub fn arena(&self) -> &'a SyntaxArena {
        self.arena
    }

    /// Restores the sink to a previous checkpoint by truncating the children list.
    #[inline(always)]
    pub fn restore(&mut self, checkpoint: usize) {
        self.children.truncate(checkpoint)
    }

    /// Finishes a node starting from the given checkpoint and adds it as a child.
    ///
    /// Tokens skipped as trivia by [`State::advance`] are already in the sink when a
    /// node finishes, so a node would otherwise keep the whitespace and comments
    /// that follow its last token and report a span past that token's end. Those
    /// trailing trivia stay in the sink as siblings of the node instead, which keeps
    /// the source text lossless while the node covers only its own tokens. Root
    /// nodes keep their trailing trivia because they span the whole source, and
    /// other roots are completed by [`ParserState::finish`].
    pub fn finish_node(&mut self, checkpoint: usize, kind: L::ElementType) -> &'a GreenNode<'a, L> {
        let end = if kind.is_root() {
            self.children.len()
        }
        else {
            let mut end = self.children.len();
            while end > checkpoint {
                match &self.children[end - 1] {
                    GreenTree::Leaf(leaf) if L::TokenType::is_ignored(&leaf.kind) => end -= 1,
                    _ => break,
                }
            }
            end
        };

        let children_count = end - checkpoint;
        let node = if children_count == 0 {
            GreenNode::new(kind, &[])
        }
        else {
            let children_iter = self.children[checkpoint..end].iter().cloned();
            let children_slice = self.arena.alloc_slice_fill_iter(children_count, children_iter);
            GreenNode::new(kind, children_slice)
        };
        let trailing: Vec<GreenTree<'a, L>> = self.children[end..].to_vec();
        self.children.truncate(checkpoint);
        let node_ref = self.arena.alloc(node);
        self.children.push(GreenTree::Node(node_ref));
        self.children.extend(trailing);
        node_ref
    }

    /// Appends the trivia left after the root node to it so the root covers the whole source.
    fn attach_trailing_trivia(&mut self, root: &'a GreenNode<'a, L>) -> &'a GreenNode<'a, L> {
        let Some(GreenTree::Node(node)) = self.children.first()
        else {
            return root;
        };
        if !std::ptr::eq(*node, root) {
            return root;
        }
        let trailing = &self.children[1..];
        if trailing.is_empty() || !trailing.iter().all(|child| matches!(child, GreenTree::Leaf(leaf) if L::TokenType::is_ignored(&leaf.kind))) {
            return root;
        }
        let children_iter = root.children.iter().copied().chain(trailing.iter().copied());
        let children_slice = self.arena.alloc_slice_fill_iter(root.children.len() + trailing.len(), children_iter);
        self.arena.alloc(GreenNode::new(root.kind, children_slice))
    }
}

/// Encapsulates incremental parsing logic.
pub struct IncrementalContext<'a, L: Language> {
    /// A cursor over the previous syntax tree.
    cursor: Cursor<'a, L>,
    /// Sorted list of edits with their accumulated deltas.
    /// Each entry contains the old range and the cumulative delta *after* this edit.
    edits: Vec<(Range<usize>, isize)>,
}

impl<'a, L: Language> IncrementalContext<'a, L> {
    /// Creates a new incremental parsing context.
    pub fn new(old_root: &'a GreenNode<'a, L>, edits: &[crate::source::TextEdit]) -> Self {
        let mut sorted_edits: Vec<_> = edits.iter().collect();
        sorted_edits.sort_by_key(|e| e.span.start);

        let mut cumulative_delta = 0isize;
        let mut processed_edits = Vec::with_capacity(edits.len());

        for edit in sorted_edits {
            let old_len = edit.span.end - edit.span.start;
            let new_len = edit.text.len();
            cumulative_delta += new_len as isize - old_len as isize;
            processed_edits.push((edit.span, cumulative_delta));
        }

        Self { cursor: Cursor::new(old_root), edits: processed_edits }
    }

    fn map_new_to_old(&self, new_pos: usize) -> Option<usize> {
        let mut current_delta = 0isize;

        for (old_range, cumulative_delta) in &self.edits {
            let new_start = (old_range.start as isize + current_delta) as usize;
            let edit_delta = *cumulative_delta - current_delta;
            let new_end = (old_range.end as isize + current_delta + edit_delta) as usize;

            if new_pos < new_start {
                return Some((new_pos as isize - current_delta) as usize);
            }
            else if new_pos < new_end {
                // Inside a dirty range
                return None;
            }

            current_delta = *cumulative_delta;
        }

        Some((new_pos as isize - current_delta) as usize)
    }

    fn is_dirty(&self, old_start: usize, old_end: usize) -> bool {
        for (old_range, _) in &self.edits {
            // Check for overlap between [old_start, old_end) and [old_range.start, old_range.end)
            if old_start < old_range.end && old_end > old_range.start {
                return true;
            }
        }
        false
    }
}

/// High-level API for parsers, coordinating token supply and tree construction.
pub struct ParserState<'a, L: Language, S: Source + ?Sized = crate::source::SourceText> {
    /// The token source providing tokens to the parser.
    pub tokens: TokenSource<L>,

    /// The sink where the syntax tree is constructed.
    pub sink: TreeSink<'a, L>,

    /// Optional context for incremental parsing.
    pub incremental: Option<IncrementalContext<'a, L>>,

    /// Collection of errors encountered during parsing.
    pub errors: Vec<OakError>,
    /// We keep a reference to help with error reporting and offset calculation.
    pub source: &'a S,
}

impl<'a, L: Language, S: Source + ?Sized> ParserState<'a, L, S> {
    /// Returns the source file's ID, if available.
    pub fn source_id(&self) -> Option<crate::source::SourceId> {
        self.source.source_id()
    }

    /// Returns the syntax arena used by this parser.
    pub fn arena(&self) -> &'a SyntaxArena {
        self.sink.arena()
    }

    /// Creates a new parser state.
    ///
    /// # Arguments
    /// * `arena` - The syntax arena to allocate nodes in.
    /// * `lex_output` - The output from the lexer, including tokens and any lexical errors.
    /// * `source` - The source text being parsed.
    /// * `capacity_hint` - Initial capacity hint for the tree sink's child vector.
    pub fn new(arena: &'a SyntaxArena, lex_output: LexOutput<L>, source: &'a S, capacity_hint: usize) -> Self {
        let (tokens, mut errors) = match lex_output.result {
            Ok(tokens) => (tokens, Vec::new()),
            Err(e) => (Tokens::default(), vec![e]),
        };
        errors.extend(lex_output.diagnostics);

        Self { tokens: TokenSource::new(tokens), sink: TreeSink::new(arena, capacity_hint), incremental: None, errors, source }
    }

    /// Creates a nested parser state that shares the same arena and source.
    /// This is useful for parsing sub-structures that should be independent but part of the same overall tree.
    pub fn nested(&self) -> ParserState<'a, L, S> {
        ParserState { tokens: self.tokens.clone(), sink: TreeSink::new(self.sink.arena, 1024), incremental: None, errors: Vec::new(), source: self.source }
    }

    /// Returns the text content of the current token.
    pub fn peek_text(&self) -> Option<std::borrow::Cow<'_, str>> {
        self.current().map(|t| self.source.get_text_in(t.span))
    }

    /// Manually promotes a node from a previous generation to the current one.
    /// This is useful for improving memory locality.
    pub fn promote(&mut self, node: &'a GreenNode<'a, L>) -> &'a GreenNode<'a, L> {
        deep_clone_node(node, self.sink.arena)
    }

    /// Sets the incremental parsing context.
    ///
    /// # Arguments
    /// * `old` - The root of the previous version of the syntax tree.
    /// * `edits` - The edits that were applied to the source text.
    pub fn set_incremental(&mut self, old: &'a GreenNode<'a, L>, edits: &[crate::source::TextEdit]) {
        self.incremental = Some(IncrementalContext::new(old, edits));
    }

    // --- Error Reporting ---

    /// Returns the current byte offset.
    pub fn current_offset(&self) -> usize {
        self.tokens.current().map(|t| t.span.start).unwrap_or_else(|| self.source.length())
    }

    /// Records a syntax error with the given message.
    pub fn syntax_error(&mut self, message: impl Into<String>) -> OakError {
        let err = OakError::syntax_error(message, self.current_offset(), self.source.source_id());
        self.errors.push(err.clone());
        err
    }

    /// Records an unexpected token error.
    pub fn record_unexpected_token(&mut self, token: impl Into<String>) {
        let err = OakError::unexpected_token(token, self.current_offset(), self.source.source_id());
        self.errors.push(err)
    }

    /// Records an expected token error.
    pub fn record_expected(&mut self, expected: impl Into<String>) {
        let offset = self.tokens.current().map(|t| t.span.start).unwrap_or(self.source.length());
        let err = OakError::expected_token(expected, offset, self.source.source_id());
        self.errors.push(err)
    }

    /// Records an expected name error.
    pub fn record_expected_name(&mut self, name_kind: impl Into<String>) {
        let offset = self.tokens.current().map(|t| t.span.start).unwrap_or(self.source.length());
        let err = OakError::expected_name(name_kind, offset, self.source.source_id());
        self.errors.push(err)
    }

    /// Records a trailing comma not allowed error.
    pub fn record_trailing_comma_not_allowed(&mut self) {
        let offset = self.tokens.current().map(|t| t.span.start).unwrap_or(self.source.length());
        let err = OakError::trailing_comma_not_allowed(offset, self.source.source_id());
        self.errors.push(err)
    }

    /// Records an unexpected end of file error.
    pub fn record_unexpected_eof(&mut self) {
        let offset = self.source.length();
        let err = OakError::unexpected_eof(offset, self.source.source_id());
        self.errors.push(err)
    }

    /// Records an unexpected end of file error and returns it.
    pub fn unexpected_eof(&mut self) -> OakError {
        let offset = self.source.length();
        let err = OakError::unexpected_eof(offset, self.source.source_id());
        self.errors.push(err.clone());
        err
    }

    // --- Token Navigation ---

    /// Returns the current token.
    #[inline]
    pub fn current(&self) -> Option<&Token<L::TokenType>> {
        self.tokens.current()
    }

    /// Returns the kind of the current token.
    #[inline]
    pub fn peek_kind(&self) -> Option<L::TokenType> {
        self.current().map(|t| t.kind)
    }

    /// Peeks at a token at the specified offset from the current position.
    #[inline]
    pub fn peek_at(&self, offset: usize) -> Option<&Token<L::TokenType>> {
        self.tokens.peek_at(offset)
    }

    /// Peeks at the Nth non-trivia token from the current position.
    pub fn peek_non_trivia_at(&self, mut n: usize) -> Option<&Token<L::TokenType>> {
        let mut offset = 0;
        while let Some(token) = self.tokens.peek_at(offset) {
            if !L::TokenType::is_ignored(&token.kind) {
                if n == 0 {
                    return Some(token);
                }
                n -= 1
            }
            offset += 1
        }
        None
    }

    /// Returns the kind of the token at the specified offset from the current position.
    #[inline]
    pub fn peek_kind_at(&self, offset: usize) -> Option<L::TokenType> {
        self.tokens.peek_at(offset).map(|t| t.kind)
    }

    /// Returns the kind of the Nth non-trivia token from the current position.
    pub fn peek_non_trivia_kind_at(&self, n: usize) -> Option<L::TokenType> {
        self.peek_non_trivia_at(n).map(|t| t.kind)
    }

    /// Checks if the parser has not yet reached the end of the token stream.
    #[inline]
    pub fn not_at_end(&self) -> bool {
        !self.tokens.is_end()
    }

    /// Checks if the current token is of the specified kind.
    #[inline]
    pub fn at(&self, kind: L::TokenType) -> bool {
        self.peek_kind() == Some(kind)
    }

    /// Checks if the current token is NOT of the specified kind.
    #[inline]
    pub fn not_at(&self, kind: L::TokenType) -> bool {
        !self.at(kind)
    }

    /// Advances the current position to the next token.
    pub fn advance(&mut self) {
        self.tokens.advance();
        self.skip_trivia()
    }

    /// Skips trivia tokens and adds them to the syntax tree.
    pub fn skip_trivia(&mut self) {
        while let Some(token) = self.tokens.current() {
            if std::intrinsics::likely(L::TokenType::is_ignored(&token.kind)) {
                self.sink.push_leaf(token.kind, token.length());
                self.tokens.advance()
            }
            else {
                break;
            }
        }
    }

    /// Advances until a token of the specified kind is found, or the end of the token stream is reached.
    pub fn advance_until(&mut self, kind: L::TokenType) {
        while std::intrinsics::likely(self.not_at_end() && !self.at(kind)) {
            self.advance()
        }
    }

    /// Advances until any token of the specified kinds is found, or the end of the token stream is reached.
    pub fn advance_until_any(&mut self, kinds: &[L::TokenType]) {
        while std::intrinsics::likely(self.not_at_end() && !kinds.iter().any(|&k| self.at(k))) {
            self.advance()
        }
    }

    /// Consumes the current token and adds it to the syntax tree as a leaf node.
    pub fn bump(&mut self) {
        if let Some(token) = self.current() {
            self.sink.push_leaf(token.kind, token.length());
            self.advance()
        }
    }

    /// Consumes the current token if it matches the specified kind.
    /// Returns `true` if the token was consumed, `false` otherwise.
    pub fn eat(&mut self, kind: L::TokenType) -> bool {
        if std::intrinsics::unlikely(self.at(kind)) {
            self.bump();
            true
        }
        else {
            false
        }
    }

    /// Expects the current token to be of the specified kind, consuming it if it matches.
    /// If the token does not match, an error is recorded and returned.
    pub fn expect(&mut self, kind: L::TokenType) -> Result<(), OakError> {
        if std::intrinsics::likely(self.eat(kind)) {
            Ok(())
        }
        else {
            let err = OakError::expected_token(format!("{:?}", kind), self.current_offset(), self.source.source_id());
            self.errors.push(err.clone());
            Err(err)
        }
    }

    /// Tries to parse a construct with a backtracking point.
    pub fn try_parse<T, F>(&mut self, parser: F) -> Result<T, OakError>
    where
        F: FnOnce(&mut Self) -> Result<T, OakError>,
    {
        let checkpoint = self.checkpoint();
        match parser(self) {
            Ok(value) => Ok(value),
            Err(err) => {
                self.restore(checkpoint);
                Err(err)
            }
        }
    }

    /// Tries to parse a construct with a backtracking point, preserving the error if it fails.
    ///
    /// This is similar to `try_parse`, but it keeps the error generated by the parser
    /// instead of potentially discarding it or generating a generic one.
    pub fn try_parse_with_error<T, F>(&mut self, parser: F) -> Result<T, OakError>
    where
        F: FnOnce(&mut Self) -> Result<T, OakError>,
    {
        let checkpoint = self.checkpoint();
        match parser(self) {
            Ok(value) => Ok(value),
            Err(err) => {
                self.restore(checkpoint);
                Err(err)
            }
        }
    }

    // --- Tree Construction ---

    /// Creates a checkpoint in the tree construction process, which can be used to finish a node later.
    /// This checkpoint includes both the token index and the tree sink's current position for backtracking.
    pub fn checkpoint(&self) -> (usize, usize) {
        (self.tokens.index(), self.sink.checkpoint())
    }

    /// Restores the parser state to a previous checkpoint.
    pub fn restore(&mut self, (token_index, tree_checkpoint): (usize, usize)) {
        self.tokens.set_index(token_index);
        self.sink.restore(tree_checkpoint)
    }

    /// Finishes a node starting from the given checkpoint and adds it as a child.
    pub fn finish_at(&mut self, checkpoint: (usize, usize), kind: L::ElementType) -> &'a GreenNode<'a, L> {
        self.sink.finish_node(checkpoint.1, kind)
    }

    /// Creates a checkpoint before an existing node.
    pub fn checkpoint_before(&mut self, node: &'a GreenNode<'a, L>) -> (usize, usize) {
        // Find the index of the node in the sink's children
        let sink_checkpoint = self.sink.checkpoint();
        for i in (0..sink_checkpoint).rev() {
            if let Some(child) = self.sink.children.get(i) {
                if let GreenTree::Node(n) = child {
                    if std::ptr::eq(*n, node) {
                        return (0, i); // token_index is hard to determine, but usually not needed for infix wrapping
                    }
                }
            }
        }
        (0, sink_checkpoint)
    }

    /// Adds an existing node as a child to the current node.
    pub fn push_child(&mut self, node: &'a GreenNode<'a, L>) {
        self.sink.push_node(node)
    }

    /// Attempts to reuse a previous syntax node at the current position.
    /// This is the core of incremental parsing.
    pub fn try_reuse(&mut self, kind: L::ElementType) -> bool {
        let Some(inc) = &mut self.incremental
        else {
            return false;
        };
        let current_index = self.tokens.index();
        let new_pos = self.tokens.current().map(|t| t.span.start).unwrap_or(self.source.length());

        #[cfg(test)]
        println!("try_reuse: kind={:?}, new_pos={}", kind, new_pos);

        let Some(target_old_pos) = inc.map_new_to_old(new_pos)
        else {
            return false;
        };

        #[cfg(test)]
        println!("Trying to reuse node at pos {};: {:?}", new_pos, kind);

        let mut steps = 0;
        const MAX_STEPS: usize = 100;

        loop {
            let start = inc.cursor.offset();
            let end = inc.cursor.end_offset();

            if std::intrinsics::likely(start == target_old_pos) {
                if let Some(node) = inc.cursor.as_node() {
                    if std::intrinsics::likely(node.kind == kind) {
                        if std::intrinsics::likely(!inc.is_dirty(start, end)) {
                            // Verify tokens
                            let node_len = node.text_len() as usize;
                            let target_new_end = new_pos + node_len;

                            // Check token stream more efficiently
                            let mut lookahead = 0;
                            let mut current_pos = new_pos;
                            let mut total_length = 0;

                            while let Some(t) = self.tokens.peek_at(lookahead) {
                                if t.span.start != current_pos {
                                    // Token stream doesn't match expected positions
                                    break;
                                }
                                total_length += t.length();
                                current_pos = t.span.end;

                                if total_length == node_len && current_pos == target_new_end {
                                    // Found the end!
                                    let tokens_consumed = lookahead + 1;
                                    let new_node = deep_clone_node(node, self.sink.arena);
                                    self.sink.push_node(new_node);
                                    self.tokens.set_index(current_index + tokens_consumed);
                                    inc.cursor.step_over();
                                    // A reused node does not own the trivia that follows it,
                                    // so consume it here to keep the current token significant.
                                    self.skip_trivia();
                                    return true;
                                }
                                else if total_length > node_len {
                                    break;
                                }
                                lookahead += 1;
                            }
                        }
                    }
                }
                if !inc.cursor.step_into() && !inc.cursor.step_next() {
                    return false;
                }
            }
            else if std::intrinsics::likely(start < target_old_pos && end > target_old_pos) {
                if !inc.cursor.step_into() {
                    // If we can't step into, try to step next
                    if !inc.cursor.step_next() {
                        return false;
                    }
                }
            }
            else if std::intrinsics::likely(end <= target_old_pos) {
                if !inc.cursor.step_next() {
                    return false;
                }
            }
            else {
                return false;
            }

            steps += 1;
            if steps > MAX_STEPS {
                return false;
            }
        }
    }

    /// Finishes the parsing process and returns the final output.
    pub fn finish(mut self, result: Result<&'a GreenNode<'a, L>, OakError>) -> crate::parser::ParseOutput<'a, L> {
        let result = result.map(|root| self.sink.attach_trailing_trivia(root));
        crate::errors::OakDiagnostics { result, diagnostics: self.errors }
    }

    /// Wraps the parsing of a node with incremental reuse support.
    ///
    /// This method first attempts to reuse an existing node of the given kind from the previous tree.
    /// If successful, it skips the provided closure and returns `Ok(())`.
    /// Otherwise, it creates a checkpoint, runs the closure to parse the node's content,
    /// and then finishes the node with the specified kind.
    pub fn incremental_node<F>(&mut self, kind: L::ElementType, f: F) -> Result<(), OakError>
    where
        F: FnOnce(&mut Self) -> Result<(), OakError>,
    {
        if self.try_reuse(kind) {
            return Ok(());
        };

        let checkpoint = self.checkpoint();
        let res = f(self);
        self.finish_at(checkpoint, kind);
        res
    }

    /// Wraps the parsing of an optional node with incremental reuse support.
    ///
    /// Similar to `incremental_node`, but the closure returns a boolean indicating
    /// if the node was actually present.
    pub fn incremental_opt<F>(&mut self, kind: L::ElementType, f: F) -> Result<bool, OakError>
    where
        F: FnOnce(&mut Self) -> Result<bool, OakError>,
    {
        if self.try_reuse(kind) {
            return Ok(true);
        };

        let checkpoint = self.checkpoint();
        match f(self) {
            Ok(true) => {
                self.finish_at(checkpoint, kind);
                Ok(true)
            }
            Ok(false) => Ok(false),
            Err(e) => {
                self.finish_at(checkpoint, kind);
                Err(e)
            }
        }
    }
}
