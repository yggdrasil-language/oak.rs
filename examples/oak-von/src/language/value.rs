use std::collections::HashMap;

/// VON value representation.
///
/// This represents the pure value of a VON element without any source code location information.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VonValue {
    /// String value.
    String(String),
    /// Number value.
    Number(f64),
    /// Boolean value.
    Boolean(bool),
    /// Null value.
    Null,
    /// Undefined value.
    Undefined,
    /// Infinity value.
    Inf,
    /// NaN value.
    Nan,
    /// Array value.
    Array(VonArray),
    /// Tuple value.
    Tuple(VonTuple),
    /// Object value.
    Object(VonObject),
    /// Enum value.
    Enum(VonEnum),
}

/// Array wrapper of VON
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonArray {
    pub elements: Vec<VonValue>,
}

/// Tuple wrapper of VON
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonTuple {
    pub elements: Vec<VonValue>,
}

/// Object wrapper of VON
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonObject {
    pub fields: Vec<VonField>,
}

/// Field of VON object
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonField {
    pub name: String,
    pub value: VonValue,
}

/// Enum value of VON
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VonEnum {
    pub variant: String,
    pub payload: Option<Box<VonValue>>,
}

impl VonValue {
    /// Returns the string slice if the value is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            VonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the number value if the value is a number.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            VonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns the boolean value if the value is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            VonValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns true if the value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, VonValue::Null)
    }

    /// Returns true if the value is undefined.
    pub fn is_undefined(&self) -> bool {
        matches!(self, VonValue::Undefined)
    }

    /// Returns true if the value is infinity.
    pub fn is_inf(&self) -> bool {
        matches!(self, VonValue::Inf)
    }

    /// Returns true if the value is NaN.
    pub fn is_nan(&self) -> bool {
        matches!(self, VonValue::Nan)
    }

    /// Returns a reference to the array if the value is an array.
    pub fn as_array(&self) -> Option<&Vec<VonValue>> {
        match self {
            VonValue::Array(VonArray { elements: a }) => Some(a),
            _ => None,
        }
    }

    /// Returns a reference to the tuple if the value is a tuple.
    pub fn as_tuple(&self) -> Option<&Vec<VonValue>> {
        match self {
            VonValue::Tuple(VonTuple { elements: t }) => Some(t),
            _ => None,
        }
    }

    /// Returns a reference to the object if the value is an object.
    pub fn as_object(&self) -> Option<&Vec<VonField>> {
        match self {
            VonValue::Object(VonObject { fields: o }) => Some(o),
            _ => None,
        }
    }

    /// Returns a reference to the enum if the value is an enum.
    pub fn as_enum(&self) -> Option<(&String, Option<&VonValue>)> {
        match self {
            VonValue::Enum(VonEnum { variant, payload }) => Some((variant, payload.as_deref())),
            _ => None,
        }
    }

    /// Gets a value from the object by key name.
    pub fn get(&self, key: &str) -> Option<&VonValue> {
        match self {
            VonValue::Object(VonObject { fields: o }) => o.iter().find(|field| field.name == key).map(|field| &field.value),
            _ => None,
        }
    }
}

impl std::fmt::Display for VonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VonValue::String(s) => write!(f, "\"{}\"", s),
            VonValue::Number(n) => write!(f, "{}", n),
            VonValue::Boolean(b) => write!(f, "{}", b),
            VonValue::Null => write!(f, "null"),
            VonValue::Undefined => write!(f, "undefined"),
            VonValue::Inf => write!(f, "inf"),
            VonValue::Nan => write!(f, "nan"),
            VonValue::Array(VonArray { elements: a }) => {
                write!(f, "[")?;
                for (i, item) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            VonValue::Tuple(VonTuple { elements: t }) => {
                write!(f, "(")?;
                for (i, item) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            VonValue::Object(VonObject { fields: o }) => {
                write!(f, "{{")?;
                for (i, VonField { name, value }) in o.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}={}", name, value)?;
                }
                write!(f, "}}")
            }
            VonValue::Enum(VonEnum { variant, payload }) => {
                write!(f, "{}", variant)?;
                if let Some(payload) = payload {
                    write!(f, " {}", payload)?;
                }
                Ok(())
            }
        }
    }
}

/// Converts from AST VonValue to pure VonValue.
pub fn from_ast(value: &crate::ast::VonValue) -> VonValue {
    match value {
        crate::ast::VonValue::Null(_) => VonValue::Null,
        crate::ast::VonValue::Boolean(b) => VonValue::Boolean(b.value),
        crate::ast::VonValue::Number(n) => VonValue::Number(n.value),
        crate::ast::VonValue::String(s) => VonValue::String(s.value.clone()),
        crate::ast::VonValue::Array(a) => VonValue::Array(VonArray { elements: a.elements.iter().map(from_ast).collect() }),
        crate::ast::VonValue::Tuple(t) => VonValue::Tuple(VonTuple { elements: t.elements.iter().map(from_ast).collect() }),
        crate::ast::VonValue::Object(o) => VonValue::Object(VonObject { fields: o.fields.iter().map(|field| VonField { name: field.name.clone(), value: from_ast(&field.value) }).collect() }),
        crate::ast::VonValue::Enum(e) => VonValue::Enum(VonEnum { variant: e.variant.clone(), payload: e.payload.as_ref().map(|p| Box::new(from_ast(p))) }),
        crate::ast::VonValue::Undefined(_) => VonValue::Undefined,
        crate::ast::VonValue::Inf(_) => VonValue::Inf,
        crate::ast::VonValue::Nan(_) => VonValue::Nan,
    }
}

/// Converts from pure VonValue to AST VonValue.
pub fn to_ast(value: &VonValue) -> crate::ast::VonValue {
    use crate::ast::{VonBoolean, VonEnum, VonInf, VonNan, VonNull, VonNumber, VonObject, VonString, VonTuple, VonUndefined, VonValue as AstVonValue};
    match value {
        VonValue::Null => AstVonValue::Null(VonNull { span: (0..0).into() }),
        VonValue::Boolean(b) => AstVonValue::Boolean(VonBoolean { span: (0..0).into(), value: *b }),
        VonValue::Number(n) => AstVonValue::Number(VonNumber { span: (0..0).into(), value: *n }),
        VonValue::String(s) => AstVonValue::String(VonString { span: (0..0).into(), value: s.clone() }),
        VonValue::Array(a) => AstVonValue::Array(crate::ast::VonArray { elements: a.elements.iter().map(to_ast).collect(), span: (0..0).into() }),
        VonValue::Tuple(t) => AstVonValue::Tuple(crate::ast::VonTuple { elements: t.elements.iter().map(to_ast).collect(), span: (0..0).into() }),
        VonValue::Object(o) => AstVonValue::Object(crate::ast::VonObject { fields: o.fields.iter().map(|field| crate::ast::VonField { span: (0..0).into(), name: field.name.clone(), value: to_ast(&field.value) }).collect(), span: (0..0).into() }),
        VonValue::Enum(e) => AstVonValue::Enum(crate::ast::VonEnum { span: (0..0).into(), variant: e.variant.clone(), payload: e.payload.as_ref().map(|p| Box::new(to_ast(p))) }),
        VonValue::Undefined => AstVonValue::Undefined(VonUndefined { span: (0..0).into() }),
        VonValue::Inf => AstVonValue::Inf(VonInf { span: (0..0).into() }),
        VonValue::Nan => AstVonValue::Nan(VonNan { span: (0..0).into() }),
    }
}
