//! Provides [`Serializer`] for [`to_string()`](super::to_string()).

use std::fmt::{Display, Write};

use serde::{
    ser::{self, Impossible, SerializeStructVariant},
    Serialize,
};

use super::Error;

/// A serializer for converting structs to mount options.
#[derive(Default, Debug)]
pub struct Serializer {
    output: String,
}

impl Serializer {
    /// Finalize serialization and return the output.
    pub fn into_output(self) -> String {
        self.output
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

    type SerializeMap = Impossible<(), Error>;

    type SerializeStruct = SerializeStruct<'a>;

    type SerializeStructVariant = SerializeStruct<'a>;

    serialize_invalid_primitives! {
        Error::BadType,
        serialize_bool: bool,
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_u128: u128,
        serialize_f32: f32,
        serialize_f64: f64,
        serialize_char: char,
        serialize_str: &str,
        serialize_bytes: &[u8],
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct {
            first: self.output.is_empty(),
            serializer: self,
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_struct(name, len)
    }
}

pub struct SerializeStruct<'a> {
    first: bool,
    serializer: &'a mut Serializer,
}

impl<'a> ser::SerializeStruct for SerializeStruct<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            self.serializer.output.push(',');
        }

        self.serializer.output.push_str(key);

        value.serialize(&mut ValueSerializer {
            serializer: self.serializer,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeStructVariant for SerializeStruct<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeStruct::end(self)
    }
}

/// Serializes values for [`Serializer`].
///
/// Sequences are serialized on separate lines, repeating the `key`.
struct ValueSerializer<'a> {
    serializer: &'a mut Serializer,
}

impl<'a> ValueSerializer<'a> {
    /// Writes the `value` to `serializer.output` as `key=value`.
    fn write_value(&mut self, value: impl Display) {
        write!(self.serializer.output, "={value}").expect("write to String never fails");
    }
}

impl<'a> ser::Serializer for &mut ValueSerializer<'a> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

    type SerializeMap = Impossible<(), Error>;

    type SerializeStruct = Impossible<(), Error>;

    type SerializeStructVariant = Impossible<(), Error>;

    serialize_primitives! {
        write_value,
        serialize_bool: bool,
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_u128: u128,
        serialize_f32: f32,
        serialize_f64: f64,
        serialize_char: char,
        serialize_str: &str,
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::BadType)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::BadType)
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        self.write_value(value);
        Ok(())
    }
}
