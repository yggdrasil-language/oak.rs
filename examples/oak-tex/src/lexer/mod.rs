#![doc = include_str!("readme.md")]
use crate::{language::TexLanguage, lexer::token_type::TexTokenType};
/// Token types for the TeX lexer.
pub mod token_type;
use oak_core::{
    Lexer, LexerCache, LexerState, OakError, Source, TextEdit,
    lexer::{CommentConfig, LexOutput, WhitespaceConfig},
};
use std::sync::LazyLock;

pub(crate) type State<'a, S> = LexerState<'a, S, TexLanguage>;

static TEX_WHITESPACE: LazyLock<WhitespaceConfig> = LazyLock::new(|| WhitespaceConfig { unicode_whitespace: true });
static TEX_COMMENT: LazyLock<CommentConfig> = LazyLock::new(|| CommentConfig { line_marker: "%", block_start: "", block_end: "", nested_blocks: false });

/// A lexer for TeX source files.
#[derive(Clone, Debug)]
pub struct TexLexer<'config> {
    /// The language configuration.
    config: &'config TexLanguage,
}

impl<'config> Lexer<TexLanguage> for TexLexer<'config> {
    /// Lexes the source text into tokens.
    fn lex<'a, S: Source + ?Sized>(&self, source: &S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<TexLanguage>) -> LexOutput<TexLanguage> {
        let mut state = State::new_with_cache(source, 0, cache);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof()
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> TexLexer<'config> {
    /// Creates a new TeX lexer with the given language configuration.
    pub fn new(config: &'config TexLanguage) -> Self {
        Self { config }
    }

    /// Runs the lexer on the current state.
    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        while state.not_at_end() {
            let safe_point = state.get_position();

            if self.skip_whitespace(state) {
                continue;
            }

            if self.skip_comment(state) {
                continue;
            }

            if self.lex_command(state) {
                continue;
            }

            if self.lex_math_delimiters(state) {
                continue;
            }

            if self.lex_braces_and_brackets(state) {
                continue;
            }

            if self.lex_special_chars(state) {
                continue;
            }

            if self.lex_number(state) {
                continue;
            }

            if self.lex_text(state) {
                continue;
            }

            state.advance_if_dead_lock(safe_point)
        }

        Ok(())
    }

    /// Skips whitespace characters based on the TeX whitespace configuration.
    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        TEX_WHITESPACE.scan(state, TexTokenType::Whitespace)
    }

    /// Skips comments based on the TeX comment configuration.
    fn skip_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        TEX_COMMENT.scan(state, TexTokenType::Comment, TexTokenType::Comment)
    }

    /// Lexes a TeX command (e.g., `\section`).
    fn lex_command<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        if state.peek() != Some('\\') {
            return false;
        }

        state.advance(1); // consume '\'

        // Read the command name
        let mut has_name = false;
        if let Some(ch) = state.peek() {
            if ch == '\\' {
                // `\\` is the row break inside an environment body, not a
                // one-character command named `\`.
                state.advance(1);
                state.add_token(TexTokenType::RowBreak, start, state.get_position());
                return true;
            }
            if ch.is_ascii_alphabetic() {
                while let Some(ch) = state.peek() {
                    if ch.is_ascii_alphabetic() {
                        state.advance(ch.len_utf8());
                        has_name = true
                    }
                    else {
                        break;
                    }
                }
            }
            else {
                // Single non-alphabetic character command (e.g., \\, \&, \$, \ )
                state.advance(ch.len_utf8());
                has_name = true
            }
        }

        if has_name {
            let end = state.get_position();
            let text = state.get_text_in((start + 1..end).into()); // Skip the backslash

            let kind = match text.as_ref() {
                "begin" => TexTokenType::BeginKeyword,
                "end" => TexTokenType::EndKeyword,
                "(" => TexTokenType::ParenInlineMathDelim,    // \(
                ")" => TexTokenType::ParenInlineMathDelim,    // \)
                "[" => TexTokenType::BracketDisplayMathDelim, // \[
                "]" => TexTokenType::BracketDisplayMathDelim, // \]
                "{" => TexTokenType::BraceDelim,              // \{
                "}" => TexTokenType::BraceDelim,              // \}
                "left" => TexTokenType::LeftCommand,
                "right" => TexTokenType::RightCommand,
                "middle" => TexTokenType::MiddleCommand,
                "limits" => TexTokenType::Limits,
                "nolimits" => TexTokenType::Nolimits,
                "documentclass" => TexTokenType::DocumentclassKeyword,
                "usepackage" => TexTokenType::UsepackageKeyword,
                "section" => TexTokenType::SectionKeyword,
                "subsection" => TexTokenType::SubsectionKeyword,
                "subsubsection" => TexTokenType::SubsubsectionKeyword,
                "chapter" => TexTokenType::ChapterKeyword,
                "part" => TexTokenType::PartKeyword,
                "title" => TexTokenType::TitleKeyword,
                "author" => TexTokenType::AuthorKeyword,
                "date" => TexTokenType::DateKeyword,
                "maketitle" => TexTokenType::MaketitleKeyword,
                "tableofcontents" => TexTokenType::TableofcontentsKeyword,
                "item" => TexTokenType::ItemKeyword,
                "label" => TexTokenType::LabelKeyword,
                "ref" => TexTokenType::RefKeyword,
                "cite" => TexTokenType::CiteKeyword,
                "includegraphics" => TexTokenType::IncludegraphicsKeyword,
                "textbf" => TexTokenType::TextbfKeyword,
                "textit" => TexTokenType::TextitKeyword,
                "texttt" => TexTokenType::TextTt,
                "textsc" => TexTokenType::TextSc,
                "emph" => TexTokenType::EmphKeyword,
                "underline" => TexTokenType::Underline,
                "frac" => TexTokenType::Frac,
                "sqrt" => TexTokenType::Sqrt,
                "sum" => TexTokenType::Sum,
                "int" => TexTokenType::Int,
                "lim" => TexTokenType::Lim,
                "alpha" => TexTokenType::Alpha,
                "beta" => TexTokenType::Beta,
                "gamma" => TexTokenType::Gamma,
                "delta" => TexTokenType::Delta,
                "epsilon" => TexTokenType::Epsilon,
                "zeta" => TexTokenType::Zeta,
                "eta" => TexTokenType::Eta,
                "theta" => TexTokenType::Theta,
                "iota" => TexTokenType::Iota,
                "kappa" => TexTokenType::Kappa,
                "lambda" => TexTokenType::Lambda,
                "mu" => TexTokenType::Mu,
                "nu" => TexTokenType::Nu,
                "xi" => TexTokenType::Xi,
                "omicron" => TexTokenType::Omicron,
                "pi" => TexTokenType::Pi,
                "rho" => TexTokenType::Rho,
                "sigma" => TexTokenType::Sigma,
                "tau" => TexTokenType::Tau,
                "upsilon" => TexTokenType::Upsilon,
                "phi" => TexTokenType::Phi,
                "chi" => TexTokenType::Chi,
                "psi" => TexTokenType::Psi,
                "omega" => TexTokenType::Omega,
                "Gamma" => TexTokenType::UpperGamma,
                "Delta" => TexTokenType::UpperDelta,
                "Theta" => TexTokenType::UpperTheta,
                "Lambda" => TexTokenType::UpperLambda,
                "Xi" => TexTokenType::UpperXi,
                "Pi" => TexTokenType::UpperPi,
                "Sigma" => TexTokenType::UpperSigma,
                "Upsilon" => TexTokenType::UpperUpsilon,
                "Phi" => TexTokenType::UpperPhi,
                "Psi" => TexTokenType::UpperPsi,
                "Omega" => TexTokenType::UpperOmega,
                _ => TexTokenType::Command,
            };

            state.add_token(kind, start, end);
            true
        }
        else {
            false
        }
    }

    fn lex_math_delimiters<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        if let Some('$') = state.peek() {
            state.advance(1);
            if let Some('$') = state.peek() {
                state.advance(1);
                state.add_token(TexTokenType::DoubleDollar, start, state.get_position());
            }
            else {
                state.add_token(TexTokenType::Dollar, start, state.get_position());
            }
            true
        }
        else {
            false
        }
    }

    fn lex_braces_and_brackets<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        if let Some(ch) = state.peek() {
            let kind = match ch {
                '{' => Some(TexTokenType::LeftBrace),
                '}' => Some(TexTokenType::RightBrace),
                '[' => Some(TexTokenType::LeftBracket),
                ']' => Some(TexTokenType::RightBracket),
                _ => None,
            };

            if let Some(kind) = kind {
                state.advance(ch.len_utf8());
                state.add_token(kind, start, state.get_position());
                return true;
            }
        }
        false
    }

    fn lex_special_chars<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        if let Some(ch) = state.peek() {
            let kind = match ch {
                '&' => Some(TexTokenType::Ampersand),
                '_' => Some(TexTokenType::Underscore),
                '^' => Some(TexTokenType::Caret),
                '~' => Some(TexTokenType::Tilde),
                '#' => Some(TexTokenType::Hash),
                '%' => Some(TexTokenType::Percent),
                '*' => Some(TexTokenType::Star),
                _ => None,
            };

            if let Some(kind) = kind {
                state.advance(ch.len_utf8());
                state.add_token(kind, start, state.get_position());
                return true;
            }
        }
        false
    }

    fn lex_number<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let mut has_digits = false;

        while let Some(ch) = state.peek() {
            if ch.is_ascii_digit() {
                state.advance(1);
                has_digits = true;
            }
            else {
                break;
            }
        }

        if has_digits {
            if let Some('.') = state.peek() {
                state.advance(1);
                while let Some(ch) = state.peek() {
                    if ch.is_ascii_digit() {
                        state.advance(1);
                    }
                    else {
                        break;
                    }
                }
            }
            state.add_token(TexTokenType::Number, start, state.get_position());
            true
        }
        else {
            false
        }
    }

    fn lex_text<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let mut has_text = false;

        while let Some(ch) = state.peek() {
            if ch.is_whitespace() || ch == '\\' || ch == '%' || ch == '$' || ch == '{' || ch == '}' || ch == '[' || ch == ']' || ch == '&' || ch == '_' || ch == '^' || ch == '~' || ch == '#' || ch.is_ascii_digit() {
                break;
            }
            state.advance(ch.len_utf8());
            has_text = true;
        }

        if has_text {
            state.add_token(TexTokenType::Text, start, state.get_position());
            true
        }
        else {
            false
        }
    }
}
