pub use keywords::ValkyrieKeywords;
pub use token_type::ValkyrieTokenType;

use crate::ValkyrieLanguage;
use oak_core::{
    Lexer, LexerState, TextEdit,
    lexer::{LexOutput, LexerCache},
    source::Source,
};

pub(crate) type State<'a, S> = LexerState<'a, S, ValkyrieLanguage>;

/// Valkyrie lexer implementation.
#[derive(Clone)]
pub struct ValkyrieLexer<'config> {
    pub(crate) config: &'config ValkyrieLanguage,
}

impl<'config> Lexer<ValkyrieLanguage> for ValkyrieLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &'a S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<ValkyrieLanguage>) -> LexOutput<ValkyrieLanguage> {
        let mut state = State::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> ValkyrieLexer<'config> {
    /// Create a new Valkyrie lexer.
    pub fn new(config: &'config ValkyrieLanguage) -> Self {
        Self { config }
    }
}

/// Token type definitions for the Valkyrie lexer.
pub mod token_type;

/// Keywords for the Valkyrie language.
pub mod keywords;

/// Lexer implementation.
mod lex;
/// TGrammar (`<% ... %>`) lexing gated by `ValkyrieLanguage::support_t_grammar`.
mod lex_tgrammar;
