#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]

mod cst_format;
mod print;

/// AST module for TypeScript.
pub mod ast;
/// Builder module for TypeScript.
pub mod builder;
/// Source formatting (`format_source`, `FormatOptions`).
pub mod formatter;

/// Language definition for TypeScript.
pub mod language;
/// Lexer for TypeScript.
pub mod lexer;

/// Parser for TypeScript.
pub mod parser;

// Re-exports
pub use crate::{
    ast::TypeScriptRoot,
    builder::TypeScriptBuilder,
    formatter::{FormatError, FormatOptions, format_source},
    language::TypeScriptLanguage,
    lexer::{TypeScriptLexer, token_type::TypeScriptTokenType},
    parser::{TypeScriptParser, element_type::TypeScriptElementType},
};

#[cfg(test)]
mod binding_pattern_unit {
    use crate::ast::Statement;
    use oak_core::{Builder, ParseSession, SourceText};
    use crate::{TypeScriptBuilder, TypeScriptLanguage};

    #[test]
    fn builds_object_binding_pattern_as_one_declaration() {
        let source = SourceText::new("const { name } = profile;");
        let language = TypeScriptLanguage::default();
        let builder = TypeScriptBuilder::new(&language);
        let mut session = ParseSession::default();
        let result = Builder::build(&builder, &source, &[], &mut session);
        let root = result.result.expect("binding pattern should parse");
        let Statement::VariableDeclaration(variable) = &root.statements[0] else {
            panic!("expected variable declaration");
        };
        assert_eq!(variable.name, "{ name }");
        assert!(variable.value.is_some());
    }
}

#[cfg(test)]
mod type_erasure_printer_unit {
    use crate::{format_source, FormatOptions};

    #[test]
    fn prints_exported_class_property_without_type_syntax() {
        let source = "export default class Chart { private width: number = 10; title: string = 'chart'; }";
        let output = format_source(source, &FormatOptions::default().with_type_erasure(true)).unwrap();
        assert_eq!(output, "export default class Chart { width = 10; title = 'chart'; }");
    }
}

#[cfg(test)]
mod control_flow_printer_unit {
    use crate::{format_source, FormatOptions};

    #[test]
    fn prints_minimal_control_flow_statements() {
        let source = "do { tick(); } while (ready); try { run(); } catch { recover(); } finally { cleanup(); }";
        let output = format_source(source, &FormatOptions::default()).unwrap();
        assert_eq!(
            output,
            "do { tick() } while (ready)\ntry { run() } catch { recover() } finally { cleanup() }"
        );
    }
}

#[cfg(feature = "lsp")]
pub use crate::lsp::{TypeScriptLanguageService, formatter::TypeScriptFormatter, highlighter::TypeScriptHighlighter};

#[cfg(feature = "mcp")]
pub use crate::mcp::serve_typescript_mcp;
