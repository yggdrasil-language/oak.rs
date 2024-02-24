use serde::ser::{self, Serialize, Serializer};

use crate::language::value::{VonValue, to_ast};
use oak_core::OakError;

/// VON 序列化实现
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    value.serialize(serializer)
}

/// Serializes a Rust type into a VON string.
pub fn to_string<T>(value: &T) -> Result<String, OakError>
where
    T: serde::Serialize,
{
    let von_value = value.serialize(VonValueSerializer {})?;
    Ok(crate::printer::print_value(&von_value, crate::printer::PrintStyle::Compact, &crate::printer::PrintOptions::default()))
}

/// Serializes a Rust type into an indented VON string.
pub fn to_string_indented<T>(value: &T, indent_width: usize) -> Result<String, OakError>
where
    T: serde::Serialize,
{
    let von_value = value.serialize(VonValueSerializer {})?;
    Ok(crate::printer::print_value(
        &von_value,
        crate::printer::PrintStyle::Indented,
        &crate::printer::PrintOptions { indent_width },
    ))
}

struct VonValueSerializer {}

impl Default for VonValueSerializer {
    fn default() -> Self {
        Self {}
    }
}

impl serde::ser::Serializer for VonValueSerializer {
    type Ok = VonValue;
    type Error = OakError;

    type SerializeSeq = VonSeqSerializer;
    type SerializeTuple = VonSeqSerializer;
    type SerializeTupleStruct = VonSeqSerializer;
    type SerializeTupleVariant = VonSeqSerializer;
    type SerializeMap = VonMapSerializer;
    type SerializeStruct = VonMapSerializer;
    type SerializeStructVariant = VonMapSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Boolean(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v as f64))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v as f64))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v as f64))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v as f64))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v as f64))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v as f64))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v as f64))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v as f64))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Number(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::String(String::from_utf8_lossy(v).to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Enum(crate::language::value::VonEnum { variant: "None".to_string(), payload: None }))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let payload = value.serialize(VonValueSerializer {})?;
        Ok(VonValue::Enum(crate::language::value::VonEnum { variant: "Some".to_string(), payload: Some(Box::new(payload)) }))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Enum(crate::language::value::VonEnum { variant: variant.to_string(), payload: None }))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let payload = value.serialize(VonValueSerializer {})?;
        Ok(VonValue::Enum(crate::language::value::VonEnum { variant: variant.to_string(), payload: Some(Box::new(payload)) }))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(VonSeqSerializer { elements: Vec::with_capacity(len.unwrap_or(0)) })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(VonSeqSerializer { elements: Vec::with_capacity(len) })
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(VonSeqSerializer { elements: Vec::with_capacity(len) })
    }

    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(VonSeqSerializer { elements: Vec::with_capacity(len) })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(VonMapSerializer { fields: Vec::with_capacity(len.unwrap_or(0)), current_key: None })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(VonMapSerializer { fields: Vec::with_capacity(len), current_key: None })
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(VonMapSerializer { fields: Vec::with_capacity(len), current_key: Some(VonValue::String(variant.to_string())) })
    }
}

struct VonSeqSerializer {
    elements: Vec<VonValue>,
}

impl serde::ser::SerializeSeq for VonSeqSerializer {
    type Ok = VonValue;
    type Error = OakError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let serializer = VonValueSerializer {};
        let item = value.serialize(serializer)?;
        self.elements.push(item);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Array(crate::language::value::VonArray { elements: self.elements }))
    }
}

impl serde::ser::SerializeTuple for VonSeqSerializer {
    type Ok = VonValue;
    type Error = OakError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let serializer = VonValueSerializer {};
        let item = value.serialize(serializer)?;
        self.elements.push(item);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Tuple(crate::language::value::VonTuple { elements: self.elements }))
    }
}

impl serde::ser::SerializeTupleStruct for VonSeqSerializer {
    type Ok = VonValue;
    type Error = OakError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let serializer = VonValueSerializer {};
        let item = value.serialize(serializer)?;
        self.elements.push(item);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Tuple(crate::language::value::VonTuple { elements: self.elements }))
    }
}

impl serde::ser::SerializeTupleVariant for VonSeqSerializer {
    type Ok = VonValue;
    type Error = OakError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let serializer = VonValueSerializer {};
        let item = value.serialize(serializer)?;
        self.elements.push(item);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Array(crate::language::value::VonArray { elements: self.elements }))
    }
}

struct VonMapSerializer {
    fields: Vec<crate::language::value::VonField>,
    current_key: Option<VonValue>,
}

impl serde::ser::SerializeMap for VonMapSerializer {
    type Ok = VonValue;
    type Error = OakError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let serializer = VonValueSerializer {};
        let key_value = key.serialize(serializer)?;
        self.current_key = Some(key_value);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let serializer = VonValueSerializer {};
        let value_value = value.serialize(serializer)?;
        if let Some(key) = self.current_key.take() {
            // Convert VonValue key to string
            let key_str = match key {
                VonValue::String(s) => s,
                VonValue::Number(n) => n.to_string(),
                VonValue::Boolean(b) => b.to_string(),
                _ => return Err(OakError::custom_error("VON keys must be strings, numbers, or booleans")),
            };
            self.fields.push(crate::language::value::VonField { name: key_str, value: value_value });
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Object(crate::language::value::VonObject { fields: self.fields }))
    }
}

impl serde::ser::SerializeStruct for VonMapSerializer {
    type Ok = VonValue;
    type Error = OakError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let serializer = VonValueSerializer {};
        let value_value = value.serialize(serializer)?;
        self.fields.push(crate::language::value::VonField { name: key.to_string(), value: value_value });
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(VonValue::Object(crate::language::value::VonObject { fields: self.fields }))
    }
}

impl serde::ser::SerializeStructVariant for VonMapSerializer {
    type Ok = VonValue;
    type Error = OakError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let serializer = VonValueSerializer {};
        let value_value = value.serialize(serializer)?;
        self.fields.push(crate::language::value::VonField { name: key.to_string(), value: value_value });
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if let Some(key) = self.current_key {
            if let VonValue::String(variant) = key {
                // If there's only one field, use it as the payload
                if self.fields.len() == 1 {
                    let field = self.fields[0].clone();
                    Ok(VonValue::Enum(crate::language::value::VonEnum { variant, payload: Some(Box::new(field.value)) }))
                }
                else {
                    // Otherwise, use all fields as the payload
                    let payload = VonValue::Object(crate::language::value::VonObject { fields: self.fields });
                    Ok(VonValue::Enum(crate::language::value::VonEnum { variant, payload: Some(Box::new(payload)) }))
                }
            }
            else {
                Ok(VonValue::Object(crate::language::value::VonObject { fields: self.fields }))
            }
        }
        else {
            Ok(VonValue::Object(crate::language::value::VonObject { fields: self.fields }))
        }
    }
}
