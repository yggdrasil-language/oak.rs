#![doc = include_str!("readme.md")]
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};

mod pretty;

/// Root node of the VON AST.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonRoot {
    /// The root value.
    pub value: VonValue,
}

impl ToSource for VonRoot {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.value.to_source(buffer)
    }
}

/// A value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VonValue {
    /// An object value.
    Object(VonObject),
    /// An array value.
    Array(VonArray),
    /// A tuple value.
    Tuple(VonTuple),
    /// A string value.
    String(VonString),
    /// A number value.
    Number(VonNumber),
    /// A boolean value.
    Boolean(VonBoolean),
    /// A null value.
    Null(VonNull),
    /// An undefined value.
    Undefined(VonUndefined),
    /// An infinity value.
    Inf(VonInf),
    /// A NaN value.
    Nan(VonNan),
    /// An enum value.
    Enum(VonEnum),
}

/// A tuple value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonTuple {
    /// The elements of the tuple.
    pub elements: Vec<VonValue>,
    /// The source span of the tuple.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl VonValue {
    /// Converts the value to a string representation.
    pub fn to_string(&self) -> String {
        match self {
            VonValue::Boolean(b) => b.value.to_string(),
            VonValue::Number(n) => n.value.to_string(),
            VonValue::Null(_) => "null".to_string(),
            VonValue::Undefined(_) => "undefined".to_string(),
            VonValue::Inf(_) => "inf".to_string(),
            VonValue::Nan(_) => "nan".to_string(),
            VonValue::String(s) => format!("\"{}\"", s.value),
            VonValue::Array(a) => {
                let elements: Vec<String> = a.elements.iter().map(|e| e.to_string()).collect();
                format!("[{}]", elements.join(","))
            }
            VonValue::Tuple(t) => {
                let elements: Vec<String> = t.elements.iter().map(|e| e.to_string()).collect();
                format!("({})", elements.join(","))
            }
            VonValue::Object(o) => {
                let fields: Vec<String> = o.fields.iter().map(|f| format!("{}={}", f.name, f.value.to_string())).collect();
                format!("{{{}}}", fields.join(","))
            }
            VonValue::Enum(e) => {
                if let Some(payload) = &e.payload {
                    format!("{} {}", e.variant, payload.to_string())
                }
                else {
                    e.variant.clone()
                }
            }
        }
    }
}

impl ToSource for VonValue {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            VonValue::Object(v) => v.to_source(buffer),
            VonValue::Array(v) => v.to_source(buffer),
            VonValue::Tuple(v) => v.to_source(buffer),
            VonValue::String(v) => v.to_source(buffer),
            VonValue::Number(v) => v.to_source(buffer),
            VonValue::Boolean(v) => v.to_source(buffer),
            VonValue::Null(v) => v.to_source(buffer),
            VonValue::Undefined(v) => v.to_source(buffer),
            VonValue::Inf(v) => v.to_source(buffer),
            VonValue::Nan(v) => v.to_source(buffer),
            VonValue::Enum(v) => v.to_source(buffer),
        }
    }
}

impl ToSource for VonTuple {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("(");
        for (i, element) in self.elements.iter().enumerate() {
            if i > 0 {
                buffer.push(",")
            }
            element.to_source(buffer)
        }
        buffer.push(")")
    }
}

/// An enum value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonEnum {
    /// The variant name.
    pub variant: String,
    /// The optional payload.
    pub payload: Option<Box<VonValue>>,
    /// The source span of the enum value.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonEnum {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(&self.variant);
        if let Some(payload) = &self.payload {
            buffer.push("(");
            payload.to_source(buffer);
            buffer.push(")");
        }
    }
}

/// An object value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonObject {
    /// The fields of the object.
    pub fields: Vec<VonField>,
    /// The source span of the object.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonObject {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("{");
        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                buffer.push(",")
            }
            field.to_source(buffer)
        }
        buffer.push("}")
    }
}

/// A field in a VON object.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonField {
    /// The name of the field.
    pub name: String,
    /// The value of the field.
    pub value: VonValue,
    /// The source span of the field.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonField {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(&self.name);
        buffer.push("=");
        self.value.to_source(buffer)
    }
}

/// An array value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonArray {
    /// The elements of the array.
    pub elements: Vec<VonValue>,
    /// The source span of the array.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonArray {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("[");
        for (i, element) in self.elements.iter().enumerate() {
            if i > 0 {
                buffer.push(",")
            }
            element.to_source(buffer)
        }
        buffer.push("]")
    }
}

/// A string value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonString {
    /// The string content.
    pub value: String,
    /// The source span of the string.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonString {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("\"");
        buffer.push(&self.value);
        buffer.push("\"")
    }
}

/// A number value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonNumber {
    /// The numeric value.
    pub value: f64,
    /// The source span of the number.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonNumber {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(&self.value.to_string())
    }
}

/// A boolean value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonBoolean {
    /// The boolean value.
    pub value: bool,
    /// The source span of the boolean.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonBoolean {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(if self.value { "true" } else { "false" })
    }
}

/// A null value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonNull {
    /// The source span of the null literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonNull {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("null")
    }
}

/// An undefined value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonUndefined {
    /// The source span of the undefined literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonUndefined {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("undefined")
    }
}

/// An infinity value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonInf {
    /// The source span of the infinity literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonInf {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("inf")
    }
}

/// A NaN (Not a Number) value in VON.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonNan {
    /// The source span of the NaN literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for VonNan {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("nan")
    }
}
