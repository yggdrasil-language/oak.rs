#![doc = include_str!("readme.md")]
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};

#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::{AsDocument, doc as pp_doc};

/// The root node of a JSON AST.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonRoot {
    /// The top-level value of the JSON document.
    pub value: JsonValueNode,
}

impl ToSource for JsonRoot {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.value.to_source(buffer)
    }
}

/// Represents any valid JSON value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "oak-pretty-print", derive(AsDocument))]
pub enum JsonValueNode {
    /// A JSON object (collection of key-value pairs).
    Object(JsonObject),
    /// A JSON array (ordered list of values).
    Array(JsonArray),
    /// A JSON string.
    String(JsonString),
    /// A JSON number (represented as f64).
    Number(JsonNumber),
    /// A JSON boolean (true or false).
    Boolean(JsonBoolean),
    /// A JSON null value.
    Null(JsonNull),
}

impl JsonValueNode {
    /// Returns the value as a string slice, if it is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValueNode::String(s) => Some(&s.value),
            _ => None,
        }
    }

    /// Returns the value as an `f64`, if it is a number.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValueNode::Number(n) => Some(n.value),
            _ => None,
        }
    }

    /// Returns the value as a `bool`, if it is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValueNode::Boolean(b) => Some(b.value),
            _ => None,
        }
    }

    /// Returns the value as a `u64`, if it is a number.
    pub fn as_u64(&self) -> Option<u64> {
        self.as_f64().map(|f| f as u64)
    }

    /// Returns the value as a reference to a `JsonArray`, if it is an array.
    pub fn as_array(&self) -> Option<&JsonArray> {
        match self {
            JsonValueNode::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the value as a reference to a `JsonObject`, if it is an object.
    pub fn as_object(&self) -> Option<&JsonObject> {
        match self {
            JsonValueNode::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Gets a value from the object by key, if this value is an object.
    pub fn get(&self, key: &str) -> Option<&JsonValueNode> {
        match self {
            JsonValueNode::Object(o) => o.get(key),
            _ => None,
        }
    }

    /// Converts the JSON value to its string representation.
    pub fn to_string(&self) -> String {
        match self {
            JsonValueNode::Null(_) => "null".to_string(),
            JsonValueNode::Boolean(b) => b.value.to_string(),
            JsonValueNode::Number(n) => {
                // Format as integer if possible
                if n.value.fract() == 0.0 { (n.value as i64).to_string() } else { n.value.to_string() }
            }
            JsonValueNode::String(s) => format!("\"{}\"", s.value),
            JsonValueNode::Array(a) => {
                let elements: Vec<String> = a.elements.iter().map(|e| e.to_string()).collect();
                format!("[{}]", elements.join(","))
            }
            JsonValueNode::Object(o) => {
                let fields: Vec<String> = o.fields.iter().map(|f| format!("\"{}\":{}", f.name.value, f.value.to_string())).collect();
                format!("{{{}}}", fields.join(","))
            }
        }
    }
}

impl std::fmt::Display for JsonValueNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<&str> for JsonValueNode {
    fn from(s: &str) -> Self {
        JsonValueNode::String(JsonString { value: s.to_string(), span: (0..0).into() })
    }
}

impl From<String> for JsonValueNode {
    fn from(s: String) -> Self {
        JsonValueNode::String(JsonString { value: s, span: (0..0).into() })
    }
}

impl From<f64> for JsonValueNode {
    fn from(f: f64) -> Self {
        JsonValueNode::Number(JsonNumber { value: f, span: (0..0).into() })
    }
}

impl From<u64> for JsonValueNode {
    fn from(u: u64) -> Self {
        JsonValueNode::Number(JsonNumber { value: u as f64, span: (0..0).into() })
    }
}

impl From<i32> for JsonValueNode {
    fn from(i: i32) -> Self {
        JsonValueNode::Number(JsonNumber { value: i as f64, span: (0..0).into() })
    }
}

impl From<i64> for JsonValueNode {
    fn from(i: i64) -> Self {
        JsonValueNode::Number(JsonNumber { value: i as f64, span: (0..0).into() })
    }
}

impl From<usize> for JsonValueNode {
    fn from(u: usize) -> Self {
        JsonValueNode::Number(JsonNumber { value: u as f64, span: (0..0).into() })
    }
}

impl From<bool> for JsonValueNode {
    fn from(b: bool) -> Self {
        JsonValueNode::Boolean(JsonBoolean { value: b, span: (0..0).into() })
    }
}

impl From<u32> for JsonValueNode {
    fn from(value: u32) -> Self {
        JsonValueNode::Number(JsonNumber { value: value as f64, span: (0..0).into() })
    }
}

impl From<f32> for JsonValueNode {
    fn from(value: f32) -> Self {
        JsonValueNode::Number(JsonNumber { value: value as f64, span: (0..0).into() })
    }
}

impl<T: Into<JsonValueNode>> From<Option<T>> for JsonValueNode {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => JsonValueNode::Null(JsonNull { span: (0..0).into() }),
        }
    }
}

impl From<()> for JsonValueNode {
    fn from(_: ()) -> Self {
        JsonValueNode::Null(JsonNull { span: (0..0).into() })
    }
}

impl From<Vec<JsonValueNode>> for JsonValueNode {
    fn from(elements: Vec<JsonValueNode>) -> Self {
        JsonValueNode::Array(JsonArray { elements, span: (0..0).into() })
    }
}

impl From<std::collections::HashMap<String, JsonValueNode>> for JsonValueNode {
    fn from(fields: std::collections::HashMap<String, JsonValueNode>) -> Self {
        let mut fields_vec: Vec<JsonField> = fields.into_iter().map(|(k, v)| JsonField { name: JsonString { value: k, span: (0..0).into() }, value: v, span: (0..0).into() }).collect();
        // Sort fields by name for consistent output
        fields_vec.sort_by(|a, b| a.name.value.cmp(&b.name.value));
        JsonValueNode::Object(JsonObject { fields: fields_vec, span: (0..0).into() })
    }
}

impl<T: Into<JsonValueNode>> FromIterator<T> for JsonValueNode {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        JsonValueNode::Array(JsonArray { elements: iter.into_iter().map(Into::into).collect(), span: (0..0).into() })
    }
}

impl ToSource for JsonValueNode {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            JsonValueNode::Object(v) => v.to_source(buffer),
            JsonValueNode::Array(v) => v.to_source(buffer),
            JsonValueNode::String(v) => v.to_source(buffer),
            JsonValueNode::Number(v) => v.to_source(buffer),
            JsonValueNode::Boolean(v) => v.to_source(buffer),
            JsonValueNode::Null(v) => v.to_source(buffer),
        }
    }
}

/// Represents a JSON object.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "oak-pretty-print", derive(AsDocument))]
#[cfg_attr(feature = "oak-pretty-print", oak(doc = 
    if _self.fields.is_empty() {
        pp_doc!("{}")
    } else {
        pp_doc!(group( [
            "{",
            indent( [line, join(_self.fields.iter(), [",", line])]),
            line,
            "}"
        ]))
    }
))]
pub struct JsonObject {
    /// The fields (key-value pairs) of the object.
    pub fields: Vec<JsonField>,
    /// The source range of the object.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl JsonObject {
    /// Returns the value associated with the given key, if it exists.
    pub fn get(&self, key: &str) -> Option<&JsonValueNode> {
        self.fields.iter().find(|f| f.name.value == key).map(|f| &f.value)
    }
}

impl ToSource for JsonObject {
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

/// Represents a single field (key-value pair) in a JSON object.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "oak-pretty-print", derive(AsDocument))]
#[cfg_attr(feature = "oak-pretty-print", oak(doc = [self.name.as_document(config), ": ", self.value.as_document(config)]))]
pub struct JsonField {
    /// The name (key) of the field.
    pub name: JsonString,
    /// The value of the field.
    pub value: JsonValueNode,
    /// The source range of the field.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for JsonField {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.name.to_source(buffer);
        buffer.push(":");
        self.value.to_source(buffer)
    }
}

/// Represents a JSON array.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "oak-pretty-print", derive(AsDocument))]
#[cfg_attr(feature = "oak-pretty-print", oak(doc = 
    if _self.elements.is_empty() {
        pp_doc!("[]")
    } else {
        pp_doc!(group( [
            "[",
            indent( [line, join(_self.elements.iter(), [",", line])]),
            line,
            "]"
        ]))
    }
))]
pub struct JsonArray {
    /// The elements of the array.
    pub elements: Vec<JsonValueNode>,
    /// The source range of the array.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for JsonArray {
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

/// Represents a JSON string literal.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "oak-pretty-print", derive(AsDocument))]
#[cfg_attr(feature = "oak-pretty-print", oak(doc = format!("\"{}\"", self.value)))]
pub struct JsonString {
    /// The string content (without quotes).
    pub value: String,
    /// The source range of the string literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for JsonString {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("\"");
        buffer.push(&self.value);
        buffer.push("\"")
    }
}

/// Represents a JSON number literal.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "oak-pretty-print", derive(AsDocument))]
#[cfg_attr(feature = "oak-pretty-print", oak(doc = self.value.to_string()))]
pub struct JsonNumber {
    /// The numeric value.
    pub value: f64,
    /// The source range of the number literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for JsonNumber {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(&self.value.to_string())
    }
}

/// Represents a JSON boolean literal.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "oak-pretty-print", derive(AsDocument))]
#[cfg_attr(feature = "oak-pretty-print", oak(doc = if self.value { "true" } else { "false" }))]
pub struct JsonBoolean {
    /// The boolean value.
    pub value: bool,
    /// The source range of the boolean literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for JsonBoolean {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(if self.value { "true" } else { "false" })
    }
}

/// Represents a JSON null literal.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "oak-pretty-print", derive(AsDocument))]
#[cfg_attr(feature = "oak-pretty-print", oak(doc = "null"))]
pub struct JsonNull {
    /// The source range of the null literal.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for JsonNull {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("null")
    }
}
