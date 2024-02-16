use oak_core::{Builder, parser::ParseSession, SourceText};
use oak_typescript::{TypeScriptBuilder, TypeScriptLanguage};

#[test]
fn repro_vmz_script_fragments() {
    let lang = TypeScriptLanguage::standard();
    for source in [
        "const el = ev?.currentTarget as HTMLElement | null;",
        "const iso = active?.getAttribute?.(\"data-vmz-date-iso\");",
        "const value = ((ev?.currentTarget as HTMLElement | null) ?? null);",
    ] {
        let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
        let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
        println!("{source}: {} diagnostics", result.diagnostics.len());
        assert!(result.result.is_ok());
        assert!(result.diagnostics.is_empty(), "{source}: {:?}", result.diagnostics);
    }
}
