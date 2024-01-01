use crate::{ast::*, builder::TypeScriptBuilder, language::TypeScriptLanguage, lexer::token_type::TypeScriptTokenType, parser::element_type::TypeScriptElementType};
use oak_core::{OakError, RedNode, RedTree, Source, SourceText};

impl<'config> TypeScriptBuilder<'config> {
    pub(crate) fn build_statement(&self, node: &RedNode<TypeScriptLanguage>, source: &SourceText) -> Result<Option<Statement>, OakError> {
        let kind = node.green.kind;
        let span = node.span();

        match kind {
            TypeScriptElementType::SourceFile | TypeScriptElementType::Root => {
                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        if let Some(stmt) = self.build_statement(&child_node, source)? {
                            return Ok(Some(stmt));
                        }
                    }
                }
                Ok(None)
            }
            TypeScriptElementType::VariableDeclaration => {
                let mut name = String::new();
                let mut ty = None;
                let mut value = None;
                let mut is_declare = false;
                let mut in_initializer = false;
                let mut declaration_kind = VariableKind::Const;
                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            if child_kind == TypeScriptElementType::BindingPattern {
                                name = source.get_text_in(child_node.span().into()).trim().to_string();
                            }
                            else if child_kind == TypeScriptElementType::IdentifierName && !in_initializer {
                                if name.is_empty() {
                                    for c in child_node.children() {
                                        if let RedTree::Leaf(l) = c {
                                            if l.kind == TypeScriptTokenType::IdentifierName {
                                                name = source.get_text_in(l.span.into()).trim().to_string();
                                                break;
                                            }
                                        }
                                    }
                                    if name.is_empty() {
                                        name = source.get_text_in(child_node.span().into()).trim().to_string();
                                    }
                                }
                            }
                            else if child_kind == TypeScriptElementType::TypeAnnotation {
                                if !self.erase_types {
                                    ty = self.build_type_annotation(&child_node, source)?
                                }
                            }
                            else if value.is_none() {
                                if let Some(expr) = self.build_expression(&child_node, source)? {
                                    value = Some(expr)
                                }
                            }
                        }
                        RedTree::Leaf(leaf) => match leaf.kind {
                            TypeScriptTokenType::IdentifierName => {
                                if name.is_empty() {
                                    name = source.get_text_in(leaf.span.into()).trim().to_string()
                                }
                            }
                            TypeScriptTokenType::Declare => is_declare = true,
                            TypeScriptTokenType::Let => declaration_kind = VariableKind::Let,
                            TypeScriptTokenType::Var => declaration_kind = VariableKind::Var,
                            TypeScriptTokenType::Const => declaration_kind = VariableKind::Const,
                            TypeScriptTokenType::Equal => in_initializer = true,
                            _ => {}
                        },
                    }
                }
                Ok(Some(Statement::VariableDeclaration(VariableDeclaration { decorators: Vec::new(), is_declare, name, kind: declaration_kind, ty, value, span: span.into() })))
            }
            TypeScriptElementType::FunctionDeclaration => {
                let mut name = String::new();
                let mut type_params = Vec::new();
                let mut params = Vec::new();
                let mut return_type = None;
                let mut body = Vec::new();
                let mut decorators = Vec::new();
                let mut is_declare = false;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::IdentifierName => {
                                    if name.is_empty() {
                                        for c in child_node.children() {
                                            if let RedTree::Leaf(l) = c {
                                                if l.kind == TypeScriptTokenType::IdentifierName {
                                                    name = source.get_text_in(l.span.into()).trim().to_string();
                                                    break;
                                                }
                                            }
                                        }
                                        if name.is_empty() {
                                            name = source.get_text_in(child_node.span().into()).trim().to_string();
                                        }
                                    }
                                }
                                TypeScriptElementType::TypeParameter => {
                                    if !self.erase_types {
                                        if let Some(tp) = self.build_type_parameter(&child_node, source)? {
                                            type_params.push(tp)
                                        }
                                    }
                                }
                                TypeScriptElementType::Parameter => {
                                    if let Some(p) = self.build_parameter(&child_node, source)? {
                                        params.push(p)
                                    }
                                }
                                TypeScriptElementType::TypeAnnotation => {
                                    if !self.erase_types {
                                        return_type = self.build_type_annotation(&child_node, source)?
                                    }
                                }
                                TypeScriptElementType::BlockStatement => {
                                    if let Some(Statement::BlockStatement(block)) = self.build_statement(&child_node, source)? {
                                        body = block.statements
                                    }
                                }
                                TypeScriptElementType::Decorator => {
                                    if let Some(d) = self.build_decorator(&child_node, source)? {
                                        decorators.push(d)
                                    }
                                }
                                _ => {}
                            }
                        }
                        RedTree::Leaf(leaf) => match leaf.kind {
                            TypeScriptTokenType::IdentifierName => {
                                if name.is_empty() {
                                    name = source.get_text_in(leaf.span.into()).to_string()
                                }
                            }
                            TypeScriptTokenType::Declare => is_declare = true,
                            _ => {}
                        },
                    }
                }
                Ok(Some(Statement::FunctionDeclaration(FunctionDeclaration { decorators, is_declare, name, type_params, params, return_type, body, span: span.into() })))
            }
            TypeScriptElementType::ClassDeclaration => {
                let mut name = String::new();
                let mut type_params = Vec::new();
                let mut extends = None;
                let mut implements = Vec::new();
                let mut is_abstract = false;
                let mut body = Vec::new();
                let mut decorators = Vec::new();
                let mut is_declare = false;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::IdentifierName => {
                                    if name.is_empty() {
                                        name = source.get_text_in(child_node.span().into()).to_string()
                                    }
                                }
                                TypeScriptElementType::TypeParameter => {
                                    if !self.erase_types {
                                        if let Some(tp) = self.build_type_parameter(&child_node, source)? {
                                            type_params.push(tp)
                                        }
                                    }
                                }
                                TypeScriptElementType::ClassBody => {
                                    for member_child in child_node.children() {
                                        if let RedTree::Node(member_node) = member_child {
                                            if let Some(member) = self.build_class_member(&member_node, source)? {
                                                body.push(member)
                                            }
                                        }
                                    }
                                }
                                TypeScriptElementType::HeritageClause => {
                                    let mut is_implements = false;
                                    for heritage_child in child_node.children() {
                                        match heritage_child {
                                            RedTree::Leaf(leaf) => {
                                                if leaf.kind == TypeScriptTokenType::Implements {
                                                    is_implements = true
                                                }
                                            }
                                            RedTree::Node(heritage_node) => {
                                                if heritage_node.green.kind == TypeScriptElementType::TypeReference {
                                                    if let Some(ty) = self.build_type_annotation(&heritage_node, source)? {
                                                        if is_implements {
                                                            if !self.erase_types {
                                                                implements.push(ty)
                                                            }
                                                        }
                                                        else {
                                                            extends = Some(ty)
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                TypeScriptElementType::Decorator => {
                                    if let Some(d) = self.build_decorator(&child_node, source)? {
                                        decorators.push(d)
                                    }
                                }
                                _ => {}
                            }
                        }
                        RedTree::Leaf(leaf) => match leaf.kind {
                            TypeScriptTokenType::IdentifierName => {
                                if name.is_empty() {
                                    name = source.get_text_in(leaf.span.into()).to_string()
                                }
                            }
                            TypeScriptTokenType::Abstract => is_abstract = true,
                            TypeScriptTokenType::Declare => is_declare = true,
                            _ => {}
                        },
                    }
                }
                Ok(Some(Statement::ClassDeclaration(ClassDeclaration { decorators, is_declare, name, type_params, extends, implements, is_abstract, body, span: span.into() })))
            }
            TypeScriptElementType::InterfaceDeclaration => {
                if self.erase_types {
                    return Ok(None);
                }
                let mut name = String::new();
                let mut type_params = Vec::new();
                let mut extends = Vec::new();
                let mut body = Vec::new();
                let mut is_declare = false;
                let mut decorators = Vec::new();

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::IdentifierName => {
                                    if name.is_empty() {
                                        name = source.get_text_in(child_node.span().into()).to_string()
                                    }
                                }
                                TypeScriptElementType::TypeParameter => {
                                    if let Some(tp) = self.build_type_parameter(&child_node, source)? {
                                        type_params.push(tp)
                                    }
                                }
                                TypeScriptElementType::TypeLiteral => {
                                    // Extract members from object type
                                    for member_child in child_node.children() {
                                        if let RedTree::Node(member_node) = member_child {
                                            if let Some(member) = self.build_class_member(&member_node, source)? {
                                                body.push(member)
                                            }
                                        }
                                    }
                                }
                                TypeScriptElementType::HeritageClause => {
                                    for heritage_child in child_node.children() {
                                        if let RedTree::Node(heritage_node) = heritage_child {
                                            if heritage_node.green.kind == TypeScriptElementType::TypeReference {
                                                if let Some(ty) = self.build_type_annotation(&heritage_node, source)? {
                                                    extends.push(ty)
                                                }
                                            }
                                        }
                                    }
                                }
                                TypeScriptElementType::Decorator => {
                                    if let Some(d) = self.build_decorator(&child_node, source)? {
                                        decorators.push(d)
                                    }
                                }
                                _ => {}
                            }
                        }
                        RedTree::Leaf(leaf) => {
                            if leaf.kind == TypeScriptTokenType::Declare {
                                is_declare = true
                            }
                            else if leaf.kind == TypeScriptTokenType::IdentifierName && name.is_empty() {
                                name = source.get_text_in(leaf.span.into()).to_string();
                            }
                        }
                    }
                }
                Ok(Some(Statement::Interface(InterfaceDeclaration { decorators, is_declare, name, type_params, extends, body, span: span.into() })))
            }
            TypeScriptElementType::TypeAliasDeclaration => {
                if self.erase_types {
                    return Ok(None);
                }
                let mut name = String::new();
                let mut type_params = Vec::new();
                let mut ty = None;
                let mut is_declare = false;
                let mut decorators = Vec::new();

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::IdentifierName => {
                                    if name.is_empty() {
                                        name = source.get_text_in(child_node.span().into()).to_string()
                                    }
                                }
                                TypeScriptElementType::TypeParameter => {
                                    if let Some(tp) = self.build_type_parameter(&child_node, source)? {
                                        type_params.push(tp)
                                    }
                                }
                                TypeScriptElementType::Decorator => {
                                    if let Some(d) = self.build_decorator(&child_node, source)? {
                                        decorators.push(d)
                                    }
                                }
                                _ => {
                                    if let Some(t) = self.build_type_annotation(&child_node, source)? {
                                        ty = Some(t)
                                    }
                                }
                            }
                        }
                        RedTree::Leaf(leaf) => {
                            if leaf.kind == TypeScriptTokenType::Declare {
                                is_declare = true
                            }
                            else if leaf.kind == TypeScriptTokenType::IdentifierName && name.is_empty() {
                                name = source.get_text_in(leaf.span.into()).to_string();
                            }
                        }
                    }
                }
                if let Some(ty) = ty { Ok(Some(Statement::TypeAlias(TypeAliasDeclaration { decorators, is_declare, name, type_params, ty, span: span.into() }))) } else { Ok(None) }
            }
            TypeScriptElementType::EnumDeclaration => {
                let mut name = String::new();
                let mut members = Vec::new();
                let mut is_declare = false;
                let mut decorators = Vec::new();

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::IdentifierName => {
                                    if name.is_empty() {
                                        name = source.get_text_in(child_node.span().into()).to_string()
                                    }
                                }
                                TypeScriptElementType::EnumMember => {
                                    if let Some(member) = self.build_enum_member(&child_node, source)? {
                                        members.push(member)
                                    }
                                }
                                TypeScriptElementType::Decorator => {
                                    if let Some(d) = self.build_decorator(&child_node, source)? {
                                        decorators.push(d)
                                    }
                                }
                                _ => {}
                            }
                        }
                        RedTree::Leaf(leaf) => {
                            if leaf.kind == TypeScriptTokenType::Declare {
                                is_declare = true
                            }
                        }
                    }
                }
                Ok(Some(Statement::Enum(EnumDeclaration { decorators, is_declare, name, members, span: span.into() })))
            }
            TypeScriptElementType::BlockStatement => {
                let mut statements = Vec::new();
                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        if let Some(stmt) = self.build_statement(&child_node, source)? {
                            statements.push(stmt)
                        }
                    }
                }
                Ok(Some(Statement::BlockStatement(BlockStatement { decorators: Vec::new(), is_declare: false, statements, span: span.into() })))
            }
            TypeScriptElementType::ExpressionStatement => {
                let mut expression = None;
                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        expression = self.build_expression(&child_node, source)?
                    }
                }
                if let Some(expr) = expression { Ok(Some(Statement::ExpressionStatement(ExpressionStatement { decorators: Vec::new(), is_declare: false, expression: expr, span: span.into() }))) } else { Ok(None) }
            }
            TypeScriptElementType::IfStatement => {
                let mut test = None;
                let mut consequent = None;
                let mut alternate = None;

                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        if test.is_none() {
                            if let Some(expr) = self.build_expression(&child_node, source)? {
                                test = Some(expr);
                                continue;
                            }
                        }

                        if consequent.is_none() {
                            if let Some(stmt) = self.build_statement(&child_node, source)? {
                                consequent = Some(stmt);
                                continue;
                            }
                        }
                        else if alternate.is_none() {
                            if let Some(stmt) = self.build_statement(&child_node, source)? {
                                alternate = Some(stmt);
                            }
                        }
                    }
                }

                if let (Some(t), Some(c)) = (test, consequent) {
                    Ok(Some(Statement::IfStatement(IfStatement { decorators: Vec::new(), is_declare: false, test: t, consequent: Box::new(c), alternate: alternate.map(Box::new), span: span.into() })))
                }
                else {
                    Ok(None)
                }
            }
            TypeScriptElementType::ReturnStatement => {
                let mut argument = None;
                for child in node.children() {
                    if let RedTree::Node(child_node) = child {
                        argument = self.build_expression(&child_node, source)?
                    }
                }
                Ok(Some(Statement::ReturnStatement(ReturnStatement { decorators: Vec::new(), is_declare: false, argument, span: span.into() })))
            }
            TypeScriptElementType::ForStatement => {
                let mut initializer = None;
                let mut test = None;
                let mut incrementor = None;
                let mut body = Box::new(Statement::BlockStatement(BlockStatement { decorators: Vec::new(), is_declare: false, statements: Vec::new(), span: (0..0).into() }));

                #[derive(PartialEq)]
                enum ForState {
                    Init,
                    Test,
                    Update,
                    Body,
                }
                let mut state = ForState::Init;

                for child in node.children() {
                    match child {
                        RedTree::Leaf(leaf) => {
                            if leaf.kind == TypeScriptTokenType::Semicolon {
                                if state == ForState::Init {
                                    state = ForState::Test;
                                }
                                else if state == ForState::Test {
                                    state = ForState::Update;
                                }
                            }
                            else if leaf.kind == TypeScriptTokenType::RightParen {
                                state = ForState::Body;
                            }
                        }
                        RedTree::Node(child_node) => match state {
                            ForState::Init => {
                                if child_node.green.kind == TypeScriptElementType::VariableDeclaration {
                                    if let Some(stmt) = self.build_statement(&child_node, source)? {
                                        initializer = Some(Box::new(stmt));
                                    }
                                    state = ForState::Test;
                                }
                                else {
                                    if let Some(stmt) = self.build_statement(&child_node, source)? {
                                        initializer = Some(Box::new(stmt));
                                    }
                                }
                            }
                            ForState::Test => {
                                if let Some(expr) = self.build_expression(&child_node, source)? {
                                    test = Some(expr);
                                }
                            }
                            ForState::Update => {
                                if let Some(expr) = self.build_expression(&child_node, source)? {
                                    incrementor = Some(expr);
                                }
                            }
                            ForState::Body => {
                                if let Some(stmt) = self.build_statement(&child_node, source)? {
                                    body = Box::new(stmt);
                                }
                            }
                        },
                    }
                }
                Ok(Some(Statement::ForStatement(ForStatement { decorators: Vec::new(), is_declare: false, initializer, test, incrementor, body, span: span.into() })))
            }
            TypeScriptElementType::WhileStatement => {
                let mut test = Expression { span: (0..0).into(), kind: Box::new(ExpressionKind::BooleanLiteral(true)) };
                let mut body = Box::new(Statement::BlockStatement(BlockStatement { decorators: Vec::new(), is_declare: false, statements: Vec::new(), span: (0..0).into() }));
                let mut past_paren = false;

                for child in node.children() {
                    match child {
                        RedTree::Leaf(leaf) => {
                            if leaf.kind == TypeScriptTokenType::RightParen {
                                past_paren = true;
                            }
                        }
                        RedTree::Node(child_node) => {
                            if past_paren {
                                if let Some(stmt) = self.build_statement(&child_node, source)? {
                                    body = Box::new(stmt);
                                }
                            }
                            else {
                                if let Some(expr) = self.build_expression(&child_node, source)? {
                                    test = expr;
                                }
                            }
                        }
                    }
                }
                Ok(Some(Statement::WhileStatement(WhileStatement { decorators: Vec::new(), is_declare: false, test, body, span: span.into() })))
            }
            TypeScriptElementType::DoWhileStatement => {
                let mut test = Expression { span: (0..0).into(), kind: Box::new(ExpressionKind::BooleanLiteral(true)) };
                let mut body = Box::new(Statement::BlockStatement(BlockStatement { decorators: Vec::new(), is_declare: false, statements: Vec::new(), span: (0..0).into() }));
                let mut seen_while = false;

                for child in node.children() {
                    match child {
                        RedTree::Leaf(leaf) => {
                            if leaf.kind == TypeScriptTokenType::While {
                                seen_while = true;
                            }
                        }
                        RedTree::Node(child_node) => {
                            if seen_while {
                                if let Some(expr) = self.build_expression(&child_node, source)? {
                                    test = expr;
                                }
                            }
                            else {
                                if let Some(stmt) = self.build_statement(&child_node, source)? {
                                    body = Box::new(stmt);
                                }
                            }
                        }
                    }
                }
                Ok(Some(Statement::DoWhileStatement(DoWhileStatement { decorators: Vec::new(), is_declare: false, test, body, span: span.into() })))
            }
            TypeScriptElementType::SwitchStatement => {
                let mut discriminant = None;
                let mut cases = Vec::new();

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::CaseClause => {
                                    let mut test = None;
                                    let mut consequent = Vec::new();

                                    for case_child in child_node.children() {
                                        if let RedTree::Node(case_node) = case_child {
                                            if test.is_none() {
                                                if let Some(expr) = self.build_expression(&case_node, source)? {
                                                    test = Some(expr);
                                                }
                                            }
                                            else {
                                                if let Some(stmt) = self.build_statement(&case_node, source)? {
                                                    consequent.push(stmt);
                                                }
                                            }
                                        }
                                    }

                                    if let Some(test) = test {
                                        cases.push(SwitchCase { test: Some(test), consequent, span: child_node.span().into() });
                                    }
                                }
                                TypeScriptElementType::DefaultClause => {
                                    let mut consequent = Vec::new();

                                    for default_child in child_node.children() {
                                        if let RedTree::Node(default_node) = default_child {
                                            if let Some(stmt) = self.build_statement(&default_node, source)? {
                                                consequent.push(stmt);
                                            }
                                        }
                                    }

                                    cases.push(SwitchCase { test: None, consequent, span: child_node.span().into() });
                                }
                                _ => {
                                    if discriminant.is_none() {
                                        if let Some(expr) = self.build_expression(&child_node, source)? {
                                            discriminant = Some(expr);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(discriminant) = discriminant { Ok(Some(Statement::SwitchStatement(SwitchStatement { decorators: Vec::new(), is_declare: false, discriminant, cases, span: span.into() }))) } else { Ok(None) }
            }
            TypeScriptElementType::TryStatement => {
                let mut block = Vec::new();
                let mut handler = None;
                let mut finalizer = None;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::BlockStatement => {
                                    if block.is_empty() {
                                        for block_child in child_node.children() {
                                            if let RedTree::Node(block_node) = block_child {
                                                if let Some(stmt) = self.build_statement(&block_node, source)? {
                                                    block.push(stmt);
                                                }
                                            }
                                        }
                                    }
                                }
                                TypeScriptElementType::FinallyClause => {
                                    let mut finalizer_block = Vec::new();
                                    for finalizer_child in child_node.children() {
                                        if let RedTree::Node(finalizer_node) = finalizer_child {
                                            if finalizer_node.green.kind == TypeScriptElementType::BlockStatement {
                                                for block_child in finalizer_node.children() {
                                                    if let RedTree::Node(block_node) = block_child {
                                                        if let Some(stmt) = self.build_statement(&block_node, source)? {
                                                            finalizer_block.push(stmt);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    finalizer = Some(finalizer_block);
                                }
                                TypeScriptElementType::CatchClause => {
                                    let mut param = None;
                                    let mut body = Vec::new();
                                    let mut in_body = false;

                                    for catch_child in child_node.children() {
                                        match catch_child {
                                            RedTree::Node(catch_node) => {
                                                let child_kind = catch_node.green.kind;
                                                if !in_body && child_kind != TypeScriptElementType::BlockStatement {
                                                    // This is likely the parameter
                                                    param = Some(source.get_text_in(catch_node.span().into()).trim().to_string());
                                                }
                                                else if child_kind == TypeScriptElementType::BlockStatement {
                                                    // This is the body block
                                                    in_body = true;
                                                    for block_child in catch_node.children() {
                                                        if let RedTree::Node(block_node) = block_child {
                                                            if let Some(stmt) = self.build_statement(&block_node, source)? {
                                                                body.push(stmt);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }

                                    // Even if body is empty, set the handler if we found a catch clause
                                    handler = Some(CatchClause { param, body, span: child_node.span().into() });
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                if !block.is_empty() { Ok(Some(Statement::TryStatement(TryStatement { decorators: Vec::new(), is_declare: false, block, handler, finalizer, span: span.into() }))) } else { Ok(None) }
            }
            TypeScriptElementType::ImportDeclaration => {
                let mut specifiers = Vec::new();
                let mut source_str = String::new();
                let mut is_type_only = false;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::ImportClause => {
                                    for clause_child in child_node.children() {
                                        if let RedTree::Node(clause_node) = clause_child {
                                            match clause_node.green.kind {
                                                TypeScriptElementType::IdentifierName => specifiers.push(ImportSpecifier::Default(source.get_text_in(clause_node.span().into()).trim().to_string())),
                                                TypeScriptElementType::NamespaceImport => {
                                                    for ns_child in clause_node.children() {
                                                        if let RedTree::Node(ns_node) = ns_child {
                                                            if ns_node.green.kind == TypeScriptElementType::IdentifierName {
                                                                specifiers.push(ImportSpecifier::Namespace(source.get_text_in(ns_node.span().into()).trim().to_string()))
                                                            }
                                                        }
                                                    }
                                                }
                                                TypeScriptElementType::NamedImports => {
                                                    for named_child in clause_node.children() {
                                                        if let RedTree::Node(named_node) = named_child {
                                                            if named_node.green.kind == TypeScriptElementType::ImportSpecifier {
                                                                let mut local = String::new();
                                                                let mut imported = String::new();
                                                                for spec_child in named_node.children() {
                                                                    if let RedTree::Node(spec_node) = spec_child {
                                                                        if spec_node.green.kind == TypeScriptElementType::IdentifierName {
                                                                            let name = source.get_text_in(spec_node.span().into()).trim().to_string();
                                                                            if imported.is_empty() { imported = name } else { local = name }
                                                                        }
                                                                    }
                                                                }
                                                                if local.is_empty() {
                                                                    local = imported.clone()
                                                                }
                                                                specifiers.push(ImportSpecifier::Named { local, imported })
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                TypeScriptElementType::StringLiteral => {
                                    let text = source.get_text_in(child_node.span().into());
                                    source_str = text[1..text.len() - 1].to_string()
                                }
                                _ => {}
                            }
                        }
                        RedTree::Leaf(leaf) => {
                            if leaf.kind == TypeScriptTokenType::Type {
                                is_type_only = true
                            }
                            else if leaf.kind == TypeScriptTokenType::StringLiteral && source_str.is_empty() {
                                let text = source.get_text_in(leaf.span.into());
                                if text.len() >= 2 {
                                    source_str = text[1..text.len() - 1].to_string();
                                }
                            }
                        }
                    }
                }
                Ok(Some(Statement::ImportDeclaration(ImportDeclaration { specifiers, module_specifier: source_str, is_type_only, span: span.into() })))
            }
            TypeScriptElementType::ExportDeclaration => {
                let mut declaration = None;
                let mut specifiers = Vec::new();
                let mut source_str = None;
                let mut is_default = false;
                let mut is_type_only = false;

                for child in node.children() {
                    match child {
                        RedTree::Node(child_node) => {
                            let child_kind = child_node.green.kind;
                            match child_kind {
                                TypeScriptElementType::NamedExports => {
                                    let mut pending_type_only = false;
                                    for named_child in child_node.children() {
                                        if let RedTree::Node(named_node) = named_child {
                                            if named_node.green.kind == TypeScriptElementType::ExportSpecifier {
                                                let mut local = String::new();
                                                let mut exported = String::new();
                                                let mut is_type_only = pending_type_only;
                                                for spec_child in named_node.children() {
                                                    if let RedTree::Node(spec_node) = spec_child {
                                                        if spec_node.green.kind == TypeScriptElementType::IdentifierName {
                                                            let name = source.get_text_in(spec_node.span().into()).trim().to_string();
                                                            if local.is_empty() { local = name } else { exported = name }
                                                        }
                                                    }
                                                    else if let RedTree::Leaf(leaf) = spec_child {
                                                        if leaf.kind == TypeScriptTokenType::Type {
                                                            is_type_only = true;
                                                        }
                                                    }
                                                }
                                                if exported.is_empty() {
                                                    exported = local.clone()
                                                }
                                                specifiers.push(ExportSpecifier { local, exported, is_type_only })
                                            }
                                        }
                                        else if let RedTree::Leaf(leaf) = named_child {
                                            pending_type_only = leaf.kind == TypeScriptTokenType::Type;
                                        }
                                    }
                                }
                                TypeScriptElementType::StringLiteral => {
                                    let text = source.get_text_in(child_node.span().into());
                                    source_str = Some(text[1..text.len() - 1].to_string())
                                }
                                _ => declaration = self.build_statement(&child_node, source)?,
                            }
                        }
                        RedTree::Leaf(leaf) => match leaf.kind {
                            TypeScriptTokenType::Default => is_default = true,
                            TypeScriptTokenType::Type => is_type_only = true,
                            TypeScriptTokenType::StringLiteral if source_str.is_none() => {
                                let text = source.get_text_in(leaf.span.into());
                                if text.len() >= 2 {
                                    source_str = Some(text[1..text.len() - 1].to_string());
                                }
                            }
                            _ => {}
                        },
                    }
                }
                Ok(Some(Statement::ExportDeclaration(ExportDeclaration { declaration: declaration.map(Box::new), specifiers, source: source_str, is_default, is_type_only, span: span.into() })))
            }
            _ => {
                // Try building as expression statement if not a recognized statement kind
                if let Some(expr) = self.build_expression(node, source)? { Ok(Some(Statement::ExpressionStatement(ExpressionStatement { decorators: Vec::new(), is_declare: false, expression: expr, span: span.into() }))) } else { Ok(None) }
            }
        }
    }
}
