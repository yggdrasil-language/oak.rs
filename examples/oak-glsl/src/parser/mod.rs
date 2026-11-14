/// Element type definitions for the GLSL parser.
pub mod element_type;

use crate::{language::GlslLanguage, lexer::token_type::GlslTokenType};
use element_type::GlslElementType;
use oak_core::{
    GreenNode, Parser, ParserState, TokenType, UniversalTokenRole,
    parser::parse_with_lexer,
    source::{Source, TextEdit},
};

/// A parser state for the GLSL language.
pub(crate) type State<'a, S> = ParserState<'a, GlslLanguage, S>;

/// A parser for the GLSL language.
pub struct GlslParser<'config> {
    /// Language configuration.
    pub(crate) config: &'config GlslLanguage,
}

impl<'config> GlslParser<'config> {
    /// Creates a new GLSL parser.
    pub fn new(config: &'config GlslLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Parser<GlslLanguage> for GlslParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl oak_core::ParseCache<GlslLanguage>) -> oak_core::ParseOutput<'a, GlslLanguage> {
        let lexer = crate::lexer::GlslLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let cp = state.checkpoint();
            loop {
                state.skip_trivia();
                if !state.not_at_end() || state.at(GlslTokenType::Eof) {
                    break;
                }

                let before = state.checkpoint().0;
                self.parse_item(state);
                // Every branch of `parse_item` must make progress, but a guard
                // keeps malformed input from looping forever.
                if state.checkpoint().0 == before && state.not_at_end() {
                    state.bump();
                }
            }
            Ok(state.finish_at(cp, GlslElementType::Root))
        })
    }
}

impl<'config> GlslParser<'config> {
    // --- Module level -------------------------------------------------------

    /// Parses a single module-level item, including its leading `layout(...)`.
    fn parse_item<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_layout(state);

        match state.peek_kind() {
            Some(GlslTokenType::Struct) => self.parse_struct(state, cp),
            Some(GlslTokenType::Directive) | Some(GlslTokenType::Semicolon) => {
                // Preprocessor lines and stray semicolons do not produce items.
                state.bump();
            }
            Some(kind) if starts_declaration(state, kind) => self.parse_declaration(state, cp),
            Some(_) => {
                state.syntax_error("expected a declaration");
                state.bump();
            }
            None => {}
        }
    }

    /// Parses a declaration that starts with optional qualifiers and a type.
    fn parse_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        let has_const = self.parse_qualifiers(state);

        // `layout(local_size_x = 64) in;` declares no variable.
        if state.at(GlslTokenType::Semicolon) {
            self.finish_terminated(state);
            state.finish_at(cp, GlslElementType::VariableDeclaration);
            return;
        }

        // `buffer Data { ... };` declares an interface block whose name is not a type.
        if state.at(GlslTokenType::Identifier) && state.peek_non_trivia_kind_at(1) == Some(GlslTokenType::LeftBrace) {
            state.bump();
            self.parse_member_block(state);
            self.finish_terminated(state);
            state.finish_at(cp, GlslElementType::VariableDeclaration);
            return;
        }

        self.parse_type(state);

        if state.at(GlslTokenType::Identifier) {
            state.bump();
        }
        else if !state.at(GlslTokenType::Semicolon) && !state.at(GlslTokenType::LeftBrace) && !state.at(GlslTokenType::LeftParen) {
            state.record_expected_name("identifier");
        }

        if state.at(GlslTokenType::LeftParen) {
            self.parse_param_list(state);
            self.parse_block(state);
            state.finish_at(cp, GlslElementType::FunctionDeclaration);
        }
        else {
            if state.at(GlslTokenType::LeftBracket) {
                self.skip_array_extent(state);
            }
            if state.eat(GlslTokenType::Assign) {
                self.parse_expression(state);
            }
            self.finish_terminated(state);
            state.finish_at(cp, if has_const { GlslElementType::GlobalConstDeclaration } else { GlslElementType::VariableDeclaration });
        }
    }

    /// Parses a function parameter list.
    fn parse_param_list<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        if !state.eat(GlslTokenType::LeftParen) {
            state.record_expected("LeftParen");
            state.finish_at(cp, GlslElementType::ParameterList);
            return;
        }
        while state.not_at_end() && !state.at(GlslTokenType::RightParen) && !state.at(GlslTokenType::Eof) {
            let before = state.checkpoint().0;
            self.parse_param(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
            if !state.eat(GlslTokenType::Comma) {
                break;
            }
        }
        if !state.eat(GlslTokenType::RightParen) {
            state.record_expected("RightParen");
        }
        state.finish_at(cp, GlslElementType::ParameterList);
    }

    /// Parses a function parameter.
    fn parse_param<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_layout(state);
        self.parse_qualifiers(state);
        self.parse_type(state);
        if state.at(GlslTokenType::Identifier) {
            state.bump();
        }
        if state.at(GlslTokenType::LeftBracket) {
            self.skip_array_extent(state);
        }
        state.finish_at(cp, GlslElementType::Parameter);
    }

    /// Parses a struct declaration.
    fn parse_struct<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump(); // struct
        self.parse_name(state);
        self.parse_member_block(state);
        self.finish_terminated(state);
        state.finish_at(cp, GlslElementType::StructDeclaration);
    }

    /// Parses a `{ member; ... }` block of a struct or an interface block.
    fn parse_member_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if !state.eat(GlslTokenType::LeftBrace) {
            state.record_expected("LeftBrace");
            return;
        }
        while state.not_at_end() && !state.at(GlslTokenType::RightBrace) && !state.at(GlslTokenType::Eof) {
            let before = state.checkpoint().0;
            self.parse_struct_member(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
        }
        if !state.eat(GlslTokenType::RightBrace) {
            state.record_expected("RightBrace");
        }
    }

    /// Parses a struct member or an interface block member.
    fn parse_struct_member<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_layout(state);
        self.parse_qualifiers(state);
        self.parse_type(state);
        if state.at(GlslTokenType::Identifier) {
            state.bump();
        }
        else {
            state.record_expected_name("identifier");
        }
        if state.at(GlslTokenType::LeftBracket) {
            self.skip_array_extent(state);
        }
        self.finish_terminated(state);
        state.finish_at(cp, GlslElementType::StructMemberDeclaration);
    }

    // --- Layout qualifiers --------------------------------------------------

    /// Parses zero or more `layout(...)` qualifiers.
    fn parse_layout<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let mut any = false;
        while state.at(GlslTokenType::Layout) {
            state.bump(); // layout

            if !state.eat(GlslTokenType::LeftParen) {
                state.record_expected("LeftParen");
                return any;
            }

            while state.not_at_end() && !state.at(GlslTokenType::RightParen) && !state.at(GlslTokenType::Eof) {
                let entry_cp = state.checkpoint();
                match state.peek_kind() {
                    Some(kind) if is_name_like(kind) => state.bump(),
                    Some(_) => state.record_expected_name("layout qualifier"),
                    None => break,
                }
                if state.eat(GlslTokenType::Assign) {
                    self.parse_expression(state);
                }
                state.finish_at(entry_cp, GlslElementType::LayoutQualifier);
                if !state.eat(GlslTokenType::Comma) {
                    break;
                }
            }

            if !state.eat(GlslTokenType::RightParen) {
                state.record_expected("RightParen");
            }
            any = true;
        }
        any
    }

    // --- Types --------------------------------------------------------------

    /// Parses a type reference such as `vec3`, `mat4`, or a struct name.
    fn parse_type<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, GlslLanguage> {
        let cp = state.checkpoint();
        if let Some(kind) = state.peek_kind() {
            if is_type_token(kind) {
                state.bump();
            }
            else if is_type_terminator(kind) {
                state.syntax_error("expected a type");
            }
            else {
                state.syntax_error("expected a type");
                state.bump();
            }
        }
        else {
            state.record_unexpected_eof();
        }

        if state.at(GlslTokenType::LeftBracket) {
            self.skip_array_extent(state);
        }

        state.finish_at(cp, GlslElementType::TypeReference)
    }

    /// Consumes a `[ ... ]` array extent. Extents are not modelled by the AST.
    fn skip_array_extent<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let mut depth = 0usize;
        while state.not_at_end() {
            match state.peek_kind() {
                Some(GlslTokenType::LeftBracket) => {
                    depth += 1;
                    state.bump();
                }
                Some(GlslTokenType::RightBracket) => {
                    state.bump();
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return;
                    }
                }
                Some(GlslTokenType::Eof) | None => return,
                Some(GlslTokenType::Semicolon) | Some(GlslTokenType::RightBrace) => return,
                Some(_) => state.bump(),
            }
        }
    }

    /// Parses an identifier used as a declared name.
    fn parse_name<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if state.at(GlslTokenType::Identifier) {
            state.bump();
        }
        else {
            state.record_expected_name("identifier");
        }
    }

    /// Parses zero or more declaration qualifiers and reports whether `const`
    /// was among them.
    fn parse_qualifiers<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let mut has_const = false;
        while let Some(kind) = state.peek_kind() {
            if !is_qualifier(kind) {
                break;
            }
            if kind == GlslTokenType::Const {
                has_const = true;
            }
            state.bump();
        }
        has_const
    }

    // --- Statements ---------------------------------------------------------

    /// Parses a compound statement.
    fn parse_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, GlslLanguage> {
        let cp = state.checkpoint();
        if !state.eat(GlslTokenType::LeftBrace) {
            state.record_expected("LeftBrace");
        }
        while state.not_at_end() && !state.at(GlslTokenType::RightBrace) && !state.at(GlslTokenType::Eof) {
            let before = state.checkpoint().0;
            self.parse_statement(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
        }
        if !state.eat(GlslTokenType::RightBrace) {
            state.record_expected("RightBrace");
        }
        state.finish_at(cp, GlslElementType::Block)
    }

    /// Parses a single statement.
    fn parse_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        match state.peek_kind() {
            Some(GlslTokenType::LeftBrace) => {
                self.parse_block(state);
            }
            Some(GlslTokenType::If) => self.parse_if_statement(state),
            Some(GlslTokenType::For) => self.parse_for_statement(state),
            Some(GlslTokenType::While) => self.parse_while_statement(state),
            Some(GlslTokenType::Do) => self.parse_do_while_statement(state),
            Some(GlslTokenType::Break) => {
                let cp = state.checkpoint();
                state.bump();
                self.finish_terminated(state);
                state.finish_at(cp, GlslElementType::BreakStatement);
            }
            Some(GlslTokenType::Continue) => {
                let cp = state.checkpoint();
                state.bump();
                self.finish_terminated(state);
                state.finish_at(cp, GlslElementType::ContinueStatement);
            }
            Some(GlslTokenType::Discard) => {
                let cp = state.checkpoint();
                state.bump();
                self.finish_terminated(state);
                state.finish_at(cp, GlslElementType::DiscardStatement);
            }
            Some(GlslTokenType::Return) => {
                let cp = state.checkpoint();
                state.bump();
                if state.not_at_end() && !state.at(GlslTokenType::Semicolon) && !state.at(GlslTokenType::RightBrace) {
                    self.parse_expression(state);
                }
                self.finish_terminated(state);
                state.finish_at(cp, GlslElementType::ReturnStatement);
            }
            Some(kind) if starts_declaration(state, kind) => self.parse_var_statement(state),
            _ => self.parse_expression_statement(state, true),
        }
    }

    /// Consumes the terminating semicolon of a statement, reporting it when missing.
    fn finish_terminated<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if !state.eat(GlslTokenType::Semicolon) {
            state.record_expected("Semicolon");
        }
    }

    /// Parses a local variable declaration.
    fn parse_var_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_qualifiers(state);
        self.parse_type(state);
        if state.at(GlslTokenType::Identifier) {
            state.bump();
        }
        else {
            state.record_expected_name("identifier");
        }
        if state.at(GlslTokenType::LeftBracket) {
            self.skip_array_extent(state);
        }
        if state.eat(GlslTokenType::Assign) {
            self.parse_expression(state);
        }
        self.finish_terminated(state);
        state.finish_at(cp, GlslElementType::VarStatement);
    }

    /// Parses an `if` statement, including `else if` chains.
    fn parse_if_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // if
        self.parse_expression(state);
        self.parse_block(state);

        if state.eat(GlslTokenType::Else) {
            if state.at(GlslTokenType::If) {
                self.parse_if_statement(state);
            }
            else {
                self.parse_block(state);
            }
        }

        state.finish_at(cp, GlslElementType::IfStatement);
    }

    /// Parses a `for` statement.
    fn parse_for_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // for

        if !state.eat(GlslTokenType::LeftParen) {
            state.record_expected("LeftParen");
        }

        // Initializer.
        let init_cp = state.checkpoint();
        if !state.at(GlslTokenType::Semicolon) && state.not_at_end() {
            if state.peek_kind().is_some_and(|kind| starts_declaration(state, kind)) {
                self.parse_var_statement(state);
            }
            else {
                self.parse_expression_statement(state, false);
            }
        }
        if state.at(GlslTokenType::Semicolon) {
            state.bump();
        }
        state.finish_at(init_cp, GlslElementType::ForInit);

        // Condition.
        let cond_cp = state.checkpoint();
        if state.not_at_end() && !state.at(GlslTokenType::Semicolon) && !state.at(GlslTokenType::RightParen) {
            self.parse_expression(state);
        }
        if state.at(GlslTokenType::Semicolon) {
            state.bump();
        }
        state.finish_at(cond_cp, GlslElementType::ForCond);

        // Update.
        let update_cp = state.checkpoint();
        if state.not_at_end() && !state.at(GlslTokenType::RightParen) {
            self.parse_expression_statement(state, false);
        }
        state.finish_at(update_cp, GlslElementType::ForUpdate);

        if !state.eat(GlslTokenType::RightParen) {
            state.record_expected("RightParen");
        }

        self.parse_block(state);
        state.finish_at(cp, GlslElementType::ForStatement);
    }

    /// Parses a `while` statement.
    fn parse_while_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // while
        self.parse_expression(state);
        self.parse_block(state);
        state.finish_at(cp, GlslElementType::WhileStatement);
    }

    /// Parses a `do ... while` statement.
    fn parse_do_while_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // do
        self.parse_block(state);
        if !state.eat(GlslTokenType::While) {
            state.record_expected("While");
        }
        self.parse_expression(state);
        self.finish_terminated(state);
        state.finish_at(cp, GlslElementType::DoWhileStatement);
    }

    /// Parses an expression or an assignment used as a statement.
    ///
    /// `terminated` controls whether a semicolon is expected after the statement.
    fn parse_expression_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, terminated: bool) {
        let cp = state.checkpoint();
        let lhs = self.parse_expression(state);

        if let Some(tokens) = assignment_operator_tokens(state) {
            let wrap_cp = state.checkpoint_before(lhs);
            for _ in 0..tokens {
                state.bump();
            }
            self.parse_expression(state);
            if terminated {
                self.finish_terminated(state);
            }
            state.finish_at(wrap_cp, GlslElementType::AssignmentStatement);
        }
        else {
            if terminated {
                self.finish_terminated(state);
            }
            state.finish_at(cp, GlslElementType::ExpressionStatement);
        }
    }

    // --- Expressions --------------------------------------------------------

    /// Parses an expression with the lowest binding power.
    fn parse_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, GlslLanguage> {
        self.parse_binary_expression(state, 0)
    }

    /// Parses a binary expression using precedence climbing.
    fn parse_binary_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, min_precedence: u8) -> &'a GreenNode<'a, GlslLanguage> {
        let mut left = self.parse_unary_expression(state);

        while state.not_at_end() {
            let Some((precedence, tokens)) = binary_operator(state)
            else {
                break;
            };
            if precedence < min_precedence {
                break;
            }

            let cp = state.checkpoint_before(left);
            for _ in 0..tokens {
                state.bump();
            }
            let _right = self.parse_binary_expression(state, precedence + 1);
            left = state.finish_at(cp, GlslElementType::ExprBinary);
        }

        left
    }

    /// Parses a prefix or postfix expression.
    fn parse_unary_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, GlslLanguage> {
        if matches!(state.peek_kind(), Some(GlslTokenType::Minus) | Some(GlslTokenType::LogicalNot) | Some(GlslTokenType::BitwiseNot) | Some(GlslTokenType::Increment) | Some(GlslTokenType::Decrement)) {
            let cp = state.checkpoint();
            state.bump();
            self.parse_unary_expression(state);
            return state.finish_at(cp, GlslElementType::ExprUnary);
        }

        self.parse_postfix_expression(state)
    }

    /// Parses member access, indexing, calls, and postfix increments on a
    /// primary expression.
    fn parse_postfix_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, GlslLanguage> {
        let mut left = self.parse_primary_expression(state);

        loop {
            match state.peek_kind() {
                Some(GlslTokenType::Dot) => {
                    let cp = state.checkpoint_before(left);
                    state.bump();
                    if let Some(kind) = state.peek_kind() {
                        if is_name_like(kind) {
                            state.bump();
                        }
                        else {
                            state.record_expected_name("member name");
                        }
                    }
                    left = state.finish_at(cp, GlslElementType::ExprMember);
                }
                Some(GlslTokenType::LeftBracket) => {
                    let cp = state.checkpoint_before(left);
                    state.bump();
                    self.parse_expression(state);
                    if !state.eat(GlslTokenType::RightBracket) {
                        state.record_expected("RightBracket");
                    }
                    left = state.finish_at(cp, GlslElementType::ExprIndex);
                }
                Some(GlslTokenType::LeftParen) => {
                    let cp = state.checkpoint_before(left);
                    self.parse_call_arguments(state);
                    left = state.finish_at(cp, GlslElementType::ExprCall);
                }
                Some(GlslTokenType::Increment) | Some(GlslTokenType::Decrement) => {
                    let cp = state.checkpoint_before(left);
                    state.bump();
                    left = state.finish_at(cp, GlslElementType::ExprUnary);
                }
                _ => break,
            }
        }

        left
    }

    /// Parses a parenthesized argument list.
    fn parse_call_arguments<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if !state.eat(GlslTokenType::LeftParen) {
            state.record_expected("LeftParen");
            return;
        }
        while state.not_at_end() && !state.at(GlslTokenType::RightParen) && !state.at(GlslTokenType::Eof) {
            let before = state.checkpoint().0;
            self.parse_expression(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
            if !state.eat(GlslTokenType::Comma) {
                break;
            }
        }
        if !state.eat(GlslTokenType::RightParen) {
            state.record_expected("RightParen");
        }
    }

    /// Parses a primary expression: a literal, a name, or a parenthesized
    /// expression.
    fn parse_primary_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, GlslLanguage> {
        let cp = state.checkpoint();

        match state.peek_kind() {
            Some(GlslTokenType::IntegerLiteral) | Some(GlslTokenType::FloatLiteral) | Some(GlslTokenType::StringLiteral) | Some(GlslTokenType::BooleanLiteral) => {
                state.bump();
                state.finish_at(cp, GlslElementType::Expr)
            }
            Some(kind) if is_name_like(kind) => {
                state.bump();
                state.finish_at(cp, GlslElementType::Expr)
            }
            Some(GlslTokenType::LeftParen) => {
                state.bump();
                self.parse_expression(state);
                if !state.eat(GlslTokenType::RightParen) {
                    state.record_expected("RightParen");
                }
                state.finish_at(cp, GlslElementType::ExprParen)
            }
            _ => {
                state.syntax_error("expected an expression");
                if state.not_at_end() && !state.at(GlslTokenType::Eof) {
                    state.bump();
                }
                state.finish_at(cp, GlslElementType::Expr)
            }
        }
    }
}

/// Returns `true` for identifiers and keyword tokens that may appear as names.
fn is_name_like(kind: GlslTokenType) -> bool {
    kind == GlslTokenType::Identifier || kind.role() == UniversalTokenRole::Keyword
}

/// Returns `true` for tokens that open a declaration rather than an expression.
fn starts_declaration<'a, S: Source + ?Sized>(state: &State<'a, S>, kind: GlslTokenType) -> bool {
    if is_qualifier(kind) {
        return true;
    }
    if is_type_keyword(kind) {
        // A type keyword followed by `(` is a constructor call such as
        // `vec3(1.0, 0.0, 0.0)`.
        return state.peek_non_trivia_kind_at(1) != Some(GlslTokenType::LeftParen);
    }
    kind == GlslTokenType::Identifier && state.peek_non_trivia_kind_at(1) == Some(GlslTokenType::Identifier)
}

/// Returns `true` for keyword tokens that qualify a declaration.
fn is_qualifier(kind: GlslTokenType) -> bool {
    matches!(
        kind,
        GlslTokenType::Const
            | GlslTokenType::Uniform
            | GlslTokenType::In
            | GlslTokenType::Out
            | GlslTokenType::Inout
            | GlslTokenType::Buffer
            | GlslTokenType::Shared
            | GlslTokenType::Precision
            | GlslTokenType::Highp
            | GlslTokenType::Mediump
            | GlslTokenType::Lowp
            | GlslTokenType::Flat
            | GlslTokenType::Smooth
            | GlslTokenType::Noperspective
            | GlslTokenType::Centroid
            | GlslTokenType::Sample
            | GlslTokenType::Invariant
            | GlslTokenType::Patch
            | GlslTokenType::Readonly
            | GlslTokenType::Writeonly
            | GlslTokenType::Coherent
            | GlslTokenType::Volatile
            | GlslTokenType::Restrict
    )
}

/// Returns `true` for builtin type keywords.
fn is_type_keyword(kind: GlslTokenType) -> bool {
    matches!(
        kind,
        GlslTokenType::Void
            | GlslTokenType::Bool
            | GlslTokenType::Int
            | GlslTokenType::Uint
            | GlslTokenType::Float
            | GlslTokenType::Double
            | GlslTokenType::Vec2
            | GlslTokenType::Vec3
            | GlslTokenType::Vec4
            | GlslTokenType::IVec2
            | GlslTokenType::IVec3
            | GlslTokenType::IVec4
            | GlslTokenType::UVec2
            | GlslTokenType::UVec3
            | GlslTokenType::UVec4
            | GlslTokenType::BVec2
            | GlslTokenType::BVec3
            | GlslTokenType::BVec4
            | GlslTokenType::DVec2
            | GlslTokenType::DVec3
            | GlslTokenType::DVec4
            | GlslTokenType::Mat2
            | GlslTokenType::Mat3
            | GlslTokenType::Mat4
            | GlslTokenType::Mat2x2
            | GlslTokenType::Mat2x3
            | GlslTokenType::Mat2x4
            | GlslTokenType::Mat3x2
            | GlslTokenType::Mat3x3
            | GlslTokenType::Mat3x4
            | GlslTokenType::Mat4x2
            | GlslTokenType::Mat4x3
            | GlslTokenType::Mat4x4
            | GlslTokenType::Sampler2D
            | GlslTokenType::Sampler3D
            | GlslTokenType::SamplerCube
            | GlslTokenType::Sampler2DArray
            | GlslTokenType::SamplerCubeArray
            | GlslTokenType::Sampler2DShadow
            | GlslTokenType::SamplerCubeShadow
            | GlslTokenType::Sampler2DArrayShadow
            | GlslTokenType::ISampler2D
            | GlslTokenType::USampler2D
            | GlslTokenType::Image2D
            | GlslTokenType::AtomicUint
    )
}

/// Returns `true` for tokens that may start a type.
fn is_type_token(kind: GlslTokenType) -> bool {
    kind == GlslTokenType::Identifier || is_type_keyword(kind)
}

/// Returns `true` for tokens that terminate a type position.
fn is_type_terminator(kind: GlslTokenType) -> bool {
    matches!(kind, GlslTokenType::Comma | GlslTokenType::RightParen | GlslTokenType::RightBrace | GlslTokenType::RightBracket | GlslTokenType::Greater | GlslTokenType::Semicolon | GlslTokenType::Assign)
}

/// Returns the precedence and token count of the binary operator at the cursor.
fn binary_operator<'a, S: Source + ?Sized>(state: &State<'a, S>) -> Option<(u8, usize)> {
    let kind = state.peek_kind()?;
    let info = match kind {
        GlslTokenType::LogicalOr => (1, 1),
        GlslTokenType::LogicalAnd => (2, 1),
        GlslTokenType::BitwiseOr => (3, 1),
        GlslTokenType::BitwiseXor => (4, 1),
        GlslTokenType::BitwiseAnd => (5, 1),
        GlslTokenType::Equal | GlslTokenType::NotEqual => (6, 1),
        GlslTokenType::Less | GlslTokenType::LessEqual | GlslTokenType::Greater | GlslTokenType::GreaterEqual => (7, 1),
        GlslTokenType::LeftShift | GlslTokenType::RightShift => (8, 1),
        GlslTokenType::Plus | GlslTokenType::Minus => (9, 1),
        GlslTokenType::Multiply | GlslTokenType::Divide | GlslTokenType::Modulo => (10, 1),
        _ => return None,
    };
    Some(info)
}

/// Returns the number of tokens that make up the assignment operator at the cursor.
fn assignment_operator_tokens<'a, S: Source + ?Sized>(state: &State<'a, S>) -> Option<usize> {
    match state.peek_kind()? {
        GlslTokenType::Assign
        | GlslTokenType::PlusAssign
        | GlslTokenType::MinusAssign
        | GlslTokenType::MultiplyAssign
        | GlslTokenType::DivideAssign
        | GlslTokenType::ModuloAssign
        | GlslTokenType::BitwiseAndAssign
        | GlslTokenType::BitwiseOrAssign
        | GlslTokenType::BitwiseXorAssign
        | GlslTokenType::LeftShiftAssign
        | GlslTokenType::RightShiftAssign => Some(1),
        _ => None,
    }
}
