use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ClassMember, Statement},
    formatter::{FormatOptions, format_source},
};

#[test]
fn repro_vmz_script_fragments() {
    let lang = TypeScriptLanguage::standard();
    for source in [
        "const el = ev?.currentTarget as HTMLElement | null;",
        "const iso = active?.getAttribute?.(\"data-vmz-date-iso\");",
        "const value = ((ev?.currentTarget as HTMLElement | null) ?? null);",
        "const c = \"vmz-ui-date-picker\" + (open ? \" is-opened\" : \"\");",
        "const c = \"vmz-ui-date-picker__day\" + (day.inMonth ? \"\" : \" is-outside\") + (day.selected ? \" is-on\" : \"\");",
        "const c = opt.selected ? 'true' : 'false';",
        "const h = (ev: KeyboardEvent) => { ev.preventDefault(); };",
        r#"const el = ev?.target
 ? (t.closest("[data-vmz-date-iso]") as HTMLElement | null)
                : ((ev?.currentTarget as HTMLElement | null) ?? null);"#,
        r#"const el =
            (t && typeof (t as HTMLElement).closest === "function"
                ? ((t as HTMLElement).closest("[data-vmz-option]") as HTMLElement | null)
                : null) || (ev?.currentTarget as HTMLElement | null);"#,
        r#"const id =
            el && typeof el.getAttribute === "function" ? el.getAttribute("data-vmz-option") : null;"#,
        r#"const type = String(file.type || "").toLowerCase();"#,
    ] {
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
        println!("{source}: {} diagnostics", result.diagnostics.len());
        if !result.diagnostics.is_empty() {
            println!("DIAGS={:?}", result.diagnostics);
        }
        assert!(result.result.is_ok());
        assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    }
}

#[test]
fn repro_assertions_inside_subscripts() {
    let lang = TypeScriptLanguage::standard();
    for source in [
        "const translated = english[ticket.id as keyof typeof english];",
        "const x = arr[i as number];",
        "const x = map[key as string];",
        "const x = list[(index as unknown) as number];",
        "const x = arr[i as number].toFixed(2);",
        "const value = input as Record<string, unknown>;",
        "const value = input as Array<Array<string>>;",
        "const value = input as A & B;",
        "const less = input as number < 3;",
    ] {
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
        assert!(result.result.is_ok(), "{source}");
        assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    }
}

#[test]
fn repro_array_type_positions() {
    let lang = TypeScriptLanguage::standard();
    for source in ["interface P { a: string[]; }", "interface P { vendors: VendorId[]; tags: string[] }", "const o = { a: string[] };", "const o = { a: [] };", "type T = { a: X[] };", "const grid: string[][] = [];"] {
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
        assert!(result.result.is_ok(), "{source}");
        assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    }
}

#[test]
fn repro_function_body_stays_outside_return_type() {
    let lang = TypeScriptLanguage::standard();
    let tail = "const STATIC_VENDORS: VendorId[] = ['cloudflare-pages', 'github-pages', 'vercel', 'netlify', 'tencent-edgeone'];\n";
    for head in [
        "function f(): string { return 'x'; }\n",
        "function f(x: string): string { return x; }\n",
        "function f(): void { g(); }\n",
        "export function f(): string { return 'x'; }\n",
        "function templ(template: string, vars: Record<string, string>): string {\n    return template.replace(/\\{\\{(\\w+)\\}\\}/g, (_m, key: string) => vars[key] || '');\n}\n",
        "function f(): { a: string } { return { a: 'x' }; }\n",
        "function f(): Array<{ a: string }> { return []; }\n",
    ] {
        let source = format!("{head}{tail}");
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source.as_str()), &[], &mut cache);
        assert!(result.result.is_ok(), "{head}");
        assert!(result.diagnostics.is_empty(), "{head}: {:#?}", result.diagnostics);
        assert_eq!(result.result.expect("root").statements.len(), 2, "{head}");
    }
}

#[test]
fn repro_return_type_keeps_body_statements() {
    let lang = TypeScriptLanguage::standard();
    let source = "function f(): string { return 'x'; }";
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    let root = result.result.expect("root");
    let Some(Statement::FunctionDeclaration(func)) = root.statements.first()
    else {
        panic!("expected function declaration, got {:?}", root.statements);
    };
    assert_eq!(func.body.len(), 1, "function body parsed as statements");
}

#[test]
fn repro_generic_declarations_keep_their_clauses() {
    // VMZ parses plain TypeScript, so `<T>` before `(` is a generic arrow and
    // never a JSX element.
    let lang = TypeScriptLanguage { jsx: false, ..TypeScriptLanguage::standard() };
    for source in [
        "function f<T>(x: T): T { return x; }",
        "function f<T extends Record<string, unknown>>(x: T): T { return x; }",
        "class Box<T> extends Base<T> implements C<T> { value!: T; }",
        "class A implements B<C>, D {}",
        "interface A<T> extends B<T> { value: T }",
        "type Box<T extends object = {}> = { value: T };",
        "type Fn = <T>(x: T) => T;",
        "const g = <T>(x: T): T => x;",
        "const g = <T extends object>(x: T): T => x;",
        "const v = a < b && c > d;",
    ] {
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
        assert!(result.result.is_ok(), "{source}");
        assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    }
}

#[test]
fn repro_typed_string_array_literal() {
    let lang = TypeScriptLanguage::standard();
    for source in [
        "const A: VendorId[] = ['a', 'b'];",
        "const A: VendorId[] = [];",
        "const A = ['a', 'b'];",
        "const A: string[] = ['a-b'];",
        "const A = ['tencent-edgeone'];",
        "const A: VendorId[] = ['cloudflare-pages', 'github-pages', 'vercel', 'netlify', 'tencent-edgeone'];",
    ] {
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
        println!("{source} => {:?}", result.diagnostics);
        assert!(result.result.is_ok(), "{source}");
        assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    }
}

#[test]
fn repro_type_only_import_and_export() {
    let lang = TypeScriptLanguage::standard();
    for source in ["import { a, type B, type C } from './x.ts';", "import { a } from './x.ts';\nimport type { B, C } from './x.ts';", "export type { B };", "export type { B, C };"] {
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
        println!("{source} => {:?}", result.diagnostics);
        assert!(result.result.is_ok(), "{source}");
        assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    }
}

#[test]
fn repro_regex_replace_with_typed_arrow() {
    let lang = TypeScriptLanguage::standard();
    for source in [
        r#"function templ(template: string): string { return template.replace(/\{\{(\w+)\}\}/g, "x"); }"#,
        r#"function templ(template: string, vars: Record<string, string>): string { return template.replace(/\{\{(\w+)\}\}/g, (_m, key: string) => vars[key] ?? ""); }"#,
        r#"const out = s.replace(/\{\{(\w+)\}\}/g, (_m, key: string) => vars[key] ?? "");"#,
        r#"const out = s.replace(/\{\{(\w+)\}\}/g, (_m, key) => vars[key] ?? "");"#,
        r#"const out = s.replace(/\{\{(\w+)\}\}/g, (_m, key: string) => vars[key] || "");"#,
    ] {
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
        assert!(result.result.is_ok(), "{source}");
        assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    }
}

#[test]
fn repro_vmz_formatter_fragments() {
    let options = FormatOptions::default();
    for source in [
        "export type ParsedOptions = Record<string, string | boolean | string[]> & { _: string[] };",
        "const FROM_SPEC_RE = /\\b(?:import|export)\\s+[^'\"\\n]*?\\s+from\\s+(['\"])([^'\"]+)\\1/g;",
        "if (/\\[[^\\]]+\\]/.test(href) || /\\/:[^/]+/.test(href)) return full;",
        "return /\\.(c?js|mjs|ts)$/i.test(token);",
    ] {
        let output = format_source(source, &options).expect("format");
        assert_eq!(format_source(&output, &options).expect("idempotent"), output, "source={source:?}");
    }
}

fn class_member_names(body: &str) -> (Vec<String>, usize) {
    let lang = TypeScriptLanguage::standard();
    let source = format!("export default class Probe {{\n{body}\n}}\n");
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source.as_str()), &[], &mut cache);
    let root = result.result.expect("root");
    let mut names = Vec::new();
    for stmt in &root.statements {
        let inner = match stmt {
            Statement::ExportDeclaration(exp) => exp.declaration.as_deref(),
            _ => None,
        };
        let Some(Statement::ClassDeclaration(class)) = inner
        else {
            continue;
        };
        for member in &class.body {
            match member {
                ClassMember::Method { name, .. } | ClassMember::Property { name, .. } => {
                    names.push(name.clone());
                }
            }
        }
    }
    (names, result.diagnostics.len())
}

#[test]
fn class_member_spans_stay_aligned_after_parenthesized_initializer() {
    let cases = [
        "    a() {\n        const z = (x) + y;\n    }\n    tail() { return 1; }",
        "    a() {\n        const z = (x) && y;\n    }\n    tail() { return 1; }",
        "    a(ev) {\n        if ((ev.key === \"ArrowDown\")) return;\n    }\n    tail() { return 1; }",
        "    a() {\n        if ((x) || y) return;\n    }\n    tail() { return 1; }",
    ];
    for body in cases {
        let (names, diags) = class_member_names(body);
        assert_eq!(diags, 0, "{body}");
        assert_eq!(names, vec!["a", "tail"], "{body}");
    }
}
