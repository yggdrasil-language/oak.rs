use crate::{
    lexer::token_type::TypeScriptTokenType,
    parser::{State, TypeScriptParser},
};
use oak_core::{OakError, parser::pratt::PrattParser, source::Source};

impl<'config> TypeScriptParser<'config> {
    pub(crate) fn parse_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        self.skip_trivia(state);
        let cp = state.checkpoint();

        // Handle decorators
        let mut has_prefix = false;
        while self.at(state, At) {
            self.parse_decorator(state)?;
            has_prefix = true;
        }

        // Handle declare
        if self.eat(state, Declare) {
            has_prefix = true;
        }

        if has_prefix {
            match state.peek_kind() {
                Some(Var) | Some(Let) | Some(Const) => {
                    self.parse_variable_declaration_content(state)?;
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::VariableDeclaration);
                }
                Some(Function) => {
                    self.parse_function_declaration_content(state)?;
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::FunctionDeclaration);
                }
                Some(Class) | Some(Abstract) => {
                    self.parse_class_declaration_content(state)?;
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ClassDeclaration);
                }
                Some(Interface) => {
                    self.parse_interface_declaration_content(state)?;
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::InterfaceDeclaration);
                }
                Some(Enum) => {
                    self.parse_enum_declaration_content(state)?;
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::EnumDeclaration);
                }
                Some(Type) if self.is_type_alias_head(state) => {
                    self.parse_type_alias_declaration_content(state)?;
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::TypeAliasDeclaration);
                }
                Some(Namespace) | Some(Module) => {
                    self.parse_namespace_declaration_content(state)?;
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::NamespaceDeclaration);
                }
                _ => {
                    PrattParser::parse(state, 0, self);
                    self.skip_trivia(state);
                    state.eat(Semicolon);
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ExpressionStatement);
                }
            };
        }
        else {
            match state.peek_kind() {
                Some(Var) | Some(Let) | Some(Const) => {
                    self.parse_variable_declaration(state)?;
                }
                Some(Function) | Some(Async) => {
                    self.parse_function_declaration(state)?;
                }
                Some(Class) | Some(Abstract) => {
                    self.parse_class_declaration(state)?;
                }
                Some(Interface) => {
                    self.parse_interface_declaration(state)?;
                }
                Some(Enum) => {
                    self.parse_enum_declaration(state)?;
                }
                Some(Type) if self.is_type_alias_head(state) => {
                    self.parse_type_alias_declaration(state)?;
                }
                Some(Namespace) | Some(Module) => {
                    self.parse_namespace_declaration(state)?;
                }
                Some(Import) => {
                    if state.peek_non_trivia_kind_at(1) == Some(Dot) {
                        // `import.meta` at the statement head is an expression, not a declaration.
                        PrattParser::parse(state, 0, self);
                        self.skip_trivia(state);
                        state.eat(Semicolon);
                        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ExpressionStatement);
                    }
                    else {
                        self.parse_import_declaration(state)?;
                    }
                }
                Some(Export) => {
                    self.parse_export_declaration(state)?;
                }
                Some(If) => {
                    self.parse_if_statement(state)?;
                }
                Some(For) => {
                    self.parse_for_statement(state)?;
                }
                Some(While) => {
                    self.parse_while_statement(state)?;
                }
                Some(Do) => {
                    self.parse_do_while_statement(state)?;
                }
                Some(Switch) => {
                    self.parse_switch_statement(state)?;
                }
                Some(Try) => {
                    self.parse_try_statement(state)?;
                }
                Some(Break) => {
                    self.parse_break_statement(state)?;
                }
                Some(Continue) => {
                    self.parse_continue_statement(state)?;
                }
                Some(Throw) => {
                    self.parse_throw_statement(state)?;
                }
                Some(Return) => {
                    self.parse_return_statement(state)?;
                }
                Some(LeftBrace) => {
                    self.parse_block(state)?;
                }
                _ => {
                    PrattParser::parse(state, 0, self);
                    self.skip_trivia(state);
                    state.eat(Semicolon);
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ExpressionStatement);
                }
            };
        }
        Ok(())
    }

    pub(crate) fn parse_decorator<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        self.expect(state, At).ok();
        PrattParser::parse(state, 16, self); // High precedence for decorator expression
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::Decorator);
        Ok(())
    }

    pub(crate) fn parse_variable_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let cp = state.checkpoint();
        self.parse_variable_declaration_content(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::VariableDeclaration);
        Ok(())
    }

    pub(crate) fn parse_variable_declaration_content<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use TypeScriptTokenType::*;
        state.bump(); // var, let, or const
        self.skip_trivia(state);
        while self.is_binding_identifier(state) || self.at(state, LeftBrace) || self.at(state, LeftBracket) {
            if self.at(state, LeftBrace) || self.at(state, LeftBracket) {
                self.parse_binding_pattern(state);
            }
            else {
                self.parse_binding_identifier(state);
            }
            if self.eat(state, Colon) {
                self.parse_type_annotation(state)?;
            }
            if self.eat(state, Equal) {
                PrattParser::parse(state, 0, self);
            }
            if !self.eat(state, Comma) {
                break;
            }
            self.skip_trivia(state);
        }
        self.eat(state, Semicolon);
        Ok(())
    }

    pub(crate) fn parse_binding_pattern<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use TypeScriptTokenType::*;
        let checkpoint = state.checkpoint();
        let (open, close) = if state.at(LeftBrace.into()) { (LeftBrace, RightBrace) } else { (LeftBracket, RightBracket) };
        state.bump();
        let mut depth = 1usize;
        while state.not_at_end() && depth > 0 {
            if state.at(open.into()) {
                depth += 1;
            }
            else if state.at(close.into()) {
                depth -= 1;
            }
            state.bump();
        }
        state.finish_at(checkpoint, crate::parser::element_type::TypeScriptElementType::BindingPattern);
    }

    pub(crate) fn parse_function_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let cp = state.checkpoint();
        self.parse_function_declaration_content(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::FunctionDeclaration);
        Ok(())
    }

    pub(crate) fn parse_function_declaration_content<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        if self.eat(state, Async) {
            self.skip_trivia(state);
        }
        self.expect(state, Function).ok();
        self.skip_trivia(state);
        // `function*` / `async function*`: the `*` sits inside the declaration
        // node so the generator marker is never dropped from the tree.
        self.eat(state, Star);
        if self.at(state, IdentifierName) {
            self.expect(state, IdentifierName).ok();
        }
        else if self.peek_kind(state).is_some_and(|kind| kind.is_contextual_keyword()) {
            // `function set (options)` / `async function get(code)`: contextual
            // keywords are legal function names, but the lexer emits dedicated
            // tokens for them instead of `IdentifierName`. Wrap the token in an
            // `IdentifierName` node (as keyword object member names do) so the
            // builder's name lookup finds it.
            let name_cp = state.checkpoint();
            state.bump();
            state.finish_at(name_cp, crate::parser::element_type::TypeScriptElementType::IdentifierName);
        }
        self.parse_type_parameters(state)?;
        self.parse_parameters(state)?;
        if self.eat(state, Colon) {
            self.parse_return_type_annotation(state)?;
        }
        if self.at(state, LeftBrace) {
            self.parse_block(state)?
        }
        else {
            self.eat(state, Semicolon);
        }
        Ok(())
    }

    pub(crate) fn parse_interface_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let cp = state.checkpoint();
        self.parse_interface_declaration_content(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::InterfaceDeclaration);
        Ok(())
    }

    pub(crate) fn parse_interface_declaration_content<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        state.bump(); // interface
        self.expect(state, IdentifierName).ok();
        self.parse_type_parameters(state)?;
        if self.eat(state, Extends) {
            loop {
                self.parse_type_reference(state)?;
                if !self.eat(state, Comma) {
                    break;
                }
            }
        }
        self.parse_block(state)?;
        Ok(())
    }

    pub(crate) fn parse_enum_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let cp = state.checkpoint();
        self.parse_enum_declaration_content(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::EnumDeclaration);
        Ok(())
    }

    pub(crate) fn parse_enum_declaration_content<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        state.bump(); // enum
        self.expect(state, IdentifierName).ok();
        self.parse_block(state)?;
        Ok(())
    }

    pub(crate) fn parse_type_alias_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let cp = state.checkpoint();
        self.parse_type_alias_declaration_content(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::TypeAliasDeclaration);
        Ok(())
    }

    pub(crate) fn parse_type_alias_declaration_content<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        state.bump(); // type
        self.expect(state, IdentifierName).ok();
        self.parse_type_parameters(state)?;
        self.expect(state, Equal).ok();
        self.parse_type_annotation(state)?;
        while state.not_at_end() && !self.at(state, Semicolon) {
            self.skip_trivia(state);
            if state.not_at_end() && !self.at(state, Semicolon) { state.bump() } else { break }
        }
        self.eat(state, Semicolon);
        Ok(())
    }

    pub(crate) fn parse_namespace_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let cp = state.checkpoint();
        self.parse_namespace_declaration_content(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::NamespaceDeclaration);
        Ok(())
    }

    pub(crate) fn parse_namespace_declaration_content<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        state.bump(); // namespace or module
        self.expect(state, IdentifierName).ok();
        self.parse_block(state)?;
        Ok(())
    }

    pub(crate) fn parse_import_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};
        let cp = state.checkpoint();
        state.bump(); // import

        self.skip_trivia(state);
        // `import type …` — leave `Type` as a leaf so the builder can record it as a `Modifier`.
        if self.at(state, Type) {
            state.bump();
            self.skip_trivia(state);
        }

        // Side-effect import: `import './x'`.
        if self.at(state, StringLiteral) {
            self.parse_import_string_literal(state);
            self.eat(state, Semicolon);
            state.finish_at(cp, TypeScriptElementType::ImportDeclaration);
            return Ok(());
        }

        let clause_cp = state.checkpoint();
        if self.at(state, LeftBrace) {
            self.parse_named_imports(state);
        }
        else if self.at(state, Star) {
            self.parse_namespace_import(state);
        }
        else if self.at(state, IdentifierName) {
            self.parse_import_identifier(state);
            self.skip_trivia(state);
            // `import def, { named }` / `import def, * as ns`
            if self.eat(state, Comma) {
                self.skip_trivia(state);
                if self.at(state, LeftBrace) {
                    self.parse_named_imports(state);
                }
                else if self.at(state, Star) {
                    self.parse_namespace_import(state);
                }
            }
        }
        state.finish_at(clause_cp, TypeScriptElementType::ImportClause);

        self.skip_trivia(state);
        if self.eat(state, From) {
            self.skip_trivia(state);
            self.parse_import_string_literal(state);
        }

        self.eat(state, Semicolon);
        state.finish_at(cp, TypeScriptElementType::ImportDeclaration);
        Ok(())
    }

    fn parse_import_identifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};
        let id_cp = state.checkpoint();
        self.expect(state, IdentifierName).ok();
        state.finish_at(id_cp, TypeScriptElementType::IdentifierName);
    }

    fn parse_import_string_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};
        let str_cp = state.checkpoint();
        self.expect(state, StringLiteral).ok();
        state.finish_at(str_cp, TypeScriptElementType::StringLiteral);
        self.parse_import_attributes(state);
    }

    /// Consumes an import attribute clause (`with { … }` / legacy `assert { … }`)
    /// that follows the module string, keeping it inside the declaration node.
    fn parse_import_attributes<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};
        self.skip_trivia(state);
        let head = state.peek_kind();
        let is_assert = head == Some(IdentifierName) && state.peek_text().is_some_and(|text| text == "assert");
        if !matches!(head, Some(With)) && !is_assert {
            return;
        }
        if state.peek_non_trivia_kind_at(1) != Some(LeftBrace) {
            return;
        }
        let cp = state.checkpoint();
        state.bump(); // `with` / `assert`
        self.skip_trivia(state);
        state.bump(); // `{`
        let mut depth = 1usize;
        while state.not_at_end() && depth > 0 {
            if state.at(LeftBrace.into()) {
                depth += 1;
            }
            else if state.at(RightBrace.into()) {
                depth -= 1;
            }
            state.bump();
        }
        state.finish_at(cp, TypeScriptElementType::ImportAttributes);
    }

    /// `type` at the statement head introduces a type alias only when a name
    /// follows it; `type = x` and `type == x` are assignment expressions.
    pub(crate) fn is_type_alias_head<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        match state.peek_non_trivia_kind_at(1) {
            Some(IdentifierName) => true,
            Some(kind) => kind.is_contextual_keyword(),
            None => false,
        }
    }

    fn parse_named_imports<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};
        let named_cp = state.checkpoint();
        self.expect(state, LeftBrace).ok();
        while state.not_at_end() && !self.at(state, RightBrace) {
            self.skip_trivia(state);
            let spec_cp = state.checkpoint();
            if self.at(state, Type) {
                state.bump();
                self.skip_trivia(state);
            }
            // Specifier names may be keywords (`default`), which the lexer
            // classifies as reserved tokens instead of `IdentifierName`.
            if self.is_binding_identifier(state) {
                self.parse_binding_identifier(state);
                self.skip_trivia(state);
                if self.eat(state, As) {
                    self.skip_trivia(state);
                    self.parse_binding_identifier(state);
                }
                state.finish_at(spec_cp, TypeScriptElementType::ImportSpecifier);
            }
            if !self.eat(state, Comma) {
                break;
            }
            self.skip_trivia(state);
        }
        self.expect(state, RightBrace).ok();
        state.finish_at(named_cp, TypeScriptElementType::NamedImports);
    }

    fn parse_namespace_import<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};
        let ns_cp = state.checkpoint();
        state.bump(); // *
        self.skip_trivia(state);
        self.expect(state, As).ok();
        self.skip_trivia(state);
        self.parse_import_identifier(state);
        state.finish_at(ns_cp, TypeScriptElementType::NamespaceImport);
    }

    pub(crate) fn parse_export_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};
        let cp = state.checkpoint();
        state.bump(); // export

        self.skip_trivia(state);
        // `export type …`
        if self.at(state, Type) {
            let type_cp = state.checkpoint();
            state.bump();
            self.skip_trivia(state);
            if self.at(state, LeftBrace) {
                self.parse_named_exports(state);
                if self.eat(state, From) {
                    self.parse_import_string_literal(state);
                }
                self.eat(state, Semicolon);
            }
            else {
                state.restore(type_cp);
                self.parse_type_alias_declaration(state)?;
            }
            state.finish_at(cp, TypeScriptElementType::ExportDeclaration);
            return Ok(());
        }

        if self.eat(state, Default) {
            self.skip_trivia(state);
            if self.at(state, LeftBrace) {
                // `export default { … }` exports an object literal expression;
                // parsing it as a statement would read `{` as a block and turn
                // every property `:` into a diagnostic.
                let ecp = state.checkpoint();
                PrattParser::parse(state, 0, self);
                self.skip_trivia(state);
                self.eat(state, Semicolon);
                state.finish_at(ecp, TypeScriptElementType::ExpressionStatement);
            }
            else {
                self.parse_statement(state)?;
            }
        }
        else if self.at(state, LeftBrace) {
            self.parse_named_exports(state);
            self.skip_trivia(state);
            if self.eat(state, From) {
                self.skip_trivia(state);
                self.parse_import_string_literal(state);
            }
            self.eat(state, Semicolon);
        }
        else if self.at(state, Star) {
            // `export * from '…'` / `export * as ns from '…'`
            let star_cp = state.checkpoint();
            state.bump(); // *
            self.skip_trivia(state);
            if self.eat(state, As) {
                self.skip_trivia(state);
                self.parse_import_identifier(state);
            }
            state.finish_at(star_cp, TypeScriptElementType::NamespaceImport);
            self.skip_trivia(state);
            self.expect(state, From).ok();
            self.skip_trivia(state);
            self.parse_import_string_literal(state);
            self.eat(state, Semicolon);
        }
        else {
            self.parse_statement(state)?
        }

        state.finish_at(cp, TypeScriptElementType::ExportDeclaration);
        Ok(())
    }

    fn parse_named_exports<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};
        let named_cp = state.checkpoint();
        self.expect(state, LeftBrace).ok();
        while state.not_at_end() && !self.at(state, RightBrace) {
            self.skip_trivia(state);
            let spec_cp = state.checkpoint();
            if self.at(state, Type) {
                state.bump();
                self.skip_trivia(state);
            }
            // Specifier names may be keywords (`default`), which the lexer
            // classifies as reserved tokens instead of `IdentifierName`.
            if self.is_binding_identifier(state) {
                self.parse_binding_identifier(state);
                self.skip_trivia(state);
                if self.eat(state, As) {
                    self.skip_trivia(state);
                    self.parse_binding_identifier(state);
                }
                state.finish_at(spec_cp, TypeScriptElementType::ExportSpecifier);
            }
            if !self.eat(state, Comma) {
                break;
            }
            self.skip_trivia(state);
        }
        self.expect(state, RightBrace).ok();
        state.finish_at(named_cp, TypeScriptElementType::NamedExports);
    }

    /// Parses one expression in a parenthesized statement head and folds a
    /// top-level comma sequence into a `SequenceExpression` node. The
    /// ECMAScript `Expression` production used by `if`, `while`, `do…while`,
    /// `switch` and the `for` init/test/update slots allows sequences, so a
    /// bare comma must not terminate the slot or leak into the following
    /// punctuation.
    fn parse_expression_or_sequence<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let seq_cp = state.checkpoint();
        PrattParser::parse(state, 0, self);
        if self.at(state, Comma) {
            while self.eat(state, Comma) {
                PrattParser::parse(state, 0, self);
            }
            state.finish_at(seq_cp, crate::parser::element_type::TypeScriptElementType::SequenceExpression);
        }
    }

    pub(crate) fn parse_if_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // if
        self.expect(state, LeftParen).ok();
        self.parse_expression_or_sequence(state);
        self.expect(state, RightParen).ok();
        self.parse_statement(state)?;
        if self.eat(state, Else) {
            self.parse_statement(state)?
        }
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::IfStatement);
        Ok(())
    }

    pub(crate) fn parse_for_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // for
        self.eat(state, Await);
        self.expect(state, LeftParen).ok();

        // Init
        self.skip_trivia(state);
        if self.at(state, Var) || self.at(state, Let) || self.at(state, Const) {
            let declaration_cp = state.checkpoint();
            self.parse_variable_declaration_content(state)?;
            state.finish_at(declaration_cp, crate::parser::element_type::TypeScriptElementType::VariableDeclaration);
            if self.eat(state, Of) {
                PrattParser::parse(state, 0, self);
                self.expect(state, RightParen).ok();
                self.parse_statement(state)?;
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ForOfStatement);
                return Ok(());
            }
            if self.eat(state, In) {
                PrattParser::parse(state, 0, self);
                self.expect(state, RightParen).ok();
                self.parse_statement(state)?;
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ForInStatement);
                return Ok(());
            }
        }
        else {
            if !self.at(state, Semicolon) {
                // `in` is a Pratt binary operator, so a bare `for (k in o)` would
                // fold into one expression. Scan the header for a top-level `in`
                // first; the left side is then parsed with relational precedence
                // excluded so it stops before the keyword.
                if self.for_header_has_top_level_in(state) {
                    PrattParser::parse(state, 10, self);
                    self.expect(state, In).ok();
                    PrattParser::parse(state, 0, self);
                    self.expect(state, RightParen).ok();
                    self.parse_statement(state)?;
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ForInStatement);
                    return Ok(());
                }
                self.parse_expression_or_sequence(state);
            }
            self.skip_trivia(state);
            if self.at(state, Of) {
                state.bump(); // of
                PrattParser::parse(state, 0, self);
                self.expect(state, RightParen).ok();
                self.parse_statement(state)?;
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ForOfStatement);
                return Ok(());
            }
            self.expect(state, Semicolon).ok();
        }

        // Test
        self.skip_trivia(state);
        if !self.at(state, Semicolon) {
            self.parse_expression_or_sequence(state);
        }
        self.expect(state, Semicolon).ok();

        // Update
        self.skip_trivia(state);
        if !self.at(state, RightParen) {
            self.parse_expression_or_sequence(state);
        }
        self.expect(state, RightParen).ok();

        self.parse_statement(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ForStatement);
        Ok(())
    }

    /// Returns `true` when the current position is inside a `for (...)` header
    /// that contains a top-level `in` before any top-level `;` or the closing
    /// paren. A top-level `=>` (arrow init) disqualifies the scan.
    fn for_header_has_top_level_in<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        use oak_core::TokenType;
        let mut depth = 0usize;
        let mut offset = 0usize;
        while let Some(token) = state.peek_at(offset) {
            if token.kind.is_ignored() {
                offset += 1;
                continue;
            }
            match token.kind {
                LeftParen | LeftBracket | LeftBrace => depth += 1,
                RightParen | RightBracket | RightBrace if depth == 0 => return false,
                RightParen | RightBracket | RightBrace => depth -= 1,
                Semicolon if depth == 0 => return false,
                Arrow if depth == 0 => return false,
                In if depth == 0 => return true,
                _ => {}
            }
            offset += 1;
        }
        false
    }

    pub(crate) fn parse_while_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // while
        self.expect(state, LeftParen).ok();
        self.parse_expression_or_sequence(state);
        self.expect(state, RightParen).ok();
        self.parse_statement(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::WhileStatement);
        Ok(())
    }

    pub(crate) fn parse_do_while_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // do
        self.parse_statement(state)?;
        self.expect(state, While).ok();
        self.expect(state, LeftParen).ok();
        self.parse_expression_or_sequence(state);
        self.expect(state, RightParen).ok();
        self.eat(state, Semicolon);
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::DoWhileStatement);
        Ok(())
    }

    pub(crate) fn parse_switch_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // switch
        self.expect(state, LeftParen).ok();
        self.parse_expression_or_sequence(state);
        self.expect(state, RightParen).ok();
        self.expect(state, LeftBrace).ok();
        while state.not_at_end() && !self.at(state, RightBrace) {
            let ccp = state.checkpoint();
            if self.eat(state, Case) {
                PrattParser::parse(state, 0, self);
                self.expect(state, Colon).ok();
                while state.not_at_end() && !self.at(state, Case) && !self.at(state, Default) && !self.at(state, RightBrace) {
                    self.parse_statement(state)?
                }
                state.finish_at(ccp, crate::parser::element_type::TypeScriptElementType::CaseClause);
            }
            else if self.eat(state, Default) {
                self.expect(state, Colon).ok();
                while state.not_at_end() && !self.at(state, Case) && !self.at(state, Default) && !self.at(state, RightBrace) {
                    self.parse_statement(state)?
                }
                state.finish_at(ccp, crate::parser::element_type::TypeScriptElementType::DefaultClause);
            }
            else {
                state.bump();
            }
        }
        self.expect(state, RightBrace).ok();
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::SwitchStatement);
        Ok(())
    }

    pub(crate) fn parse_try_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // try
        self.parse_block(state)?;
        if self.at(state, Catch) {
            let ccp = state.checkpoint();
            state.bump();
            if self.eat(state, LeftParen) {
                self.expect(state, IdentifierName).ok();
                self.expect(state, RightParen).ok();
            }
            self.parse_block(state)?;
            state.finish_at(ccp, crate::parser::element_type::TypeScriptElementType::CatchClause);
        }
        if self.eat(state, Finally) {
            let fcp = state.checkpoint();
            self.parse_block(state)?;
            state.finish_at(fcp, crate::parser::element_type::TypeScriptElementType::FinallyClause);
        }
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::TryStatement);
        Ok(())
    }

    pub(crate) fn parse_break_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // break
        if self.at(state, IdentifierName) {
            state.bump();
        }
        self.eat(state, Semicolon);
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::BreakStatement);
        Ok(())
    }

    pub(crate) fn parse_continue_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // continue
        if self.at(state, IdentifierName) {
            state.bump();
        }
        self.eat(state, Semicolon);
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ContinueStatement);
        Ok(())
    }

    pub(crate) fn parse_throw_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // throw
        PrattParser::parse(state, 0, self);
        self.eat(state, Semicolon);
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ThrowStatement);
        Ok(())
    }

    pub(crate) fn parse_return_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // return
        if !self.at(state, Semicolon) && !self.at(state, RightBrace) {
            PrattParser::parse(state, 0, self);
        }
        self.eat(state, Semicolon);
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ReturnStatement);
        Ok(())
    }
}
