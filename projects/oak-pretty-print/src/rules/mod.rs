use crate::{Document, FormatResult, state::FormatContext};
use alloc::{boxed::Box, vec::Vec};
use oak_core::{
    errors::OakError,
    language::Language,
    tree::{RedLeaf, RedNode},
};

/// Trait for defining formatting rules
/// 
/// This trait is used to define custom formatting rules for AST nodes and tokens.
/// It supports language-specific configuration and custom formatting state.
/// 
/// The `C` type parameter represents the language-specific configuration.
/// The `S` type parameter represents the formatting state.
pub trait FormatRule<L: Language, C: Clone, S: Clone = crate::state::FormatState> {
    /// The name of the rule
    fn name(&self) -> &str;

    /// The priority of the rule (higher numbers indicate higher priority)
    fn priority(&self) -> u8 {
        0
    }

    /// Checks if the rule applies to the given node
    fn applies_to_node(&self, _node: &RedNode<L>) -> bool {
        false
    }

    /// Checks if the rule applies to the given token
    fn applies_to_token(&self, _token: &RedLeaf<L>) -> bool {
        false
    }

    /// Applies the formatting rule to a node, returning an optional Document
    /// 
    /// # Parameters
    /// - `node`: The AST node to format
    /// - `context`: The formatting context, including configuration and state
    /// - `source`: The source code string
    /// - `format_children`: A closure to format child nodes
    /// 
    /// # Returns
    /// An optional `Document` representing the formatted node
    fn apply_node<'a>(&self, node: &RedNode<L>, context: &FormatContext<L, C, S>, source: &'a str, format_children: &dyn Fn(&RedNode<L>) -> FormatResult<Document<'a>>) -> FormatResult<Option<Document<'a>>>;

    /// Applies the formatting rule to a token, returning an optional Document
    /// 
    /// # Parameters
    /// - `token`: The AST token to format
    /// - `context`: The formatting context, including configuration and state
    /// - `source`: The source code string
    /// 
    /// # Returns
    /// An optional `Document` representing the formatted token
    fn apply_token<'a>(&self, token: &RedLeaf<L>, context: &FormatContext<L, C, S>, source: &'a str) -> FormatResult<Option<Document<'a>>>;

    /// Checks if the rule conflicts with another rule
    fn conflicts_with(&self, _other: &dyn FormatRule<L, C, S>) -> bool {
        false
    }
}

/// A collection of formatting rules
/// 
/// This struct holds a collection of formatting rules that can be applied to AST nodes.
/// It supports language-specific configuration and custom formatting state.
/// 
/// The `C` type parameter represents the language-specific configuration.
/// The `S` type parameter represents the formatting state.
pub struct RuleSet<L: Language, C: Clone, S: Clone = crate::state::FormatState> {
    rules: Vec<Box<dyn FormatRule<L, C, S>>>,
}

impl<L: Language, C: Clone, S: Clone> Default for RuleSet<L, C, S> {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

impl<L: Language, C: Clone, S: Clone> RuleSet<L, C, S> {
    /// Creates a new rule set
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a rule to the set
    pub fn add_rule(&mut self, rule: Box<dyn FormatRule<L, C, S>>) -> FormatResult<()> {
        // Check for rule conflicts
        for existing_rule in &self.rules {
            if rule.conflicts_with(existing_rule.as_ref()) || existing_rule.conflicts_with(rule.as_ref()) {
                return Err(OakError::format_error(format!("Rule conflict between '{}' and '{}'", existing_rule.name(), rule.name())));
            }
        }

        self.rules.push(rule);

        // Sort by priority
        self.rules.sort_by(|a, b| b.priority().cmp(&a.priority()));

        Ok(())
    }

    /// Adds multiple rules to the set
    pub fn add_rules(&mut self, rules: Vec<Box<dyn FormatRule<L, C, S>>>) -> FormatResult<()> {
        for rule in rules {
            self.add_rule(rule)?
        }
        Ok(())
    }

    /// Gets rules applicable to a specific node
    pub fn applicable_rules_for_node<'a>(&'a self, node: &'a RedNode<L>) -> impl Iterator<Item = &'a dyn FormatRule<L, C, S>> + 'a {
        self.rules.iter().filter(move |rule| rule.applies_to_node(node)).map(|rule| rule.as_ref())
    }

    /// Gets rules applicable to a specific token
    pub fn applicable_rules_for_token<'a>(&'a self, token: &'a RedLeaf<L>) -> impl Iterator<Item = &'a dyn FormatRule<L, C, S>> + 'a {
        self.rules.iter().filter(move |rule| rule.applies_to_token(token)).map(|rule| rule.as_ref())
    }

    /// Applies all applicable node rules and returns the first successful Document
    pub fn apply_node_rules<'a>(&self, node: &RedNode<L>, context: &FormatContext<L, C, S>, source: &'a str, format_children: &dyn Fn(&RedNode<L>) -> FormatResult<Document<'a>>) -> FormatResult<Option<Document<'a>>> {
        for rule in self.applicable_rules_for_node(node) {
            if let Some(doc) = rule.apply_node(node, context, source, format_children)? {
                return Ok(Some(doc));
            }
        }
        Ok(None)
    }

    /// Applies all applicable token rules and returns the first successful Document
    pub fn apply_token_rules<'a>(&self, token: &RedLeaf<L>, context: &FormatContext<L, C, S>, source: &'a str) -> FormatResult<Option<Document<'a>>> {
        for rule in self.applicable_rules_for_token(token) {
            if let Some(doc) = rule.apply_token(token, context, source)? {
                return Ok(Some(doc));
            }
        }
        Ok(None)
    }
}
