/// Element types for the TeX parser.
pub mod element_type;

use crate::{
    language::TexLanguage,
    lexer::{TexLexer, token_type::TexTokenType},
    parser::element_type::TexElementType,
};
use oak_core::{
    GreenNode, OakError,
    parser::{ParseCache, ParseOutput, Parser, ParserState, parse_with_lexer},
    source::{Source, TextEdit},
};

/// TeX parser state.
pub(crate) type State<'a, S> = ParserState<'a, TexLanguage, S>;

/// A parser for TeX source files.
pub struct TexParser<'config> {
    /// The language configuration.
    pub(crate) config: &'config TexLanguage,
}

impl<'config> TexParser<'config> {
    /// Creates a new TeX parser.
    pub fn new(config: &'config TexLanguage) -> Self {
        Self { config }
    }

    /// Parses a single TeX item (command, environment, group, etc.).
    fn parse_item<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        match state.peek_kind() {
            Some(TexTokenType::BeginKeyword) => self.parse_environment(state),
            Some(TexTokenType::Dollar) => self.parse_math_delimiter(state, TexTokenType::Dollar, TexElementType::InlineMath),
            Some(TexTokenType::DoubleDollar) => self.parse_math_delimiter(state, TexTokenType::DoubleDollar, TexElementType::DisplayMath),
            Some(TexTokenType::ParenInlineMathDelim) => self.parse_math_delimiter(state, TexTokenType::ParenInlineMathDelim, TexElementType::ParenInlineMath),
            Some(TexTokenType::BracketDisplayMathDelim) => self.parse_math_delimiter(state, TexTokenType::BracketDisplayMathDelim, TexElementType::BracketDisplayMath),
            Some(TexTokenType::LeftCommand) => self.parse_left_right(state),
            Some(TexTokenType::Limits) | Some(TexTokenType::Nolimits) => self.parse_limits(state),
            Some(kind) if self.is_command_like(kind) => self.parse_command(state),
            Some(TexTokenType::LeftBrace) => self.parse_group(state),
            Some(TexTokenType::Caret) => self.parse_superscript(state),
            Some(TexTokenType::Underscore) => self.parse_subscript(state),
            _ => {
                state.bump();
                Ok(())
            }
        }
    }

    /// Returns true if the token kind is considered command-like.
    fn is_command_like(&self, kind: TexTokenType) -> bool {
        match kind {
            TexTokenType::Backslash
            | TexTokenType::Command
            | TexTokenType::RightCommand
            | TexTokenType::MiddleCommand
            | TexTokenType::EndKeyword
            | TexTokenType::DocumentclassKeyword
            | TexTokenType::UsepackageKeyword
            | TexTokenType::SectionKeyword
            | TexTokenType::SubsectionKeyword
            | TexTokenType::SubsubsectionKeyword
            | TexTokenType::ChapterKeyword
            | TexTokenType::PartKeyword
            | TexTokenType::TitleKeyword
            | TexTokenType::AuthorKeyword
            | TexTokenType::DateKeyword
            | TexTokenType::MaketitleKeyword
            | TexTokenType::TableofcontentsKeyword
            | TexTokenType::ItemKeyword
            | TexTokenType::LabelKeyword
            | TexTokenType::RefKeyword
            | TexTokenType::CiteKeyword
            | TexTokenType::IncludegraphicsKeyword
            | TexTokenType::TextbfKeyword
            | TexTokenType::TextitKeyword
            | TexTokenType::EmphKeyword
            | TexTokenType::LeftCommand
            | TexTokenType::Frac
            | TexTokenType::Sqrt
            | TexTokenType::Sum
            | TexTokenType::Int
            | TexTokenType::Lim
            | TexTokenType::Alpha
            | TexTokenType::Beta
            | TexTokenType::Gamma
            | TexTokenType::Delta
            | TexTokenType::Epsilon => true,
            _ => false,
        }
    }

    /// Parses a TeX environment (e.g., `\begin{...} ... \end{...}`).
    fn parse_environment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();

        // Parse \begin{name} — the name is always the first mandatory argument
        // so the builder can extract it from the BeginEnvironment CST node.
        let begin_checkpoint = state.checkpoint();
        state.expect(TexTokenType::BeginKeyword)?;
        if state.at(TexTokenType::LeftBrace) {
            self.parse_mandatory_argument(state)?;
        }
        state.finish_at(begin_checkpoint, TexElementType::BeginEnvironment);

        // Parse content until \end{...}
        while state.not_at_end() && !state.at(TexTokenType::EndKeyword) {
            self.parse_item(state)?;
        }

        // Parse \end{name}
        if state.at(TexTokenType::EndKeyword) {
            let end_checkpoint = state.checkpoint();
            state.bump();
            if state.at(TexTokenType::LeftBrace) {
                self.parse_mandatory_argument(state)?;
            }
            state.finish_at(end_checkpoint, TexElementType::EndEnvironment);
        }

        state.finish_at(checkpoint, TexElementType::Environment);
        Ok(())
    }

    fn parse_superscript<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        state.expect(TexTokenType::Caret)?;

        if state.at(TexTokenType::LeftBrace) {
            self.parse_group(state)?;
        }
        else {
            state.bump();
        }

        state.finish_at(checkpoint, TexElementType::Superscript);
        Ok(())
    }

    fn parse_subscript<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        state.expect(TexTokenType::Underscore)?;

        if state.at(TexTokenType::LeftBrace) {
            self.parse_group(state)?;
        }
        else {
            state.bump();
        }

        state.finish_at(checkpoint, TexElementType::Subscript);
        Ok(())
    }

    fn parse_command<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        let kind = state.peek_kind().unwrap_or(TexTokenType::Command);

        state.bump(); // consume \ or command keyword

        // Optional trailing star: \section*, \frac*, ...
        if state.at(TexTokenType::Star) {
            state.bump();
        }

        // Parse optional [...] and required {...} arguments.
        while state.at(TexTokenType::LeftBracket) || state.at(TexTokenType::LeftBrace) {
            if state.at(TexTokenType::LeftBracket) {
                self.parse_optional_argument(state)?;
            }
            else {
                self.parse_mandatory_argument(state)?;
            }
        }

        // Preserve specific command element kinds for known commands,
        // otherwise fall back to the generic Command element.
        let _node_kind = match kind {
            TexTokenType::Frac | TexTokenType::Sqrt | TexTokenType::Sum | TexTokenType::Int | TexTokenType::Lim | TexTokenType::Alpha | TexTokenType::Beta | TexTokenType::Gamma | TexTokenType::Delta | TexTokenType::Epsilon => kind,
            _ => TexTokenType::Command,
        };

        state.finish_at(checkpoint, _node_kind.into());
        Ok(())
    }

    fn parse_group<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        state.expect(TexTokenType::LeftBrace)?;
        while state.not_at_end() && !state.at(TexTokenType::RightBrace) {
            self.parse_item(state)?;
        }
        state.expect(TexTokenType::RightBrace)?;
        state.finish_at(checkpoint, TexElementType::Group);
        Ok(())
    }

    /// Parse a math block delimited by `$...$`, `$$...$$`, `\(...\)` or
    /// `\[...\]`. The same lexed token kind is used for opening and
    /// closing, so we match on that.
    fn parse_math_delimiter<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, open_kind: TexTokenType, element_kind: TexElementType) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        state.bump(); // consume opening delimiter
        while state.not_at_end() && !state.at(open_kind) {
            self.parse_item(state)?;
        }
        // Closing delimiter is optional on EOF for forgiving parsing.
        if state.at(open_kind) {
            state.bump();
        }
        state.finish_at(checkpoint, element_kind);
        Ok(())
    }

    /// Parse `\left<delim> ... \right<delim>` (optionally with
    /// `\middle<delim>` tokens inside the content).
    fn parse_left_right<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();

        // consume `\left`
        state.bump();
        // consume the opening delimiter (single token or group arg)
        self.consume_delimiter_token(state)?;

        while state.not_at_end() && !state.at(TexTokenType::RightCommand) {
            self.parse_item(state)?;
        }

        // consume `\right` + its delimiter if present
        if state.at(TexTokenType::RightCommand) {
            state.bump();
            self.consume_delimiter_token(state)?;
        }

        state.finish_at(checkpoint, TexElementType::LeftRightPair);
        Ok(())
    }

    /// Parse a `\limits` or `\nolimits` command.
    fn parse_limits<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        let kind = state.peek_kind().unwrap_or(TexTokenType::Limits);
        state.bump();
        let element = match kind {
            TexTokenType::Nolimits => TexElementType::Nolimits,
            _ => TexElementType::Limits,
        };
        state.finish_at(checkpoint, element);
        Ok(())
    }

    /// Consume a single delimiter token after `\left`, `\middle` or `\right`.
    /// Handles `\left\{` and `\left(`, but also the invisible `\left.`.
    fn consume_delimiter_token<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        if !state.not_at_end() {
            return Ok(());
        }
        match state.peek_kind() {
            // One char delimiters: ( ) [ ] . | / < > =
            Some(TexTokenType::LeftParen)
            | Some(TexTokenType::RightParen)
            | Some(TexTokenType::LeftBracket)
            | Some(TexTokenType::RightBracket)
            | Some(TexTokenType::Dot)
            | Some(TexTokenType::Pipe)
            | Some(TexTokenType::Slash)
            | Some(TexTokenType::Less)
            | Some(TexTokenType::Greater)
            | Some(TexTokenType::Equal) => {
                state.bump();
                Ok(())
            }
            // Delimiter commands like \{ \} \langle \rangle \lvert \rvert etc.
            Some(TexTokenType::BraceDelim) | Some(TexTokenType::Command) | Some(TexTokenType::Backslash) => {
                state.bump();
                Ok(())
            }
            // Grouped delimiter: \left{\bigcap} ... unlikely in practice,
            // but we tolerate it by consuming a single mandatory argument.
            Some(TexTokenType::LeftBrace) => self.parse_group(state),
            _ => {
                // Unknown or no delimiter — be forgiving and move on.
                state.bump();
                Ok(())
            }
        }
    }

    fn parse_optional_argument<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        state.expect(TexTokenType::LeftBracket)?;
        while state.not_at_end() && !state.at(TexTokenType::RightBracket) {
            self.parse_item(state)?;
        }
        state.expect(TexTokenType::RightBracket)?;
        state.finish_at(checkpoint, TexElementType::OptionalArgument);
        Ok(())
    }

    fn parse_mandatory_argument<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        state.expect(TexTokenType::LeftBrace)?;
        while state.not_at_end() && !state.at(TexTokenType::RightBrace) {
            self.parse_item(state)?;
        }
        state.expect(TexTokenType::RightBrace)?;
        state.finish_at(checkpoint, TexElementType::MandatoryArgument);
        Ok(())
    }
}

impl<'config> Parser<TexLanguage> for TexParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<TexLanguage>) -> ParseOutput<'a, TexLanguage> {
        let lexer = TexLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();

            while state.not_at_end() {
                self.parse_item(state)?
            }

            Ok(state.finish_at(checkpoint, TexElementType::Root))
        })
    }
}
