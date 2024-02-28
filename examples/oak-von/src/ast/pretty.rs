//! VON AST → `oak_pretty_print::Document`。

use oak_core::source::{SourceBuffer, ToSource};
use oak_pretty_print::{AsDocument, Document, SOFT_LINE as soft_line, doc, indent};

use super::{VonArray, VonEnum, VonField, VonObject, VonTuple, VonValue};

fn leaf_document(value: &VonValue) -> Document<'static> {
    let mut buffer = SourceBuffer::new();
    value.to_source(&mut buffer);
    Document::text(buffer.to_string())
}

impl AsDocument for VonValue {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        match self {
            VonValue::Object(object) => object.as_document(config),
            VonValue::Array(array) => array.as_document(config),
            VonValue::Tuple(tuple) => tuple.as_document(config),
            VonValue::Enum(enum_value) => enum_value.as_document(config),
            _ => leaf_document(self),
        }
    }
}

impl AsDocument for VonObject {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        if self.fields.is_empty() {
            return Document::text("{}");
        }
        let fields = Document::join(self.fields.iter().map(|field| field.as_document(config)), doc!(",", soft_line));
        Document::group(doc!("{", indent(doc!(soft_line, fields)), "}"))
    }
}

impl AsDocument for VonField {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        doc!(self.name.as_str(), "=", self.value.as_document(config))
    }
}

impl AsDocument for VonArray {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        if self.elements.is_empty() {
            return Document::text("[]");
        }
        let items = Document::join(self.elements.iter().map(|element| element.as_document(config)), doc!(",", soft_line));
        Document::group(doc!("[", indent(doc!(soft_line, items)), "]"))
    }
}

impl AsDocument for VonTuple {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        if self.elements.is_empty() {
            return Document::text("()");
        }
        if self.elements.len() == 1 {
            return leaf_document(&VonValue::Tuple(self.clone()));
        }
        let items = Document::join(self.elements.iter().map(|element| element.as_document(config)), doc!(",", soft_line));
        Document::group(doc!("(", indent(doc!(soft_line, items)), ")"))
    }
}

impl AsDocument for VonEnum {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        match &self.payload {
            None => Document::text(self.variant.clone()),
            Some(payload) => doc!(self.variant.as_str(), " ", payload.as_document(config)),
        }
    }
}
