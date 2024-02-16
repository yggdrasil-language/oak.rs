use oak_core::{Builder, parser::ParseSession, SourceText};
use oak_typescript::{TypeScriptBuilder, TypeScriptLanguage};

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
    ] {
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
        println!("{source}: {} diagnostics", result.diagnostics.len());
        if !result.diagnostics.is_empty() { println!("DIAGS={:?}", result.diagnostics); }
        assert!(result.result.is_ok());
        assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    }
}
