//! Module-level binding and reference facts for rename-safe bundling.
//!
//! [`module_scope_facts`] reads a built [`TypeScriptRoot`] and reports every
//! module-level binding with the source spans of its declaration and of its
//! value-position references.
//!
//! The results are only meaningful when the AST was built with type erasure
//! **off** ([`crate::TypeScriptBuilder::with_type_erasure`]` (false), the
//! default): with types erased, type references are invisible to this module
//! and `has_type_references` will be wrong.
//!
//! When `incomplete` is `true`, the facts may be missing references (dynamic
//! `eval`, JSX, template interpolations, ambiguous computed keys, unrecoverable
//! declaration spans, duplicate bindings, namespaces, interface bodies — the
//! parser does not yet record interface members or heritage in the AST). A
//! consumer **must refuse to rename** any binding while the flag is set:
//! over-rejecting is safe, under-reporting a reference is not.
//!
//! `references` are value-position uses in source order. Only spans the AST
//! actually carries are recorded; no span is ever invented by name matching.
//! The single exception is [`crate::ast::TypeAnnotation::Query`] (`typeof X`),
//! which stores no span: there a whole-word textual match may only set the
//! conservative `has_type_references` flag, never a reference span.

use std::collections::HashMap;

use core::range::Range;

use crate::ast::{
    ClassMember, Decorator, Expression, ExpressionKind, ExportDeclaration, ExportSpecifier, FunctionParam, ImportDeclaration,
    ImportSpecifier, ObjectProperty, Statement, TemplateElement, TypeScriptRoot, TypeAnnotation, TypeParameter, VariableDeclaration,
    VariableKind,
};

/// Module-level facts about the bindings of one [`TypeScriptRoot`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModuleScopeFacts {
    /// Module-level bindings in source order.
    pub bindings: Vec<ModuleBindingFacts>,
    /// `true` when references may be missing; consumers must refuse renames.
    pub incomplete: bool,
}

/// One module-level binding with its declaration and reference spans.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModuleBindingFacts {
    /// The bound name.
    pub name: String,
    /// What kind of declaration introduced the binding.
    pub kind: ModuleBindingKind,
    /// Source span of the binding token.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub declaration: Range<usize>,
    /// Value-position references in source order.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range::vec"))]
    pub references: Vec<Range<usize>>,
    /// `true` when the name is used in a type position (span not recorded).
    pub has_type_references: bool,
}

/// The declaration form that introduced a module-level binding.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModuleBindingKind {
    /// `function` declaration.
    Function,
    /// `class` declaration.
    Class,
    /// `enum` declaration.
    Enum,
    /// `var` binding (including `var` hoisted out of module-level blocks).
    Var,
    /// `let` binding.
    Let,
    /// `const` binding.
    Const,
    /// `import def from "m"` default specifier.
    ImportDefault,
    /// `import { named } from "m"` named specifier.
    ImportNamed,
    /// `import * as ns from "m"` namespace specifier.
    ImportNamespace,
}

/// Collects the module-level bindings and their references from `root`.
pub fn module_scope_facts(root: &TypeScriptRoot) -> ModuleScopeFacts {
    let mut walker = ScopeWalker::new();
    for statement in &root.statements {
        walker.module_statement(statement);
    }
    for statement in &root.statements {
        walker.walk_statement(statement);
    }
    ModuleScopeFacts { bindings: walker.bindings, incomplete: walker.incomplete }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FrameKind {
    Module,
    Function,
    Block,
}

/// One lexical scope frame. `Some(index)` slots exist only in the module
/// frame and point at [`ModuleBindingFacts`]; every other entry is a shadow.
struct Frame {
    kind: FrameKind,
    entries: Vec<(String, Option<usize>)>,
}

struct ScopeWalker {
    bindings: Vec<ModuleBindingFacts>,
    binding_index: HashMap<String, usize>,
    frames: Vec<Frame>,
    incomplete: bool,
}

impl ScopeWalker {
    fn new() -> Self {
        Self {
            bindings: Vec::new(),
            binding_index: HashMap::new(),
            frames: vec![Frame { kind: FrameKind::Module, entries: Vec::new() }],
            incomplete: false,
        }
    }

    fn push(&mut self, kind: FrameKind) {
        self.frames.push(Frame { kind, entries: Vec::new() });
    }

    fn pop(&mut self) {
        self.frames.pop().expect("the module frame is never popped");
    }

    /// Registers `name` in the current frame unless the frame already knows it.
    fn register(&mut self, name: &str, slot: Option<usize>) {
        let frame = self.frames.last_mut().expect("the module frame is always present");
        if !frame.entries.iter().any(|(entry_name, _)| entry_name == name) {
            frame.entries.push((name.to_string(), slot));
        }
    }

    /// Registers a hoisted `var` shadow in the nearest function-or-module frame.
    fn hoist_register(&mut self, name: &str) {
        let index = self.frames.iter().rposition(|frame| frame.kind != FrameKind::Block).unwrap_or(0);
        let frame = &mut self.frames[index];
        if !frame.entries.iter().any(|(entry_name, _)| entry_name == name) {
            frame.entries.push((name.to_string(), None));
        }
    }

    fn resolve_value(&mut self, name: &str, span: Range<usize>) {
        for frame in self.frames.iter().rev() {
            let slot = frame.entries.iter().find(|(entry_name, _)| entry_name == name).map(|(_, slot)| *slot);
            if let Some(slot) = slot {
                if let Some(index) = slot {
                    self.bindings[index].references.push(span);
                }
                return;
            }
        }
    }

    fn resolve_type(&mut self, name: &str) {
        for frame in self.frames.iter().rev() {
            let slot = frame.entries.iter().find(|(entry_name, _)| entry_name == name).map(|(_, slot)| *slot);
            if let Some(slot) = slot {
                if let Some(index) = slot {
                    self.bindings[index].has_type_references = true;
                }
                return;
            }
        }
    }

    /// Rule 5: the AST cannot tell a plain key from a computed `[key]`, so a
    /// key that reads like a module binding marks the facts incomplete.
    fn check_key_name(&mut self, name: &str) {
        if self.binding_index.contains_key(name) {
            self.incomplete = true;
        }
    }

    // ---------------------------------------------------------------- phase 1

    fn add_module_binding(&mut self, name: &str, kind: ModuleBindingKind, declaration: &Range<usize>) {
        if name.is_empty() {
            self.incomplete = true;
            return;
        }
        if declaration.start == declaration.end {
            self.incomplete = true;
        }
        if self.binding_index.contains_key(name) {
            self.incomplete = true;
            return;
        }
        let index = self.bindings.len();
        self.bindings.push(ModuleBindingFacts {
            name: name.to_string(),
            kind,
            declaration: declaration.clone(),
            references: Vec::new(),
            has_type_references: false,
        });
        self.binding_index.insert(name.to_string(), index);
        let frame = &mut self.frames[0];
        match frame.entries.iter_mut().find(|(entry_name, _)| entry_name == name) {
            Some(entry) => entry.1 = Some(index),
            None => frame.entries.push((name.to_string(), Some(index))),
        }
    }

    fn declared_binding(&mut self, name: &str, kind: ModuleBindingKind, name_span: &Range<usize>) {
        if name.is_empty() {
            self.incomplete = true;
            return;
        }
        self.add_module_binding(name, kind, name_span);
    }

    fn variable_bindings(&mut self, declaration: &VariableDeclaration) {
        let kind = match declaration.kind {
            VariableKind::Var => ModuleBindingKind::Var,
            VariableKind::Let => ModuleBindingKind::Let,
            VariableKind::Const => ModuleBindingKind::Const,
        };
        for declarator in &declaration.declarators {
            if declarator.binding_spans.len() != declarator.binding_names.len() {
                self.incomplete = true;
            }
            for (position, name) in declarator.binding_names.iter().enumerate() {
                match declarator.binding_spans.get(position) {
                    Some(span) => self.add_module_binding(name, kind.clone(), span),
                    None => self.add_module_binding(name, kind.clone(), &Range { start: declarator.span.start, end: declarator.span.start }),
                }
            }
        }
    }

    fn import_bindings(&mut self, declaration: &ImportDeclaration) {
        if declaration.is_type_only() {
            for specifier in &declaration.specifiers {
                self.register(import_local(specifier), None);
            }
            return;
        }
        for specifier in &declaration.specifiers {
            match specifier {
                ImportSpecifier::Default { local, span } => self.add_module_binding(local, ModuleBindingKind::ImportDefault, span),
                ImportSpecifier::Namespace { local, local_span, .. } => {
                    self.add_module_binding(local, ModuleBindingKind::ImportNamespace, local_span)
                }
                ImportSpecifier::Named { local, local_span, type_modifier, .. } => {
                    if type_modifier.is_some() {
                        self.register(local.as_str(), None);
                    }
                    else {
                        self.add_module_binding(local, ModuleBindingKind::ImportNamed, local_span);
                    }
                }
            }
        }
    }

    fn module_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::FunctionDeclaration(declaration) => self.declared_binding(&declaration.name, ModuleBindingKind::Function, &declaration.name_span),
            Statement::ClassDeclaration(declaration) => self.declared_binding(&declaration.name, ModuleBindingKind::Class, &declaration.name_span),
            Statement::Enum(declaration) => self.declared_binding(&declaration.name, ModuleBindingKind::Enum, &declaration.name_span),
            Statement::VariableDeclaration(declaration) => self.variable_bindings(declaration),
            Statement::ImportDeclaration(declaration) => self.import_bindings(declaration),
            Statement::ExportDeclaration(declaration) => {
                if declaration.is_type_only() {
                    return;
                }
                if let Some(inner) = &declaration.declaration {
                    self.module_statement(inner);
                }
            }
            other => {
                let mut hoisted = Vec::new();
                collect_var_declarators_one(other, &mut hoisted);
                for (name, span) in hoisted {
                    self.add_module_binding(&name, ModuleBindingKind::Var, &span);
                }
            }
        }
    }

    // ---------------------------------------------------------------- phase 2

    fn walk_statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.walk_statement(statement);
        }
    }

    /// Traverses one statement body, giving a non-block body its own scope so
    /// declarations inside it shadow the module bindings.
    fn walk_body(&mut self, statement: &Statement) {
        if matches!(statement, Statement::BlockStatement(_)) {
            self.walk_statement(statement);
            return;
        }
        self.push(FrameKind::Block);
        self.prescan_declarations(std::slice::from_ref(statement));
        self.walk_statement(statement);
        self.pop();
    }

    fn walk_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                self.walk_decorators(&declaration.decorators);
                for declarator in &declaration.declarators {
                    if let Some(ty) = &declarator.ty {
                        self.walk_type(ty);
                    }
                    if let Some(value) = &declarator.value {
                        self.walk_expression(value);
                    }
                }
            }
            Statement::FunctionDeclaration(declaration) => {
                self.walk_decorators(&declaration.decorators);
                self.enter_function(None, &declaration.type_params, &declaration.params, declaration.return_type.as_ref(), &declaration.body);
            }
            Statement::ClassDeclaration(declaration) => {
                self.walk_decorators(&declaration.decorators);
                self.enter_class(None, &declaration.type_params, declaration.extends.as_ref(), &declaration.implements, &declaration.body);
            }
            Statement::ExpressionStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.walk_expression(&statement.expression);
            }
            Statement::ImportDeclaration(_) => {}
            Statement::ExportDeclaration(declaration) => self.walk_export(declaration),
            Statement::Interface(declaration) => {
                // The parser does not yet produce member or heritage nodes for
                // an interface body, so the AST carries none of the type names
                // written inside it. Any reference from an interface to a
                // module binding is therefore invisible: refuse all renames in
                // such a module until the parser grows interface members.
                self.incomplete = true;
                self.walk_decorators(&declaration.decorators);
                self.push(FrameKind::Block);
                self.register_type_params(&declaration.type_params);
                for parameter in &declaration.type_params {
                    self.walk_type_param_types(parameter);
                }
                for extended in &declaration.extends {
                    self.walk_type(extended);
                }
                for member in &declaration.body {
                    self.walk_type_member(member);
                }
                self.pop();
            }
            Statement::TypeAlias(declaration) => {
                self.walk_decorators(&declaration.decorators);
                self.push(FrameKind::Block);
                self.register_type_params(&declaration.type_params);
                for parameter in &declaration.type_params {
                    self.walk_type_param_types(parameter);
                }
                self.walk_type(&declaration.ty);
                self.pop();
            }
            Statement::Enum(declaration) => {
                self.walk_decorators(&declaration.decorators);
                for member in &declaration.members {
                    if let Some(initializer) = &member.initializer {
                        self.walk_expression(initializer);
                    }
                }
            }
            Statement::ReturnStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                if let Some(argument) = &statement.argument {
                    self.walk_expression(argument);
                }
            }
            Statement::IfStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.walk_expression(&statement.test);
                self.walk_body(&statement.consequent);
                if let Some(alternate) = &statement.alternate {
                    self.walk_body(alternate);
                }
            }
            Statement::WhileStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.walk_expression(&statement.test);
                self.walk_body(&statement.body);
            }
            Statement::DoWhileStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.walk_body(&statement.body);
                self.walk_expression(&statement.test);
            }
            Statement::ForStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.push(FrameKind::Block);
                if let Some(initializer) = &statement.initializer {
                    self.prescan_declarations(std::slice::from_ref(initializer));
                    self.walk_statement(initializer);
                }
                if let Some(test) = &statement.test {
                    self.walk_expression(test);
                }
                if let Some(incrementor) = &statement.incrementor {
                    self.walk_expression(incrementor);
                }
                self.walk_body(&statement.body);
                self.pop();
            }
            Statement::ForInStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.push(FrameKind::Block);
                self.prescan_declarations(std::slice::from_ref(&statement.left));
                self.walk_statement(&statement.left);
                self.walk_expression(&statement.right);
                self.walk_body(&statement.body);
                self.pop();
            }
            Statement::ForOfStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.push(FrameKind::Block);
                self.prescan_declarations(std::slice::from_ref(&statement.left));
                self.walk_statement(&statement.left);
                self.walk_expression(&statement.right);
                self.walk_body(&statement.body);
                self.pop();
            }
            Statement::SwitchStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.walk_expression(&statement.discriminant);
                self.push(FrameKind::Block);
                for case in &statement.cases {
                    self.prescan_declarations(&case.consequent);
                }
                for case in &statement.cases {
                    if let Some(test) = &case.test {
                        self.walk_expression(test);
                    }
                    self.walk_statements(&case.consequent);
                }
                self.pop();
            }
            Statement::TryStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.push(FrameKind::Block);
                self.prescan_declarations(&statement.block);
                self.walk_statements(&statement.block);
                self.pop();
                if let Some(handler) = &statement.handler {
                    self.push(FrameKind::Block);
                    if let Some(param) = &handler.param {
                        self.register(param, None);
                    }
                    self.prescan_declarations(&handler.body);
                    self.walk_statements(&handler.body);
                    self.pop();
                }
                if let Some(finalizer) = &statement.finalizer {
                    self.push(FrameKind::Block);
                    self.prescan_declarations(finalizer);
                    self.walk_statements(finalizer);
                    self.pop();
                }
            }
            Statement::ThrowStatement(statement) => {
                self.walk_decorators(&statement.decorators);
                self.walk_expression(&statement.argument);
            }
            Statement::BreakStatement(_) | Statement::ContinueStatement(_) => {}
            Statement::BlockStatement(statement) => {
                self.push(FrameKind::Block);
                self.prescan_declarations(&statement.statements);
                self.walk_statements(&statement.statements);
                self.pop();
            }
            Statement::Namespace(_) => self.incomplete = true,
        }
    }

    fn walk_export(&mut self, declaration: &ExportDeclaration) {
        if declaration.is_type_only() {
            if declaration.source.is_none() {
                for specifier in &declaration.specifiers {
                    if let ExportSpecifier::Named { local, .. } = specifier {
                        self.resolve_type(local);
                    }
                }
            }
            return;
        }
        if let Some(inner) = &declaration.declaration {
            self.walk_statement(inner);
        }
        if declaration.source.is_none() {
            for specifier in &declaration.specifiers {
                if let ExportSpecifier::Named { local, local_span, type_modifier, .. } = specifier {
                    if type_modifier.is_some() {
                        self.resolve_type(local);
                    }
                    else if local_span.start == local_span.end {
                        self.incomplete = true;
                    }
                    else {
                        self.resolve_value(local, local_span.clone());
                    }
                }
            }
        }
    }

    fn enter_function(
        &mut self,
        name_shadow: Option<&str>,
        type_params: &[TypeParameter],
        params: &[FunctionParam],
        return_type: Option<&TypeAnnotation>,
        body: &[Statement],
    ) {
        self.push(FrameKind::Function);
        if let Some(name) = name_shadow {
            if !name.is_empty() {
                self.register(name, None);
            }
        }
        self.register_type_params(type_params);
        for param in params {
            if param.binding_names.is_empty() {
                if !param.name.is_empty() {
                    self.register(&param.name, None);
                }
            }
            else {
                for bound in &param.binding_names {
                    self.register(bound, None);
                }
            }
        }
        for parameter in type_params {
            self.walk_type_param_types(parameter);
        }
        for param in params {
            self.walk_decorators(&param.decorators);
            if let Some(ty) = &param.ty {
                self.walk_type(ty);
            }
            if let Some(default) = &param.default {
                self.walk_expression(default);
            }
        }
        if let Some(return_type) = return_type {
            self.walk_type(return_type);
        }
        let mut hoisted = Vec::new();
        collect_var_declarators(body, &mut hoisted);
        for (name, _) in hoisted {
            self.hoist_register(&name);
        }
        self.prescan_declarations(body);
        self.walk_statements(body);
        self.pop();
    }

    fn enter_class(
        &mut self,
        name_shadow: Option<&str>,
        type_params: &[TypeParameter],
        extends: Option<&TypeAnnotation>,
        implements: &[TypeAnnotation],
        members: &[ClassMember],
    ) {
        self.push(FrameKind::Block);
        if let Some(name) = name_shadow {
            if !name.is_empty() {
                self.register(name, None);
            }
        }
        self.register_type_params(type_params);
        for parameter in type_params {
            self.walk_type_param_types(parameter);
        }
        if let Some(extends) = extends {
            self.walk_type(extends);
        }
        for implemented in implements {
            self.walk_type(implemented);
        }
        for member in members {
            self.walk_class_member(member);
        }
        self.pop();
    }

    fn walk_class_member(&mut self, member: &ClassMember) {
        match member {
            ClassMember::Property { decorators, name, ty, initializer, .. } => {
                self.check_key_name(name);
                self.walk_decorators(decorators);
                if let Some(ty) = ty {
                    self.walk_type(ty);
                }
                if let Some(initializer) = initializer {
                    self.walk_expression(initializer);
                }
            }
            ClassMember::Method { decorators, name, type_params, params, return_type, body, .. } => {
                self.check_key_name(name);
                self.walk_decorators(decorators);
                self.enter_function(None, type_params, params, return_type.as_ref(), body);
            }
        }
    }

    fn walk_type_member(&mut self, member: &ClassMember) {
        match member {
            ClassMember::Property { name, ty, .. } => {
                self.check_key_name(name);
                if let Some(ty) = ty {
                    self.walk_type(ty);
                }
            }
            ClassMember::Method { name, type_params, params, return_type, .. } => {
                self.check_key_name(name);
                self.push(FrameKind::Block);
                self.register_type_params(type_params);
                for parameter in type_params {
                    self.walk_type_param_types(parameter);
                }
                for param in params {
                    if let Some(ty) = &param.ty {
                        self.walk_type(ty);
                    }
                }
                if let Some(return_type) = return_type {
                    self.walk_type(return_type);
                }
                self.pop();
            }
        }
    }

    fn walk_decorators(&mut self, decorators: &[Decorator]) {
        for decorator in decorators {
            self.walk_expression(&decorator.expression);
        }
    }

    fn register_type_params(&mut self, type_params: &[TypeParameter]) {
        for parameter in type_params {
            if !parameter.name.is_empty() {
                self.register(&parameter.name, None);
            }
        }
    }

    fn walk_type_param_types(&mut self, parameter: &TypeParameter) {
        if let Some(constraint) = &parameter.constraint {
            self.walk_type(constraint);
        }
        if let Some(default) = &parameter.default {
            self.walk_type(default);
        }
    }

    fn walk_expression(&mut self, expression: &Expression) {
        match &*expression.kind {
            ExpressionKind::Identifier(name) => self.resolve_value(name, expression.span.clone()),
            ExpressionKind::TemplateString(text) => {
                if text.contains("${") {
                    self.incomplete = true;
                }
            }
            ExpressionKind::UnaryExpression { argument, .. } | ExpressionKind::UpdateExpression { argument, .. } => {
                self.walk_expression(argument)
            }
            ExpressionKind::BinaryExpression { left, right, .. } => {
                self.walk_expression(left);
                self.walk_expression(right);
            }
            ExpressionKind::ConditionalExpression { test, consequent, alternate, .. } => {
                self.walk_expression(test);
                self.walk_expression(consequent);
                self.walk_expression(alternate);
            }
            ExpressionKind::MemberExpression { object, property, computed, .. } => {
                self.walk_expression(object);
                if *computed {
                    self.walk_expression(property);
                }
            }
            ExpressionKind::CallExpression { func, args } => {
                if let ExpressionKind::Identifier(name) = &*func.kind {
                    if name == "eval" {
                        self.incomplete = true;
                    }
                }
                self.walk_expression(func);
                for arg in args {
                    self.walk_expression(arg);
                }
            }
            ExpressionKind::NewExpression { func, args } => {
                self.walk_expression(func);
                for arg in args {
                    self.walk_expression(arg);
                }
            }
            ExpressionKind::AssignmentExpression { left, right, .. } => {
                self.walk_expression(left);
                self.walk_expression(right);
            }
            ExpressionKind::AsExpression { expression, type_annotation } => {
                self.walk_expression(expression);
                self.walk_type(type_annotation);
            }
            ExpressionKind::TypeAssertionExpression { expression, type_annotation } => {
                self.walk_expression(expression);
                self.walk_type(type_annotation);
            }
            ExpressionKind::ArrowFunction { type_params, params, return_type, body, .. } => {
                self.enter_function(None, type_params, params, return_type.as_ref(), std::slice::from_ref(body));
            }
            ExpressionKind::ObjectLiteral { properties } => {
                for property in properties {
                    match property {
                        ObjectProperty::Property { name, value, shorthand, .. } => {
                            if !*shorthand {
                                self.check_key_name(name);
                            }
                            self.walk_expression(value);
                        }
                        ObjectProperty::Method { name, params, return_type, body, .. } => {
                            self.check_key_name(name);
                            self.enter_function(None, &[], params, return_type.as_ref(), body);
                        }
                        ObjectProperty::Spread(expression) => self.walk_expression(expression),
                    }
                }
            }
            ExpressionKind::ArrayLiteral { elements } | ExpressionKind::SequenceExpression(elements) => {
                for element in elements {
                    self.walk_expression(element);
                }
            }
            ExpressionKind::SpreadElement(inner) | ExpressionKind::AwaitExpression(inner) | ExpressionKind::NonNullExpression(inner) => {
                self.walk_expression(inner)
            }
            ExpressionKind::YieldExpression(inner) => {
                if let Some(inner) = inner {
                    self.walk_expression(inner);
                }
            }
            ExpressionKind::ImportExpression { module_specifier } => self.walk_expression(module_specifier),
            ExpressionKind::FunctionExpression { name, type_params, params, return_type, body, .. } => {
                self.enter_function(name.as_deref(), type_params, params, return_type.as_ref(), body);
            }
            ExpressionKind::ClassExpression { name, type_params, extends, implements, body } => {
                self.enter_class(name.as_deref(), type_params, extends.as_ref(), implements, body);
            }
            ExpressionKind::TaggedTemplateExpression { tag, template } => {
                self.walk_expression(tag);
                self.walk_expression(template);
            }
            ExpressionKind::JsxElement(_) | ExpressionKind::JsxFragment(_) | ExpressionKind::JsxSelfClosingElement(_) => {
                self.incomplete = true;
            }
            ExpressionKind::NumericLiteral(_)
            | ExpressionKind::StringLiteral(_)
            | ExpressionKind::BigIntLiteral(_)
            | ExpressionKind::BooleanLiteral(_)
            | ExpressionKind::NullLiteral
            | ExpressionKind::RegexLiteral(_)
            | ExpressionKind::MetaProperty { .. } => {}
        }
    }

    fn walk_type(&mut self, annotation: &TypeAnnotation) {
        match annotation {
            TypeAnnotation::Identifier(name) => self.resolve_type(name),
            TypeAnnotation::Reference { name, args } => {
                self.resolve_type(name);
                for arg in args {
                    self.walk_type(arg);
                }
            }
            TypeAnnotation::Predefined(_) | TypeAnnotation::Literal(_) | TypeAnnotation::Infer(_) => {}
            TypeAnnotation::Array(inner) | TypeAnnotation::KeyOf(inner) => self.walk_type(inner),
            TypeAnnotation::Tuple(members) | TypeAnnotation::Union(members) | TypeAnnotation::Intersection(members) => {
                for member in members {
                    self.walk_type(member);
                }
            }
            TypeAnnotation::Function { type_params, args, return_type } => {
                self.push(FrameKind::Block);
                self.register_type_params(type_params);
                for parameter in type_params {
                    self.walk_type_param_types(parameter);
                }
                for param in args {
                    if let Some(ty) = &param.ty {
                        self.walk_type(ty);
                    }
                    if let Some(default) = &param.default {
                        self.walk_expression(default);
                    }
                }
                self.walk_type(return_type);
                self.pop();
            }
            TypeAnnotation::Object(members) => {
                for member in members {
                    self.walk_type_member(member);
                }
            }
            TypeAnnotation::Query(text) => {
                for word in text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')) {
                    if let Some(&index) = self.binding_index.get(word) {
                        self.bindings[index].has_type_references = true;
                    }
                }
            }
            TypeAnnotation::Conditional { check_type, extends_type, true_type, false_type } => {
                self.walk_type(check_type);
                self.walk_type(extends_type);
                self.walk_type(true_type);
                self.walk_type(false_type);
            }
            TypeAnnotation::Mapped { key_name, key_type, value_type, .. } => {
                self.push(FrameKind::Block);
                self.register(key_name, None);
                self.walk_type(key_type);
                self.walk_type(value_type);
                self.pop();
            }
            TypeAnnotation::TemplateLiteral(elements) => {
                for element in elements {
                    if let TemplateElement::Type(inner) = element {
                        self.walk_type(inner);
                    }
                }
            }
        }
    }

    /// Registers the names a scope declares directly so that references before
    /// the declaration still resolve to the shadow, not to the module binding.
    fn prescan_declarations(&mut self, statements: &[Statement]) {
        for statement in statements {
            match statement {
                Statement::VariableDeclaration(declaration) if declaration.kind != VariableKind::Var => {
                    for declarator in &declaration.declarators {
                        for name in &declarator.binding_names {
                            self.register(name, None);
                        }
                    }
                }
                Statement::FunctionDeclaration(declaration) => {
                    if !declaration.name.is_empty() {
                        self.register(&declaration.name, None);
                    }
                }
                Statement::ClassDeclaration(declaration) => {
                    if !declaration.name.is_empty() {
                        self.register(&declaration.name, None);
                    }
                }
                Statement::Enum(declaration) => {
                    if !declaration.name.is_empty() {
                        self.register(&declaration.name, None);
                    }
                }
                _ => {}
            }
        }
    }
}

fn import_local(specifier: &ImportSpecifier) -> &str {
    match specifier {
        ImportSpecifier::Default { local, .. } | ImportSpecifier::Namespace { local, .. } | ImportSpecifier::Named { local, .. } => local,
    }
}

/// Collects every `var` binding declared in `statements`, descending through
/// block-shaped statements but never into function, class or namespace bodies.
fn collect_var_declarators(statements: &[Statement], out: &mut Vec<(String, Range<usize>)>) {
    for statement in statements {
        collect_var_declarators_one(statement, out);
    }
}

fn collect_var_declarators_one(statement: &Statement, out: &mut Vec<(String, Range<usize>)>) {
    match statement {
        Statement::VariableDeclaration(declaration) if declaration.kind == VariableKind::Var => {
            for declarator in &declaration.declarators {
                for (position, name) in declarator.binding_names.iter().enumerate() {
                    let span = match declarator.binding_spans.get(position) {
                        Some(span) => span.clone(),
                        None => Range { start: declarator.span.start, end: declarator.span.start },
                    };
                    out.push((name.clone(), span));
                }
            }
        }
        Statement::BlockStatement(block) => collect_var_declarators(&block.statements, out),
        Statement::IfStatement(statement) => {
            collect_var_declarators_one(&statement.consequent, out);
            if let Some(alternate) = &statement.alternate {
                collect_var_declarators_one(alternate, out);
            }
        }
        Statement::WhileStatement(statement) => collect_var_declarators_one(&statement.body, out),
        Statement::DoWhileStatement(statement) => collect_var_declarators_one(&statement.body, out),
        Statement::ForStatement(statement) => {
            if let Some(initializer) = &statement.initializer {
                collect_var_declarators_one(initializer, out);
            }
            collect_var_declarators_one(&statement.body, out);
        }
        Statement::ForInStatement(statement) => {
            collect_var_declarators_one(&statement.left, out);
            collect_var_declarators_one(&statement.body, out);
        }
        Statement::ForOfStatement(statement) => {
            collect_var_declarators_one(&statement.left, out);
            collect_var_declarators_one(&statement.body, out);
        }
        Statement::SwitchStatement(statement) => {
            for case in &statement.cases {
                collect_var_declarators(&case.consequent, out);
            }
        }
        Statement::TryStatement(statement) => {
            collect_var_declarators(&statement.block, out);
            if let Some(handler) = &statement.handler {
                collect_var_declarators(&handler.body, out);
            }
            if let Some(finalizer) = &statement.finalizer {
                collect_var_declarators(finalizer, out);
            }
        }
        _ => {}
    }
}
