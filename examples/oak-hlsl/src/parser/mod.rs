/// Element type definitions for the HLSL parser.
pub mod element_type;

use crate::{language::HlslLanguage, lexer::token_type::HlslTokenType};
use element_type::HlslElementType;
use oak_core::{
    GreenNode, OakError, Parser, ParserState, TokenType, UniversalTokenRole,
    parser::parse_with_lexer,
    source::{Source, TextEdit},
};

/// A parser state for the HLSL language.
pub(crate) type State<'a, S> = ParserState<'a, HlslLanguage, S>;

/// Attributes understood by the HLSL frontend. Unknown names are still parsed
/// structurally, but reported through a diagnostic.
const KNOWN_ATTRIBUTES: &[&str] =
    &["allow_uav_condition", "branch", "clipplane", "domain", "earlydepthstencil", "fastopt", "flatten", "instance", "loop", "maxvertexcount", "numthreads", "outputcontrolpoints", "outputtopology", "partitioning", "patchconstantfunc", "shader", "unroll"];

/// A parser for the HLSL language.
pub struct HlslParser<'config> {
    /// Language configuration.
    pub(crate) config: &'config HlslLanguage,
}

impl<'config> HlslParser<'config> {
    /// Creates a new HLSL parser with the given language configuration.
    pub fn new(config: &'config HlslLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Parser<HlslLanguage> for HlslParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl oak_core::ParseCache<HlslLanguage>) -> oak_core::ParseOutput<'a, HlslLanguage> {
        let lexer = crate::lexer::HlslLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let cp = state.checkpoint();
            loop {
                state.skip_trivia();
                if !state.not_at_end() || state.at(HlslTokenType::Eof) {
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
            Ok(state.finish_at(cp, HlslElementType::Root))
        })
    }
}

impl<'config> HlslParser<'config> {
    // --- Module level -------------------------------------------------------

    /// Parses a single module-level item, including its leading attributes.
    fn parse_item<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_attributes(state);

        match state.peek_kind() {
            Some(HlslTokenType::Struct) => self.parse_struct(state, cp),
            Some(HlslTokenType::Cbuffer) | Some(HlslTokenType::Tbuffer) => self.skip_block_declaration(state),
            Some(HlslTokenType::Typedef) => self.parse_typedef(state, cp),
            Some(HlslTokenType::Using) => self.parse_using_alias(state, cp),
            Some(HlslTokenType::Include)
            | Some(HlslTokenType::Define)
            | Some(HlslTokenType::Undef)
            | Some(HlslTokenType::If_)
            | Some(HlslTokenType::Ifdef)
            | Some(HlslTokenType::Ifndef)
            | Some(HlslTokenType::Else_)
            | Some(HlslTokenType::Elif)
            | Some(HlslTokenType::Endif)
            | Some(HlslTokenType::Line)
            | Some(HlslTokenType::Pragma)
            | Some(HlslTokenType::Semicolon) => {
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

    /// Parses a `cbuffer` or `tbuffer` block. These buffer blocks are outside
    /// the current frontend surface, so only their tokens are consumed.
    fn skip_block_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while state.not_at_end() && !state.at(HlslTokenType::LeftBrace) && !state.at(HlslTokenType::Semicolon) && !state.at(HlslTokenType::Eof) {
            state.bump();
        }
        if state.at(HlslTokenType::LeftBrace) {
            self.skip_balanced_braces(state);
            state.eat(HlslTokenType::Semicolon);
        }
        else if state.at(HlslTokenType::Semicolon) {
            state.bump();
        }
    }

    /// Consumes a balanced `{ ... }` group.
    fn skip_balanced_braces<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let mut depth = 0usize;
        while state.not_at_end() {
            match state.peek_kind() {
                Some(HlslTokenType::LeftBrace) => {
                    depth += 1;
                    state.bump();
                }
                Some(HlslTokenType::RightBrace) => {
                    state.bump();
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return;
                    }
                }
                Some(HlslTokenType::Eof) | None => return,
                Some(_) => state.bump(),
            }
        }
    }

    /// Parses a legacy `typedef Type Name;` alias.
    fn parse_typedef<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump(); // typedef
        self.parse_type(state);
        self.parse_name(state);
        self.finish_terminated(state);
        state.finish_at(cp, HlslElementType::TypeAliasDeclaration);
    }

    /// Parses a `using Name = Type;` alias.
    fn parse_using_alias<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump(); // using
        self.parse_name(state);
        if !state.eat(HlslTokenType::Assign) {
            state.record_expected("Assign");
        }
        self.parse_type(state);
        self.finish_terminated(state);
        state.finish_at(cp, HlslElementType::TypeAliasDeclaration);
    }

    /// Parses a declaration that starts with optional qualifiers and a type.
    fn parse_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        let has_const = self.parse_qualifiers(state);
        self.parse_type(state);

        if state.at(HlslTokenType::Identifier) {
            state.bump();
        }
        else if !state.at(HlslTokenType::LeftParen) {
            state.record_expected_name("identifier");
        }

        if state.at(HlslTokenType::LeftParen) {
            self.parse_param_list(state);
            if state.eat(HlslTokenType::Colon) {
                self.parse_semantic(state);
            }
            if state.at(HlslTokenType::LeftBrace) {
                self.parse_block(state);
            }
            else if state.at(HlslTokenType::Semicolon) {
                state.bump();
            }
            state.finish_at(cp, HlslElementType::FunctionDeclaration);
        }
        else {
            if state.eat(HlslTokenType::Colon) {
                self.parse_colon_tail(state);
            }
            if state.at(HlslTokenType::LeftBracket) {
                self.skip_array_extent(state);
            }
            if state.eat(HlslTokenType::Assign) {
                self.parse_expression(state);
            }
            self.finish_terminated(state);
            state.finish_at(cp, if has_const { HlslElementType::GlobalConstDeclaration } else { HlslElementType::VariableDeclaration });
        }
    }

    /// Parses the tail of a `:` in a declaration, which is either a resource
    /// register or a semantic name.
    fn parse_colon_tail<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if state.at(HlslTokenType::Register) {
            self.parse_register(state);
        }
        else {
            self.parse_semantic(state);
        }
    }

    /// Parses a `register(b0)` clause.
    fn parse_register<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // register
        if !state.eat(HlslTokenType::LeftParen) {
            state.record_expected("LeftParen");
            state.finish_at(cp, HlslElementType::RegisterClause);
            return;
        }
        while state.not_at_end() && !state.at(HlslTokenType::RightParen) {
            if state.at(HlslTokenType::Comma) {
                state.bump();
            }
            else if state.at(HlslTokenType::Eof) || state.at(HlslTokenType::Semicolon) || state.at(HlslTokenType::LeftBrace) {
                break;
            }
            else {
                state.bump();
            }
        }
        if !state.eat(HlslTokenType::RightParen) {
            state.record_expected("RightParen");
        }
        state.finish_at(cp, HlslElementType::RegisterClause);
    }

    /// Parses a semantic name such as `SV_Target` or `TEXCOORD0`.
    fn parse_semantic<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if let Some(kind) = state.peek_kind() {
            if is_name_like(kind) {
                state.bump();
            }
            else {
                state.record_expected_name("semantic");
            }
        }
        else {
            state.record_unexpected_eof();
        }
    }

    /// Parses a function parameter list.
    fn parse_param_list<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        if !state.eat(HlslTokenType::LeftParen) {
            state.record_expected("LeftParen");
            state.finish_at(cp, HlslElementType::ParameterList);
            return;
        }
        while state.not_at_end() && !state.at(HlslTokenType::RightParen) {
            let before = state.checkpoint().0;
            self.parse_param(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
            if !state.eat(HlslTokenType::Comma) {
                break;
            }
        }
        if !state.eat(HlslTokenType::RightParen) {
            state.record_expected("RightParen");
        }
        state.finish_at(cp, HlslElementType::ParameterList);
    }

    /// Parses a function parameter.
    fn parse_param<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_attributes(state);
        self.parse_qualifiers(state);
        self.parse_type(state);
        if state.at(HlslTokenType::Identifier) {
            state.bump();
        }
        if state.eat(HlslTokenType::Colon) {
            self.parse_semantic(state);
        }
        if state.at(HlslTokenType::LeftBracket) {
            self.skip_array_extent(state);
        }
        state.finish_at(cp, HlslElementType::Parameter);
    }

    /// Parses a struct declaration.
    fn parse_struct<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, cp: (usize, usize)) {
        state.bump(); // struct
        self.parse_name(state);

        if !state.eat(HlslTokenType::LeftBrace) {
            state.record_expected("LeftBrace");
        }
        while state.not_at_end() && !state.at(HlslTokenType::RightBrace) {
            let before = state.checkpoint().0;
            self.parse_struct_member(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
        }
        if !state.eat(HlslTokenType::RightBrace) {
            state.record_expected("RightBrace");
        }
        if !state.eat(HlslTokenType::Semicolon) {
            state.record_expected("Semicolon");
        }
        state.finish_at(cp, HlslElementType::StructDeclaration);
    }

    /// Parses a struct member.
    fn parse_struct_member<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_attributes(state);
        self.parse_qualifiers(state);
        self.parse_type(state);
        if state.at(HlslTokenType::Identifier) {
            state.bump();
        }
        else {
            state.record_expected_name("identifier");
        }
        if state.eat(HlslTokenType::Colon) {
            self.parse_semantic(state);
        }
        if state.at(HlslTokenType::LeftBracket) {
            self.skip_array_extent(state);
        }
        self.finish_terminated(state);
        state.finish_at(cp, HlslElementType::StructMemberDeclaration);
    }

    // --- Attributes ---------------------------------------------------------

    /// Parses zero or more `[...]` attributes. Returns `true` when at least one
    /// was parsed.
    fn parse_attributes<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let mut any = false;
        while state.at(HlslTokenType::LeftBracket) {
            self.parse_attribute(state);
            any = true;
        }
        any
    }

    /// Parses one attribute such as `[numthreads(64, 1, 1)]`.
    fn parse_attribute<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // [

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

        if state.eat(HlslTokenType::LeftParen) {
            while state.not_at_end() && !state.at(HlslTokenType::RightParen) {
                let before = state.checkpoint().0;
                self.parse_expression(state);
                if state.checkpoint().0 == before {
                    state.bump();
                }
                if !state.eat(HlslTokenType::Comma) {
                    break;
                }
            }
            if !state.eat(HlslTokenType::RightParen) {
                state.record_expected("RightParen");
            }
        }

        if !state.eat(HlslTokenType::RightBracket) {
            state.record_expected("RightBracket");
        }

        if let Some((text, offset)) = name {
            if !KNOWN_ATTRIBUTES.contains(&text.as_str()) {
                let error = OakError::syntax_error(format!("unknown HLSL attribute `{text}`"), offset, state.source.source_id());
                state.errors.push(error);
            }
        }

        state.finish_at(cp, HlslElementType::Attribute);
    }

    // --- Types --------------------------------------------------------------

    /// Parses a type reference such as `float4`, `RWStructuredBuffer<float4>`,
    /// or a user-defined struct name.
    fn parse_type<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, HlslLanguage> {
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

        if state.at(HlslTokenType::Less) {
            state.bump();
            while state.not_at_end() && !state.at(HlslTokenType::Greater) {
                let before = state.checkpoint().0;
                self.parse_type(state);
                if state.checkpoint().0 == before {
                    state.bump();
                }
                if !state.eat(HlslTokenType::Comma) {
                    break;
                }
            }
            if !state.eat(HlslTokenType::Greater) {
                state.record_expected("Greater");
            }
        }

        if state.at(HlslTokenType::LeftBracket) {
            self.skip_array_extent(state);
        }

        state.finish_at(cp, HlslElementType::TypeReference)
    }

    /// Consumes a `[ ... ]` array extent. Extents are not modelled by the AST.
    fn skip_array_extent<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let mut depth = 0usize;
        while state.not_at_end() {
            match state.peek_kind() {
                Some(HlslTokenType::LeftBracket) => {
                    depth += 1;
                    state.bump();
                }
                Some(HlslTokenType::RightBracket) => {
                    state.bump();
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return;
                    }
                }
                Some(HlslTokenType::Eof) | None => return,
                Some(HlslTokenType::Semicolon) | Some(HlslTokenType::RightBrace) => return,
                Some(_) => state.bump(),
            }
        }
    }

    /// Parses an identifier used as a declared name.
    fn parse_name<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if state.at(HlslTokenType::Identifier) {
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
            if kind == HlslTokenType::Const {
                has_const = true;
            }
            state.bump();
        }
        has_const
    }

    // --- Statements ---------------------------------------------------------

    /// Parses a compound statement.
    fn parse_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, HlslLanguage> {
        let cp = state.checkpoint();
        if !state.eat(HlslTokenType::LeftBrace) {
            state.record_expected("LeftBrace");
        }
        while state.not_at_end() && !state.at(HlslTokenType::RightBrace) {
            let before = state.checkpoint().0;
            self.parse_statement(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
        }
        if !state.eat(HlslTokenType::RightBrace) {
            state.record_expected("RightBrace");
        }
        state.finish_at(cp, HlslElementType::Block)
    }

    /// Parses a single statement.
    fn parse_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        match state.peek_kind() {
            Some(HlslTokenType::LeftBrace) => {
                self.parse_block(state);
            }
            Some(HlslTokenType::LeftBracket) => {
                self.parse_attributes(state);
                self.parse_statement(state);
            }
            Some(HlslTokenType::If) => self.parse_if_statement(state),
            Some(HlslTokenType::For) => self.parse_for_statement(state),
            Some(HlslTokenType::While) => self.parse_while_statement(state),
            Some(HlslTokenType::Do) => self.parse_do_while_statement(state),
            Some(HlslTokenType::Break) => {
                let cp = state.checkpoint();
                state.bump();
                self.finish_terminated(state);
                state.finish_at(cp, HlslElementType::BreakStatement);
            }
            Some(HlslTokenType::Continue) => {
                let cp = state.checkpoint();
                state.bump();
                self.finish_terminated(state);
                state.finish_at(cp, HlslElementType::ContinueStatement);
            }
            Some(HlslTokenType::Discard) => {
                let cp = state.checkpoint();
                state.bump();
                self.finish_terminated(state);
                state.finish_at(cp, HlslElementType::DiscardStatement);
            }
            Some(HlslTokenType::Return) => {
                let cp = state.checkpoint();
                state.bump();
                if state.not_at_end() && !state.at(HlslTokenType::Semicolon) && !state.at(HlslTokenType::RightBrace) {
                    self.parse_expression(state);
                }
                self.finish_terminated(state);
                state.finish_at(cp, HlslElementType::ReturnStatement);
            }
            Some(kind) if starts_declaration(state, kind) => self.parse_var_statement(state),
            _ => self.parse_expression_statement(state, true),
        }
    }

    /// Consumes the terminating semicolon of a statement, reporting it when missing.
    fn finish_terminated<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if !state.eat(HlslTokenType::Semicolon) {
            state.record_expected("Semicolon");
        }
    }

    /// Parses a local variable declaration.
    fn parse_var_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.parse_qualifiers(state);
        self.parse_type(state);
        if state.at(HlslTokenType::Identifier) {
            state.bump();
        }
        else {
            state.record_expected_name("identifier");
        }
        if state.eat(HlslTokenType::Colon) {
            self.parse_semantic(state);
        }
        if state.at(HlslTokenType::LeftBracket) {
            self.skip_array_extent(state);
        }
        if state.eat(HlslTokenType::Assign) {
            self.parse_expression(state);
        }
        self.finish_terminated(state);
        state.finish_at(cp, HlslElementType::VarStatement);
    }

    /// Parses an `if` statement, including `else if` chains.
    fn parse_if_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // if
        self.parse_expression(state);
        self.parse_block(state);

        if state.eat(HlslTokenType::Else) {
            if state.at(HlslTokenType::If) {
                self.parse_if_statement(state);
            }
            else {
                self.parse_block(state);
            }
        }

        state.finish_at(cp, HlslElementType::IfStatement);
    }

    /// Parses a `for` statement.
    fn parse_for_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // for

        if !state.eat(HlslTokenType::LeftParen) {
            state.record_expected("LeftParen");
        }

        // Initializer.
        let init_cp = state.checkpoint();
        if !state.at(HlslTokenType::Semicolon) && state.not_at_end() {
            if state.peek_kind().is_some_and(|kind| starts_declaration(state, kind)) {
                self.parse_var_statement(state);
            }
            else {
                self.parse_expression_statement(state, false);
            }
        }
        if state.at(HlslTokenType::Semicolon) {
            state.bump();
        }
        state.finish_at(init_cp, HlslElementType::ForInit);

        // Condition.
        let cond_cp = state.checkpoint();
        if state.not_at_end() && !state.at(HlslTokenType::Semicolon) && !state.at(HlslTokenType::RightParen) {
            self.parse_expression(state);
        }
        if state.at(HlslTokenType::Semicolon) {
            state.bump();
        }
        state.finish_at(cond_cp, HlslElementType::ForCond);

        // Update.
        let update_cp = state.checkpoint();
        if state.not_at_end() && !state.at(HlslTokenType::RightParen) {
            self.parse_expression_statement(state, false);
        }
        state.finish_at(update_cp, HlslElementType::ForUpdate);

        if !state.eat(HlslTokenType::RightParen) {
            state.record_expected("RightParen");
        }

        self.parse_block(state);
        state.finish_at(cp, HlslElementType::ForStatement);
    }

    /// Parses a `while` statement.
    fn parse_while_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // while
        self.parse_expression(state);
        self.parse_block(state);
        state.finish_at(cp, HlslElementType::WhileStatement);
    }

    /// Parses a `do ... while` statement.
    fn parse_do_while_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.bump(); // do
        self.parse_block(state);
        if !state.eat(HlslTokenType::While) {
            state.record_expected("While");
        }
        self.parse_expression(state);
        self.finish_terminated(state);
        state.finish_at(cp, HlslElementType::DoWhileStatement);
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
            state.finish_at(wrap_cp, HlslElementType::AssignmentStatement);
        }
        else {
            if terminated {
                self.finish_terminated(state);
            }
            state.finish_at(cp, HlslElementType::ExpressionStatement);
        }
    }

    // --- Expressions --------------------------------------------------------

    /// Parses an expression with the lowest binding power.
    fn parse_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, HlslLanguage> {
        self.parse_binary_expression(state, 0)
    }

    /// Parses a binary expression using precedence climbing.
    fn parse_binary_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, min_precedence: u8) -> &'a GreenNode<'a, HlslLanguage> {
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
            left = state.finish_at(cp, HlslElementType::ExprBinary);
        }

        left
    }

    /// Parses a prefix or postfix expression.
    fn parse_unary_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, HlslLanguage> {
        if matches!(state.peek_kind(), Some(HlslTokenType::Minus) | Some(HlslTokenType::LogicalNot) | Some(HlslTokenType::BitwiseNot) | Some(HlslTokenType::Increment) | Some(HlslTokenType::Decrement)) {
            let cp = state.checkpoint();
            state.bump();
            self.parse_unary_expression(state);
            return state.finish_at(cp, HlslElementType::ExprUnary);
        }

        self.parse_postfix_expression(state)
    }

    /// Parses member access, indexing, calls, and postfix increments on a
    /// primary expression.
    fn parse_postfix_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, HlslLanguage> {
        let mut left = self.parse_primary_expression(state);

        loop {
            match state.peek_kind() {
                Some(HlslTokenType::Dot) => {
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
                    left = state.finish_at(cp, HlslElementType::ExprMember);
                }
                Some(HlslTokenType::LeftBracket) => {
                    let cp = state.checkpoint_before(left);
                    state.bump();
                    self.parse_expression(state);
                    if !state.eat(HlslTokenType::RightBracket) {
                        state.record_expected("RightBracket");
                    }
                    left = state.finish_at(cp, HlslElementType::ExprIndex);
                }
                Some(HlslTokenType::LeftParen) => {
                    let cp = state.checkpoint_before(left);
                    self.parse_call_arguments(state);
                    left = state.finish_at(cp, HlslElementType::ExprCall);
                }
                Some(HlslTokenType::Increment) | Some(HlslTokenType::Decrement) => {
                    let cp = state.checkpoint_before(left);
                    state.bump();
                    left = state.finish_at(cp, HlslElementType::ExprUnary);
                }
                _ => break,
            }
        }

        left
    }

    /// Parses a parenthesized argument list.
    fn parse_call_arguments<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if !state.eat(HlslTokenType::LeftParen) {
            state.record_expected("LeftParen");
            return;
        }
        while state.not_at_end() && !state.at(HlslTokenType::RightParen) {
            let before = state.checkpoint().0;
            self.parse_expression(state);
            if state.checkpoint().0 == before {
                state.bump();
            }
            if !state.eat(HlslTokenType::Comma) {
                break;
            }
        }
        if !state.eat(HlslTokenType::RightParen) {
            state.record_expected("RightParen");
        }
    }

    /// Parses a primary expression: a literal, a name, or a parenthesized
    /// expression.
    fn parse_primary_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, HlslLanguage> {
        let cp = state.checkpoint();

        match state.peek_kind() {
            Some(HlslTokenType::NumberLiteral) | Some(HlslTokenType::StringLiteral) | Some(HlslTokenType::BooleanLiteral) => {
                state.bump();
                state.finish_at(cp, HlslElementType::Expr)
            }
            Some(kind) if is_name_like(kind) => {
                state.bump();
                state.finish_at(cp, HlslElementType::Expr)
            }
            Some(HlslTokenType::LeftParen) => {
                state.bump();
                self.parse_expression(state);
                if !state.eat(HlslTokenType::RightParen) {
                    state.record_expected("RightParen");
                }
                state.finish_at(cp, HlslElementType::ExprParen)
            }
            _ => {
                state.syntax_error("expected an expression");
                if state.not_at_end() && !state.at(HlslTokenType::Eof) {
                    state.bump();
                }
                state.finish_at(cp, HlslElementType::Expr)
            }
        }
    }
}

/// Returns `true` for identifiers and keyword tokens that may appear as names.
fn is_name_like(kind: HlslTokenType) -> bool {
    kind == HlslTokenType::Identifier || kind.role() == UniversalTokenRole::Keyword
}

/// Returns `true` for tokens that open a declaration rather than an expression.
fn starts_declaration<'a, S: Source + ?Sized>(state: &State<'a, S>, kind: HlslTokenType) -> bool {
    if is_qualifier(kind) {
        return true;
    }
    if is_type_keyword(kind) {
        // A type keyword followed by `(` is a constructor call such as
        // `float4(1.0, 0.0, 0.0, 1.0)`.
        return state.peek_non_trivia_kind_at(1) != Some(HlslTokenType::LeftParen);
    }
    kind == HlslTokenType::Identifier && state.peek_non_trivia_kind_at(1) == Some(HlslTokenType::Identifier)
}

/// Returns `true` for keyword tokens that qualify a declaration.
fn is_qualifier(kind: HlslTokenType) -> bool {
    matches!(
        kind,
        HlslTokenType::Static
            | HlslTokenType::Const
            | HlslTokenType::Uniform
            | HlslTokenType::Varying
            | HlslTokenType::In
            | HlslTokenType::Out
            | HlslTokenType::Inout
            | HlslTokenType::Inline
            | HlslTokenType::Extern
            | HlslTokenType::Shared
            | HlslTokenType::Groupshared
            | HlslTokenType::Volatile
            | HlslTokenType::Precise
            | HlslTokenType::Nointerpolation
            | HlslTokenType::Linear
            | HlslTokenType::Centroid
            | HlslTokenType::Sample
            | HlslTokenType::Noperspective
            | HlslTokenType::Target
    )
}

/// Returns `true` for builtin type keywords.
fn is_type_keyword(kind: HlslTokenType) -> bool {
    matches!(
        kind,
        HlslTokenType::Bool
            | HlslTokenType::Int
            | HlslTokenType::Uint
            | HlslTokenType::Half
            | HlslTokenType::Float
            | HlslTokenType::Double
            | HlslTokenType::Min16float
            | HlslTokenType::Min10float
            | HlslTokenType::Min16int
            | HlslTokenType::Min12int
            | HlslTokenType::Min16uint
            | HlslTokenType::Bool2
            | HlslTokenType::Bool3
            | HlslTokenType::Bool4
            | HlslTokenType::Int2
            | HlslTokenType::Int3
            | HlslTokenType::Int4
            | HlslTokenType::Uint2
            | HlslTokenType::Uint3
            | HlslTokenType::Uint4
            | HlslTokenType::Half2
            | HlslTokenType::Half3
            | HlslTokenType::Half4
            | HlslTokenType::Float2
            | HlslTokenType::Float3
            | HlslTokenType::Float4
            | HlslTokenType::Double2
            | HlslTokenType::Double3
            | HlslTokenType::Double4
            | HlslTokenType::Float2x2
            | HlslTokenType::Float2x3
            | HlslTokenType::Float2x4
            | HlslTokenType::Float3x2
            | HlslTokenType::Float3x3
            | HlslTokenType::Float3x4
            | HlslTokenType::Float4x2
            | HlslTokenType::Float4x3
            | HlslTokenType::Float4x4
            | HlslTokenType::Double2x2
            | HlslTokenType::Double2x3
            | HlslTokenType::Double2x4
            | HlslTokenType::Double3x2
            | HlslTokenType::Double3x3
            | HlslTokenType::Double3x4
            | HlslTokenType::Double4x2
            | HlslTokenType::Double4x3
            | HlslTokenType::Double4x4
            | HlslTokenType::Texture1D
            | HlslTokenType::Texture2D
            | HlslTokenType::Texture3D
            | HlslTokenType::TextureCube
            | HlslTokenType::Texture1DArray
            | HlslTokenType::Texture2DArray
            | HlslTokenType::TextureCubeArray
            | HlslTokenType::Texture2DMS
            | HlslTokenType::Texture2DMSArray
            | HlslTokenType::Sampler
            | HlslTokenType::SamplerState
            | HlslTokenType::SamplerComparisonState
            | HlslTokenType::Buffer
            | HlslTokenType::StructuredBuffer
            | HlslTokenType::ByteAddressBuffer
            | HlslTokenType::RWBuffer
            | HlslTokenType::RWStructuredBuffer
            | HlslTokenType::RWByteAddressBuffer
            | HlslTokenType::AppendStructuredBuffer
            | HlslTokenType::ConsumeStructuredBuffer
    )
}

/// Returns `true` for tokens that may start a type or a type argument.
fn is_type_token(kind: HlslTokenType) -> bool {
    kind == HlslTokenType::Identifier || is_type_keyword(kind)
}

/// Returns `true` for tokens that terminate a type position.
fn is_type_terminator(kind: HlslTokenType) -> bool {
    matches!(kind, HlslTokenType::Comma | HlslTokenType::RightParen | HlslTokenType::RightBrace | HlslTokenType::RightBracket | HlslTokenType::Greater | HlslTokenType::Semicolon | HlslTokenType::Assign | HlslTokenType::Colon)
}

/// Returns the precedence and token count of the binary operator at the cursor.
fn binary_operator<'a, S: Source + ?Sized>(state: &State<'a, S>) -> Option<(u8, usize)> {
    let kind = state.peek_kind()?;
    let info = match kind {
        HlslTokenType::LogicalOr => (1, 1),
        HlslTokenType::LogicalAnd => (2, 1),
        HlslTokenType::BitwiseOr => (3, 1),
        HlslTokenType::BitwiseXor => (4, 1),
        HlslTokenType::BitwiseAnd => (5, 1),
        HlslTokenType::Equal | HlslTokenType::NotEqual => (6, 1),
        HlslTokenType::Less | HlslTokenType::LessEqual | HlslTokenType::Greater | HlslTokenType::GreaterEqual => (7, 1),
        HlslTokenType::LeftShift | HlslTokenType::RightShift => (8, 1),
        HlslTokenType::Plus | HlslTokenType::Minus => (9, 1),
        HlslTokenType::Multiply | HlslTokenType::Divide | HlslTokenType::Modulo => (10, 1),
        _ => return None,
    };
    Some(info)
}

/// Returns the number of tokens that make up the assignment operator at the cursor.
fn assignment_operator_tokens<'a, S: Source + ?Sized>(state: &State<'a, S>) -> Option<usize> {
    match state.peek_kind()? {
        HlslTokenType::Assign
        | HlslTokenType::PlusAssign
        | HlslTokenType::MinusAssign
        | HlslTokenType::MultiplyAssign
        | HlslTokenType::DivideAssign
        | HlslTokenType::ModuloAssign
        | HlslTokenType::BitwiseAndAssign
        | HlslTokenType::BitwiseOrAssign
        | HlslTokenType::BitwiseXorAssign
        | HlslTokenType::LeftShiftAssign
        | HlslTokenType::RightShiftAssign => Some(1),
        _ => None,
    }
}
