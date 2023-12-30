#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]

mod printer;

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

#[cfg(test)]
mod object_literal_unit {
    use crate::{format_source, FormatOptions};

    #[test]
    fn preserves_object_property_names_during_type_erasure() {
        let source = "const params = { id: 'sku-1', tab: \"security\" };";
        let output = format_source(source, &FormatOptions::default().with_type_erasure(true)).unwrap();
        assert_eq!(output, "const params = { id: 'sku-1', tab: 'security' }");
    }
}

#[cfg(test)]
mod function_type_unit {
    use crate::{format_source, FormatOptions};

    #[test]
    fn formats_function_types_inside_object_aliases() {
        let source = r#"type Api = { "parse-subject": (subject: string) => ParsedSubject; "section-for-gitmoji": (gitmoji: string | undefined) => ReleaseSection; };"#;
        let output = format_source(source, &FormatOptions::default()).unwrap();
        assert!(output.contains("type Api = {"));
        assert!(output.contains("\"parse-subject\": (subject: string) => ParsedSubject"));
        assert!(output.contains("\"section-for-gitmoji\": (gitmoji: string | undefined) => ReleaseSection"));
    }

    #[test]
    fn formats_nifty_native_contract_shape() {
        let source = r#"import type { ParsedSubject, ReleaseSection } from "./types.js";
export type GitmojiExports = {
    "known-gitmojis": () => string[];
    "validate-subject": (subject: string) => boolean;
    "leading-gitmoji": (subject: string) => string | undefined;
    "section-for-gitmoji": (gitmoji: string | undefined) => ReleaseSection;
    "author-mention": (email: string, authorName: string, authorMapJson: string) => string;
};
export type GitExports = {
    "resolve-range": (
        repoRoot: string,
        version: string | undefined,
        fromRef: string | undefined,
        toRef: string | undefined,
    ) => { version: string; "from-ref"?: string; "to-ref": string };
    "list-tag-infos": (repoRoot: string) => Array<{ name: string; "short-hash": string }>;
};"#;
        format_source(source, &FormatOptions::default()).expect("Nifty native type contract should format");
    }

    #[test]
    fn formats_nifty_native_function_tail() {
        let source = r#"const cached: NiftyNative | undefined;
export function loadNiftyNative(): NiftyNative {
    if (cached) {
        return cached;
    }
    const key = `${process.platform}-${process.arch}`;
    const pkg = PLATFORM_PACKAGES[key];
    if (!pkg) {
        throw new Error(`Unsupported platform for Nifty native bindings: ${key}`);
    }
    const require = createRequire(import.meta.url);
    const binding = require(pkg).default as NativeBinding;
    cached = wrapBinding(binding);
    return cached;
}

export function mapTagInfo(raw: { name: string; "short-hash": string }): TagInfo {
    return { name: raw.name, shortHash: raw["short-hash"] };
}"#;
        format_source(source, &FormatOptions::default()).expect("Nifty native function tail should format");
    }

    #[test]
    fn formats_optional_properties_in_function_return_types() {
        let source = r#"export function authPayload(options: AuthCliOptions): { otp?: string; totpSecret?: string; token?: string; npm: string } {
    return { npm: options.npm };
}"#;
        format_source(source, &FormatOptions::default()).expect("optional return properties should format");
    }

    #[test]
    fn formats_inline_type_import_specifiers() {
        let source = r#"import { value, type Value } from "./value.js";"#;
        format_source(source, &FormatOptions::default()).expect("inline type imports should format");
    }

    #[test]
    fn formats_async_function_exports() {
        let source = r#"export async function loadValue(): Promise<string> { return "ok"; }"#;
        format_source(source, &FormatOptions::default()).expect("async exports should format");
    }

    #[test]
    fn formats_inline_type_export_specifiers() {
        let source = r#"export { value, type Value } from "./value.js";"#;
        format_source(source, &FormatOptions::default()).expect("inline type exports should format");
    }

    #[test]
    fn formats_nifty_cli_index_exports() {
        let source = r#"export { runCli } from "./main.js";
export { bumpSemver, bumpWorkspace, runBump, type BumpKind, type BumpOptions, type BumpReport } from "./bump.js";
export { runPublish, type PublishOptions, type PublishReport } from "./publish.js";
export { runTrust, type TrustReport } from "./trust.js";
export { runUpdate } from "./update.js";
export { runLint } from "./lint.js";
export { runUpload } from "./upload.js";
export { installNativeFromOptions, type InstallNativeOptions } from "./install-native.js";
export { findWorkspaceRoot, listWorkspacePackages, type WorkspacePackage } from "./workspace.js";"#;
        let output = format_source(source, &FormatOptions::default()).expect("nifty cli exports should format");
        assert!(output.contains("type BumpKind"));
        assert!(output.contains("type WorkspacePackage"));
    }

    #[test]
    fn formats_const_asserted_array_literals() {
        format_source("const values = [\"a\"] as const;", &FormatOptions::default())
            .expect("const assertions should format");
    }

    #[test]
    fn formats_readonly_array_type_members() {
        format_source(
            "type Config = { values: readonly string[] | undefined };",
            &FormatOptions::default(),
        )
        .expect("readonly array type members should format");
    }

    #[test]
    fn formats_readonly_array_record_types() {
        format_source(
            "type Presets = Record<NiftyFormatPreset, readonly string[] | undefined>;",
            &FormatOptions::default(),
        )
        .expect("readonly array record types should format");
    }

    #[test]
    fn formats_nifty_format_presets_source() {
        let source = include_str!("E:/vmz 全栈开发/npm-tools/projects/packages/nifty/src/config/formatPresets.ts");
        format_source(source, &FormatOptions::default()).expect("Nifty format presets should format");
    }

    #[test]
    fn formats_preset_record_declaration() {
        format_source(
            "const PRESET_INCLUDES: Record<NiftyFormatPreset, readonly string[] | undefined> = { default: undefined };",
            &FormatOptions::default(),
        )
        .expect("preset record declaration should format");
    }

    #[test]
    fn formats_typed_arrow_return_config_source() {
        let source = r#"const DEFAULT_ENV = (cwd: string): NiftyConfigEnv => ({
    mode: process.env.NODE_ENV ?? "development",
    command: "serve",
    layout: detectProjectLayout(cwd),
});"#;
        format_source(source, &FormatOptions::default()).expect("typed arrow return should format");
    }

    #[test]
    fn formats_for_of_config_source() {
        format_source(
            "for (const name of CONFIG_FILE_NAMES) { const candidate = join(here, name); }",
            &FormatOptions::default(),
        )
        .expect("for-of should format");
    }

    #[test]
    fn formats_multiline_type_exports() {
        format_source(
            "export type {\n    CommitRecord,\n    GithubAuthor,\n} from \"./types.js\";",
            &FormatOptions::default(),
        )
        .expect("multiline type exports should format");
    }

    #[test]
    fn formats_import_meta_expression() {
        format_source("const require = createRequire(import.meta.url);", &FormatOptions::default()).expect("import.meta should format");
    }
}

#[cfg(feature = "lsp")]
pub use crate::lsp::{TypeScriptLanguageService, formatter::TypeScriptFormatter, highlighter::TypeScriptHighlighter};

#[cfg(feature = "mcp")]
pub use crate::mcp::serve_typescript_mcp;



