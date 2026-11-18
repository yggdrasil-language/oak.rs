//! 注册表行为用例：按语言 id 分派、未知语言失败闭合、文档类型不匹配失败。

use std::any::Any;

use oak_formats::contract::{FormatError, FormatOptions, PrintStyle, Printer};
use oak_formats::printer::{print_document, register_printer};

struct Marker(String);

struct MarkerPrinter;

impl Printer for MarkerPrinter {
    fn language_id(&self) -> &str {
        "marker"
    }

    fn print(&self, document: &dyn Any, style: PrintStyle, options: &FormatOptions) -> Result<String, FormatError> {
        let marker = document.downcast_ref::<Marker>().ok_or_else(|| FormatError::WrongDocument { expected: "Marker".to_string() })?;
        let indent = if style == PrintStyle::Indented { " ".repeat(options.indent_width) } else { String::new() };
        Ok(format!("{indent}{}", marker.0))
    }
}

#[test]
fn unknown_language_fails_closed() {
    let error = print_document("not-registered", &Marker("x".to_string()), PrintStyle::Compact, &FormatOptions::default())
        .expect_err("unregistered language must fail");
    assert!(matches!(error, FormatError::Unsupported { .. }), "{error:?}");
}

#[test]
fn registered_printer_receives_the_document() {
    register_printer(&["marker", "mk"], Box::new(MarkerPrinter));
    let compact = print_document("mk", &Marker("payload".to_string()), PrintStyle::Compact, &FormatOptions::default()).expect("compact");
    assert_eq!(compact, "payload");
    let indented = print_document("marker", &Marker("payload".to_string()), PrintStyle::Indented, &FormatOptions::default()).expect("indented");
    assert_eq!(indented, "    payload");
}

#[test]
fn wrong_document_type_is_rejected() {
    register_printer(&["marker-strict"], Box::new(MarkerPrinter));
    let error = print_document("marker-strict", &"not a marker".to_string(), PrintStyle::Compact, &FormatOptions::default())
        .expect_err("wrong document type must fail");
    assert!(matches!(error, FormatError::WrongDocument { .. }), "{error:?}");
}
