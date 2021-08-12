use std::collections::HashMap;

use serde::Serialize;

pub enum Value {
    Null,
    Boolean(bool),
    Integer(i128),
    Float(f64),
    String(String),
    Array(Vec<Self>),
    Object(HashMap<String, Value>),
}

impl Value {
    #[inline]
    pub fn serialize_from<T: Serialize>(value: &T) -> Result<Value, Error> {
        value.serialize(Serializer)
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i8> for Value {
    #[inline]
    fn from(value: i8) -> Self {
        Self::Integer(value.into())
    }
}

impl From<i16> for Value {
    #[inline]
    fn from(value: i16) -> Self {
        Self::Integer(value.into())
    }
}

impl From<i32> for Value {
    #[inline]
    fn from(value: i32) -> Self {
        Value::Integer(value.into())
    }
}

impl From<i64> for Value {
    #[inline]
    fn from(value: i64) -> Self {
        Self::Integer(value.into())
    }
}

impl From<i128> for Value {
    #[inline]
    fn from(value: i128) -> Self {
        Self::Integer(value)
    }
}

impl From<u8> for Value {
    #[inline]
    fn from(value: u8) -> Self {
        Self::Integer(value.into())
    }
}

impl From<u16> for Value {
    #[inline]
    fn from(value: u16) -> Self {
        Self::Integer(value.into())
    }
}

impl From<u32> for Value {
    #[inline]
    fn from(value: u32) -> Self {
        Self::Integer(value.into())
    }
}

impl From<u64> for Value {
    #[inline]
    fn from(value: u64) -> Self {
        Self::Integer(value.into())
    }
}

impl From<f32> for Value {
    #[inline]
    fn from(value: f32) -> Self {
        Self::Float(value.into())
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<String> for Value {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    #[inline]
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<char> for Value {
    #[inline]
    fn from(value: char) -> Self {
        Self::String(value.into())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    #[inline]
    fn from(value: Vec<T>) -> Self {
        Self::Array(value.into_iter().map(|x| x.into()).collect())
    }
}

impl<T: Into<Value>> From<HashMap<String, T>> for Value {
    #[inline]
    fn from(value: HashMap<String, T>) -> Self {
        Self::Object(value.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

#[derive(Debug)]
pub enum Error {
    NonStringKeyUnsupported,
    BytesUnsupported,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl serde::ser::StdError for Error {}
impl serde::ser::Error for Error {
    fn custom<T: core::fmt::Display>(_msg: T) -> Self {
        unimplemented!()
    }
}

struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeTuple;
    type SerializeTupleStruct = SerializeTupleStruct;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.into())
    }

    #[inline]
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::BytesUnsupported)
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let mut map = HashMap::new();
        map.insert(variant.into(), Value::Null);
        Ok(Value::Object(map))
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let mut map = HashMap::new();
        map.insert(variant.into(), Value::Null);
        Ok(Value::Object(map))
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq(Vec::with_capacity(len.unwrap_or_default())))
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeTuple(Vec::with_capacity(len)))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeTupleStruct(Vec::with_capacity(len)))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant(variant, Vec::with_capacity(len)))
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap(HashMap::new(), None))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct(HashMap::new()))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant(variant, HashMap::new()))
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        true
    }
}

struct SerializeSeq(Vec<Value>);
impl serde::ser::SerializeSeq for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0.push(value.serialize(Serializer)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Array(self.0))
    }
}

struct SerializeTuple(Vec<Value>);
impl serde::ser::SerializeTuple for SerializeTuple {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0.push(value.serialize(Serializer)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Array(self.0))
    }
}

struct SerializeTupleStruct(Vec<Value>);
impl serde::ser::SerializeTupleStruct for SerializeTupleStruct {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0.push(value.serialize(Serializer)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Array(self.0))
    }
}

struct SerializeTupleVariant(&'static str, Vec<Value>);
impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.1.push(value.serialize(Serializer)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut map = HashMap::new();
        map.insert(self.0.into(), Value::Array(self.1));
        Ok(Value::Object(map))
    }
}

struct SerializeMap(HashMap<String, Value>, Option<String>);
impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        if let Value::String(key) = key.serialize(Serializer)? {
            self.1 = Some(key);
            return Ok(());
        }

        Err(Error::NonStringKeyUnsupported)
    }

    #[inline]
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0
            .insert(self.1.take().unwrap(), value.serialize(Serializer)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Object(self.0))
    }
}

struct SerializeStruct(HashMap<String, Value>);
impl serde::ser::SerializeStruct for SerializeStruct {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.0.insert(key.into(), value.serialize(Serializer)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Object(self.0))
    }
}

struct SerializeStructVariant(&'static str, HashMap<String, Value>);
impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.1.insert(key.into(), value.serialize(Serializer)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let mut map = HashMap::new();
        map.insert(self.0.into(), Value::Object(self.1));
        Ok(Value::Object(map))
    }
}
