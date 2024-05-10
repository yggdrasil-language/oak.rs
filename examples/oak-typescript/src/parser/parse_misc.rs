use crate::{
    lexer::TypeScriptTokenType,
    parser::{State, TypeScriptParser},
};
use oak_core::{OakError, TokenType, source::Source};

impl<'config> TypeScriptParser<'config> {
    pub(crate) fn peek_kind<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Option<TypeScriptTokenType> {
        self.skip_trivia(state);
        state.peek_kind().map(|k| k.try_into().unwrap())
    }

    pub(crate) fn skip_trivia<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while state.not_at_end() && state.current().map(|t| t.kind.is_ignored()).unwrap_or(false) {
            state.bump();
        }
    }

    pub(crate) fn expect<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, kind: TypeScriptTokenType) -> Result<(), OakError> {
        self.skip_trivia(state);
        state.expect(kind)
    }

    pub(crate) fn eat<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, kind: TypeScriptTokenType) -> bool {
        self.skip_trivia(state);
        state.eat(kind)
    }

    pub(crate) fn at<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, kind: TypeScriptTokenType) -> bool {
        self.skip_trivia(state);
        state.at(kind.into())
    }

    pub(crate) fn is_binding_identifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        self.skip_trivia(state);
        let kind = state.peek_kind().and_then(|k| k.try_into().ok());
        let keyword_name = state.peek_text().is_some_and(|text| TypeScriptTokenType::from_keyword(&text).is_some());
        kind == Some(TypeScriptTokenType::IdentifierName) || keyword_name
    }

    pub(crate) fn parse_binding_identifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::parser::element_type::TypeScriptElementType;
        let cp = state.checkpoint();
        if self.is_binding_identifier(state) {
            state.bump();
            state.finish_at(cp, TypeScriptElementType::IdentifierName);
        }
        else {
            self.expect(state, TypeScriptTokenType::IdentifierName).ok();
        }
    }

    pub(crate) fn parse_parameters<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        self.expect(state, LeftParen).ok();
        while state.not_at_end() && !self.at(state, RightParen) {
            self.skip_trivia(state);
            let cp = state.checkpoint();

            // Handle parameter decorators
            while self.at(state, At) {
                self.parse_decorator(state)?;
            }

            if self.at(state, IdentifierName) {
                self.expect(state, IdentifierName).ok();
                // Skip type annotation
                if self.eat(state, Colon) {
                    self.parse_type_annotation(state)?;
                }
            }
            else {
                state.bump();
            }
            state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::Parameter);
            self.eat(state, Comma);
        }
        self.expect(state, RightParen).ok();
        Ok(())
    }

    pub(crate) fn parse_type_annotation<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        self.parse_type_annotation_with_arrow(state, false)
    }

    pub(crate) fn parse_return_type_annotation<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        self.parse_type_annotation_with_arrow(state, true)
    }

    /// Consumes the type that follows `as` inside an expression.
    ///
    /// [`Self::parse_type_annotation`] only knows the stop tokens that follow a
    /// `:` annotation. An assertion can sit anywhere in an expression
    /// (`items[item.id as keyof typeof items]`), so this scanner tracks its own
    /// nesting and stops at the first token that cannot continue a type at the
    /// top level.
    pub(crate) fn parse_assertion_type<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};

        self.skip_trivia(state);
        let cp = state.checkpoint();
        let mut depth = 0usize;
        let mut angles = 0usize;
        let mut consumed = false;

        while state.not_at_end() {
            let Some(kind) = self.peek_kind(state)
            else {
                break;
            };
            let top = depth == 0 && angles == 0;
            if consumed && top && assertion_type_terminator(kind) {
                break;
            }
            match kind {
                LeftParen | LeftBracket | LeftBrace => depth += 1,
                RightParen | RightBracket | RightBrace => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                Less => {
                    if !angle_group_closes(state) {
                        break;
                    }
                    angles += 1;
                }
                Greater => {
                    if angles == 0 {
                        break;
                    }
                    angles -= 1;
                }
                // `Array<Array<string>>` closes two angle groups at once.
                RightShift => {
                    if angles < 2 {
                        break;
                    }
                    angles -= 2;
                }
                _ => {}
            }
            state.bump();
            consumed = true;
        }

        if consumed {
            state.finish_at(cp, TypeScriptElementType::TypeAnnotation);
        }
    }

    fn parse_type_annotation_with_arrow<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, stop_at_arrow: bool) -> Result<(), OakError> {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};

        let cp = state.checkpoint();
        let mut parens = 0usize;
        let mut brackets = 0usize;
        let mut braces = 0usize;
        let mut angles = 0usize;
        let mut consumed = false;
        let mut last_kind = None;

        while state.not_at_end() {
            let Some(kind) = self.peek_kind(state)
            else {
                break;
            };
            let top = parens == 0 && brackets == 0 && braces == 0 && angles == 0;
            if consumed && top && matches!(kind, Comma | Semicolon | RightParen | RightBrace | Equal) {
                break;
            }
            if consumed && top && stop_at_arrow {
                if matches!(kind, Arrow) {
                    break;
                }
                // A return type ends where the function body starts. `{` may also
                // open an object type literal, so it only ends the annotation when
                // the previous token already completed a type operand
                // (`: Foo {`, `: Foo[] {`, `: Array<Foo> {`).
                if matches!(kind, LeftBrace) && !type_operand_may_start(last_kind) {
                    break;
                }
            }
            match kind {
                LeftParen => parens += 1,
                RightParen if parens > 0 => parens -= 1,
                LeftBracket => brackets += 1,
                RightBracket if brackets > 0 => brackets -= 1,
                LeftBrace => braces += 1,
                RightBrace if braces > 0 => braces -= 1,
                Less => angles += 1,
                Greater if angles > 0 => angles -= 1,
                _ => {}
            }
            state.bump();
            consumed = true;
            last_kind = Some(kind);
        }

        if consumed {
            state.finish_at(cp, TypeScriptElementType::TypeAnnotation);
        }
        Ok(())
    }

    pub(crate) fn parse_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        self.expect(state, LeftBrace).ok();
        while state.not_at_end() && !self.at(state, RightBrace) {
            let before = state.checkpoint();
            self.parse_statement(state)?;
            if state.checkpoint().0 == before.0 && state.not_at_end() {
                state.bump();
            }
        }
        self.expect(state, RightBrace).ok();
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::BlockStatement);
        Ok(())
    }
}

/// Token kinds after which a type operand (object type literal, tuple, ...) may start.
fn type_operand_may_start(previous: Option<TypeScriptTokenType>) -> bool {
    use TypeScriptTokenType::*;
    matches!(previous, Some(Colon | Pipe | Ampersand | Less | Comma | Question | Readonly | New | Infer))
}

/// Token kinds that end a type at the top level of an `as` assertion.
fn assertion_type_terminator(kind: TypeScriptTokenType) -> bool {
    use TypeScriptTokenType::*;
    matches!(
        kind,
        Comma
            | Semicolon
            | Colon
            | Question
            | Arrow
            | As
            | In
            | Instanceof
            | RightParen
            | RightBracket
            | RightBrace
            | Plus
            | Minus
            | Star
            | Slash
            | Percent
            | StarStar
            | PlusPlus
            | MinusMinus
            | LessEqual
            | GreaterEqual
            | EqualEqual
            | NotEqual
            | EqualEqualEqual
            | NotEqualEqual
            | AmpersandAmpersand
            | PipePipe
            | QuestionQuestion
            | Equal
            | PlusEqual
            | MinusEqual
            | StarEqual
            | SlashEqual
            | PercentEqual
            | StarStarEqual
            | LeftShift
            | LeftShiftEqual
            | RightShift
            | RightShiftEqual
            | UnsignedRightShift
            | UnsignedRightShiftEqual
            | AmpersandEqual
            | PipeEqual
            | CaretEqual
            | AmpersandAmpersandEqual
            | PipePipeEqual
            | QuestionQuestionEqual
    )
}

/// Returns true when the `<` at the current position opens a generic argument
/// list that closes at the same level before any top-level type terminator.
fn angle_group_closes<S: Source + ?Sized>(state: &State<'_, S>) -> bool {
    use TypeScriptTokenType::*;
    let mut depth = 0usize;
    let mut offset = 1usize;
    while let Some(kind) = state.peek_non_trivia_kind_at(offset) {
        match kind {
            Less => depth += 1,
            Greater => {
                if depth == 0 {
                    return true;
                }
                depth -= 1;
            }
            // `Array<Array<string>>` closes two angle groups at once.
            RightShift => return depth <= 1,
            _ if depth == 0 && assertion_type_terminator(kind) => return false,
            _ => {}
        }
        offset += 1;
        if offset > 64 {
            return false;
        }
    }
    false
}
