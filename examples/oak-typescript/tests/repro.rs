use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
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
