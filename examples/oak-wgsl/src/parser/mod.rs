/// WGSL element types.
pub mod element_type;

use crate::{language::WgslLanguage, lexer::token_type::WgslTokenType};
use element_type::WgslElementType;
use oak_core::{
    GreenNode, OakError, TokenType, UniversalTokenRole,
    parser::{Parser, ParserState, parse_with_lexer},
    source::{Source, TextEdit},
};

/// A parser state for the WGSL language.
pub(crate) type State<'a, S> = ParserState<'a, WgslLanguage, S>;

/// Attributes understood by the WGSL frontend. Unknown names are still parsed
/// structurally, but reported through a diagnostic.
const KNOWN_ATTRIBUTES: &[&str] = &[
    "align",
    "binding",
    "blend_src",
    "builtin",
    "compute",
    "const",
    "diagnostic",
    "early_depth_test",
    "fragment",
    "group",
    "id",
    "index",
    "interpolate",
    "invariant",
    "location",
    "must_use",
    "read",
    "read_write",
    "size",
    "storage",
    "vertex",
    "workgroup_size",
    "write",
];

/// A parser for the WGSL language.
pub struct WgslParser<'config> {
    pub(crate) config: &'config WgslLanguage,
}

impl<'config> WgslParser<'config> {
    /// Creates a new WGSL parser.
    pub fn new(config: &'config WgslLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Parser<WgslLanguage> for WgslParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl oak_core::ParseCache<WgslLanguage>) -> oak_core::ParseOutput<'a, WgslLanguage> {
        let lexer = crate::lexer::WgslLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let cp = state.checkpoint();
            while state.not_at_end() {
                state.skip_trivia();
                if !state.not_at_end() {
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
            Ok(state.finish_at(cp, WgslElementType::Root))
        })
    }
}

impl<'config> WgslParser<'config> {
    // --- Module level -------------------------------------------------------

    /// Parses a single module-level item, including its leading attributes.
    fn parse_item<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        let has_attributes = self.parse_attributes(state);

        match state.peek_kind() {
            Some(WgslTokenType::StructKw) => self.parse_struct(state, cp),
            Some(WgslTokenType::VarKw) => self.parse_global_var(state, cp),
            Some(WgslTokenType::LetKw) | Some(WgslTokenType::ConstKw) => self.parse_global_const(state, cp),
            Some(WgslTokenType::FnKw) => self.parse_function(state, cp),
            Some(WgslTokenType::AliasKw) | Some(WgslTokenType::TypeKw) => self.parse_type_alias(state, cp),
            Some(WgslTokenType::EnableKw) | Some(WgslTokenType::RequiresKw) => self.parse_directive(state, cp),
            Some(_) => {
                if has_attributes {
                    state.syntax_error("expected a declaration after attributes");
                }
                state.bump();
            }
            None => {}
        }
    }

    /// Parses a module-scope directive such as `enable f16;`.
    fn parse_directive<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump();
        while state.not_at_end() && !state.at(WgslTokenType::Semicolon) {
            state.bump();
        }
        if !state.eat(WgslTokenType::Semicolon) {
            state.record_expected("Semicolon");
        }
        state.finish_at(cp, WgslElementType::Directive);
    }

    /// Parses a module-scope `var` declaration.
    fn parse_global_var<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump(); // var

        if state.eat(WgslTokenType::Lt) {
            while state.not_at_end() && !state.at(WgslTokenType::Gt) {
                if state.at(WgslTokenType::Comma) {
                    state.bump();
                }
                else if state.at(WgslTokenType::Gt) {
                    break;
                }
                else if state.at(WgslTokenType::RightBrace) || state.at(WgslTokenType::Semicolon) {
                    break;
                }
                else {
                    state.bump();
                }
            }
            if !state.eat(WgslTokenType::Gt) {
                state.record_expected("Gt");
            }
        }

        self.parse_name(state);

        if state.eat(WgslTokenType::Colon) {
            self.parse_type(state);
        }
        if state.eat(WgslTokenType::Assign) {
            self.parse_expression(state);
        }
        if !state.eat(WgslTokenType::Semicolon) {
            state.record_expected("Semicolon");
        }
        state.finish_at(cp, WgslElementType::Variable);
    }

    /// Parses a module-scope `const` (or a module-scope `let`) declaration.
    fn parse_global_const<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump();
        self.parse_name(state);
        if state.eat(WgslTokenType::Colon) {
            self.parse_type(state);
        }
        if state.eat(WgslTokenType::Assign) {
            self.parse_expression(state);
        }
        if !state.eat(WgslTokenType::Semicolon) {
            state.record_expected("Semicolon");
        }
        state.finish_at(cp, WgslElementType::GlobalConst);
    }

    /// Parses a type alias declaration.
    fn parse_type_alias<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump(); // alias / type
        self.parse_name(state);
        if !state.eat(WgslTokenType::Assign) {
            state.record_expected("Assign");
        }
        self.parse_type(state);
        if !state.eat(WgslTokenType::Semicolon) {
            state.record_expected("Semicolon");
        }
        state.finish_at(cp, WgslElementType::TypeAlias);
    }

    /// Parses a function declaration.
    fn parse_function<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump(); // fn
        self.parse_name(state);

        if !state.eat(WgslTokenType::LeftParen) {
            state.record_expected("LeftParen");
        }
        while state.not_at_end() && !state.at(WgslTokenType::RightParen) {
            let before = state.checkpoint().0;
            self.parse_param(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
            if !state.eat(WgslTokenType::Comma) {
                break;
            }
        }
        if !state.eat(WgslTokenType::RightParen) {
            state.record_expected("RightParen");
        }

        if state.eat(WgslTokenType::Arrow) {
            self.parse_type(state);
        }

        self.parse_block(state);
        state.finish_at(cp, WgslElementType::Function);
    }

    /// Parses a function parameter.
    fn parse_param<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_attributes(state);
        self.parse_binding_name(state);
        if !state.eat(WgslTokenType::Colon) {
            state.record_expected("Colon");
        }
        self.parse_type(state);
        state.finish_at(cp, WgslElementType::Param);
    }

    /// Parses a struct declaration.
    fn parse_struct<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump(); // struct
        self.parse_name(state);

        if !state.eat(WgslTokenType::LeftBrace) {
            state.record_expected("LeftBrace");
        }
        while state.not_at_end() && !state.at(WgslTokenType::RightBrace) {
            let before = state.checkpoint().0;
            self.parse_struct_member(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
            if !state.eat(WgslTokenType::Comma) {
                if state.at(WgslTokenType::RightBrace) {
                    break;
                }
            }
        }
        if !state.eat(WgslTokenType::RightBrace) {
            state.record_expected("RightBrace");
        }
        state.finish_at(cp, WgslElementType::Struct);
    }

    /// Parses a struct member.
    fn parse_struct_member<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_attributes(state);
        self.parse_binding_name(state);
        if !state.eat(WgslTokenType::Colon) {
            state.record_expected("Colon");
        }
        self.parse_type(state);
        state.finish_at(cp, WgslElementType::StructMember);
    }

    // --- Attributes ---------------------------------------------------------

    /// Parses zero or more attributes. Returns `true` when at least one was parsed.
    fn parse_attributes<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let mut any = false;
        while state.at(WgslTokenType::At) {
            self.parse_attribute(state);
            any = true;
        }
        any
    }

    /// Parses one attribute such as `@group(0)`.
    fn parse_attribute<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // @

        let mut name = None;
        if let Some(kind) = state.peek_kind() {
            if is_name_like(kind) {
                let offset = state.current_offset();
                name = state.peek_text().map(|text| (text.to_string(), offset));
                state.bump();
            }
            else {
                state.record_expected_name("attribute");
            }
        }

        if state.eat(WgslTokenType::LeftParen) {
            while state.not_at_end() && !state.at(WgslTokenType::RightParen) {
                let before = state.checkpoint().0;
                self.parse_expression(state);
                if state.checkpoint().0 == before {
                    state.bump();
                }
                if !state.eat(WgslTokenType::Comma) {
                    break;
                }
            }
            if !state.eat(WgslTokenType::RightParen) {
                state.record_expected("RightParen");
            }
        }

        if let Some((text, offset)) = name {
            if !KNOWN_ATTRIBUTES.contains(&text.as_str()) {
                let error = OakError::syntax_error(format!("unknown WGSL attribute `{text}`"), offset, state.source.source_id());
                state.errors.push(error);
            }
        }

        state.finish_at(cp, WgslElementType::Attribute);
    }

    // --- Types --------------------------------------------------------------

    /// Parses a type reference such as `vec3<f32>` or `array<f32, 64>`.
    fn parse_type<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, WgslLanguage> {
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

        if state.at(WgslTokenType::Lt) {
            state.bump();
            while state.not_at_end() && !state.at(WgslTokenType::Gt) {
                let before = state.checkpoint().0;
                self.parse_type(state);
                if state.checkpoint().0 == before {
                    state.bump();
                }
                if !state.eat(WgslTokenType::Comma) {
                    break;
                }
            }
            if !state.eat(WgslTokenType::Gt) {
                state.record_expected("Gt");
            }
        }

        state.finish_at(cp, WgslElementType::Type)
    }

    /// Parses an identifier used as a declared name.
    fn parse_name<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if state.at(WgslTokenType::Identifier) {
            state.bump();
        }
        else {
            state.record_expected_name("identifier");
        }
    }

    /// Parses a name that may collide with a builtin keyword, such as a struct
    /// member called `position` or a parameter called `vertex_index`.
    fn parse_binding_name<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if let Some(kind) = state.peek_kind() {
            if is_name_like(kind) {
                state.bump();
                return;
            }
        }
        state.record_expected_name("identifier");
    }

    // --- Statements ---------------------------------------------------------

    /// Parses a compound statement.
    fn parse_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, WgslLanguage> {
        let cp = state.checkpoint();
        if !state.eat(WgslTokenType::LeftBrace) {
            state.record_expected("LeftBrace");
        }
        while state.not_at_end() && !state.at(WgslTokenType::RightBrace) {
            let before = state.checkpoint().0;
            self.parse_statement(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
        }
        if !state.eat(WgslTokenType::RightBrace) {
            state.record_expected("RightBrace");
        }
        state.finish_at(cp, WgslElementType::Block)
    }

    /// Parses a single statement.
    fn parse_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        match state.peek_kind() {
            Some(WgslTokenType::LeftBrace) => {
                self.parse_block(state);
            }
            Some(WgslTokenType::VarKw) | Some(WgslTokenType::LetKw) | Some(WgslTokenType::ConstKw) => self.parse_var_statement(state),
            Some(WgslTokenType::IfKw) => self.parse_if_statement(state),
            Some(WgslTokenType::ForKw) => self.parse_for_statement(state),
            Some(WgslTokenType::WhileKw) => self.parse_while_statement(state),
            Some(WgslTokenType::LoopKw) => self.parse_loop_statement(state),
            Some(WgslTokenType::BreakKw) => {
                let cp = state.checkpoint();
                state.bump();
                self.finish_terminated(state);
                state.finish_at(cp, WgslElementType::BreakStatement);
            }
            Some(WgslTokenType::ContinueKw) => {
                let cp = state.checkpoint();
                state.bump();
                self.finish_terminated(state);
                state.finish_at(cp, WgslElementType::ContinueStatement);
            }
            Some(WgslTokenType::DiscardKw) => {
                let cp = state.checkpoint();
                state.bump();
                self.finish_terminated(state);
                state.finish_at(cp, WgslElementType::DiscardStatement);
            }
            Some(WgslTokenType::ReturnKw) => {
                let cp = state.checkpoint();
                state.bump();
                if state.not_at_end() && !state.at(WgslTokenType::Semicolon) && !state.at(WgslTokenType::RightBrace) {
                    self.parse_expression(state);
                }
                self.finish_terminated(state);
                state.finish_at(cp, WgslElementType::ReturnStatement);
            }
            _ => self.parse_expression_statement(state, true),
        }
    }

    /// Consumes the terminating semicolon of a statement, reporting it when missing.
    fn finish_terminated<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if !state.eat(WgslTokenType::Semicolon) {
            state.record_expected("Semicolon");
        }
    }

    /// Parses a local `var`/`let`/`const` declaration.
    fn parse_var_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // var / let / const
        self.parse_name(state);
        if state.eat(WgslTokenType::Colon) {
            self.parse_type(state);
        }
        if state.eat(WgslTokenType::Assign) {
            self.parse_expression(state);
        }
        self.finish_terminated(state);
        state.finish_at(cp, WgslElementType::VarStatement);
    }

    /// Parses an `if` statement, including `else if` chains.
    fn parse_if_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // if
        self.parse_expression(state);
        self.parse_block(state);

        if state.eat(WgslTokenType::ElseKw) {
            if state.at(WgslTokenType::IfKw) {
                self.parse_if_statement(state);
            }
            else {
                self.parse_block(state);
            }
        }

        state.finish_at(cp, WgslElementType::IfStatement);
    }

    /// Parses a `for` statement.
    fn parse_for_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // for

        if !state.eat(WgslTokenType::LeftParen) {
            state.record_expected("LeftParen");
        }

        // Initializer.
        let init_cp = state.checkpoint();
        if !state.at(WgslTokenType::Semicolon) && state.not_at_end() {
            if state.at(WgslTokenType::VarKw) || state.at(WgslTokenType::LetKw) || state.at(WgslTokenType::ConstKw) {
                self.parse_var_statement(state);
            }
            else {
                self.parse_expression_statement(state, true);
            }
        }
        if state.at(WgslTokenType::Semicolon) {
            state.bump();
        }
        state.finish_at(init_cp, WgslElementType::ForInit);

        // Condition.
        let cond_cp = state.checkpoint();
        if state.not_at_end() && !state.at(WgslTokenType::Semicolon) && !state.at(WgslTokenType::RightParen) {
            self.parse_expression(state);
        }
        if state.at(WgslTokenType::Semicolon) {
            state.bump();
        }
        state.finish_at(cond_cp, WgslElementType::ForCond);

        // Update.
        let update_cp = state.checkpoint();
        if state.not_at_end() && !state.at(WgslTokenType::RightParen) {
            self.parse_expression_statement(state, false);
        }
        state.finish_at(update_cp, WgslElementType::ForUpdate);

        if !state.eat(WgslTokenType::RightParen) {
            state.record_expected("RightParen");
        }

        self.parse_block(state);
        state.finish_at(cp, WgslElementType::ForStatement);
    }

    /// Parses a `while` statement.
    fn parse_while_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // while
        self.parse_expression(state);
        self.parse_block(state);
        state.finish_at(cp, WgslElementType::WhileStatement);
    }

    /// Parses a `loop` statement.
    fn parse_loop_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // loop
        self.parse_block(state);
        state.finish_at(cp, WgslElementType::LoopStatement);
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
            state.finish_at(wrap_cp, WgslElementType::AssignmentStatement);
        }
        else {
            if terminated {
                self.finish_terminated(state);
            }
            state.finish_at(cp, WgslElementType::ExpressionStatement);
        }
    }

    // --- Expressions --------------------------------------------------------

    /// Parses an expression with the lowest binding power.
    fn parse_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, WgslLanguage> {
        self.parse_binary_expression(state, 0)
    }

    /// Parses a binary expression using precedence climbing.
    fn parse_binary_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, min_precedence: u8) -> &'a GreenNode<'a, WgslLanguage> {
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
            left = state.finish_at(cp, WgslElementType::ExprBinary);
        }

        left
    }

    /// Parses a prefix expression.
    fn parse_unary_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, WgslLanguage> {
        if matches!(state.peek_kind(), Some(WgslTokenType::Minus) | Some(WgslTokenType::Bang) | Some(WgslTokenType::Tilde)) {
            let cp = state.checkpoint();
            state.bump();
            self.parse_unary_expression(state);
            return state.finish_at(cp, WgslElementType::ExprUnary);
        }

        self.parse_postfix_expression(state)
    }

    /// Parses member access, indexing, and calls on a primary expression.
    fn parse_postfix_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, WgslLanguage> {
        let mut left = self.parse_primary_expression(state);

        loop {
            match state.peek_kind() {
                Some(WgslTokenType::Dot) => {
                    let cp = state.checkpoint_before(left);
                    state.bump();
                    if state.at(WgslTokenType::Identifier) {
                        state.bump();
                    }
                    else {
                        state.record_expected_name("member name");
                    }
                    left = state.finish_at(cp, WgslElementType::ExprMember);
                }
                Some(WgslTokenType::LeftBracket) => {
                    let cp = state.checkpoint_before(left);
                    state.bump();
                    self.parse_expression(state);
                    if !state.eat(WgslTokenType::RightBracket) {
                        state.record_expected("RightBracket");
                    }
                    left = state.finish_at(cp, WgslElementType::ExprIndex);
                }
                Some(WgslTokenType::LeftParen) => {
                    let cp = state.checkpoint_before(left);
                    self.parse_call_arguments(state);
                    left = state.finish_at(cp, WgslElementType::ExprCall);
                }
                _ => break,
            }
        }

        left
    }

    /// Parses a parenthesized argument list.
    fn parse_call_arguments<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if !state.eat(WgslTokenType::LeftParen) {
            state.record_expected("LeftParen");
            return;
        }
        while state.not_at_end() && !state.at(WgslTokenType::RightParen) {
            let before = state.checkpoint().0;
            self.parse_expression(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
            if !state.eat(WgslTokenType::Comma) {
                break;
            }
        }
        if !state.eat(WgslTokenType::RightParen) {
            state.record_expected("RightParen");
        }
    }

    /// Parses a primary expression: a literal, a name, a generic constructor,
    /// or a parenthesized expression.
    fn parse_primary_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, WgslLanguage> {
        let cp = state.checkpoint();

        match state.peek_kind() {
            Some(WgslTokenType::IntegerLiteral) | Some(WgslTokenType::FloatLiteral) | Some(WgslTokenType::BoolLiteral) | Some(WgslTokenType::StringLiteral) => {
                state.bump();
                state.finish_at(cp, WgslElementType::Expr)
            }
            Some(kind) if is_name_like(kind) => {
                if is_generic_constructor_call(state) {
                    state.bump(); // name
                    state.bump(); // <
                    while state.not_at_end() && !state.at(WgslTokenType::Gt) {
                        let before = state.checkpoint().0;
                        self.parse_type(state);
                        if state.checkpoint().0 == before {
                            state.bump();
                        }
                        if !state.eat(WgslTokenType::Comma) {
                            break;
                        }
                    }
                    if !state.eat(WgslTokenType::Gt) {
                        state.record_expected("Gt");
                    }
                    state.finish_at(cp, WgslElementType::ExprTypeCtor)
                }
                else {
                    state.bump();
                    state.finish_at(cp, WgslElementType::Expr)
                }
            }
            Some(WgslTokenType::LeftParen) => {
                state.bump();
                self.parse_expression(state);
                if !state.eat(WgslTokenType::RightParen) {
                    state.record_expected("RightParen");
                }
                state.finish_at(cp, WgslElementType::ExprParen)
            }
            _ => {
                state.syntax_error("expected an expression");
                if state.not_at_end() {
                    state.bump();
                }
                state.finish_at(cp, WgslElementType::Expr)
            }
        }
    }
}

/// Returns `true` for identifiers and keyword tokens that may appear as names.
fn is_name_like(kind: WgslTokenType) -> bool {
    kind == WgslTokenType::Identifier || kind.role() == UniversalTokenRole::Keyword
}

/// Returns `true` for tokens that may start a type or a type argument.
fn is_type_token(kind: WgslTokenType) -> bool {
    matches!(kind.role(), UniversalTokenRole::Name | UniversalTokenRole::Keyword | UniversalTokenRole::Literal)
}

/// Returns `true` for tokens that terminate a type position.
fn is_type_terminator(kind: WgslTokenType) -> bool {
    matches!(kind, WgslTokenType::Comma | WgslTokenType::RightParen | WgslTokenType::RightBrace | WgslTokenType::RightBracket | WgslTokenType::Gt | WgslTokenType::Semicolon | WgslTokenType::Assign)
}

/// Detects `name<...>(` so that `vec3<f32>(...)` is parsed as a generic
/// constructor call instead of a chain of comparisons.
fn is_generic_constructor_call<'a, S: Source + ?Sized>(state: &State<'a, S>) -> bool {
    if state.peek_non_trivia_kind_at(1) != Some(WgslTokenType::Lt) {
        return false;
    }

    let mut depth = 0usize;
    let mut offset = 1;
    while let Some(kind) = state.peek_non_trivia_kind_at(offset) {
        match kind {
            WgslTokenType::Lt => depth += 1,
            WgslTokenType::Gt => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
                if depth == 0 {
                    return state.peek_non_trivia_kind_at(offset + 1) == Some(WgslTokenType::LeftParen);
                }
            }
            WgslTokenType::Comma => {}
            kind if matches!(kind.role(), UniversalTokenRole::Name | UniversalTokenRole::Keyword | UniversalTokenRole::Literal) => {}
            _ => return false,
        }
        offset += 1;
        if offset > 128 {
            return false;
        }
    }

    false
}

/// Returns the precedence and token count of the binary operator at the cursor.
fn binary_operator<'a, S: Source + ?Sized>(state: &State<'a, S>) -> Option<(u8, usize)> {
    let kind = state.peek_kind()?;
    let info = match kind {
        WgslTokenType::PipePipe => (1, 1),
        WgslTokenType::AmpersandAmpersand => (2, 1),
        WgslTokenType::Pipe => (3, 1),
        WgslTokenType::Caret => (4, 1),
        WgslTokenType::Ampersand => (5, 1),
        WgslTokenType::EqEq | WgslTokenType::BangEq | WgslTokenType::Ne => (6, 1),
        WgslTokenType::Lt | WgslTokenType::Le | WgslTokenType::Ge => (7, 1),
        WgslTokenType::Gt => {
            if state.peek_non_trivia_kind_at(1) == Some(WgslTokenType::Gt) {
                // `>>=` belongs to the assignment parser, not to a shift expression.
                if state.peek_non_trivia_kind_at(2) == Some(WgslTokenType::Assign) {
                    return None;
                }
                (8, 2)
            }
            else {
                (7, 1)
            }
        }
        WgslTokenType::LeftShift | WgslTokenType::LtLt | WgslTokenType::RightShift | WgslTokenType::GtGt => (8, 1),
        WgslTokenType::Plus | WgslTokenType::Minus => (9, 1),
        WgslTokenType::Star | WgslTokenType::Slash | WgslTokenType::Percent => (10, 1),
        _ => return None,
    };
    Some(info)
}

/// Returns the number of tokens that make up the assignment operator at the cursor.
fn assignment_operator_tokens<'a, S: Source + ?Sized>(state: &State<'a, S>) -> Option<usize> {
    match state.peek_kind()? {
        WgslTokenType::Assign
        | WgslTokenType::PlusAssign
        | WgslTokenType::MinusAssign
        | WgslTokenType::StarAssign
        | WgslTokenType::SlashAssign
        | WgslTokenType::PercentAssign
        | WgslTokenType::AmpersandAssign
        | WgslTokenType::PipeAssign
        | WgslTokenType::CaretAssign
        | WgslTokenType::LeftShiftAssign
        | WgslTokenType::RightShiftAssign => Some(1),
        _ => None,
    }
}
