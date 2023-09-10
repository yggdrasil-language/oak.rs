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
                Some(Type) => {
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
                Some(Function) => {
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
                Some(Type) => {
                    self.parse_type_alias_declaration(state)?;
                }
                Some(Namespace) | Some(Module) => {
                    self.parse_namespace_declaration(state)?;
                }
                Some(Import) => {
                    self.parse_import_declaration(state)?;
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
        while state.at(IdentifierName.into()) {
            self.expect(state, IdentifierName).ok();
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

    pub(crate) fn parse_function_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let cp = state.checkpoint();
        self.parse_function_declaration_content(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::FunctionDeclaration);
        Ok(())
    }

    pub(crate) fn parse_function_declaration_content<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        state.bump(); // function
        self.skip_trivia(state);
        if self.at(state, IdentifierName) {
            self.expect(state, IdentifierName).ok();
        }
        self.parse_parameters(state)?;
        // Skip return type annotation
        if self.eat(state, Colon) {
            while state.not_at_end() && !self.at(state, LeftBrace) && !self.at(state, Semicolon) {
                self.skip_trivia(state);
                if state.not_at_end() && !self.at(state, LeftBrace) && !self.at(state, Semicolon) { state.bump() } else { break }
            }
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
        self.expect(state, Equal).ok();
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
        use crate::lexer::token_type::TypeScriptTokenType::*;
        use crate::parser::element_type::TypeScriptElementType;
        let cp = state.checkpoint();
        state.bump(); // import

        self.skip_trivia(state);
        // `import type …` — leave `Type` as a leaf so the builder can set `is_type_only`.
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
        use crate::lexer::token_type::TypeScriptTokenType::*;
        use crate::parser::element_type::TypeScriptElementType;
        let id_cp = state.checkpoint();
        self.expect(state, IdentifierName).ok();
        state.finish_at(id_cp, TypeScriptElementType::IdentifierName);
    }

    fn parse_import_string_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        use crate::parser::element_type::TypeScriptElementType;
        let str_cp = state.checkpoint();
        self.expect(state, StringLiteral).ok();
        state.finish_at(str_cp, TypeScriptElementType::StringLiteral);
    }

    fn parse_named_imports<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        use crate::parser::element_type::TypeScriptElementType;
        let named_cp = state.checkpoint();
        self.expect(state, LeftBrace).ok();
        while state.not_at_end() && !self.at(state, RightBrace) {
            self.skip_trivia(state);
            if self.at(state, IdentifierName) {
                let spec_cp = state.checkpoint();
                self.parse_import_identifier(state);
                self.skip_trivia(state);
                if self.eat(state, As) {
                    self.skip_trivia(state);
                    self.parse_import_identifier(state);
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
        use crate::lexer::token_type::TypeScriptTokenType::*;
        use crate::parser::element_type::TypeScriptElementType;
        let ns_cp = state.checkpoint();
        state.bump(); // *
        self.skip_trivia(state);
        self.expect(state, As).ok();
        self.skip_trivia(state);
        self.parse_import_identifier(state);
        state.finish_at(ns_cp, TypeScriptElementType::NamespaceImport);
    }

    pub(crate) fn parse_export_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // export

        self.skip_trivia(state);
        if self.eat(state, Default) {
            self.parse_statement(state)?;
        }
        else if self.at(state, LeftBrace) {
            // export { a, b as c }
            state.bump();
            while state.not_at_end() && !self.at(state, RightBrace) {
                self.skip_trivia(state);
                if self.at(state, IdentifierName) {
                    state.bump();
                    if self.eat(state, As) {
                        self.expect(state, IdentifierName).ok();
                    }
                }
                if !self.eat(state, Comma) {
                    break;
                }
            }
            self.expect(state, RightBrace).ok();
            if self.eat(state, From) {
                self.expect(state, StringLiteral).ok();
            }
            self.eat(state, Semicolon);
        }
        else if self.at(state, Star) {
            // export * from '...'
            state.bump();
            if self.eat(state, As) {
                self.expect(state, IdentifierName).ok();
            }
            self.expect(state, From).ok();
            self.expect(state, StringLiteral).ok();
            self.eat(state, Semicolon);
        }
        else {
            self.parse_statement(state)?
        }

        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ExportDeclaration);
        Ok(())
    }

    pub(crate) fn parse_if_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // if
        self.expect(state, LeftParen).ok();
        PrattParser::parse(state, 0, self);
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
        self.expect(state, LeftParen).ok();

        // Init
        self.skip_trivia(state);
        if self.at(state, Var) || self.at(state, Let) || self.at(state, Const) {
            let cp = state.checkpoint();
            self.parse_variable_declaration_content(state)?;
            state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::VariableDeclaration);
        }
        else {
            if !self.at(state, Semicolon) {
                PrattParser::parse(state, 0, self);
            }
            self.expect(state, Semicolon).ok();
        }

        // Test
        self.skip_trivia(state);
        if !self.at(state, Semicolon) {
            PrattParser::parse(state, 0, self);
        }
        self.expect(state, Semicolon).ok();

        // Update
        self.skip_trivia(state);
        if !self.at(state, RightParen) {
            PrattParser::parse(state, 0, self);
        }
        self.expect(state, RightParen).ok();

        self.parse_statement(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ForStatement);
        Ok(())
    }

    pub(crate) fn parse_while_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        state.bump(); // while
        self.expect(state, LeftParen).ok();
        PrattParser::parse(state, 0, self);
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
        PrattParser::parse(state, 0, self);
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
        PrattParser::parse(state, 0, self);
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
