//! TypeScript RedTree `FormatRule` registry.

mod import_declaration;
mod export_declaration;
mod variable_declaration;

use oak_pretty_print::RuleSet;

use crate::{formatter::FormatOptions, language::TypeScriptLanguage};

pub use import_declaration::ImportDeclarationRule;
pub use export_declaration::ExportDeclarationRule;
pub use variable_declaration::VariableDeclarationRule;

/// Build the default TypeScript RedTree rule set.
pub fn default_rule_set() -> RuleSet<TypeScriptLanguage, FormatOptions> {
    let mut rules = RuleSet::new();
    rules.add_rule(Box::new(ExportDeclarationRule)).expect("export_declaration rule");
    rules.add_rule(Box::new(VariableDeclarationRule)).expect("variable_declaration rule");
    rules.add_rule(Box::new(ImportDeclarationRule)).expect("import_declaration rule");
    rules
}
