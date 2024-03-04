#![doc = include_str!("readme.md")]
mod build_access;
mod build_anonymous_class;
mod build_binary;
mod build_block;
mod build_call;
mod build_class;
mod build_control;
mod build_expr;
mod build_lambda;
mod build_literal;
mod build_micro;
mod build_namespace;
mod build_object;
mod build_pratt;
mod build_root;
mod build_stmt;
mod build_template;
mod build_unary;
mod utils;

use crate::{ValkyrieLanguage, ValkyrieParser};
use oak_core::{Builder, BuilderCache, OakDiagnostics, Parser, TextEdit, builder::BuildOutput, source::Source};

/// Extracts text from source using a range.
///
/// # Parameters
/// - `source`: The source code provider
/// - `range`: The range of text to extract
///
/// # Returns
/// The extracted text as a string.
pub(crate) fn text(source: &(impl Source + ?Sized), range: oak_core::Range<usize>) -> String {
    source.get_text_in(range).to_string()
}

/// 从源码区间取出标识符文本；raw id（`` `any` ``）去掉两侧反引号。
pub(crate) fn identifier_text(source: &(impl Source + ?Sized), range: oak_core::Range<usize>) -> String {
    let raw = source.get_text_in(range);
    if raw.len() >= 2 && raw.starts_with('`') && raw.ends_with('`') { raw[1..raw.len() - 1].to_string() } else { raw.to_string() }
}

/// Valkyrie builder for constructing AST from parsed syntax trees.
///
/// The Valkyrie builder takes the green tree produced by the parser and transforms it into a
/// structured AST that can be used for further analysis or code generation.
///
/// # V Language Example
/// ```v
/// // Example of using the builder to parse and build AST from V code
/// import oak_valkyrie
///
/// fn main() {
///     let source = "class Person {\n    name: String\n    age: i32\n}\n";
///     let language = oak_valkyrie::ValkyrieLanguage::default();
///     let builder = oak_valkyrie::ValkyrieBuilder::new(&language);
///     
///     // Parse and build AST
///     let result = builder.build(source, &[], &mut oak_core::builder::DefaultBuilderCache::new());
///     
///     match result.result {
///         Ok(ast) => println("AST built successfully: {:?}", ast),
///         Err(error) => println("Error building AST: {:?}", error),
///     }
/// }
/// ```
pub struct ValkyrieBuilder<'config> {
    config: &'config ValkyrieLanguage,
}

impl<'config> ValkyrieBuilder<'config> {
    /// Create a new Valkyrie builder.
    ///
    /// # Parameters
    /// - `config`: The Valkyrie language configuration
    ///
    /// # Returns
    /// A new instance of `ValkyrieBuilder`.
    pub fn new(config: &'config ValkyrieLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<ValkyrieLanguage> for ValkyrieBuilder<'config> {
    /// Builds an AST from the parsed green tree.
    ///
    /// # Parameters
    /// - `source`: The source code provider
    /// - `edits`: Text edits to apply to the source
    /// - `_cache`: Builder cache (not used in this implementation)
    ///
    /// # Returns
    /// A `BuildOutput` containing the built AST or an error, along with any diagnostics.
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<ValkyrieLanguage>) -> BuildOutput<ValkyrieLanguage> {
        let parser = ValkyrieParser::new(self.config);

        let mut parse_cache = oak_core::parser::session::ParseSession::<ValkyrieLanguage>::default();
        let parse_result = parser.parse(source, edits, &mut parse_cache);

        match parse_result.result {
            Ok(green_tree) => match self.build_root(green_tree, source) {
                Ok(ast_root) => OakDiagnostics { result: Ok(ast_root), diagnostics: parse_result.diagnostics },
                Err(build_error) => {
                    let mut diagnostics = parse_result.diagnostics;
                    diagnostics.push(build_error.clone());
                    OakDiagnostics { result: Err(build_error), diagnostics }
                }
            },
            Err(e) => OakDiagnostics { result: Err(e), diagnostics: parse_result.diagnostics },
        }
    }
}
